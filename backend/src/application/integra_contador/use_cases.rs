use std::sync::Arc;

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::application::clientes::ClienteRepositorio;
use crate::application::certificados::CertificadoCriptografia;
use crate::application::integra_contador::ports::{
    AuditoriaIntegraContador, AutenticaProcuradorCliente, CatalogoEscritoriosIntegra,
    CertificadoA1Provider, ClienteIntegraContador, ContextoIntegraContador, ContratantePlataforma,
    EntregaIntegraContador, EventoAuditoriaIntegra, EventoChamadaIntegra,
    RepositorioIdempotencia, ReservaIdempotencia, ResultadoReservaIdempotencia,
    TokensSerproRepositorio, ValidadorProcuracao,
};
use crate::domain::integra_contador::assinatura::{AssinadorXml, TermoAutorizacao};
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, PedidoDados, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::integra_contador::servicos::{EstadoCatalogo, resolver_servico};
use crate::domain::serpro::ambiente::AmbienteSerpro;

#[derive(Debug, Clone)]
pub struct ComandoIntegraContador {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub documento_contribuinte: String,
    pub verbo: VerboIntegraContador,
    pub pedido_dados: PedidoDados,
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
    pub ambiente: AmbienteSerpro,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoIntegraContador {
    pub ambiente: AmbienteSerpro,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub estado: String,
    pub status: u16,
    pub mensagem: &'static str,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
    pub codigos_mensagem: Vec<String>,
}

pub struct ExecutarIntegraContadorUseCase {
    contexto: Arc<dyn CatalogoEscritoriosIntegra>,
    cliente: Arc<dyn ClienteIntegraContador>,
    idempotencia: Arc<dyn RepositorioIdempotencia>,
    auditoria: Arc<dyn AuditoriaIntegraContador>,
    procuracoes: Arc<dyn ValidadorProcuracao>,
    operator_header_enabled: bool,
}

impl ExecutarIntegraContadorUseCase {
    pub fn novo(
        contexto: Arc<dyn CatalogoEscritoriosIntegra>,
        cliente: Arc<dyn ClienteIntegraContador>,
        idempotencia: Arc<dyn RepositorioIdempotencia>,
        auditoria: Arc<dyn AuditoriaIntegraContador>,
        procuracoes: Arc<dyn ValidadorProcuracao>,
        operator_header_enabled: bool,
    ) -> Self {
        Self {
            contexto,
            cliente,
            idempotencia,
            auditoria,
            procuracoes,
            operator_header_enabled,
        }
    }

    pub async fn consultar_procuracoes(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: String,
        ambiente: AmbienteSerpro,
        request_id: String,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        let contexto = self
            .contexto
            .resolver_contexto(escritorio_id, usuario_id, &documento_contribuinte)
            .await?;
        let dados = serde_json::to_string(&serde_json::json!({
            "outorgante": contexto.contribuinte.numero,
            "tipoOutorgante": contexto.contribuinte.tipo.to_string(),
            "outorgado": contexto.autor_pedido_dados.numero,
            "tipoOutorgado": contexto.autor_pedido_dados.tipo.to_string(),
        }))
        .map_err(|_| ErroIntegraContador::PayloadInvalido)?;

        self.executar(ComandoIntegraContador {
            escritorio_id,
            usuario_id,
            documento_contribuinte,
            verbo: VerboIntegraContador::Consultar,
            pedido_dados: PedidoDados {
                id_sistema: "PROCURACOES".to_owned(),
                id_servico: "OBTERPROCURACAO41".to_owned(),
                versao_sistema: "1".to_owned(),
                dados,
            },
            confirmacao_explicita: false,
            chave_idempotencia: None,
            ambiente,
            request_id,
        })
        .await
    }

