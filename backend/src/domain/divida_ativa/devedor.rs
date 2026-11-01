use rust_decimal::Decimal;
use crate::domain::clientes::CpfCnpj;

#[derive(Debug, Clone)]
pub struct ConsultaDevedor {
    pub escritorio_id: uuid::Uuid,
    pub cliente_id: Option<uuid::Uuid>,
    pub chave: CpfCnpj,
}

#[derive(Debug, Clone)]
pub struct InscricaoResumo {
    pub numero_inscricao: String,
    pub numero_processo: Option<String>,
    pub situacao_descricao: String,
    pub valor_total_consolidado: Decimal,
}

#[derive(Debug, Clone)]
pub struct ResultadoConsultaDevedor {
    pub devedor: String,
    pub tipo_devedor: String,
    pub inscricoes: Vec<InscricaoResumo>,
}

#[derive(Debug, Clone)]
pub struct InscricaoDetalhe {
    pub inscricao: String,
    pub processo_administrativo: Option<String>,
    pub situacao: String,
    pub valor_total_consolidado: Decimal,
    pub indicador_prescricao: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FonteConsulta {
    CpfCnpj,
    CodigoContribuinte,
}
