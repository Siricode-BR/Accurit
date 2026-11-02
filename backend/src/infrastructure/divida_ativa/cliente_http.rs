//! Cliente HTTP da SERPRO para a Dívida Ativa (Fase 4). Implementa a
//! porta `ClienteDividaAtiva`: `GET {base}/devedor/{cpfCnpj}` e
//! `GET {base}/inscricao/{numero}` com autenticação Bearer (trial
//! fixo ou OAuth2 client-credentials), retries idempotentes (GET),
//! Retry-After com backoff+jitter em 429/5xx/timeout, e DTOs Serde
//! tolerantes (objeto plano ou lista). O JSON bruto da SERPRO nunca
//! passa para a camada acima — só VOs de domínio.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, TimeDelta, Utc};
use rand::Rng;
use secrecy::ExposeSecret;
use serde::Deserialize;
use tokio::sync::Mutex;
use url::Url;

use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::erros::ErroDividaAtiva;
use crate::domain::divida_ativa::ports::ClienteDividaAtiva;
use crate::domain::divida_ativa::{
    EstadoConsulta, FonteConsulta, InscricaoDetalhe, InscricaoResumo, ResultadoConsultaDevedor,
    mascarar_documento, parse_moeda_ptbr_estrito,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::{EndpointSerpro, EstrategiaAutenticacaoSerpro};

const MAX_TENTATIVAS: u32 = 3;
const MARGEM_RENOVACAO_TOKEN: TimeDelta = TimeDelta::seconds(60);
const ATRASO_BASE_MS: u64 = 500;

#[derive(Clone)]
pub struct ClienteDividaAtivaHttp {
    http: reqwest::Client,
    tokens_oauth: Arc<Mutex<HashMap<String, TokenEmCache>>>,
}

#[derive(Clone)]
struct TokenEmCache {
    bearer: String,
    expiracao: DateTime<Utc>,
}

impl ClienteDividaAtivaHttp {
    pub fn novo(http: reqwest::Client) -> Self {
        Self {
            http,
            tokens_oauth: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// DTO de entrada tolerante: a SERPRO responde objeto plano (P2) ou
/// lista (P3/multi-inscrição); `untagged` cobre os dois.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RespostaDevedorSerpro {
    Objeto(DevedorSerproDto),
    Lista(Vec<DevedorSerproDto>),
    Nada,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevedorSerproDto {
    numero_inscricao: Option<String>,
    numero_processo: Option<String>,
    situacao_inscricao: Option<String>,
    situacao_descricao: Option<String>,
    nome_devedor: Option<String>,
    tipo_devedor: Option<String>,
    valor_total_consolidado_moeda: Option<String>,
    cpf_cnpj: Option<String>,
    nome_unidade: Option<String>,
}

/// DTO de entrada da inscrição: a resposta é objeto plano com muitos
/// campos opcionais e valores monetários pt-BR.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InscricaoSerproDto {
    inscricao: Option<String>,
    processo_administrativo: Option<String>,
    situacao: Option<String>,
    data_inscricao: Option<String>,
    orgao_origem: Option<String>,
    codigo_natureza_receita: Option<String>,
    nome_receita: Option<String>,
    valor_total_inscrito_moeda: Option<String>,
    valor_total_consolidado_moeda: Option<String>,
    valor_remanescente_moeda: Option<String>,
    indicador_prescricao_sv8: Option<String>,
    nome_unidade: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RespostaInscricaoSerpro {
    Objeto(InscricaoSerproDto),
    Lista(Vec<InscricaoSerproDto>),
    Nada,
}

/// Token OAuth2 (apenas produção — o trial usa bearer fixo).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct TokenOAuthSerpro {
    access_token: Option<String>,
    expires_in: Option<u64>,
}

fn montar_url(base: &Url, caminho: &str) -> Result<Url, ErroDividaAtiva> {
    let base_str = base.as_str().trim_end_matches('/');
    if base_str.ends_with("/api/v1") {
        Url::parse(&format!("{base_str}/{caminho}"))
    } else {
        // Contrato: a base termina em `/api/v1` (quick-start SERPRO).
        Url::parse(&format!("{base_str}/api/v1/{caminho}"))
    }
    .map_err(|_| ErroDividaAtiva::RespostaInvalida)
}

impl ClienteDividaAtivaHttp {
    fn atraso_backoff(esta_tentativa: u32) -> Duration {
        let milisegundos = ATRASO_BASE_MS * u64::from(esta_tentativa);
        let jitter: u64 = rand::thread_rng().gen_range(0..=250);
        Duration::from_millis(milisegundos + jitter)
    }

    async fn bearer_para(
        &self,
        autenticacao: &EstrategiaAutenticacaoSerpro,
    ) -> Result<Option<String>, ErroDividaAtiva> {
        match autenticacao {
            EstrategiaAutenticacaoSerpro::BearerFixo(token) => {
                Ok(Some(token.expose_secret().to_owned()))
            }
            EstrategiaAutenticacaoSerpro::Oauth2Producao {
                token_url,
                consumer_key,
                consumer_secret,
            } => {
                let chave_cache = token_url.as_str().to_owned();
                {
                    let cache = self.tokens_oauth.lock().await;
                    if let Some(entrada) = cache.get(&chave_cache) {
                        if entrada.expiracao > Utc::now() {
                            return Ok(Some(entrada.bearer.clone()));
                        }
                    }
                }

                let novo_token = self
                    .obter_token_oauth(token_url, consumer_key, consumer_secret)
                    .await?;
                let mut cache = self.tokens_oauth.lock().await;
                cache.insert(chave_cache, novo_token.clone());
                Ok(Some(novo_token.bearer))
            }
        }
    }

    async fn obter_token_oauth(
        &self,
        token_url: &Url,
        consumer_key: &secrecy::SecretString,
        consumer_secret: &secrecy::SecretString,
    ) -> Result<TokenEmCache, ErroDividaAtiva> {
        let basic = STANDARD.encode(format!(
            "{}:{}",
            consumer_key.expose_secret(),
            consumer_secret.expose_secret()
        ));

        let resposta = self
            .http
            .post(token_url.clone())
            .header("Authorization", format!("Basic {basic}"))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|_| ErroDividaAtiva::NaoAutorizado)?;

        if resposta.status().as_u16() == 401 || resposta.status().as_u16() == 403 {
            return Err(ErroDividaAtiva::NaoAutorizado);
        }
        if !resposta.status().is_success() {
            return Err(ErroDividaAtiva::ServicoIndisponivel);
        }

        let corpo: TokenOAuthSerpro = resposta
            .json()
            .await
            .map_err(|_| ErroDividaAtiva::RespostaInvalida)?;
        let bearer = corpo
            .access_token
            .filter(|valor| !valor.trim().is_empty())
            .ok_or(ErroDividaAtiva::NaoAutorizado)?;
        let expires_in = corpo.expires_in.unwrap_or(3_295);
        let expiracao = Utc::now()
            + TimeDelta::try_seconds(i64::try_from(expires_in).unwrap_or(3_600))
                .unwrap_or(TimeDelta::hours(1))
            - MARGEM_RENOVACAO_TOKEN;
        Ok(TokenEmCache { bearer, expiracao })
    }

    async fn invalidar_token_oauth(&self, token_url: &Url) {
        let mut cache = self.tokens_oauth.lock().await;
        cache.remove(token_url.as_str());
    }

    async fn consultar_com_retentativas(
        &self,
        url: &Url,
        endpoint: &EndpointSerpro,
        cpf_operador: Option<String>,
        path_servico: &str,
    ) -> Result<(serde_json::Value, u32), ErroDividaAtiva> {
        let mut renovou = false;

        for esta_tentativa in 1..=MAX_TENTATIVAS {
            let bearer = match self.bearer_para(&endpoint.autenticacao).await {
                Ok(token) => token,
                Err(erro) => return Err(erro),
            };
            let mut pedido = self
                .http
                .get(url.clone())
                .header("accept", "application/json");
            if let Some(bearer) = bearer {
                pedido = pedido.header("Authorization", format!("Bearer {bearer}"));
            }
            if let Some(cpf) = &cpf_operador {
                // D3: só vai quando habilitado; o valor vem de
                // `usuarios.cpf` do contexto autenticado, nunca do input.
                pedido = pedido.header("x-cpf-usuario", cpf);
            }
            tracing::debug!(
                produto = "DIVIDA_ATIVA",
                ambiente = %endpoint.ambiente,
                path = %path_servico,
                tentativa = esta_tentativa,
                "consultando SERPRO"
            );

            let resposta = match pedido.send().await {
                Ok(resposta) => resposta,
                // Timeout/erro de transporte: GET é idempotente, retry.
                Err(_) => {
                    if esta_tentativa < MAX_TENTATIVAS {
                        tokio::time::sleep(Self::atraso_backoff(esta_tentativa)).await;
                        continue;
                    }
                    return Err(ErroDividaAtiva::ServicoIndisponivel);
                }
            };

            let status = resposta.status();
            match status.as_u16() {
                200 => {
                    let corpo = resposta
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|_| ErroDividaAtiva::RespostaInvalida)?;
                    return Ok((corpo, esta_tentativa));
                }
                401 => {
                    if let EstrategiaAutenticacaoSerpro::Oauth2Producao { token_url, .. } =
                        &endpoint.autenticacao
                    {
                        if !renovou {
                            renovou = true;
                            self.invalidar_token_oauth(token_url).await;
                            continue;
                        }
                    }
                    return Err(ErroDividaAtiva::NaoAutorizado);
                }
                403 => return Err(ErroDividaAtiva::ProxyNegado),
                404 => return Err(ErroDividaAtiva::NaoEncontrado),
                429 => {
                    if esta_tentativa < MAX_TENTATIVAS {
                        let espera = resposta
                            .headers()
                            .get("retry-after")
                            .and_then(|valor| valor.to_str().ok())
                            .and_then(|valor| valor.trim().parse::<u64>().ok())
                            .map(Duration::from_secs)
                            .unwrap_or_else(|| Self::atraso_backoff(esta_tentativa));
                        tokio::time::sleep(espera).await;
                        continue;
                    }
                    return Err(ErroDividaAtiva::LimiteSerproAtingido);
                }
                status_5xx if (500..600).contains(&status_5xx) => {
                    if esta_tentativa < MAX_TENTATIVAS {
                        tokio::time::sleep(Self::atraso_backoff(esta_tentativa)).await;
                        continue;
                    }
                    return Err(ErroDividaAtiva::ServicoIndisponivel);
                }
                _ => return Err(ErroDividaAtiva::RespostaInvalida),
            }
        }

        Err(ErroDividaAtiva::ServicoIndisponivel)
    }

    fn traduz_devedor(&self, dto: DevedorSerproDto) -> Result<InscricaoResumo, ErroDividaAtiva> {
        let situacao_descricao = dto.situacao_descricao.unwrap_or_default();
        let valor = dto
            .valor_total_consolidado_moeda
            .map(|texto| parse_moeda_ptbr_estrito(&texto))
            .transpose()
            .map_err(|_| ErroDividaAtiva::RespostaInvalida)?
            .unwrap_or_default();

        Ok(InscricaoResumo {
            numero_inscricao: dto.numero_inscricao.unwrap_or_else(|| String::from("?")),
            numero_processo: dto.numero_processo,
            situacao_inscricao: dto.situacao_inscricao,
            situacao_descricao,
            nome_devedor: dto.nome_devedor,
            tipo_devedor: dto
                .tipo_devedor
                .unwrap_or_else(|| String::from("PRINCIPAL")),
            valor_total_consolidado: valor,
            cpf_cnpj: dto.cpf_cnpj,
            nome_unidade: dto.nome_unidade,
        })
    }

    fn traduz_inscricao(
        &self,
        dto: InscricaoSerproDto,
        numero_inscricao_fallback: &str,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        let converter =
            |texto: Option<String>| -> Result<Option<rust_decimal::Decimal>, ErroDividaAtiva> {
                texto
                    .map(|valor| parse_moeda_ptbr_estrito(&valor))
                    .transpose()
                    .map_err(|_| ErroDividaAtiva::RespostaInvalida)
            };

        Ok(InscricaoDetalhe {
            ambiente: AmbienteSerpro::Trial,
            estado: EstadoConsulta::SemPendencias,
            fonte: FonteConsulta::CpfCnpj,
            chave_mascarada: mascarar_inscricao(&numero_inscricao_fallback),
            horario_consulta: chrono::Utc::now(),
            tentativas: 1,
            numero_inscricao: dto
                .inscricao
                .unwrap_or_else(|| numero_inscricao_fallback.to_owned()),
            processo_administrativo: dto.processo_administrativo,
            situacao: dto.situacao.unwrap_or_default(),
            data_inscricao: dto.data_inscricao,
            orgao_origem: dto.orgao_origem,
            codigo_natureza_receita: dto.codigo_natureza_receita,
            nome_receita: dto.nome_receita,
            valor_total_inscrito: converter(dto.valor_total_inscrito_moeda)?,
            valor_total_consolidado: converter(dto.valor_total_consolidado_moeda)?,
            valor_remanescente: converter(dto.valor_remanescente_moeda)?,
            indicador_prescricao: dto
                .indicador_prescricao_sv8
                .map(|valor| match valor.trim().to_uppercase().as_str() {
                    "SIM" => Ok(true),
                    "NAO" => Ok(false),
                    _ => Err(ErroDividaAtiva::RespostaInvalida),
                })
                .transpose()?,
            nome_unidade: dto.nome_unidade,
        })
    }
}

#[async_trait]
impl ClienteDividaAtiva for ClienteDividaAtivaHttp {
    async fn consultar_devedor(
        &self,
        endpoint: &EndpointSerpro,
        chave: &CpfCnpj,
        cpf_operador: Option<String>,
    ) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva> {
        let url = montar_url(&endpoint.url_base, &format!("devedor/{}", chave.como_str()))?;
        let (corpo, tentativas) = self
            .consultar_com_retentativas(&url, endpoint, cpf_operador, "devedor")
            .await?;

        let dados: RespostaDevedorSerpro =
            serde_json::from_value(corpo).map_err(|_| ErroDividaAtiva::RespostaInvalida)?;

        let inscricoes: Vec<InscricaoResumo> = match dados {
            RespostaDevedorSerpro::Objeto(dto) => match dto.numero_inscricao {
                // `{}` (200 vazio) não é erro: é SEM_PENDENCIAS (P2).
                Some(_) => vec![self.traduz_devedor(dto)?],
                None => Vec::new(),
            },
            RespostaDevedorSerpro::Lista(lista) => {
                let mut resultado = Vec::with_capacity(lista.len());
                for dto in lista {
                    resultado.push(self.traduz_devedor(dto)?);
                }
                resultado
            }
            RespostaDevedorSerpro::Nada => Vec::new(),
        };

        let estado = if inscricoes.is_empty() {
            EstadoConsulta::SemPendencias
        } else {
            EstadoConsulta::ComPendencias
        };

        Ok(ResultadoConsultaDevedor {
            ambiente: endpoint.ambiente,
            estado,
            fonte: FonteConsulta::CpfCnpj,
            chave_mascarada: mascarar_documento(chave.como_str()),
            horario_consulta: Utc::now(),
            tentativas,
            inscricoes,
        })
    }

    async fn consultar_inscricao(
        &self,
        endpoint: &EndpointSerpro,
        numero_inscricao: &str,
        cpf_operador: Option<String>,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        let url = montar_url(&endpoint.url_base, &format!("inscricao/{numero_inscricao}"))?;
        let (corpo, tentativas) = self
            .consultar_com_retentativas(&url, endpoint, cpf_operador, "inscricao")
            .await?;

        let dados: RespostaInscricaoSerpro =
            serde_json::from_value(corpo).map_err(|_| ErroDividaAtiva::RespostaInvalida)?;
        let dto = match dados {
            RespostaInscricaoSerpro::Objeto(dto) => dto,
            RespostaInscricaoSerpro::Lista(lista) => lista
                .into_iter()
                .next()
                .ok_or(ErroDividaAtiva::RespostaInvalida)?,
            RespostaInscricaoSerpro::Nada => return Err(ErroDividaAtiva::NaoEncontrado),
        };

        let mut detalhe = self.traduz_inscricao(dto, numero_inscricao)?;
        detalhe.ambiente = endpoint.ambiente;
        detalhe.estado = EstadoConsulta::ComPendencias;
        detalhe.fonte = FonteConsulta::CpfCnpj;
        detalhe.chave_mascarada = mascarar_inscricao(numero_inscricao);
        detalhe.horario_consulta = Utc::now();
        detalhe.tentativas = tentativas;
        Ok(detalhe)
    }
}

fn mascarar_inscricao(numero: &str) -> String {
    if numero.len() <= 4 {
        return String::from("?");
    }
    format!("***{}", &numero[numero.len() - 3..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_base_conclui_em_api_v1() {
        let base = Url::parse("https://gateway.test/consulta-divida-ativa-trial/api/v1")
            .expect("url base");
        let url = montar_url(&base, "devedor/09781911768").expect("url");
        assert_eq!(
            url.as_str(),
            "https://gateway.test/consulta-divida-ativa-trial/api/v1/devedor/09781911768"
        );
    }

    #[test]
    fn url_base_sem_api_v1_acrescenta() {
        let base = Url::parse("https://gateway.test/consulta-divida-ativa-trial").expect("url");
        let url = montar_url(&base, "inscricao/9061700096419").expect("url");
        assert_eq!(
            url.as_str(),
            "https://gateway.test/consulta-divida-ativa-trial/api/v1/inscricao/9061700096419"
        );
    }

    #[test]
    fn dto_lista_untagged_nao_compila_duplicado() {
        // Guarda do contrato: `untagged` cobre objeto OU lista.
        let um = serde_json::from_str::<RespostaDevedorSerpro>(
            r#"{"numeroInscricao":"10 5 03","numeroProcesso":"10166"}"#,
        )
        .expect("objeto");
        assert!(matches!(um, RespostaDevedorSerpro::Objeto(_)));

        let varios =
            serde_json::from_str::<RespostaDevedorSerpro>(r#"[{"numeroInscricao":"10 5 03"}]"#)
                .expect("lista");
        assert!(matches!(varios, RespostaDevedorSerpro::Lista(_)));

        let vazio = serde_json::from_str::<RespostaDevedorSerpro>("[]").expect("lista vazia");
        assert!(matches!(vazio, RespostaDevedorSerpro::Lista(_)));
    }

    #[test]
    fn prescricao_sim_nao() {
        let dto: InscricaoSerproDto =
            serde_json::from_str(r#"{"inscricao":"1","indicadorPrescricaoSV8":"SIM"}"#)
                .expect("dto");
        assert_eq!(dto.indicador_prescricao_sv8, Some("SIM".to_owned()));
        let dto: InscricaoSerproDto =
            serde_json::from_str(r#"{"inscricao":"1","indicadorPrescricaoSV8":"NAO"}"#)
                .expect("dto");
        assert_eq!(dto.indicador_prescricao_sv8, Some("NAO".to_owned()));
    }

    #[test]
    fn backoff_cresce_com_jitter_limitado() {
        let atraso = ClienteDividaAtivaHttp::atraso_backoff(2);
        assert!(atraso >= Duration::from_millis(1000) && atraso <= Duration::from_millis(1250));
    }
}
