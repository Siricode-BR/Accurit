use serde::{Deserialize, Serialize};

use crate::application::integra_contador::EstadoProcuracaoCliente;
use crate::application::integra_contador::EstadoProcuracaoServico;
use crate::application::integra_contador::ResultadoIntegraContador;
use crate::application::integra_contador::ResultadoValidarProcuracao;
use crate::application::integra_contador::ports::LinhaCatalogoServico;
use crate::application::integra_contador::ports::RegistroProcuracao;
use crate::domain::integra_contador::envelope::{PedidoDados, VerboIntegraContador};
use crate::domain::integra_contador::servicos::{EstadoCatalogo, ServicoIntegraContador};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::presentation::serpro::ambiente::AmbienteResolvido;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoExecucao {
    pub documento_contribuinte: String,
    pub verbo: VerboIntegraContador,
    pub pedido_dados: PedidoDados,
    #[serde(default)]
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoConsultaProcuracoes {
    pub documento_contribuinte: String,
}

// ===== FASE A — procurações persistentes e catálogo configurável =====

/// `POST /procuracoes/validar` (ou `.../revalidar`, que força a consulta).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoValidarProcuracao {
    pub cliente_id: uuid::Uuid,
}

/// `GET /procuracoes?clienteId=…`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsultaProcuracoesQuery {
    pub cliente_id: uuid::Uuid,
}

/// Registro persistido de procuração (sem o `payload_redigido` — dado
/// tolerado do SERPRO, auditável internamente, não há por que expor).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcuracaoRegistroResponse {
    pub id: uuid::Uuid,
    pub cliente_id: Option<uuid::Uuid>,
    pub ambiente: AmbienteSerpro,
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    pub codigo_servico: Option<String>,
    pub status: String,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub verificado_em: chrono::DateTime<chrono::Utc>,
    pub origem: String,
}

impl From<ResultadoValidarProcuracao> for ProcuracaoRegistroResponse {
    fn from(resultado: ResultadoValidarProcuracao) -> Self {
        Self::de_registro(&resultado.registro)
    }
}

impl ProcuracaoRegistroResponse {
    pub fn de_registro(registro: &RegistroProcuracao) -> Self {
        Self {
            id: registro.id,
            cliente_id: registro.cliente_id,
            ambiente: registro.ambiente,
            contribuinte_cpf_cnpj: registro.contribuinte_cpf_cnpj.clone(),
            procurador_cpf_cnpj: registro.procurador_cpf_cnpj.clone(),
            codigo_servico: registro.codigo_servico.clone(),
            status: registro.status.clone(),
            valido_de: registro.valido_de,
            valido_ate: registro.valido_ate,
            verificado_em: registro.verificado_em,
            origem: registro.origem.clone(),
        }
    }
}

/// `POST /procuracoes/validar` — registro persistido + fonte da resposta.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoValidarProcuracaoResponse {
    /// `CACHE` (verificação recente, dentro do TTL) ou `SERPRO`.
    pub fonte: &'static str,
    pub registro: ProcuracaoRegistroResponse,
}