    pub async fn executar(
        &self,
        comando: ComandoIntegraContador,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        if comando.ambiente == AmbienteSerpro::Producao {
            return Err(ErroIntegraContador::ProducaoBloqueada);
        }

        if comando.verbo.exige_confirmacao() && !comando.confirmacao_explicita {
            return Err(ErroIntegraContador::ConfirmacaoNecessaria);
        }

        let servico = resolver_servico(
            &comando.pedido_dados.id_sistema,
            &comando.pedido_dados.id_servico,
            &comando.pedido_dados.versao_sistema,
            comando.verbo,
        )?;
        if servico.estado != EstadoCatalogo::Habilitado {
            return Err(ErroIntegraContador::ServicoNaoHabilitado);
        }

        let mut contexto = self
            .contexto
            .resolver_contexto(
                comando.escritorio_id,
                comando.usuario_id,
                &comando.documento_contribuinte,
            )
            .await?;
        let payload = validar_dados(&comando.pedido_dados.dados)?;
        validar_payload_procuracoes(&comando.pedido_dados, &payload, &contexto)?;

        if servico.regra_procuracao.se_aplica()
            && contexto.autor_pedido_dados != contexto.contribuinte
        {
            let codigos = servico.regra_procuracao.codigos_possiveis();
            let mut autorizada = false;
            for codigo in codigos {
                autorizada |= self
                    .procuracoes
                    .validar(&contexto, &codigo, comando.ambiente)
                    .await?;
            }
            if !autorizada {
                return Err(ErroIntegraContador::ProcuracaoNegada);
            }
        }

        if !self.operator_header_enabled {
            contexto.cpf_operador = None;
        } else if contexto.cpf_operador.as_ref().is_some_and(|cpf| {
            !crate::domain::clientes::CpfCnpj::novo(cpf).is_ok_and(|documento| {
                documento.tipo() == crate::domain::clientes::TipoDocumento::Cpf
            })
        }) {
            return Err(ErroIntegraContador::ConfiguracaoIncompleta);
        }

        let payload_hash = hash_payload(&comando.pedido_dados.dados);
        let reserva = if comando.verbo.exige_confirmacao() {
            Some(criar_reserva(&comando, &payload_hash)?)
        } else {
            None
        };
        if let Some(reserva) = &reserva {
            match self.idempotencia.reservar(reserva).await? {
                ResultadoReservaIdempotencia::Nova => {}
                ResultadoReservaIdempotencia::Concluida { .. } => {
                    return Err(ErroIntegraContador::ConflitoIdempotencia);
                }
            }
            self.auditoria
                .registrar_mutacao(&EventoAuditoriaIntegra {
                    escritorio_id: comando.escritorio_id,
                    usuario_id: comando.usuario_id,
                    acao: "INTEGRA_MUTACAO_SOLICITADA",
                    verbo: comando.verbo,
                    id_sistema: servico.id_sistema.to_owned(),
                    id_servico: servico.id_servico.to_owned(),
                    ambiente: comando.ambiente,
                    estado: "EM_ANDAMENTO".to_owned(),
                    payload_hash: payload_hash.clone(),
                })
                .await?;
        }

        let request = IntegraContadorRequest {
            contratante: contexto.contratante.clone(),
            autor_pedido_dados: contexto.autor_pedido_dados.clone(),
            contribuinte: contexto.contribuinte.clone(),
            pedido_dados: comando.pedido_dados.clone(),
        };
        let entrega = self
            .cliente
            .executar(
                comando.verbo,
                &request,
                comando.ambiente,
                contexto.cpf_operador.as_deref(),
                &comando.request_id,
            )
            .await;

        match entrega {
            Ok(entrega) => {
                self.finalizar_reserva(
                    reserva.as_ref(),
                    comando.verbo,
                    &entrega,
                    &payload_hash,
                    &comando,
                )
                .await?;
                self.auditoria
                    .registrar_chamada(&EventoChamadaIntegra {
                        escritorio_id: comando.escritorio_id,
                        usuario_id: comando.usuario_id,
                        verbo: comando.verbo,
                        id_sistema: servico.id_sistema.to_owned(),
                        id_servico: servico.id_servico.to_owned(),
                        ambiente: comando.ambiente,
                        estado: entrega.estado.clone(),
                        status_code: Some(i32::from(entrega.http_status)),
                        referencia_resposta: entrega.referencia_resposta.clone(),
                        request_id: comando.request_id,
                        bilhetavel: bilhetavel(comando.verbo, entrega.http_status),
                    })
                    .await?;
                Ok(mapear_resultado(
                    comando.ambiente,
                    comando.verbo,
                    servico.id_sistema,
                    servico.id_servico,
                    entrega,
                ))
            }
            Err(erro) => {
                let estado = if comando.verbo.exige_confirmacao() {
                    "INDETERMINADO"
                } else {
                    "FALHA_TRANSPORTE"
                };
                if let Some(reserva) = &reserva {
                    self.idempotencia
                        .atualizar(
                            reserva,
                            estado,
                            None,
                            Some(serde_json::json!({"estado": estado})),
                        )
                        .await?;
                    self.auditoria
                        .registrar_mutacao(&EventoAuditoriaIntegra {
                            escritorio_id: comando.escritorio_id,
                            usuario_id: comando.usuario_id,
                            acao: "INTEGRA_MUTACAO_RESULTADO",
                            verbo: comando.verbo,
                            id_sistema: servico.id_sistema.to_owned(),
                            id_servico: servico.id_servico.to_owned(),
                            ambiente: comando.ambiente,
                            estado: estado.to_owned(),
                            payload_hash,
                        })
                        .await?;
                }
                self.auditoria
                    .registrar_chamada(&EventoChamadaIntegra {
                        escritorio_id: comando.escritorio_id,
                        usuario_id: comando.usuario_id,
                        verbo: comando.verbo,
                        id_sistema: servico.id_sistema.to_owned(),
                        id_servico: servico.id_servico.to_owned(),
                        ambiente: comando.ambiente,
                        estado: estado.to_owned(),
                        status_code: None,
                        referencia_resposta: None,
                        request_id: comando.request_id,
                        bilhetavel: false,
                    })
                    .await?;
                Err(erro)
            }
        }
    }

