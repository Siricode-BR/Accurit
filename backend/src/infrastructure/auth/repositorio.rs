use std::str::FromStr;

use chrono::{DateTime, Utc};
use diesel::sql_types::{BigInt, Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::{QueryableByName, RunQueryDsl};
use uuid::Uuid;

use crate::application::auth::{
    AdministradorRepositorio, AuditoriaRepositorio, EventoAuditoria, RepositorioError,
    SessaoAdminRepositorio, SessaoRepositorio, UsuarioRepositorio,
};
use crate::domain::auth::{
    Administrador, Audiencia, Escritorio, Papel, Sessao, SessaoAdmin, SessaoAdminComAdministrador,
    SessaoComUsuario, Usuario,
};
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct RepositorioDiesel {
    pool: DbPool,
}

impl RepositorioDiesel {
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

#[derive(QueryableByName)]
struct EscritorioRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    slug: String,
}

#[derive(QueryableByName)]
struct UsuarioRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    senha_hash: String,
    #[diesel(sql_type = Text)]
    papel: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    ativo: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    bloqueado_ate: Option<DateTime<Utc>>,
}

#[derive(QueryableByName)]
struct SessaoUsuarioRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    usuario_id: Uuid,
    #[diesel(sql_type = Text)]
    audiencia: String,
    #[diesel(sql_type = SqlUuid)]
    familia_id: Uuid,
    #[diesel(sql_type = Text)]
    refresh_hash: String,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    expira_em: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    revogada_em: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    ip: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    user_agent: Option<String>,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    senha_hash: String,
    #[diesel(sql_type = Text)]
    papel: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    ativo: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    bloqueado_ate: Option<DateTime<Utc>>,
}

fn para_usuario(row: UsuarioRow) -> Result<Usuario, RepositorioError> {
    let papel = Papel::from_str(&row.papel).map_err(|_| RepositorioError::Dados)?;
    Ok(Usuario {
        id: row.id,
        escritorio_id: row.escritorio_id,
        email: row.email,
        senha_hash: row.senha_hash,
        papel,
        ativo: row.ativo,
        bloqueado_ate: row.bloqueado_ate,
    })
}

impl UsuarioRepositorio for RepositorioDiesel {
    fn buscar_escritorio_por_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Escritorio>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query("SELECT id, slug FROM escritorios WHERE slug = $1")
            .bind::<Text, _>(slug)
            .get_result::<EscritorioRow>(&mut conexao)
            .optional_result();

        match resultado {
            Ok(Some(row)) => Ok(Some(Escritorio {
                id: row.id,
                slug: row.slug,
            })),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn buscar_por_escritorio_e_email(
        &self,
        escritorio_id: Uuid,
        email: &str,
    ) -> Result<Option<Usuario>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT id, escritorio_id, email, senha_hash, papel, ativo, bloqueado_ate \
             FROM usuarios WHERE escritorio_id = $1 AND lower(email) = lower($2)",
        )
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Text, _>(email)
        .get_result::<UsuarioRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => para_usuario(row).map(Some),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn buscar_por_id(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
    ) -> Result<Option<Usuario>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT id, escritorio_id, email, senha_hash, papel, ativo, bloqueado_ate \
             FROM usuarios WHERE escritorio_id = $1 AND id = $2",
        )
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<SqlUuid, _>(usuario_id)
        .get_result::<UsuarioRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => para_usuario(row).map(Some),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn registrar_falha(
        &self,
        usuario_id: Uuid,
        max_tentativas: u32,
        bloqueio_secs: u64,
    ) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        let max_tentativas = i32::try_from(max_tentativas).unwrap_or(i32::MAX);
        let bloqueio_secs = i64::try_from(bloqueio_secs).unwrap_or(i64::MAX);

        diesel::sql_query(
            "UPDATE usuarios SET falhas_consecutivas = falhas_consecutivas + 1, \
             bloqueado_ate = CASE WHEN falhas_consecutivas + 1 >= $2 \
             THEN NOW() + ($3 * INTERVAL '1 second') ELSE bloqueado_ate END \
             WHERE id = $1",
        )
        .bind::<SqlUuid, _>(usuario_id)
        .bind::<Integer, _>(max_tentativas)
        .bind::<BigInt, _>(bloqueio_secs)
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }

    fn registrar_sucesso(&self, usuario_id: Uuid) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE usuarios SET falhas_consecutivas = 0, bloqueado_ate = NULL, \
             ultimo_login_em = NOW() WHERE id = $1",
        )
        .bind::<SqlUuid, _>(usuario_id)
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }
}