impl From<ResultadoValidarProcuracao> for ResultadoValidarProcuracaoResponse {
    fn from(resultado: ResultadoValidarProcuracao) -> Self {
        Self {
            fonte: resultado.fonte,
            registro: ProcuracaoRegistroResponse::de_registro(&resultado.registro),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoProcuracaoServicoResponse {
    pub id_sistema: String,
    pub id_servico: String,
    pub verbo: VerboIntegraContador,
    pub habilitado: bool,
    /// `NAO_EXIGIDA` | `AUTORIZADA` | `CONSULTADA` | `PENDENTE`.
    pub estado: String,
}

impl From<EstadoProcuracaoServico> for EstadoProcuracaoServicoResponse {
    fn from(estado: EstadoProcuracaoServico) -> Self {
        Self {
            id_sistema: estado.id_sistema,
            id_servico: estado.id_servico,
            verbo: estado.verbo,
            habilitado: estado.habilitado,
            estado: estado.estado,
        }
    }
}

/// `GET /procuracoes` — estado consolidado da procuração de um cliente.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoProcuracaoClienteResponse {
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    pub status: Option<String>,
    pub verificado_em: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub servicos: Vec<EstadoProcuracaoServicoResponse>,
}

impl From<EstadoProcuracaoCliente> for EstadoProcuracaoClienteResponse {
    fn from(estado: EstadoProcuracaoCliente) -> Self {
        Self {
            contribuinte_cpf_cnpj: estado.contribuinte_cpf_cnpj,
            procurador_cpf_cnpj: estado.procurador_cpf_cnpj,
            status: estado.status,
            verificado_em: estado.verificado_em,
            valido_de: estado.valido_de,
            valido_ate: estado.valido_ate,
            servicos: estado
                .servicos
                .into_iter()
                .map(EstadoProcuracaoServicoResponse::from)
                .collect(),
        }
    }
}

// ===== FASE A — catálogo configurável (painel admin, ADR-0013) =====

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinhaCatalogoServicoResponse {
    pub id: uuid::Uuid,
    pub id_sistema: String,
    pub id_servico: String,
    pub verbo: VerboIntegraContador,
    pub versao_sistema: Option<String>,
    pub versao_confirmada: bool,
    pub habilitado: bool,
    pub apenas_consulta: bool,
    pub requer_procuracao: bool,
    pub codigos_procuracao: Vec<String>,
    pub descricao: String,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

impl From<LinhaCatalogoServico> for LinhaCatalogoServicoResponse {
    fn from(linha: LinhaCatalogoServico) -> Self {
        Self {
            id: linha.id,
            id_sistema: linha.id_sistema,
            id_servico: linha.id_servico,
            verbo: linha.verbo,
            versao_sistema: linha.versao_sistema,
            versao_confirmada: linha.versao_confirmada,
            habilitado: linha.habilitado,
            apenas_consulta: linha.apenas_consulta,
            requer_procuracao: linha.requer_procuracao,
            codigos_procuracao: linha.codigos_procuracao,
            descricao: linha.descricao,
            atualizado_em: linha.atualizado_em,
        }
    }
}

/// `PATCH /admin/integra-contador/catalogo/{sistema}/{servico}` —
/// liga/desliga uma linha do catálogo (validada contra o catálogo de
/// referência do domínio; versão pendente permanece fail-closed).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoHabilitarServico {
    pub habilitado: bool,
}

/// Requisição de validação de procuração e-CAC de um cliente (modelo
/// Software House): o backend assina o termo com o cert A1 do escritório,
/// chama AUTENTICAPROCURADOR e guarda o token por cliente. O frontend
/// nunca recebe o token — apenas o status autorizado/pendente.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoAutenticarProcurador {
    pub cliente_id: uuid::Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoAutenticarProcuradorResponse {
    pub cliente_id: uuid::Uuid,
    pub status: &'static str,
    pub expira_em: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusIntegraContadorResponse {
    pub ambiente_efetivo: AmbienteSerpro,
    pub producao_habilitada: bool,
    pub assinatura_habilitada: bool,
    pub motivo_bloqueio_producao: &'static str,
    pub motivo_bloqueio_assinatura: &'static str,
    pub servicos: Vec<ServicoIntegraContadorResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicoIntegraContadorResponse {
    pub id_sistema: &'static str,
    pub id_servico: &'static str,
    pub versao_sistema: Option<&'static str>,
    pub verbo: VerboIntegraContador,
    pub habilitado: bool,
    pub requer_confirmacao: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoIntegraContadorResponse {
    pub ambiente: AmbienteSerpro,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub estado: String,
    pub status_serpro: u16,
    pub mensagem: &'static str,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
    pub codigos_mensagem: Vec<String>,
}

impl StatusIntegraContadorResponse {
    pub fn novo(ambiente: AmbienteResolvido) -> Self {
        let servicos = crate::domain::integra_contador::servicos::catalogo_de_referencia()
            .into_iter()
            .map(ServicoIntegraContadorResponse::de)
            .collect();
        Self {
            ambiente_efetivo: ambiente.ambiente,
            producao_habilitada: false,
            assinatura_habilitada: false,
            motivo_bloqueio_producao: "Confirmação contratual SERPRO e aprovação D5 pendentes.",
            motivo_bloqueio_assinatura: "Validação de cadeia ICP-Brasil e LCR não está configurada.",
            servicos,
        }
    }
}

impl ServicoIntegraContadorResponse {
    fn de(servico: ServicoIntegraContador) -> Self {
        Self {
            id_sistema: servico.id_sistema,
            id_servico: servico.id_servico,
            versao_sistema: servico.versao_sistema,
            verbo: servico.verbo,
            habilitado: servico.estado == EstadoCatalogo::Habilitado,
            requer_confirmacao: servico.verbo.exige_confirmacao(),
        }
    }
}

impl From<ResultadoIntegraContador> for ResultadoIntegraContadorResponse {
    fn from(resultado: ResultadoIntegraContador) -> Self {
        Self {
            ambiente: resultado.ambiente,
            verbo: resultado.verbo,
            id_sistema: resultado.id_sistema,
            id_servico: resultado.id_servico,
            estado: resultado.estado,
            status_serpro: resultado.status,
            mensagem: resultado.mensagem,
            referencia_resposta: resultado.referencia_resposta,
            tempo_espera_segundos: resultado.tempo_espera_segundos,
            etag: resultado.etag,
            expires: resultado.expires,
            codigos_mensagem: resultado.codigos_mensagem,
        }
    }
}
