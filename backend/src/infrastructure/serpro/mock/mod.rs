use crate::application::serpro::ports::{
    CertificadosSerproConsulta, ErroTokenSerpro, EscritorioSerproRepositorio,
    FornecedorTokenSerpro, InvalicadorCachesSerpro, RepositorioError,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

// --- Mock Serpro Service ---

/// Mock service for SERPRO gateway in TRIAL environment.
/// Retorna respostas determinísticas baseadas em cenários definidos.
/// Só deve ser ativado quando SERPRO_TRIAL_MOCK=true e ambiente = TRIAL.
pub struct MockSerproService;

impl MockSerproService {
    /// Verifica se o mock deve ser ativado
    pub fn esta_ativo(ambiente: AmbienteSerpro, mock_ativo: bool) -> bool {
        ambiente == AmbienteSerpro::Trial && mock_ativo
    }

    /// Obtém o cenário esperado para um serviço específico
    pub fn obter_cenario(servico: &str) -> Option<MockCenario> {
        match servico {
            "DIVIDA_ATIVA" => Some(MockCenario::DividaAtiva),
            "INTEGRA_CONTADOR" => Some(MockCenario::IntegraContador),
            "PROCURACAO" => Some(MockCenario::Procuracao),
            _ => None,
        }
    }

    /// Obtém todas as cenários disponíveis para a API de listagem
    pub fn obter_todos_cenarios() -> Vec<MockCenarioInfo> {
        vec![
            MockCenarioInfo {
                id: "cenario_1".into(),
                servico: "DIVIDA_ATIVA".into(),
                titulo: "Consulta de Dívida Ativa - Sem pendências".into(),
                descricao: "Devedor com inscrição regular, sem protestos ou penhoras".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "SUCESSO".into(),
            },
            MockCenarioInfo {
                id: "cenario_2".into(),
                servico: "DIVIDA_ATIVA".into(),
                titulo: "Consulta de Dívida Ativa - Com pendências".into(),
                descricao: "Devedor com protestos ou penhoras ativos".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "COM_PENDENCIAS".into(),
            },
            MockCenarioInfo {
                id: "cenario_3".into(),
                servico: "DIVIDA_ATIVA".into(),
                titulo: "Consulta de Dívida Ativa - Não encontrado".into(),
                descricao: "Inscrição/devedor não encontrado na base".into(),
                documento_exemplo: "98765432109".into(),
                resultado_esperado: "NAO_ENCONTRADO".into(),
            },
            MockCenarioInfo {
                id: "cenario_4".into(),
                servico: "INTEGRA_CONTADOR".into(),
                titulo: "Executar procuração - Sucesso".into(),
                descricao: "Procuração válida, execução concluída com sucesso".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "SUCESSO".into(),
            },
            MockCenarioInfo {
                id: "cenario_5".into(),
                servico: "INTEGRA_CONTADOR".into(),
                titulo: "Executar procuração - Acesso negado".into(),
                descricao: "Procuração negada ou sem validade".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "ACESSO_NEGADO".into(),
            },
            MockCenarioInfo {
                id: "cenario_6".into(),
                servico: "INTEGRA_CONTADOR".into(),
                titulo: "Executar procuração - Limite excedido".into(),
                descricao: "Limite de chamadas/mês excedido".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "LIMITE_EXCEDIDO".into(),
            },
            MockCenarioInfo {
                id: "cenario_7".into(),
                servico: "INTEGRA_CONTADOR".into(),
                titulo: "Executar procuração - Indisponível".into(),
                descricao: "Serviço temporariamente indisponível".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "INDISPONIVEL".into(),
            },
            MockCenarioInfo {
                id: "cenario_8".into(),
                servico: "PROCURACAO".into(),
                titulo: "OBTERPROCURACAO41 - Encontrado".into(),
                descricao: "Procuração encontrada com todos os dados válidos".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "SUCESSO".into(),
            },
            MockCenarioInfo {
                id: "cenario_9".into(),
                servico: "PROCURACAO".into(),
                titulo: "OBTERPROCURACAO41 - Negado (403)".into(),
                descricao: "Procuração negada ou sem certificado A1".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "ACESSO_NEGADO".into(),
            },
            MockCenarioInfo {
                id: "cenario_10".into(),
                servico: "PROCURACAO".into(),
                titulo: "ENVIOXMLASSINADO81 - Sucesso".into(),
                descricao: "XML assinado (simulado) enviado com sucesso".into(),
                documento_exemplo: "12345678901".into(),
                resultado_esperado: "SUCESSO".into(),
            },
        ]
    }
}

// --- Mock Cenários ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockCenario {
    DividaAtiva,
    IntegraContador,
    Procuracao,
}