impl SessaoRepositorio for RepositorioDiesel {
    fn criar(&self, sessao: &Sessao) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "INSERT INTO sessoes \
             (id, escritorio_id, usuario_id, audiencia, familia_id, refresh_hash, \
              criado_em, expira_em, revogada_em, ip, user_agent) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::inet, $11)",
        )
        .bind::<SqlUuid, _>(sessao.id)
        .bind::<SqlUuid, _>(sessao.escritorio_id)
        .bind::<SqlUuid, _>(sessao.usuario_id)
        .bind::<Text, _>(sessao.audiencia.como_str_banco())
        .bind::<SqlUuid, _>(sessao.familia_id)
        .bind::<Text, _>(&sessao.refresh_hash)
        .bind::<Timestamptz, _>(sessao.criado_em)
        .bind::<Timestamptz, _>(sessao.expira_em)
        .bind::<Nullable<Timestamptz>, _>(sessao.revogada_em)
        .bind::<Nullable<Text>, _>(sessao.ip.as_deref())
        .bind::<Nullable<Text>, _>(sessao.user_agent.as_deref())
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }

    fn buscar_por_hash(
        &self,
        refresh_hash: &str,
    ) -> Result<Option<SessaoComUsuario>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT s.id, s.escritorio_id, s.usuario_id, s.audiencia::text AS audiencia, \
             s.familia_id, s.refresh_hash::text AS refresh_hash, s.criado_em, s.expira_em, \
             s.revogada_em, s.ip::text AS ip, s.user_agent, \
             u.email, u.senha_hash, u.papel, u.ativo, u.bloqueado_ate \
             FROM sessoes s \
             JOIN usuarios u ON u.id = s.usuario_id AND u.escritorio_id = s.escritorio_id \
             WHERE s.refresh_hash = $1",
        )
        .bind::<Text, _>(refresh_hash)
        .get_result::<SessaoUsuarioRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => {
                let audiencia =
                    Audiencia::from_str(&row.audiencia).map_err(|_| RepositorioError::Dados)?;
                let papel = Papel::from_str(&row.papel).map_err(|_| RepositorioError::Dados)?;
                Ok(Some(SessaoComUsuario {
                    sessao: Sessao {
                        id: row.id,
                        escritorio_id: row.escritorio_id,
                        usuario_id: row.usuario_id,
                        audiencia,
                        familia_id: row.familia_id,
                        refresh_hash: row.refresh_hash,
                        criado_em: row.criado_em,
                        expira_em: row.expira_em,
                        revogada_em: row.revogada_em,
                        ip: row.ip,
                        user_agent: row.user_agent,
                    },
                    usuario: Usuario {
                        id: row.usuario_id,
                        escritorio_id: row.escritorio_id,
                        email: row.email,
                        senha_hash: row.senha_hash,
                        papel,
                        ativo: row.ativo,
                        bloqueado_ate: row.bloqueado_ate,
                    },
                }))
            }
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn revogar(&self, sessao_id: Uuid) -> Result<bool, RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE sessoes SET revogada_em = NOW() \
             WHERE id = $1 AND revogada_em IS NULL",
        )
        .bind::<SqlUuid, _>(sessao_id)
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }

    fn revogar_familia(&self, familia_id: Uuid) -> Result<u64, RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE sessoes SET revogada_em = NOW() \
             WHERE familia_id = $1 AND revogada_em IS NULL",
        )
        .bind::<SqlUuid, _>(familia_id)
        .execute(&mut conexao)
        .map(|linhas| linhas as u64)
        .map_err(|_| RepositorioError::Dados)
    }
}

