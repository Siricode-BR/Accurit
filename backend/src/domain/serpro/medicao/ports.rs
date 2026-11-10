//! Fase G — Portas de medição de uso SERpro.
//! Implementadas pela infrastructure (Diesel).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::serpro::medicao::entidades::{
    AgregadoConsumoSerpro, AssinaturaEscritorio, BloqueioSerpro, ChamadaSerpro,
    ComandoRegistrarBloqueio, ComandoRegistrarChamada, PlanoEscritorio, ResultadoRegistroChamada,
};
use crate::domain::serpro::medicao::erros::ErroMedicao;

/// Repositório de chamadas SERpro (raw, append-only).
#[async_trait]
pub trait RepositorioChamadasSerpro: Send + Sync {
    async fn registrar(
        &self,
        comando: &ComandoRegistrarChamada,
    ) -> Result<ChamadaSerpro, ErroMedicao>;

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<ChamadaSerpro>, ErroMedicao>;

    async fn listar_por_periodo(
        &self,
        escritorio_id: Uuid,
        produto: Option<String>,
        ambiente: Option<String>,
        periodo: Option<String>,
        referencia_inicio: DateTime<Utc>,
        referencia_fim: DateTime<Utc>,
    ) -> Result<Vec<ChamadaSerpro>, ErroMedicao>;
}

/// Repositório de agregados de consumo.
#[async_trait]
pub trait RepositorioAgregadosConsumo: Send + Sync {
    async fn upsert(
        &self,
        agregado: &AgregadoConsumoSerpro,
    ) -> Result<AgregadoConsumoSerpro, ErroMedicao>;

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        periodo: Option<String>,
        referencia_inicio: Option<DateTime<Utc>>,
        referencia_fim: Option<DateTime<Utc>>,
    ) -> Result<Vec<AgregadoConsumoSerpro>, ErroMedicao>;
}

/// Repositório de planos de escritório.
#[async_trait]
pub trait RepositorioPlanosEscritorio: Send + Sync {
    async fn listar_ativos(&self) -> Result<Vec<PlanoEscritorio>, ErroMedicao>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<PlanoEscritorio>, ErroMedicao>;
}

/// Repositório de assinaturas de escritório.
#[async_trait]
pub trait RepositorioAssinaturasEscritorio: Send + Sync {
    async fn buscar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<AssinaturaEscritorio>, ErroMedicao>;

    async fn criar(
        &self,
        assinatura: &AssinaturaEscritorio,
    ) -> Result<AssinaturaEscritorio, ErroMedicao>;

    async fn atualizar(
        &self,
        assinatura: &AssinaturaEscritorio,
    ) -> Result<AssinaturaEscritorio, ErroMedicao>;
}

/// Repositório de bloqueios SERpro.
#[async_trait]
pub trait RepositorioBloqueiosSerpro: Send + Sync {
    async fn registrar(
        &self,
        comando: &ComandoRegistrarBloqueio,
    ) -> Result<BloqueioSerpro, ErroMedicao>;

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<BloqueioSerpro>, ErroMedicao>;
}

/// Serviço de medição: registra chamadas, verifica limites, atualiza agregados.
#[async_trait]
pub trait ServicoMedicaoSerpro: Send + Sync {
    /// Registra uma chamada SERpro e verifica se o escritório está dentro do limite.
    async fn registrar_chamada(
        &self,
        comando: &ComandoRegistrarChamada,
    ) -> Result<ResultadoRegistroChamada, ErroMedicao>;

    /// Registra uma chamada bloqueada localmente.
    async fn registrar_bloqueio(
        &self,
        comando: &ComandoRegistrarBloqueio,
    ) -> Result<BloqueioSerpro, ErroMedicao>;

    /// Atualiza agregados de consumo para um período.
    async fn atualizar_agregados(
        &self,
        escritorio_id: Uuid,
        referencia: DateTime<Utc>,
    ) -> Result<(), ErroMedicao>;

    /// Verifica se o escritório pode fazer mais chamadas neste mês.
    async fn pode_fazer_chamada(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao>;
}