    async fn finalizar_reserva(
        &self,
        reserva: Option<&ReservaIdempotencia>,
        verbo: VerboIntegraContador,
        entrega: &EntregaIntegraContador,
        payload_hash: &str,
        comando: &ComandoIntegraContador,
    ) -> Result<(), ErroIntegraContador> {
        let Some(reserva) = reserva else {
            return Ok(());
        };
        let estado = match entrega.http_status {
            504 => "INDETERMINADO",
            202 | 204 => "PROCESSANDO",
            200..=299 => "CONCLUIDO",
            _ => "REJEITADO",
        };
        self.idempotencia
            .atualizar(
                reserva,
                estado,
                Some(i32::from(entrega.http_status)),
                Some(serde_json::json!({
                    "estado": estado,
                    "referenciaResposta": entrega.referencia_resposta,
                })),
            )
            .await?;
        self.auditoria
            .registrar_mutacao(&EventoAuditoriaIntegra {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                acao: "INTEGRA_MUTACAO_RESULTADO",
                verbo,
                id_sistema: reserva.id_sistema.clone(),
                id_servico: reserva.id_servico.clone(),
                ambiente: comando.ambiente,
                estado: estado.to_owned(),
                payload_hash: payload_hash.to_owned(),
            })
            .await
    }
}

/// Comando para autenticar procurador via AUTENTICAPROCURADOR/ENVIOXMLASSINADO81.
#[derive(Debug, Clone)]
pub struct ComandoAutenticarProcurador {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Uuid,
    pub ambiente: AmbienteSerpro,
    pub request_id: String,
}

/// Resultado da autenticação do procurador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoAutenticarProcurador {
    pub jwt_token: String,
    pub expira_em: chrono::DateTime<chrono::Utc>,
}

/// Caso de uso para autenticar o procurador (escritório) junto ao SERPRO
/// via endpoint AUTENTICAPROCURADOR/ENVIOXMLASSINADO81.
/// 
/// Fluxo (modelo Software House):
/// 1. Busca o cliente (contribuinte) e o certificado A1 ativo do escritório
/// 2. Gera o Termo de Autorização com 3 atores: Contratante (SaaS, da
///    configuração), AutorPedidoDados (Escritório, via contexto) e
///    Contribuinte (cliente)
/// 3. Assina o XML com o certificado A1 do escritório
/// 4. Envia para SERPRO via `/Apoiar` (AUTENTICAPROCURADOR/ENVIOXMLASSINADO81)
/// 5. Salva o jwt_token retornado vinculado ao escritório e cliente
pub struct AutenticarProcuradorUseCase {
    contexto: Arc<dyn CatalogoEscritoriosIntegra>,
    cliente_repositorio: Arc<dyn ClienteRepositorio>,
    certificado_provider: Arc<dyn CertificadoA1Provider>,
    criptografia: Arc<dyn CertificadoCriptografia>,
    assinador: Arc<dyn AssinadorXml>,
    autentica_cliente: Arc<dyn AutenticaProcuradorCliente>,
    tokens_repositorio: Arc<dyn TokensSerproRepositorio>,
    auditoria: Arc<dyn AuditoriaIntegraContador>,
    /// Identidade da plataforma SaaS (Contratante). `None` quando a
    /// configuração não informa `SERPRO_CONTRATANTE_CNPJ`/`_NOME` — a
    /// operação falha fechada com `ConfiguracaoIncompleta`.
    contratante_plataforma: Option<ContratantePlataforma>,
}

