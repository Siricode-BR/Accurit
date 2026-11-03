use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Jsonb, Nullable, Text, Uuid as SqlUuid};
use serde_json::Value;
use uuid::Uuid;

use crate::application::integra_contador::ports::{
    AuditoriaIntegraContador, CatalogoEscritoriosIntegra, ContextoIntegraContador,
    EventoAuditoriaIntegra, EventoChamadaIntegra, RepositorioIdempotencia, ReservaIdempotencia,
    ResultadoReservaIdempotencia,
};
use crate::domain::integra_contador::envelope::Identificacao;
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct IntegraContadorRepositorioDiesel {
    pool: DbPool,
}

impl IntegraContadorRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroIntegraContador,
    > {
        self.pool
            .get()
            .map_err(|_| ErroIntegraContador::Persistencia)
    }
}

#[derive(QueryableByName)]
struct ContextoRow {
    #[diesel(sql_type = Nullable<Text>)]
    cnpj: Option<String>,
    #[diesel(sql_type = Text)]
    cpf_operador: String,
    #[diesel(sql_type = Text)]
    documento_contribuinte: String,
}

#[async_trait::async_trait]
impl CatalogoEscritoriosIntegra for IntegraContadorRepositorioDiesel {
    async fn resolver_contexto(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: &str,
    ) -> Result<ContextoIntegraContador, ErroIntegraContador> {
        let documento = crate::domain::clientes::CpfCnpj::novo(documento_contribuinte)
            .map_err(|_| ErroIntegraContador::PayloadInvalido)?;
        let documento = documento.como_str().to_owned();
        let repositorio = self.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let linha = diesel::sql_query(
                "SELECT e.cnpj, u.cpf AS cpf_operador, c.cpf_cnpj AS documento_contribuinte \
                 FROM escritorios e \
                 INNER JOIN usuarios u \
                   ON u.escritorio_id = e.id AND u.id = $2 AND u.ativo = TRUE \
                 INNER JOIN clientes c \
                   ON c.escritorio_id = e.id AND c.cpf_cnpj = $3 \
                 WHERE e.id = $1",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<SqlUuid, _>(usuario_id)
            .bind::<Text, _>(documento)
            .get_result::<ContextoRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;

            let cnpj = linha
                .cnpj
                .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;
            let contratante = Identificacao::nova(&cnpj)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
            let contribuinte = Identificacao::nova(&linha.documento_contribuinte)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;

            Ok(ContextoIntegraContador {
                contratante: contratante.clone(),
                autor_pedido_dados: contratante,
                contribuinte,
                cpf_operador: Some(linha.cpf_operador),
            })
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

#[derive(QueryableByName)]
struct IdempotenciaRow {
    #[diesel(sql_type = Text)]
    verbo: String,
    #[diesel(sql_type = Text)]
    id_sistema: String,
    #[diesel(sql_type = Text)]
    id_servico: String,
    #[diesel(sql_type = Text)]
    versao_sistema: String,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Text)]
    payload_hash: String,
    #[diesel(sql_type = Text)]
    estado: String,
    #[diesel(sql_type = Nullable<Integer>)]
    resposta_status: Option<i32>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    resposta_redigida: Option<Value>,
}

#[async_trait::async_trait]
impl RepositorioIdempotencia for IntegraContadorRepositorioDiesel {
    async fn reservar(
        &self,
        reserva: &ReservaIdempotencia,
    ) -> Result<ResultadoReservaIdempotencia, ErroIntegraContador> {
        let repositorio = self.clone();
        let reserva = reserva.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let inseridos = diesel::sql_query(
                "INSERT INTO idempotencia_operacoes \
                 (id, escritorio_id, chave_idempotencia, verbo, id_sistema, id_servico, \
                  versao_sistema, ambiente, payload_hash, estado) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'EM_ANDAMENTO') \
                 ON CONFLICT (escritorio_id, chave_idempotencia) DO NOTHING",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(&reserva.chave)
            .bind::<Text, _>(reserva.verbo.como_str())
            .bind::<Text, _>(&reserva.id_sistema)
            .bind::<Text, _>(&reserva.id_servico)
            .bind::<Text, _>(&reserva.versao_sistema)
            .bind::<Text, _>(reserva.ambiente.as_str())
            .bind::<Text, _>(&reserva.payload_hash)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            if inseridos == 1 {
                return Ok(ResultadoReservaIdempotencia::Nova);
            }

            let existente = diesel::sql_query(
                "SELECT verbo, id_sistema, id_servico, versao_sistema, ambiente, payload_hash, \
                        estado, resposta_status, resposta_redigida \
                 FROM idempotencia_operacoes \
                 WHERE escritorio_id = $1 AND chave_idempotencia = $2",
            )
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(&reserva.chave)
            .get_result::<IdempotenciaRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;

            let mesmo_pedido = existente.verbo == reserva.verbo.como_str()
                && existente.id_sistema == reserva.id_sistema
                && existente.id_servico == reserva.id_servico
                && existente.versao_sistema == reserva.versao_sistema
                && existente.ambiente == reserva.ambiente.as_str()
                && existente.payload_hash == reserva.payload_hash;
            if !mesmo_pedido {
                return Err(ErroIntegraContador::ConflitoIdempotencia);
            }
            if existente.estado == "CONCLUIDO" || existente.estado == "REJEITADO" {
                return Ok(ResultadoReservaIdempotencia::Concluida {
                    status: existente.resposta_status,
                    resposta_redigida: existente.resposta_redigida,
                });
            }
            Err(ErroIntegraContador::ConflitoIdempotencia)
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn atualizar(
        &self,
        reserva: &ReservaIdempotencia,
        estado: &str,
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let reserva = reserva.clone();
        let estado = estado.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let atualizados = diesel::sql_query(
                "UPDATE idempotencia_operacoes \
                 SET estado = $1, resposta_status = $2, resposta_redigida = $3, \
                     processado_em = CASE \
                       WHEN $1 IN ('CONCLUIDO', 'REJEITADO', 'INDETERMINADO') THEN NOW() \
                       ELSE processado_em END \
                 WHERE escritorio_id = $4 AND chave_idempotencia = $5 AND payload_hash = $6",
            )
            .bind::<Text, _>(estado)
            .bind::<Nullable<Integer>, _>(status)
            .bind::<Nullable<Jsonb>, _>(resposta_redigida)
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(reserva.chave)
            .bind::<Text, _>(reserva.payload_hash)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            if atualizados != 1 {
                return Err(ErroIntegraContador::Persistencia);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

#[async_trait::async_trait]
impl AuditoriaIntegraContador for IntegraContadorRepositorioDiesel {
    async fn registrar_chamada(
        &self,
        evento: &EventoChamadaIntegra,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let evento = evento.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            diesel::sql_query(
                "INSERT INTO chamadas_operacionais \
                 (id, escritorio_id, usuario_id, verbo, id_sistema, id_servico, ambiente, \
                  estado, status_code, referencia_resposta, request_id, bilhetavel) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(evento.escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(evento.usuario_id))
            .bind::<Text, _>(evento.verbo.como_str())
            .bind::<Text, _>(evento.id_sistema)
            .bind::<Text, _>(evento.id_servico)
            .bind::<Text, _>(evento.ambiente.as_str())
            .bind::<Text, _>(evento.estado)
            .bind::<Nullable<Integer>, _>(evento.status_code)
            .bind::<Nullable<Text>, _>(evento.referencia_resposta)
            .bind::<Nullable<Text>, _>(Some(evento.request_id))
            .bind::<Bool, _>(evento.bilhetavel)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn registrar_mutacao(
        &self,
        evento: &EventoAuditoriaIntegra,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let evento = evento.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let valores = serde_json::json!({
                "verbo": evento.verbo.como_str(),
                "idSistema": evento.id_sistema,
                "idServico": evento.id_servico,
                "ambiente": evento.ambiente.as_str(),
                "estado": evento.estado,
                "payloadHash": evento.payload_hash,
            });
            diesel::sql_query(
                "INSERT INTO auditoria_eventos \
                 (id, escritorio_id, usuario_id, acao, entidade, entidade_id, valor_novo) \
                 VALUES ($1, $2, $3, $4, 'INTEGRA_CONTADOR', $5, $6)",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(evento.escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(evento.usuario_id))
            .bind::<Text, _>(evento.acao)
            .bind::<Nullable<SqlUuid>, _>(Some(Uuid::now_v7()))
            .bind::<Nullable<Jsonb>, _>(Some(valores))
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

pub struct ProcuracaoNaoVerificavel;

#[async_trait::async_trait]
impl crate::application::integra_contador::ports::ValidadorProcuracao for ProcuracaoNaoVerificavel {
    async fn validar(
        &self,
        _contexto: &ContextoIntegraContador,
        _codigo_servico: &str,
        _ambiente: AmbienteSerpro,
    ) -> Result<bool, ErroIntegraContador> {
        Ok(false)
    }
}
