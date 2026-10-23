use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use ipnet::IpNet;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::application::admin::{
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EventoAuditoriaAdmin,
    RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{auditoria_eventos, escritorios, usuarios};

#[derive(Clone)]
pub struct AdminRepositorioDiesel {
    pool: DbPool,
}

impl AdminRepositorioDiesel {
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

impl EscritorioRepositorio for AdminRepositorioDiesel {
    fn criar(&self, escritorio: &EscritorioNovo) -> Result<Uuid, RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        let escritorio_id = Uuid::now_v7();

        diesel::insert_into(escritorios)
            .values((
                id.eq(escritorio_id),
                slug.eq(&escritorio.slug),
                nome.eq(&escritorio.nome),
                criado_em.eq(chrono::Utc::now()),
                atualizado_em.eq(chrono::Utc::now()),
            ))
            .execute(&mut conexao)
            .map(|_| escritorio_id)
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) = e
                {
                    RepositorioError::EscritorioExistente
                } else {
                    RepositorioError::Dados
                }
            })
    }

    fn buscar_por_slug(&self, slug_alvo: &str) -> Result<Option<Uuid>, RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        escritorios
            .filter(slug.eq(slug_alvo))
            .select(id)
            .first(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }
}

impl UsuarioAdminRepositorio for AdminRepositorioDiesel {
    fn criar(&self, usuario: &UsuarioNovo) -> Result<Uuid, RepositorioError> {
        use self::usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        let usuario_id = Uuid::now_v7();

        diesel::insert_into(usuarios)
            .values((
                id.eq(usuario_id),
                escritorio_id.eq(usuario.escritorio_id),
                email.eq(&usuario.email),
                cpf.eq(&usuario.cpf),
                senha_hash.eq(&usuario.senha_hash),
                papel.eq(usuario.papel.como_str()),
                ativo.eq(true),
                criado_em.eq(chrono::Utc::now()),
            ))
            .execute(&mut conexao)
            .map(|_| usuario_id)
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) = e
                {
                    RepositorioError::UsuarioExistente
                } else {
                    RepositorioError::Dados
                }
            })
    }

    fn buscar_por_escritorio_e_email(
        &self,
        escritorio_id_alvo: Uuid,
        email_alvo: &str,
    ) -> Result<Option<Uuid>, RepositorioError> {
        use self::usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        usuarios
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(email.eq(email_alvo))
            .select(id)
            .first(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }
}

impl AuditoriaAdminRepositorio for AdminRepositorioDiesel {
    fn registrar(&self, evento: &EventoAuditoriaAdmin) -> Result<(), RepositorioError> {
        use self::auditoria_eventos::dsl::*;

        let mut conexao = self.conexao()?;
        let valor_novo_json: JsonValue =
            serde_json::to_value(&evento.valor_novo).map_err(|_| RepositorioError::Dados)?;
        let valor_anterior_json: Option<JsonValue> = evento
            .valor_anterior
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|_| RepositorioError::Dados)?;
        let ip_net = evento.ip.map(IpNet::from);

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