impl AutenticarProcuradorUseCase {
    pub fn novo(
        contexto: Arc<dyn CatalogoEscritoriosIntegra>,
        cliente_repositorio: Arc<dyn ClienteRepositorio>,
        certificado_provider: Arc<dyn CertificadoA1Provider>,
        criptografia: Arc<dyn CertificadoCriptografia>,
        assinador: Arc<dyn AssinadorXml>,
        autentica_cliente: Arc<dyn AutenticaProcuradorCliente>,
        tokens_repositorio: Arc<dyn TokensSerproRepositorio>,
        auditoria: Arc<dyn AuditoriaIntegraContador>,
        contratante_plataforma: Option<ContratantePlataforma>,
    ) -> Self {
        Self {
            contexto,
            cliente_repositorio,
            certificado_provider,
            criptografia,
            assinador,
            autentica_cliente,
            tokens_repositorio,
            auditoria,
            contratante_plataforma,
        }
    }

    pub async fn executar(
        &self,
        comando: ComandoAutenticarProcurador,
    ) -> Result<ResultadoAutenticarProcurador, ErroIntegraContador> {
        // 0. Identidade da plataforma SaaS (Contratante) — fail-closed: sem
        //    `SERPRO_CONTRATANTE_CNPJ`/`SERPRO_CONTRATANTE_NOME` o Termo de
        //    Autorização não pode ser montado; nenhuma chamada ao SERPRO ocorre.
        let contratante_plataforma = self
            .contratante_plataforma
            .as_ref()
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 1. Busca o cliente para obter CPF/CNPJ e nome
        let cliente = self
            .cliente_repositorio
            .buscar_por_id(comando.escritorio_id, comando.cliente_id)
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 2. Resolve o contexto (CNPJ e nome do escritório autor e do contribuinte)
        let contexto = self
            .contexto
            .resolver_contexto(
                comando.escritorio_id,
                comando.usuario_id,
                cliente.documento.como_str(),
            )
            .await?;

        // 3. Busca o certificado A1 ativo do escritório
        let certificado = self
            .certificado_provider
            .obter_ativo(comando.escritorio_id)
            .await?
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 5. Constrói o Termo de Autorização com os 3 atores do modelo
        //    Software House: Contratante (a plataforma SaaS, da configuração),
        //    AutorPedidoDados (escritório/Procurador, do contexto) e
        //    Contribuinte (cliente).
        let termo = TermoAutorizacao {
            data_assinatura: Utc::now().date_naive(),
            vigencia: (Utc::now() + chrono::Duration::days(365)).date_naive(),
            atores: crate::domain::integra_contador::assinatura::AtoresTermoAutorizacao {
                contratante_numero: contratante_plataforma.numero.clone(),
                contratante_nome: contratante_plataforma.nome.clone(),
                autor_pedido_dados_numero: contexto.autor_pedido_dados.numero.clone(),
                autor_pedido_dados_nome: contexto.escritorio_nome.clone(),
                contribuinte_numero: contexto.contribuinte.numero.clone(),
                contribuinte_nome: cliente.nome_razao_social.clone(),
            },
        };

        // 6. Decifra o PFX A1 e a senha do escritório em memória; as
        //    cópias usadas na assinatura são zeroizadas pelo AssinadorXmlImpl
        //    ao final (o trait recebe &mut [u8] para isso).
        let mut pfx_der = self
            .criptografia
            .decifrar(&certificado.conteudo_cifrado)
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
        let mut senha = certificado
            .senha_cifrada
            .as_deref()
            .map(|cifrada| self.criptografia.decifrar(cifrada))
            .transpose()
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 7. Assina o XML (o AssinadorXmlImpl zeroiza pfx_der/senha ao fim)
        let xml_assinado = self
            .assinador
            .assinar(&termo, &mut pfx_der, &mut senha)
            .await?;

        // 8. Constrói a requisição para o endpoint `/Apoiar`
        //    (AUTENTICAPROCURADOR/ENVIOXMLASSINADO81). O envelope declara o
        //    contratante SaaS (modelo C), não o escritório.
        let request = IntegraContadorRequest {
            contratante: Identificacao::nova(&contratante_plataforma.numero)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?,
            autor_pedido_dados: contexto.autor_pedido_dados.clone(),
            contribuinte: contexto.contribuinte.clone(),
            pedido_dados: PedidoDados {
                id_sistema: "AUTENTICAPROCURADOR".to_owned(),
                id_servico: "ENVIOXMLASSINADO81".to_owned(),
                versao_sistema: "1.0".to_owned(),
                dados: serde_json::to_string(&serde_json::json!({
                    "xml": xml_assinado.xml_base64
                }))
                .map_err(|_| ErroIntegraContador::PayloadInvalido)?,
            },
        };

        // 9. Envia para SERPRO
        let resultado = self
            .autentica_cliente
            .enviar_xml_assinado(
                &request,
                &xml_assinado.xml_base64,
                comando.ambiente,
                &comando.request_id,
            )
            .await?;

        // 10. Salva o token JWT no banco
        self.tokens_repositorio
            .salvar_jwt_procurador(
                comando.escritorio_id,
                comando.cliente_id,
                comando.ambiente,
                &resultado.jwt_token,
                resultado.expira_em,
            )
            .await?;

        // 11. Auditoria
        self.auditoria
            .registrar_chamada(&EventoChamadaIntegra {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                verbo: VerboIntegraContador::Apoiar,
                id_sistema: "AUTENTICAPROCURADOR".to_owned(),
                id_servico: "ENVIOXMLASSINADO81".to_owned(),
                ambiente: comando.ambiente,
                estado: "CONCLUIDO".to_owned(),
                status_code: Some(200),
                referencia_resposta: None,
                request_id: comando.request_id,
                bilhetavel: false,
            })
            .await?;

        Ok(ResultadoAutenticarProcurador {
            jwt_token: resultado.jwt_token,
            expira_em: resultado.expira_em,
        })
    }
}

