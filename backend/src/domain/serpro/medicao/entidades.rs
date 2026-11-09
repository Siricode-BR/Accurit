//! Fase G — Entidades de medição de uso SERPRO.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::enums::{MotivoBloqueio, OrigemChamada, PeriodoAgregado, ResultadoChamada};
use crate::domain::serpro::produto::ProdutoSerpro;

/// Chamada feita ao SERPRO (registro bruto, append-only).
#[derive(Debug, Clone)]
pub struct ChamadaSerpro {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub verbo: String,
    pub servico: Option<String>,
    pub ambiente: AmbienteSerpro,
    pub cliente_id: Option<Uuid>,
    pub usuario_id: Option<Uuid>,
    pub origem: OrigemChamada,
    pub resultado: ResultadoChamada,
    pub status_http: Option<u16>,
    pub duracao_ms: u64,
    pub request_id: String,
    pub criado_em: DateTime<Utc>,
}

/// Agregado de consumo por escritório/produto/ambiente/período.
#[derive(Debug, Clone)]
pub struct AgregadoConsumoSerpro {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub ambiente: AmbienteSerpro,
    pub periodo: PeriodoAgregado,
    pub referencia_periodo: DateTime<Utc>,
    pub total_chamadas: i64,
    pub chamadas_sucesso: i64,
    pub chamadas_4xx: i64,
    pub chamadas_5xx: i64,
    pub chamadas_timeout: i64,
    pub chamadas_indeterminado: i64,
    pub duracao_total_ms: i64,
    pub atualizado_em: DateTime<Utc>,
}

/// Plano de consumo (quantas chamadas/mês cada escritório pode fazer).
#[derive(Debug, Clone)]
pub struct PlanoEscritorio {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub chamadas_mes: i64,
    pub preco_mensal_cents: i64,
    pub ativo: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Assinatura de um escritório (qual plano está ativo).
#[derive(Debug, Clone)]
pub struct AssinaturaEscritorio {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub plano_id: Uuid,
    pub chamadas_mes_limite: i64,
    pub bloqueio_ao_exceder: bool,
    pub ativo: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Chamada bloqueada localmente (não conta como chamada SERPRO).
#[derive(Debug, Clone)]
pub struct BloqueioSerpro {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub motivo: MotivoBloqueio,
    pub contexto: Option<String>,
    pub criado_em: DateTime<Utc>,
}

/// Comando para registrar uma chamada SERpro (chamado pelos wrappers de medição).
#[derive(Debug, Clone)]
pub struct ComandoRegistrarChamada {
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub verbo: String,
    pub servico: Option<String>,
    pub ambiente: AmbienteSerpro,
    pub cliente_id: Option<Uuid>,
    pub usuario_id: Option<Uuid>,
    pub origem: OrigemChamada,
    pub resultado: ResultadoChamada,
    pub status_http: Option<u16>,
    pub duracao_ms: u64,
    pub request_id: String,
}

/// Comando para registrar uma chamada bloqueada.
#[derive(Debug, Clone)]
pub struct ComandoRegistrarBloqueio {
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub motivo: MotivoBloqueio,
    pub contexto: Option<String>,
}

/// Resultado do registro de chamada (se foi aceita/bloqueada por plano).
#[derive(Debug, Clone)]
pub struct ResultadoRegistroChamada {
    pub chamada_aceita: bool,
    pub chamadas_usadas_mes: i64,
    pub chamadas_limite_mes: i64,
    pub percentual_utilizacao: f64,
}
