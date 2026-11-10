use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::Value;

use crate::application::serpro::ObterConfiguracaoError;
use crate::infrastructure::serpro::mock::MockSerproService;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::serpro::ambiente::AmbienteResolvido;
use crate::presentation::serpro::dto::AmbienteResponse;
use crate::presentation::serpro::state::SerproState;

/// GET /api/v1/serpro/ambiente — leitura do ambiente efetivo do
/// escritório do token do portal. O ambiente foi resolvido **uma vez** pelo
/// middleware `resolver_ambiente` e vive na extensão `AmbienteResolvido`
/// (snapshot imutável da requisição; D8 de DECISOES-PENDENTES).
pub async fn obter_ambiente(
    State(_estado): State<SerproState>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
) -> Result<Response, ApiError> {
    let _ = contexto.escritorio_id; // o snapshot já refletiu o token do portal

    Ok((
        StatusCode::OK,
        axum::Json(AmbienteResponse {
            ambiente_efetivo: ambiente.ambiente,
            atualizado_em: ambiente.atualizado_em,
        }),
    )
        .into_response())
}

/// Mapeamento compartilhado de `ObterConfiguracaoError` para erros HTTP —
/// usado pelo middleware de snapshot e por quem chama o use case diretamente.
pub(crate) fn map_obter_para_api(erro: ObterConfiguracaoError) -> ApiError {
    match erro {
        ObterConfiguracaoError::EscritorioNaoEncontrado => ApiError::nao_encontrado("escritório"),
        ObterConfiguracaoError::Repositorio(repositorio) => map_repositorio(repositorio),
    }
}

pub(crate) fn map_repositorio(
    erro: crate::application::serpro::ports::RepositorioError,
) -> ApiError {
    use crate::application::serpro::ports::RepositorioError;
    match erro {
        RepositorioError::Indisponivel => ApiError::indisponivel(),
        RepositorioError::NaoEncontrado => ApiError::nao_encontrado("escritório"),
        _ => ApiError::erro_interno(),
    }
}

/// Resultado padrão do mock SERPRO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MockResultado {
    pub sucesso: bool,
    pub codigo: String,
    pub mensagem: String,
    pub dados: Option<Value>,
}

/// Lista de cenários de mock disponíveis
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MockCenario {
    pub id: String,
    pub servico: String,
    pub titulo: String,
    pub descricao: String,
    pub documento_exemplo: String,
    pub resultado_esperado: String,
}

