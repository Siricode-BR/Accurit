//! Simulador determinístico do SERPRO para o ambiente TRIAL.
//!
//! Ativado por `SERPRO_TRIAL_MOCK` **apenas** quando o ambiente efetivo do
//! escritório é TRIAL. Em PRODUÇÃO o wrapper sempre delega ao cliente real.
//! Os cenários são derivados do último dígito do documento, conforme a
//! tabela documentada em `docs/Serpro/mocks-trial.md`.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use secrecy::SecretString;

use crate::application::integra_contador::ports::{
    AutenticaProcuradorCliente, AutenticarProcuradorResultado, ClienteIntegraContador,
    EntregaIntegraContador,
};
use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::devedor::{
    EstadoConsulta, FonteConsulta, InscricaoDetalhe, InscricaoResumo, ResultadoConsultaDevedor,
};
use crate::domain::divida_ativa::erros::ErroDividaAtiva;
use crate::domain::divida_ativa::ports::ClienteDividaAtiva;
use crate::domain::integra_contador::envelope::{
    IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::EndpointSerpro;

#[derive(Debug, Clone, Copy)]
pub enum ServicoMock {
    DividaAtiva,
    IntegraContador,
    Procuracao,
}

impl ServicoMock {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::DividaAtiva => "DIVIDA_ATIVA",
            Self::IntegraContador => "INTEGRA_CONTADOR",
            Self::Procuracao => "PROCURACAO",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CenarioMock {
    pub id: &'static str,
    pub servico: ServicoMock,
    pub titulo: &'static str,
    pub descricao: &'static str,
    pub documento_exemplo: &'static str,
    pub resultado_esperado: &'static str,
}

/// Documentos de exemplo válidos (CPF/CNPJ) por cenário.
pub fn cenarios() -> Vec<CenarioMock> {
    vec![
        CenarioMock {
            id: "divida-com-pendencias",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — com pendências",
            descricao: "Devedor com inscrições ativas; documento terminado em 0 ou 1.",
            documento_exemplo: "52998224725",
            resultado_esperado: "COM_PENDENCIAS",
        },
        CenarioMock {
            id: "divida-sem-pendencias",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — sem pendências",
            descricao: "Devedor regular, sem inscrições; documento terminado em 2 ou 3.",
            documento_exemplo: "11144477735",
            resultado_esperado: "SEM_PENDENCIAS",
        },
        CenarioMock {
            id: "divida-nao-encontrado",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — não encontrado",
            descricao: "Documento terminado em 4.",
            documento_exemplo: "12345678909",
            resultado_esperado: "NAO_ENCONTRADO",
        },
        CenarioMock {
            id: "divida-limite",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — limite excedido",
            descricao: "Documento terminado em 6 retorna HTTP 429.",
            documento_exemplo: "39053344705",
            resultado_esperado: "LIMITE_EXCEDIDO",
        },
        CenarioMock {
            id: "divida-indisponivel",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — indisponível",
            descricao: "Documento terminado em 7 retorna HTTP 503.",
            documento_exemplo: "86288366757",
            resultado_esperado: "INDISPONIVEL",
        },
        CenarioMock {
            id: "divida-indeterminado",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — indeterminado",
            descricao: "Documento terminado em 8 retorna HTTP 504 (estado indeterminado).",
            documento_exemplo: "95835384068",
            resultado_esperado: "INDETERMINADO",
        },
        CenarioMock {
            id: "divida-resposta-invalida",
            servico: ServicoMock::DividaAtiva,
            titulo: "Dívida Ativa — resposta inválida",
            descricao: "Documento terminado em 9 retorna resposta fora do contrato.",
            documento_exemplo: "15350946056",
            resultado_esperado: "RESPOSTA_INVALIDA",
        },
        CenarioMock {
            id: "integra-sucesso",
            servico: ServicoMock::IntegraContador,
            titulo: "Integra Contador — execução concluída",
            descricao: "Documento terminado em 0 ou 1.",
            documento_exemplo: "52998224725",
            resultado_esperado: "SUCESSO",
        },
        CenarioMock {
            id: "integra-acesso-negado",
            servico: ServicoMock::Procuracao,
            titulo: "Procuração — acesso negado",
            descricao: "Documento terminado em 4: OBTERPROCURACAO41/autenticar retorna 403.",
            documento_exemplo: "12345678909",
            resultado_esperado: "ACESSO_NEGADO",
        },
        CenarioMock {
            id: "integra-limite",
            servico: ServicoMock::IntegraContador,
            titulo: "Integra Contador — limite excedido",
            descricao: "Documento terminado em 6 retorna HTTP 429.",
            documento_exemplo: "39053344705",
            resultado_esperado: "LIMITE_EXCEDIDO",
        },
        CenarioMock {
            id: "integra-indisponivel",
            servico: ServicoMock::IntegraContador,
            titulo: "Integra Contador — indisponível",
            descricao: "Documento terminado em 7 retorna HTTP 503.",
            documento_exemplo: "86288366757",
            resultado_esperado: "INDISPONIVEL",
        },
        CenarioMock {
            id: "integra-indeterminado",
            servico: ServicoMock::IntegraContador,
            titulo: "Integra Contador — indeterminado",
            descricao: "Documento terminado em 8 retorna HTTP 504.",
            documento_exemplo: "95835384068",
            resultado_esperado: "INDETERMINADO",
        },
    ]
}

pub fn mock_ativo(ambiente: AmbienteSerpro, configurado: bool) -> bool {
    configurado && ambiente == AmbienteSerpro::Trial
}

fn ultimo_digito(documento: &str) -> u32 {
    documento
        .chars()
        .rfind(|c| c.is_ascii_digit())
        .and_then(|c| c.to_digit(10))
        .unwrap_or(0)
}

fn mascarar(documento: &str) -> String {
    let digitos: String = documento.chars().filter(char::is_ascii_digit).collect();
    if digitos.len() <= 4 {
        return "***".to_owned();
    }
    format!("***{}", &digitos[digitos.len() - 4..])
}

fn valor(centavos: i64) -> Decimal {
    Decimal::new(centavos, 2)
}

// ===== Dívida Ativa =====

pub struct ClienteDividaAtivaMock {
    interno: Arc<dyn ClienteDividaAtiva>,
    ativo: bool,
}

impl ClienteDividaAtivaMock {
    pub fn novo(interno: Arc<dyn ClienteDividaAtiva>, ativo: bool) -> Self {
        Self { interno, ativo }
    }
}

#[async_trait]
impl ClienteDividaAtiva for ClienteDividaAtivaMock {
    async fn consultar_devedor(
        &self,
        endpoint: &EndpointSerpro,
        chave: &CpfCnpj,
        cpf_operador: Option<String>,
        request_id: &str,
    ) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva> {
        if !mock_ativo(endpoint.ambiente, self.ativo) {
            return self
                .interno
                .consultar_devedor(endpoint, chave, cpf_operador, request_id)
                .await;
        }

        let documento = chave.como_str();
        match ultimo_digito(documento) {
            4 => Err(ErroDividaAtiva::NaoEncontrado),
            5 => Err(ErroDividaAtiva::ProxyNegado),
            6 => Err(ErroDividaAtiva::LimiteSerproAtingido),
            7 => Err(ErroDividaAtiva::ServicoIndisponivel),
            8 => Err(ErroDividaAtiva::Indeterminado),
            9 => Err(ErroDividaAtiva::RespostaInvalida),
            digito => {
                let com_pendencias = digito <= 1;
                let inscricoes = if com_pendencias {
                    vec![
                        InscricaoResumo {
                            numero_inscricao: "9061700096419".to_owned(),
                            numero_processo: Some("10880.000000/2024-10".to_owned()),
                            situacao_inscricao: Some("ATIVA".to_owned()),
                            situacao_descricao: "Inscrição ativa em cobrança".to_owned(),
                            nome_devedor: Some("Contribuinte Simulado Ltda".to_owned()),
                            tipo_devedor: "PJ".to_owned(),
                            valor_total_consolidado: valor(150_000),
                            cpf_cnpj: Some(documento.to_owned()),
                            nome_unidade: Some("PFN Simulada".to_owned()),
                        },
                        InscricaoResumo {
                            numero_inscricao: "9061700096427".to_owned(),
                            numero_processo: None,
                            situacao_inscricao: Some("ATIVA".to_owned()),
                            situacao_descricao: "Inscrição ativa em cobrança".to_owned(),
                            nome_devedor: Some("Contribuinte Simulado Ltda".to_owned()),
                            tipo_devedor: "PJ".to_owned(),
                            valor_total_consolidado: valor(32_000),
                            cpf_cnpj: Some(documento.to_owned()),
                            nome_unidade: Some("PFN Simulada".to_owned()),
                        },
                    ]
                } else {
                    Vec::new()
                };

                Ok(ResultadoConsultaDevedor {
                    ambiente: AmbienteSerpro::Trial,
                    estado: if com_pendencias {
                        EstadoConsulta::ComPendencias
                    } else {
                        EstadoConsulta::SemPendencias
                    },
                    fonte: FonteConsulta::CpfCnpj,
                    chave_mascarada: mascarar(documento),
                    horario_consulta: Utc::now(),
                    tentativas: 1,
                    inscricoes,
                })
            }
        }
    }

    async fn consultar_inscricao(
        &self,
        endpoint: &EndpointSerpro,
        numero_inscricao: &str,
        cpf_operador: Option<String>,
        request_id: &str,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        if !mock_ativo(endpoint.ambiente, self.ativo) {
            return self
                .interno
                .consultar_inscricao(endpoint, numero_inscricao, cpf_operador, request_id)
                .await;
        }

        match ultimo_digito(numero_inscricao) {
            4 => Err(ErroDividaAtiva::NaoEncontrado),
            6 => Err(ErroDividaAtiva::LimiteSerproAtingido),
            7 => Err(ErroDividaAtiva::ServicoIndisponivel),
            8 => Err(ErroDividaAtiva::Indeterminado),
            9 => Err(ErroDividaAtiva::RespostaInvalida),
            digito => {
                let com_pendencias = digito <= 1;
                Ok(InscricaoDetalhe {
                    ambiente: AmbienteSerpro::Trial,
                    estado: if com_pendencias {
                        EstadoConsulta::ComPendencias
                    } else {
                        EstadoConsulta::SemPendencias
                    },
                    fonte: FonteConsulta::CpfCnpj,
                    chave_mascarada: mascarar(numero_inscricao),
                    horario_consulta: Utc::now(),
                    tentativas: 1,
                    numero_inscricao: numero_inscricao.to_owned(),
                    processo_administrativo: Some("10880.000000/2024-10".to_owned()),
                    situacao: if com_pendencias {
                        "ATIVA".to_owned()
                    } else {
                        "REGULAR".to_owned()
                    },
                    data_inscricao: Some("2024-01-15".to_owned()),
                    orgao_origem: Some("Receita Federal do Brasil".to_owned()),
                    codigo_natureza_receita: Some("1210".to_owned()),
                    nome_receita: Some("Multa por atraso".to_owned()),
                    valor_total_inscrito: Some(valor(150_000)),
                    valor_total_consolidado: Some(valor(150_000)),
                    valor_remanescente: Some(valor(120_000)),
                    indicador_prescricao: Some(false),
                    nome_unidade: Some("PFN Simulada".to_owned()),
                })
            }
        }
    }
}

// ===== Integra Contador =====

pub struct ClienteIntegraContadorMock {
    interno: Arc<dyn ClienteIntegraContador>,
    ativo: bool,
}

impl ClienteIntegraContadorMock {
    pub fn novo(interno: Arc<dyn ClienteIntegraContador>, ativo: bool) -> Self {
        Self { interno, ativo }
    }
}

#[async_trait]
impl ClienteIntegraContador for ClienteIntegraContadorMock {
    async fn executar(
        &self,
        verbo: VerboIntegraContador,
        request: &IntegraContadorRequest,
        ambiente: AmbienteSerpro,
        cpf_operador: Option<&str>,
        request_id: &str,
    ) -> Result<EntregaIntegraContador, ErroIntegraContador> {
        if !mock_ativo(ambiente, self.ativo) {
            return self
                .interno
                .executar(verbo, request, ambiente, cpf_operador, request_id)
                .await;
        }

        match ultimo_digito(&request.contribuinte.numero) {
            4 => Err(ErroIntegraContador::AcessoNegado),
            6 => Err(ErroIntegraContador::LimiteExcedido),
            7 => Err(ErroIntegraContador::Indisponivel),
            8 => Err(ErroIntegraContador::Indeterminado),
            9 => Err(ErroIntegraContador::RespostaInvalida),
            _ => Ok(EntregaIntegraContador {
                http_status: 200,
                estado: "CONCLUIDO".to_owned(),
                resposta: Some(IntegraContadorResponse {
                    status: 200,
                    mensagens: Vec::new(),
                    dados: Some(
                        serde_json::json!({
                            "origem": "MOCK",
                            "idServico": request.pedido_dados.id_servico,
                            "verbo": verbo.como_str(),
                            "procuracao": "ENCONTRADA",
                            "validade": "2026-12-31",
                        })
                        .to_string(),
                    ),
                }),
                referencia_resposta: Some("MOCK-INTEGRA-001".to_owned()),
                tempo_espera_segundos: None,
                etag: None,
                expires: None,
            }),
        }
    }
}

pub struct AutenticaProcuradorMock {
    interno: Arc<dyn AutenticaProcuradorCliente>,
    ativo: bool,
}

impl AutenticaProcuradorMock {
    pub fn novo(interno: Arc<dyn AutenticaProcuradorCliente>, ativo: bool) -> Self {
        Self { interno, ativo }
    }
}

#[async_trait]
impl AutenticaProcuradorCliente for AutenticaProcuradorMock {
    async fn enviar_xml_assinado(
        &self,
        request: &IntegraContadorRequest,
        xml_base64: &str,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<AutenticarProcuradorResultado, ErroIntegraContador> {
        if !mock_ativo(ambiente, self.ativo) {
            return self
                .interno
                .enviar_xml_assinado(request, xml_base64, ambiente, request_id)
                .await;
        }

        match ultimo_digito(&request.contribuinte.numero) {
            4 => Err(ErroIntegraContador::AcessoNegado),
            6 => Err(ErroIntegraContador::LimiteExcedido),
            7 => Err(ErroIntegraContador::Indisponivel),
            _ => {
                let amanha = Utc::now().date_naive() + chrono::Days::new(1);
                let expira_em = Utc.from_utc_datetime(
                    &amanha
                        .and_hms_opt(0, 0, 0)
                        .ok_or(ErroIntegraContador::RespostaInvalida)?,
                );
                Ok(AutenticarProcuradorResultado {
                    jwt_token: SecretString::from("mock-procurador-jwt"),
                    expira_em,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_so_existe_em_trial() {
        assert!(mock_ativo(AmbienteSerpro::Trial, true));
        assert!(!mock_ativo(AmbienteSerpro::Trial, false));
        assert!(
            !mock_ativo(AmbienteSerpro::Producao, true),
            "produção nunca pode usar o simulador"
        );
    }

    #[test]
    fn cenarios_cobrem_todos_os_servicos_e_documentos_sao_digitos() {
        let cenarios = cenarios();
        assert!(
            cenarios
                .iter()
                .any(|c| matches!(c.servico, ServicoMock::DividaAtiva))
        );
        assert!(
            cenarios
                .iter()
                .any(|c| matches!(c.servico, ServicoMock::IntegraContador))
        );
        assert!(
            cenarios
                .iter()
                .any(|c| matches!(c.servico, ServicoMock::Procuracao))
        );
        for cenario in cenarios {
            assert!(
                cenario.documento_exemplo.len() >= 11
                    && cenario
                        .documento_exemplo
                        .bytes()
                        .all(|b| b.is_ascii_digit()),
                "documento de exemplo deve conter apenas dígitos"
            );
        }
    }
}