fn criar_reserva(
    comando: &ComandoIntegraContador,
    payload_hash: &str,
) -> Result<ReservaIdempotencia, ErroIntegraContador> {
    let chave = comando
        .chave_idempotencia
        .as_ref()
        .filter(|chave| {
            (16..=64).contains(&chave.len())
                && chave
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        })
        .ok_or(ErroIntegraContador::IdempotenciaNecessaria)?;

    Ok(ReservaIdempotencia {
        escritorio_id: comando.escritorio_id,
        chave: chave.clone(),
        verbo: comando.verbo,
        id_sistema: comando.pedido_dados.id_sistema.clone(),
        id_servico: comando.pedido_dados.id_servico.clone(),
        versao_sistema: comando.pedido_dados.versao_sistema.clone(),
        ambiente: comando.ambiente,
        payload_hash: payload_hash.to_owned(),
    })
}

fn validar_dados(dados: &str) -> Result<serde_json::Value, ErroIntegraContador> {
    if dados.len() > 262_144 {
        return Err(ErroIntegraContador::PayloadInvalido);
    }
    serde_json::from_str(dados).map_err(|_| ErroIntegraContador::PayloadInvalido)
}

fn validar_payload_procuracoes(
    pedido: &PedidoDados,
    payload: &serde_json::Value,
    contexto: &ContextoIntegraContador,
) -> Result<(), ErroIntegraContador> {
    if pedido.id_sistema != "PROCURACOES" || pedido.id_servico != "OBTERPROCURACAO41" {
        return Ok(());
    }
    let tipo_outorgante = contexto.contribuinte.tipo.to_string();
    let tipo_outorgado = contexto.autor_pedido_dados.tipo.to_string();
    let valido = payload
        .get("outorgante")
        .and_then(serde_json::Value::as_str)
        == Some(contexto.contribuinte.numero.as_str())
        && payload
            .get("tipoOutorgante")
            .and_then(serde_json::Value::as_str)
            == Some(tipo_outorgante.as_str())
        && payload.get("outorgado").and_then(serde_json::Value::as_str)
            == Some(contexto.autor_pedido_dados.numero.as_str())
        && payload
            .get("tipoOutorgado")
            .and_then(serde_json::Value::as_str)
            == Some(tipo_outorgado.as_str());
    if !valido {
        return Err(ErroIntegraContador::PayloadInvalido);
    }
    Ok(())
}