/// GET /api/v1/serpro/mock/cenarios
/// Retorna a lista de cenários de teste disponíveis para o cliente
/// Apenas ativo=true quando ambiente for TRIAL e mock ativo
pub async fn handler_mock_cenarios(
    State(_state): State<SerproState>,
) -> Result<Json<MockResultadoListagem>, ApiError> {
    let ativo = MockSerproService::esta_ativo(
        crate::domain::serpro::ambiente::AmbienteSerpro::Trial,
        // Na prática, isso viria da configuração do ambiente
        true,
    );

    let cenarios: Vec<MockCenario> = if ativo {
        MockSerproService::obter_todos_cenarios()
            .into_iter()
            .map(|cenario| MockCenario {
                id: cenario.id,
                servico: cenario.servico,
                titulo: cenario.titulo,
                descricao: cenario.descricao,
                documento_exemplo: cenario.documento_exemplo,
                resultado_esperado: cenario.resultado_esperado,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(Json(MockResultadoListagem { ativo, cenarios }))
}

/// Resposta para a API de cenários de mock
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MockResultadoListagem {
    pub ativo: bool,
    pub cenarios: Vec<MockCenario>,
}

/// Mock response for Dívida Ativa consultations
fn mock_divida_ativa(_servico: &str, documento: &str) -> MockResultado {
    // Extrair o último dígito do documento para determinar o cenário
    let ultimo_digito: u32 = documento
        .chars()
        .rfind(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap_or(0))
        .unwrap_or(0);

    match ultimo_digito {
        0..=2 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "Consulta realizada com sucesso".into(),
            dados: Some(serde_json::json!({
                "devedor": documento,
                "situacao": "REGULAR",
                "valor": "R$ 1.500,00",
            })),
        },
        3 | 4 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "Consulta realizada com sucesso".into(),
            dados: Some(serde_json::json!({
                "devedor": documento,
                "situacao": "COM_PENDENCIAS",
                "pendencias": ["CERTIDÃO NEGATIVA"],
                "valor": "R$ 3.200,00",
            })),
        },
        5 | 6 => MockResultado {
            sucesso: false,
            codigo: "NAO_ENCONTRADO".into(),
            mensagem: "Inscrição não encontrada na base de dados".into(),
            dados: None,
        },
        7 | 8 => MockResultado {
            sucesso: false,
            codigo: "ACESSO_NEGADO".into(),
            mensagem: "Procuração negada ou inválida".into(),
            dados: None,
        },
        9 => MockResultado {
            sucesso: false,
            codigo: "LIMITE_EXCEDIDO".into(),
            mensagem: "Limite de consultas mensais excedido".into(),
            dados: None,
        },
        _ => MockResultado {
            sucesso: false,
            codigo: "RESPOSTA_INVALIDA".into(),
            mensagem: "Resposta do serviço inválida".into(),
            dados: None,
        },
    }
}

/// Mock response for Integra Contador consultations
fn mock_integra_contador(_servico: &str, documento: &str) -> MockResultado {
    // Usar o último dígito do documento para determinar cenário
    let ultimo_digito: u32 = documento
        .chars()
        .rfind(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap_or(0))
        .unwrap_or(0);

    match ultimo_digito {
        0 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "Procuração executada com sucesso".into(),
            dados: Some(serde_json::json!({
                "status": "CONCLUIDO",
                "descricao": "Procedimento concluído sem observações",
            })),
        },
        1 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "Consulta realizada com sucesso".into(),
            dados: Some(serde_json::json!({
                "resultado": "EMPREENDIMENTO_ENCONTRADO",
                "descricao": "Dados do empreendimento encontrados com sucesso",
            })),
        },
        2 => MockResultado {
            sucesso: false,
            codigo: "NAO_ENCONTRADO".into(),
            mensagem: "Procuração ou inscrição não encontrada".into(),
            dados: None,
        },
        3 => MockResultado {
            sucesso: false,
            codigo: "ACESSO_NEGADO".into(),
            mensagem: "Procuração negada - certificado A1 inválido ou inexistente".into(),
            dados: None,
        },
        4 => MockResultado {
            sucesso: false,
            codigo: "LIMITE_EXCEDIDO".into(),
            mensagem: "Limite de chamadas/mês excedido".into(),
            dados: None,
        },
        5 => MockResultado {
            sucesso: false,
            codigo: "INDISPONIVEL".into(),
            mensagem: "Serviço temporariamente indisponível".into(),
            dados: None,
        },
        6 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "ENVIOXMLASSINADO81 concluído".into(),
            dados: Some(serde_json::json!({
                "xml_id": "XML-2024-00123",
                "status": "ASSINADO",
                "descricao": "XML assinado (simulado) com sucesso",
            })),
        },
        7 => MockResultado {
            sucesso: true,
            codigo: "SUCESSO".into(),
            mensagem: "OBTERPROCURACAO41 concluído".into(),
            dados: Some(serde_json::json!({
                "procuracao": "ENCONTRADA",
                "data_validade": "2025-12-31",
                "descricao": "Procuração encontrada e validada com sucesso",
            })),
        },
        _ => MockResultado {
            sucesso: false,
            codigo: "RESPOSTA_INVALIDA".into(),
            mensagem: "Resposta do serviço inválida".into(),
            dados: None,
        },
    }
}

/// Mock endpoint that returns deterministic responses based on scenario
/// This is intended for TRIAL environment only
pub async fn handler_mock_servico(
    Path((servico, documento)): Path<(String, String)>,
) -> Result<Json<MockResultado>, ApiError> {
    let resultado = match servico.as_str() {
        "DIVIDA_ATIVA" => mock_divida_ativa(&servico, &documento),
        "INTEGRA_CONTADOR" => mock_integra_contador(&servico, &documento),
        _ => MockResultado {
            sucesso: false,
            codigo: "SERVIÇO_NÃO_SUPORTADO".into(),
            mensagem: format!("Serviço '{}' não suportado em modo mock", servico),
            dados: None,
        },
    };

    Ok(Json(resultado))
}

/// Redireciona chamadas SERPRO para mock quando em trial mode
/// Esta função pode ser usada como middleware ou wrapper ao redor
/// das chamadas reais do cliente SERPRO
pub async fn handler_mock_wrapper(
    _metodo: String,
    caminho: String,
) -> Result<Json<MockResultado>, ApiError> {
    // Verifica se deve usar mock (trial + mock ativo)
    // Na implementação real, isso verificaria o ambiente atual
    let usar_mock = true; // Placeholder - seria verificado a configuração

    if usar_mock {
        // Parse the service and document from the path
        // Exemplo: /divida-ativa/12345678901 ou /integra-contador/12345678901
        let partes: Vec<&str> = caminho.split('/').collect();
        let servico = if partes.len() >= 2 {
            partes[0].to_string()
        } else {
            String::new()
        };
        let documento = if partes.len() >= 3 {
            partes[2].to_string()
        } else {
            String::new()
        };

        handler_mock_servico(Path((servico, documento))).await
    } else {
        // Em produção, deveria passar para o handler real
        // Por enquanto, retorna erro indicando que o mock não está ativo
        Ok(Json(MockResultado {
            sucesso: false,
            codigo: "MOCK_DESATIVADO".into(),
            mensagem: "Mock SERPRO desativado - ambiente de produção".into(),
            dados: None,
        }))
    }
}
