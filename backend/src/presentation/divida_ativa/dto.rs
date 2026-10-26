//! DTOs de saída do consultivo de Dívida Ativa: apenas serialização
//! de leitura — a API transporta valores canônicos (Decimal como
//! string com ponto decimal, conforme SPEC §4.5) e a UI formata em
//! pt-BR. Sem campos de negociação/parcelamento (não existem).

use rust_decimal::Decimal;
use serde::Serialize;

use crate::domain::divida_ativa::devedor::InscricaoDevedor as DadoInscricao;
use crate::domain::divida_ativa::inscricao::{
    InscricaoDetalhe, ReceitaPrincipal, SistemaOrigem, UnidadeServico,
};

/// Inscrição retornada por `/devedor/{cpfCnpj}` (SPEC §4.3/§4.5).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoDevedorDto {
    pub numero_inscricao: Option<String>,
    pub numero_processo: Option<String>,
    pub situacao_inscricao: Option<String>,
    pub situacao_descricao: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    /// Decimal canônico (`4457.56`) — nunca f32/f64.
    pub valor_total_consolidado: Option<String>,
    /// Documento do item em dígitos (DV validado no domínio).
    pub cpf_cnpj: Option<String>,
    pub codigo_sida: Option<String>,
    pub nome_unidade: Option<String>,
    pub codigo_comprot: Option<String>,
    pub codigo_uorg: Option<String>,
}

impl InscricaoDevedorDto {
    pub fn do_dominio(dado: &DadoInscricao) -> Self {
        Self {
            numero_inscricao: dado.numero_inscricao.clone(),
            numero_processo: dado.numero_processo.clone(),
            situacao_inscricao: dado.situacao_inscricao.clone(),
            situacao_descricao: dado.situacao_descricao.clone(),
            nome_devedor: dado.nome_devedor.clone(),
            tipo_devedor: dado.tipo_devedor.clone(),
            valor_total_consolidado: moeda(dado.valor_total_consolidado),
            cpf_cnpj: dado
                .cpf_cnpj
                .as_ref()
                .map(|documento| documento.como_str().to_owned()),
            codigo_sida: dado.codigo_sida.clone(),
            nome_unidade: dado.nome_unidade.clone(),
            codigo_comprot: dado.codigo_comprot.clone(),
            codigo_uorg: dado.codigo_uorg.clone(),
        }
    }
}

fn moeda(valor: Option<Decimal>) -> Option<String> {
    valor.map(|valor| valor.to_string())
}

/// Unidade SERPRO (PFN) — código e nome, leitura pura.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnidadeServicoDto {
    pub numero: String,
    pub nome: Option<String>,
}

impl UnidadeServicoDto {
    pub fn do_dominio(unidade: &Option<UnidadeServico>) -> Option<Self> {
        unidade.as_ref().map(|u| Self {
            numero: u.numero.clone(),
            nome: u.nome.clone(),
        })
    }
}

/// Receita principal — código/nome, leitura pura.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceitaPrincipalDto {
    pub codigo: String,
    pub nome: Option<String>,
}

impl ReceitaPrincipalDto {
    pub fn do_dominio(receita: &Option<ReceitaPrincipal>) -> Option<Self> {
        receita.as_ref().map(|r| Self {
            codigo: r.codigo.clone(),
            nome: r.nome.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SistemaOrigemDto {
    pub codigo: String,
    pub descricao: Option<String>,
}

impl SistemaOrigemDto {
    pub fn do_dominio(origem: &Option<SistemaOrigem>) -> Option<Self> {
        origem.as_ref().map(|o| Self {
            codigo: o.codigo.clone(),
            descricao: o.descricao.clone(),
        })
    }
}

/// Detalhe de `/inscricao/{numeroInscricao}` (SPEC §4.3/§4.5):
/// exibição integral dos dados de leitura.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoDetalheDto {
    /// Formatação fiel do retorno: `90 6 17 000964-19`.
    #[serde(rename = "inscricao")]
    pub inscricao_formatada: String,
    /// Dígitos (URL de nova consulta na UI).
    pub inscricao_digitos: String,
    pub processo_administrativo: Option<String>,
    pub situacao: Option<String>,
    pub data_inscricao: Option<String>,
    pub numero_pfn_responsavel: Option<String>,
    pub nome_pfn_responsavel: Option<String>,
    pub numero_pfn_inscricao: Option<String>,
    pub nome_pfn_inscricao: Option<String>,
    pub receita_principal: Option<ReceitaPrincipalDto>,
    /// `indicadorPrescricaoSV8` traduzido (`SIM`/`NAO` → bool).
    pub prescrito: Option<bool>,
    pub valor_inscrito: Option<String>,
    pub valor_consolidado: Option<String>,
    pub valor_remanescente: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub nome_unidade: Option<String>,
    pub sistema_origem: Option<SistemaOrigemDto>,
}

impl InscricaoDetalheDto {
    pub fn do_dominio(dado: &InscricaoDetalhe) -> Self {
        Self {
            inscricao_formatada: dado.inscricao_formatada.clone(),
            inscricao_digitos: dado.inscricao_digitos.clone(),
            processo_administrativo: dado.processo_administrativo.clone(),
            situacao: dado.situacao_descricao.clone(),
            data_inscricao: dado.data_inscricao.clone(),
            numero_pfn_responsavel: dado.unidade_responsavel.as_ref().map(|u| u.numero.clone()),
            nome_pfn_responsavel: dado
                .unidade_responsavel
                .as_ref()
                .and_then(|u| u.nome.clone()),
            numero_pfn_inscricao: dado.unidade_inscricao.as_ref().map(|u| u.numero.clone()),
            nome_pfn_inscricao: dado.unidade_inscricao.as_ref().and_then(|u| u.nome.clone()),
            receita_principal: ReceitaPrincipalDto::do_dominio(&dado.receita_principal),
            prescrito: dado.prescrito,
            valor_inscrito: moeda(dado.valor_inscrito),
            valor_consolidado: moeda(dado.valor_consolidado),
            valor_remanescente: moeda(dado.valor_remanescente),
            nome_devedor: dado.nome_devedor.clone(),
            tipo_devedor: dado.tipo_devedor.clone(),
            nome_unidade: dado.nome_unidade.clone(),
            sistema_origem: SistemaOrigemDto::do_dominio(&dado.sistema_origem),
        }
    }
}

/// Corpo da resposta de `/divida-ativa/devedor/{cpfCnpj}` (SPEC
/// §4.4/§4.5: ambiente e horário explícitos; estados distintos).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevedorResponse {
    pub ambiente: crate::domain::serpro::ambiente::AmbienteSerpro,
    pub estado: &'static str,
    pub fonte_consulta: &'static str,
    pub chave_mascarada: String,
    pub horario_consulta: String,
    pub tentativas: u8,
    pub inscricoes: Vec<InscricaoDevedorDto>,
}

/// Corpo da resposta de `/divida-ativa/inscricao/{numeroInscricao}`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoResponse {
    pub ambiente: crate::domain::serpro::ambiente::AmbienteSerpro,
    pub estado: &'static str,
    pub chave_mascarada: String,
    pub horario_consulta: String,
    pub detalhe: InscricaoDetalheDto,
}
