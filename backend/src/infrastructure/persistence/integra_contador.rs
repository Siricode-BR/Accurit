use async_trait::async_trait;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::application::integra_contador::ports::{
    CatalogoEscritoriosIntegra, RepositorioIdempotencia, ReservaIdempotencia,
};
use crate::application::serpro::ports::RepositorioError;
use crate::domain::integra_contador::envelope::{IntegraContadorResponse, VerboIntegraContador};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::infrastructure::persistence::pool::{DbConnection, DbPool};
use crate::infrastructure::persistence::schema::{
    escritorios as schema_escritorios, idempotencia_operacoes as schema_idempotencia,
};

#[derive(Clone)]
pub struct IntegraContadorRepositorioDiesel {
    pool: DbPool,
}

impl IntegraContadorRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(&self) -> Result<DbConnection, RepositorioError> {
        self.pool.get().map_err(|_| RepositorioError::Indisponivel)
    }

    fn buscar_reserva(
        &self,
        escritorio: Uuid,
        chave: &str,
    ) -> Result<Option<ReservaDb>, RepositorioError> {
        use schema_idempotencia::dsl::*;

        idempotencia_operacoes
            .filter(escritorio_id.eq(escritorio))
            .filter(chave_idempotencia.eq(chave))
            .select((payload_hash, estado, resposta_redigida, resposta_status))
            .first::<ReservaDb>(&mut self.conexao()?)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::idempotencia_operacoes)]
struct ReservaDb {
    payload_hash: String,
    estado: String,
    resposta_redigida: Option<Value>,
    resposta_status: Option<i32>,
}

impl TryFrom<ReservaDb> for ReservaIdempotencia {
    type Error = RepositorioError;

    fn try_from(db: ReservaDb) -> Result<Self, Self::Error> {
        let resposta = db
            .resposta_redigida
            .map(serde_json::from_value::<IntegraContadorResponse>)
            .transpose()
            .map_err(|_| RepositorioError::Dados)?;

        Ok(Self {
            payload_hash: db.payload_hash,
            resposta,
            http_status: db
                .resposta_status
                .and_then(|status| u16::try_from(status).ok()),
            estado: db.estado,
            criada_agora: false,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::idempotencia_operacoes)]
struct NovaReserva<'a> {
    id: Uuid,
    escritorio_id: Uuid,
    chave_idempotencia: &'a str,
    verbo: &'a str,
    id_sistema: &'a str,
    id_servico: &'a str,
    versao_sistema: &'a str,
    ambiente: &'a str,
    payload_hash: &'a str,
    estado: &'a str,
}

#[async_trait]
impl RepositorioIdempotencia for IntegraContadorRepositorioDiesel {
    async fn verificar(
        &self,
        escritorio: Uuid,
        chave: &str,
    ) -> Result<Option<ReservaIdempotencia>, RepositorioError> {
        let repositorio = self.clone();
        let chave = chave.to_owned();
        tokio::task::spawn_blocking(move || {
            repositorio
                .buscar_reserva(escritorio, &chave)?
                .map(ReservaIdempotencia::try_from)
                .transpose()
        })
        .await
        .map_err(|_| RepositorioError::Dados)?
    }

    async fn reservar(
        &self,
        escritorio: Uuid,
        chave: &str,
        verbo: VerboIntegraContador,
        sistema: &str,
        servico: &str,
        versao: &str,
        ambiente: AmbienteSerpro,
        hash: &str,
    ) -> Result<ReservaIdempotencia, RepositorioError> {
        let repositorio = self.clone();
        let chave = chave.to_owned();
        let sistema = sistema.to_owned();
        let servico = servico.to_owned();
        let versao = versao.to_owned();
        let hash = hash.to_owned();
        let verbo_str = verbo.path().to_owned();
        let ambiente_str = ambiente.as_str().to_owned();
        tokio::task::spawn_blocking(move || {
            use schema_idempotencia::dsl::*;

            let mut conexao = repositorio.conexao()?;
            let inseridas = diesel::insert_into(idempotencia_operacoes)
                .values(NovaReserva {
                    id: Uuid::now_v7(),
                    escritorio_id: escritorio,
                    chave_idempotencia: &chave,
                    verbo: &verbo_str,
                    id_sistema: &sistema,
                    id_servico: &servico,
                    versao_sistema: &versao,
                    ambiente: &ambiente_str,
                    payload_hash: &hash,
                    estado: "RESERVADA",
                })
                .on_conflict((escritorio_id, chave_idempotencia))
                .do_nothing()
                .execute(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;

            let reserva = idempotencia_operacoes
                .filter(escritorio_id.eq(escritorio))
                .filter(chave_idempotencia.eq(&chave))
                .select((payload_hash, estado, resposta_redigida, resposta_status))
                .first::<ReservaDb>(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;
            let mut reserva = ReservaIdempotencia::try_from(reserva)?;
            reserva.criada_agora = inseridas == 1;
            Ok(reserva)
        })
        .await
        .map_err(|_| RepositorioError::Dados)?
    }

    async fn finalizar(
        &self,
        escritorio: Uuid,
        chave: &str,
        novo_estado: &str,
        resposta: Option<&IntegraContadorResponse>,
        status_http: Option<u16>,
    ) -> Result<(), RepositorioError> {
        let resposta = resposta
            .map(serde_json::to_value)
            .transpose()
            .map_err(|_| RepositorioError::Dados)?;
        let status_http = status_http.map(i32::from);
        let chave = chave.to_owned();
        let novo_estado = novo_estado.to_owned();
        let repositorio = self.clone();

        tokio::task::spawn_blocking(move || {
            use schema_idempotencia::dsl::*;

            let mut conexao = repositorio.conexao()?;
            let atualizadas = diesel::update(
                idempotencia_operacoes
                    .filter(escritorio_id.eq(escritorio))
                    .filter(chave_idempotencia.eq(chave))
                    .filter(estado.eq("RESERVADA")),
            )
            .set((
                estado.eq(novo_estado),
                resposta_redigida.eq(resposta),
                resposta_status.eq(status_http),
                processado_em.eq(Some(chrono::Utc::now())),
            ))
            .execute(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

            if atualizadas != 1 {
                return Err(RepositorioError::Dados);
            }
            Ok(())
        })
        .await
        .map_err(|_| RepositorioError::Dados)?
    }
}

#[async_trait]
impl CatalogoEscritoriosIntegra for IntegraContadorRepositorioDiesel {
    async fn buscar_cnpj(&self, escritorio: Uuid) -> Result<Option<String>, RepositorioError> {
        let repositorio = self.clone();
        tokio::task::spawn_blocking(move || {
            use schema_escritorios::dsl::*;

            escritorios
                .filter(id.eq(escritorio))
                .select(cnpj)
                .first::<Option<String>>(&mut repositorio.conexao()?)
                .optional()
                .map(|resultado| resultado.flatten())
                .map_err(|_| RepositorioError::Dados)
        })
        .await
        .map_err(|_| RepositorioError::Dados)?
    }
}