impl AuditoriaRepositorio for RepositorioDiesel {
    fn registrar(&self, evento: &EventoAuditoria) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        let valor_novo =
            serde_json::to_string(&evento.valor_novo).map_err(|_| RepositorioError::Dados)?;
        let ip = evento.ip.map(|ip| ip.to_string());

        diesel::sql_query(
            "INSERT INTO auditoria_eventos \
             (id, escritorio_id, usuario_id, acao, entidade, entidade_id, valor_novo, ip) \
             VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::inet)",
        )
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<Nullable<SqlUuid>, _>(evento.escritorio_id)
        .bind::<Nullable<SqlUuid>, _>(evento.usuario_id)
        .bind::<Text, _>(evento.acao)
        .bind::<Nullable<Text>, _>(Some(evento.entidade))
        .bind::<Nullable<SqlUuid>, _>(evento.entidade_id)
        .bind::<Text, _>(valor_novo)
        .bind::<Nullable<Text>, _>(ip.as_deref())
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }
}

#[derive(QueryableByName)]
struct AdministradorRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    senha_hash: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    ativo: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    bloqueado_ate: Option<DateTime<Utc>>,
}

#[derive(QueryableByName)]
struct SessaoAdminRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    administrador_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    familia_id: Uuid,
    #[diesel(sql_type = Text)]
    refresh_hash: String,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    expira_em: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    revogada_em: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    ip: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    user_agent: Option<String>,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    senha_hash: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    ativo: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    bloqueado_ate: Option<DateTime<Utc>>,
}

impl AdministradorRepositorio for RepositorioDiesel {
    fn buscar_por_email(&self, email: &str) -> Result<Option<Administrador>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT id, email, senha_hash, ativo, bloqueado_ate \
             FROM administradores WHERE lower(email) = lower($1)",
        )
        .bind::<Text, _>(email)
        .get_result::<AdministradorRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => Ok(Some(Administrador {
                id: row.id,
                email: row.email,
                senha_hash: row.senha_hash,
                ativo: row.ativo,
                bloqueado_ate: row.bloqueado_ate,
            })),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn buscar_por_id(
        &self,
        administrador_id: Uuid,
    ) -> Result<Option<Administrador>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT id, email, senha_hash, ativo, bloqueado_ate \
             FROM administradores WHERE id = $1",
        )
        .bind::<SqlUuid, _>(administrador_id)
        .get_result::<AdministradorRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => Ok(Some(Administrador {
                id: row.id,
                email: row.email,
                senha_hash: row.senha_hash,
                ativo: row.ativo,
                bloqueado_ate: row.bloqueado_ate,
            })),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn registrar_falha(
        &self,
        administrador_id: Uuid,
        max_tentativas: u32,
        bloqueio_secs: u64,
    ) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        let max_tentativas = i32::try_from(max_tentativas).unwrap_or(i32::MAX);
        let bloqueio_secs = i64::try_from(bloqueio_secs).unwrap_or(i64::MAX);

        diesel::sql_query(
            "UPDATE administradores SET falhas_consecutivas = falhas_consecutivas + 1, \
             bloqueado_ate = CASE WHEN falhas_consecutivas + 1 >= $2 \
             THEN NOW() + ($3 * INTERVAL '1 second') ELSE bloqueado_ate END \
             WHERE id = $1",
        )
        .bind::<SqlUuid, _>(administrador_id)
        .bind::<Integer, _>(max_tentativas)
        .bind::<BigInt, _>(bloqueio_secs)
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }

    fn registrar_sucesso(&self, administrador_id: Uuid) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE administradores SET falhas_consecutivas = 0, bloqueado_ate = NULL, \
             ultimo_login_em = NOW() WHERE id = $1",
        )
        .bind::<SqlUuid, _>(administrador_id)
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }
}