fn hash_payload(dados: &str) -> String {
    hex::encode(Sha256::digest(dados.as_bytes()))
}

fn bilhetavel(verbo: VerboIntegraContador, status: u16) -> bool {
    !matches!(
        (verbo, status),
        (VerboIntegraContador::Apoiar, _)
            | (VerboIntegraContador::Monitorar, _)
            | (_, 204 | 304 | 400 | 401 | 404 | 429 | 500 | 503 | 504)
    )
}

fn mapear_resultado(
    ambiente: AmbienteSerpro,
    verbo: VerboIntegraContador,
    id_sistema: &str,
    id_servico: &str,
    entrega: EntregaIntegraContador,
) -> ResultadoIntegraContador {
    let mensagem = match entrega.http_status {
        202 => "A operação está em processamento.",
        204 => "O processamento continua; consulte o estado antes de repetir.",
        304 => "O recurso não foi alterado.",
        504 => "Estado indeterminado; não reenvie a operação.",
        200..=299 => "Operação concluída.",
        _ => "A SERPRO rejeitou ou não conseguiu processar a operação.",
    };
    ResultadoIntegraContador {
        ambiente,
        verbo,
        id_sistema: id_sistema.to_owned(),
        id_servico: id_servico.to_owned(),
        estado: entrega.estado,
        status: entrega.http_status,
        mensagem,
        referencia_resposta: entrega.referencia_resposta,
        tempo_espera_segundos: entrega.tempo_espera_segundos,
        etag: entrega.etag,
        expires: entrega.expires,
        codigos_mensagem: entrega
            .resposta
            .unwrap_or_else(
                || crate::domain::integra_contador::IntegraContadorResponse {
                    status: 0,
                    mensagens: Vec::new(),
                    dados: None,
                },
            )
            .mensagens
            .into_iter()
            .filter(|mensagem| !mensagem.codigo.is_empty())
            .map(|mensagem| mensagem.codigo)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calcula_hash_estavel_do_payload() {
        assert_eq!(
            hash_payload(r#"{"x":1}"#),
            "5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22"
        );
    }

    #[test]
    fn rechaca_chave_de_idempotencia_curta() {
        let comando = ComandoIntegraContador {
            escritorio_id: Uuid::nil(),
            usuario_id: Uuid::nil(),
            documento_contribuinte: "52998224725".to_owned(),
            verbo: VerboIntegraContador::Declarar,
            pedido_dados: PedidoDados {
                id_sistema: "PGDASD".to_owned(),
                id_servico: "TRANSDECLARACAO11".to_owned(),
                versao_sistema: "1".to_owned(),
                dados: "{}".to_owned(),
            },
            confirmacao_explicita: true,
            chave_idempotencia: Some("short".to_owned()),
            ambiente: AmbienteSerpro::Trial,
            request_id: "request".to_owned(),
        };

        assert_eq!(
            criar_reserva(&comando, "hash"),
            Err(ErroIntegraContador::IdempotenciaNecessaria)
        );
    }

    #[test]
    fn status_sem_conteudo_nao_e_bilhetavel() {
        assert!(!bilhetavel(VerboIntegraContador::Consultar, 204));
        assert!(!bilhetavel(VerboIntegraContador::Apoiar, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 202));
    }

    #[test]
    fn regras_bilhetamento_cinco_verbos() {
        // Apoiar e Monitorar nunca são bilhetados
        assert!(!bilhetavel(VerboIntegraContador::Apoiar, 200));
        assert!(!bilhetavel(VerboIntegraContador::Monitorar, 200));

        // Consultar, Declarar, Emitir são bilhetados em 200 e 202
        assert!(bilhetavel(VerboIntegraContador::Consultar, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 200));
        assert!(bilhetavel(VerboIntegraContador::Emitir, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 202));
        assert!(bilhetavel(VerboIntegraContador::Emitir, 202));

        // Códigos não bilhetados em nenhum verbo
        for status in [204, 304, 400, 401, 404, 429, 500, 503, 504] {
            assert!(!bilhetavel(VerboIntegraContador::Consultar, status));
            assert!(!bilhetavel(VerboIntegraContador::Declarar, status));
            assert!(!bilhetavel(VerboIntegraContador::Emitir, status));
        }
    }
}
