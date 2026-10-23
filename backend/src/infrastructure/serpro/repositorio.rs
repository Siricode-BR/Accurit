use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use std::str::FromStr;
use uuid::Uuid;

use crate::application::serpro::ports::{
    AuditoriaSerproRepositorio, CertificadoAtivoResumo, CertificadosSerproConsulta,
    CredencialArmazenada, EscritorioSerproRepositorio, EventoAuditoriaSerpro,
    InvalicadorCachesSerpro, RepositorioError,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{
    auditoria_eventos, certificados_a1, credenciais_serpro, escritorios, procuracoes, tokens_serpro,
};

#[derive(Clone)]
pub struct SerproRepositorioDiesel {
    pool: DbPool,
}

impl SerproRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        RepositorioError,
    > {
        self.pool.get().map_err(|_| RepositorioError::Indisponivel)
    }
}

impl EscritorioSerproRepositorio for SerproRepositorioDiesel {
    fn buscar_modo_trial(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<(bool, i64, chrono::DateTime<chrono::Utc>)>, RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        escritorios
            .filter(id.eq(escritorio_id))
            .select((modo_trial_serpro, versao, atualizado_em))
            .first(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }

    fn atualizar_modo_trial(
        &self,
        escritorio_id: Uuid,
        modo_trial: bool,
        versao_esperada: i64,
    ) -> Result<(i64, chrono::DateTime<chrono::Utc>), RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        let agora = chrono::Utc::now();

        // RETURNING: a única troca atomicamente, respeitando a versão
        // esperada (concorrência otimista do toggle).
        let atualizado = diesel::update(
            escritorios
                .filter(id.eq(escritorio_id))
                .filter(versao.eq(versao_esperada)),
        )
        .set((
            modo_trial_serpro.eq(modo_trial),
            versao.eq(versao_esperada + 1),
            atualizado_em.eq(agora),
        ))
        .returning((versao, atualizado_em))
        .get_result::<(i64, chrono::DateTime<chrono::Utc>)>(&mut conexao)
        .optional()
        .map_err(|_| RepositorioError::Dados)?;

        if let Some((versao_nova, quando)) = atualizado {
            return Ok((versao_nova, quando));
        }

        // Zero linhas: o escritório não existe OU a versão divergiu
        // (outra operação concorrente venceu no repositório). Distinguir:
        if escritorios
            .filter(id.eq(escritorio_id))
            .count()
            .get_result::<i64>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?
            > 0
        {
            Err(RepositorioError::Concorrencia)
        } else {
            Err(RepositorioError::NaoEncontrado)
        }
    }

    fn buscar_credencial(
        &self,
        escritorio_id_param: Uuid,
        produto_serpro: ProdutoSerpro,
        ambiente_serpro: AmbienteSerpro,
    ) -> Result<Option<CredencialArmazenada>, RepositorioError> {
        use self::credenciais_serpro::dsl::*;

        let mut conexao = self.conexao()?;
        let produto_str = produto_serpro.as_str();
        let ambiente_str = ambiente_serpro.as_str();

        type CredencialRow = (
            Uuid,
            Uuid,
            String,
            String,
            Vec<u8>,
            Vec<u8>,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        );

        let row = credenciais_serpro
            .filter(
                escritorio_id
                    .eq(escritorio_id_param)
                    .and(produto.eq(produto_str))
                    .and(ambiente.eq(ambiente_str)),
            )
            .select((
                id,
                escritorio_id,
                produto,
                ambiente,
                consumer_key_cifrado,
                consumer_secret_cifrado,
                criado_em,
                atualizado_em,
            ))
            .first::<CredencialRow>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?;

        row.map(|r| {
            Ok(CredencialArmazenada {
                id: r.0,
                escritorio_id: r.1,
                produto: ProdutoSerpro::from_str(&r.2).map_err(|_| RepositorioError::Dados)?,
                ambiente: AmbienteSerpro::from_str(&r.3).map_err(|_| RepositorioError::Dados)?,
                consumer_key_cifrado: r.4,
                consumer_secret_cifrado: r.5,
                criado_em: r.6,
                atualizado_em: r.7,
            })
        })
        .transpose()
    }
}

impl CertificadosSerproConsulta for SerproRepositorioDiesel {
    fn buscar_certificado_em_validade(
        &self,
        param_escritorio_id: Uuid,
        agora: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<CertificadoAtivoResumo>, RepositorioError> {
        use self::certificados_a1::dsl::*;

        let mut conexao = self.conexao()?;
        certificados_a1
            .filter(
                escritorio_id
                    .eq(param_escritorio_id)
                    .and(ativo.eq(true))
                    .and(valido_de.le(agora))
                    .and(valido_ate.gt(agora)),
            )
            .order_by(valido_ate.desc())
            .select((id, fingerprint_sha256, valido_ate))
            .first::<(Uuid, String, chrono::DateTime<chrono::Utc>)>(&mut conexao)
            .optional()
            .map(|linha| {
                linha.map(|(id_cert, fingerprint, validade)| CertificadoAtivoResumo {
                    id: id_cert,
                    fingerprint_sha256: fingerprint,
                    valido_ate: validade,
                })
            })
            .map_err(|_| RepositorioError::Dados)
    }
}

impl InvalicadorCachesSerpro for SerproRepositorioDiesel {
    /// ADR-0005 §6: o toggle muda o ambiente de todos os serviços do
    /// escritório; tokens e caches de procuração dos ambientes anteriores
    /// são descartados. Idempotente.
    fn invalidar_todos_do_tenant(&self, escritorio_id_param: Uuid) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;

        diesel::delete(
            tokens_serpro::table.filter(tokens_serpro::escritorio_id.eq(escritorio_id_param)),
        )
        .execute(&mut conexao)
        .map_err(|_| RepositorioError::Dados)?;

        diesel::delete(
            procuracoes::table.filter(procuracoes::escritorio_id.eq(escritorio_id_param)),
        )
        .execute(&mut conexao)
        .map_err(|_| RepositorioError::Dados)?;

        Ok(())
    }
}

impl AuditoriaSerproRepositorio for SerproRepositorioDiesel {
    fn registrar(&self, evento: &EventoAuditoriaSerpro) -> Result<(), RepositorioError> {
        use self::auditoria_eventos::dsl::*;

        let mut conexao = self.conexao()?;
        let valor_novo_json =
            serde_json::to_value(&evento.valor_novo).map_err(|_| RepositorioError::Dados)?;
        let valor_anterior_json = evento
            .valor_anterior
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|_| RepositorioError::Dados)?;
        let ip_net = evento.ip.map(ipnet::IpNet::from);

        diesel::insert_into(auditoria_eventos)
            .values((
                id.eq(Uuid::now_v7()),
                escritorio_id.eq(evento.escritorio_id),
                usuario_id.eq(None::<Uuid>),
                administrador_id.eq(Some(evento.administrador_id)),
                acao.eq(evento.acao),
                entidade.eq(Some(evento.entidade)),
                entidade_id.eq(Some(evento.entidade_id)),
                valor_novo.eq(valor_novo_json),
                valor_anterior.eq(valor_anterior_json),
                ip.eq(ip_net),
            ))
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Os testes de integração (constraints do banco, RETURNING e invalidação)
    // ficam em backend/tests/ — aqui só garantimos que o repositório é
    // compartilhável entre threads gerenciadas pelo Axum.
    #[test]
    fn repositorio_e_send_sync() {
        fn eh_send_sync<T: Send + Sync>() {}
        eh_send_sync::<SerproRepositorioDiesel>();
    }
}