#[derive(Debug, Clone)]
pub struct MockCenarioInfo {
    pub id: String,
    pub servico: String,
    pub titulo: String,
    pub descricao: String,
    pub documento_exemplo: String,
    pub resultado_esperado: String,
}

// --- Mock Implementations for Ports ---
// (ainda não plugarão o trial — ficam disponíveis para as próximas etapas)

/// Mock implementation of FornecedorTokenSerpro
/// Retorna token mock quando ativo, senão retorna erro
#[allow(dead_code)]
struct MockTokenFornecedor;

impl FornecedorTokenSerpro for MockTokenFornecedor {
    fn obter_token(
        &self,
        _escritorio_id: Uuid,
        _produto: ProdutoSerpro,
        _ambiente: AmbienteSerpro,
    ) -> impl std::future::Future<Output = Result<String, ErroTokenSerpro>> + Send {
        let fut = async move {
            if MockSerproService::esta_ativo(_ambiente, true) {
                Ok("mock-token-12345".to_string())
            } else {
                Err(ErroTokenSerpro::Indisponivel)
            }
        };
        fut
    }
}

/// Mock implementation of CertificadosSerproConsulta
/// Retorna certificado mock ou None
#[allow(dead_code)]
struct MockCertificados;

impl CertificadosSerproConsulta for MockCertificados {
    fn buscar_certificado_em_validade(
        &self,
        _escritorio_id: Uuid,
        _agora: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<crate::application::serpro::ports::CertificadoAtivoResumo>, RepositorioError>
    {
        // No mock, não retornamos certificado real
        // Apenas indicamos que não há certificado A1 no trial
        Ok(None)
    }
}

/// Mock implementation of InvalicadorCachesSerpro
/// Registra que caches foram invalidados (no mock, apenas registra)
#[allow(dead_code)]
struct MockInvalidador;

impl InvalicadorCachesSerpro for MockInvalidador {
    fn invalidar_todos_do_tenant(&self, _escritorio_id: Uuid) -> Result<(), RepositorioError> {
        // No mock, apenas registra que invalidou
        Ok(())
    }
}

/// Mock implementation of EscritorioSerproRepositorio
/// Versão mock que retorna modo trial simulado
#[allow(dead_code)]
struct MockEscritorioRepositorio;

impl EscritorioSerproRepositorio for MockEscritorioRepositorio {
    fn buscar_modo_trial(
        &self,
        _escritorio_id: Uuid,
    ) -> Result<Option<(bool, i64, chrono::DateTime<chrono::Utc>)>, RepositorioError> {
        Ok(Some((true, 1, Utc::now())))
    }

    fn atualizar_modo_trial(
        &self,
        _escritorio_id: Uuid,
        _modo_trial: bool,
        _versao_esperada: i64,
    ) -> Result<(i64, chrono::DateTime<chrono::Utc>), RepositorioError> {
        Ok((1, Utc::now()))
    }

    fn buscar_credencial(
        &self,
        _escritorio_id: Uuid,
        _produto: ProdutoSerpro,
        _ambiente: AmbienteSerpro,
    ) -> Result<Option<crate::application::serpro::ports::CredencialArmazenada>, RepositorioError>
    {
        Ok(None)
    }
}

// --- Mock HTTP Handlers ---

/// Handler para GET /api/v1/serpro/mock/cenarios
/// Retorna a lista de cenários de teste disponíveis
pub async fn handler_mock_cenarios() -> Result<Value, axum::Error> {
    let cenarios = MockSerproService::obter_todos_cenarios();
    let json = serde_json::json!({
        "ativo": true,
        "cenarios": cenarios.iter().map(|c| serde_json::json!({
            "id": c.id,
            "servico": c.servico,
            "titulo": c.titulo,
            "descricao": c.descricao,
            "documentoExemplo": c.documento_exemplo,
            "resultadoEsperado": c.resultado_esperado,
        })).collect::<Vec<_>>()
    });
    Ok(json)
}

/// Handler para chamadas mockadas do SERPRO
/// Redireciona chamadas para respostas determinísticas quando em trial mock mode
pub async fn handler_mock_redirect(_caminho: String) -> Result<Value, axum::Error> {
    // Parse the path to determine the service and scenario
    // This is a simplified mock redirection
    // Na implementação real, isso seria mais sofisticado
    // e integraria com o mesmo ponto de cliente da medição (G.1)

    // Por enquanto, retorna um erro indicando que a rota mock
    // precisa de implementação mais detalhada
    Ok(serde_json::json!({
        "erro": "Rota mock requer implementação específica por serviço"
    }))
}