impl SessaoAdminRepositorio for RepositorioDiesel {
    fn criar(&self, sessao: &SessaoAdmin) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "INSERT INTO sessoes_admin \
             (id, administrador_id, familia_id, refresh_hash, criado_em, expira_em, \
              revogada_em, ip, user_agent) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9)",
        )
        .bind::<SqlUuid, _>(sessao.id)
        .bind::<SqlUuid, _>(sessao.administrador_id)
        .bind::<SqlUuid, _>(sessao.familia_id)
        .bind::<Text, _>(&sessao.refresh_hash)
        .bind::<Timestamptz, _>(sessao.criado_em)
        .bind::<Timestamptz, _>(sessao.expira_em)
        .bind::<Nullable<Timestamptz>, _>(sessao.revogada_em)
        .bind::<Nullable<Text>, _>(sessao.ip.as_deref())
        .bind::<Nullable<Text>, _>(sessao.user_agent.as_deref())
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }

    fn buscar_por_hash(
        &self,
        refresh_hash: &str,
    ) -> Result<Option<SessaoAdminComAdministrador>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let resultado = diesel::sql_query(
            "SELECT s.id, s.administrador_id, s.familia_id, s.refresh_hash::text AS refresh_hash, \
             s.criado_em, s.expira_em, s.revogada_em, s.ip::text AS ip, s.user_agent, \
             a.email, a.senha_hash, a.ativo, a.bloqueado_ate \
             FROM sessoes_admin s \
             JOIN administradores a ON a.id = s.administrador_id \
             WHERE s.refresh_hash = $1",
        )
        .bind::<Text, _>(refresh_hash)
        .get_result::<SessaoAdminRow>(&mut conexao)
        .optional_result();

        match resultado {
            Ok(Some(row)) => Ok(Some(SessaoAdminComAdministrador {
                sessao: SessaoAdmin {
                    id: row.id,
                    administrador_id: row.administrador_id,
                    familia_id: row.familia_id,
                    refresh_hash: row.refresh_hash,
                    criado_em: row.criado_em,
                    expira_em: row.expira_em,
                    revogada_em: row.revogada_em,
                    ip: row.ip,
                    user_agent: row.user_agent,
                },
                administrador: Administrador {
                    id: row.administrador_id,
                    email: row.email,
                    senha_hash: row.senha_hash,
                    ativo: row.ativo,
                    bloqueado_ate: row.bloqueado_ate,
                },
            })),
            Ok(None) => Ok(None),
            Err(_) => Err(RepositorioError::Dados),
        }
    }

    fn revogar(&self, sessao_id: Uuid) -> Result<bool, RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE sessoes_admin SET revogada_em = NOW() \
             WHERE id = $1 AND revogada_em IS NULL",
        )
        .bind::<SqlUuid, _>(sessao_id)
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }

    fn revogar_familia(&self, familia_id: Uuid) -> Result<u64, RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "UPDATE sessoes_admin SET revogada_em = NOW() \
             WHERE familia_id = $1 AND revogada_em IS NULL",
        )
        .bind::<SqlUuid, _>(familia_id)
        .execute(&mut conexao)
        .map(|linhas| linhas as u64)
        .map_err(|_| RepositorioError::Dados)
    }
}

trait OptionalResult<T> {
    fn optional_result(self) -> Result<Option<T>, diesel::result::Error>;
}

impl<T> OptionalResult<T> for Result<T, diesel::result::Error> {
    fn optional_result(self) -> Result<Option<T>, diesel::result::Error> {
        match self {
            Ok(valor) => Ok(Some(valor)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(erro) => Err(erro),
        }
    }
}
