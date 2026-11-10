use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use ipnet::IpNet;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::application::admin::{
    AdministradorAtualizar, AdministradorDto, AdministradorNovo, AdministradorRepositorio,
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, Paginacao, RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{
    administradores, auditoria_eventos, escritorios, usuarios,
};

diesel::define_sql_function! {
    /// `lower(text)` do PostgreSQL — o e-mail é único sem diferenciar maiúsculas.
    fn lower(valor: diesel::sql_types::Text) -> diesel::sql_types::Text;
}

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
                cnpj.eq(&escritorio.cnpj),
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

    fn buscar_por_cnpj(&self, cnpj_alvo: &str) -> Result<Option<Uuid>, RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        escritorios
            .filter(cnpj.eq(cnpj_alvo))
            .select(id)
            .first(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }

    fn listar(
        &self,
        paginacao: &Paginacao,
    ) -> Result<(Vec<EscritorioResumo>, i64), RepositorioError> {
        use self::escritorios::dsl::*;

        let mut conexao = self.conexao()?;
        let limite = i64::from(paginacao.por_pagina);
        let deslocamento = i64::from(paginacao.pagina - 1) * limite;

        let total = escritorios
            .count()
            .get_result::<i64>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        let itens = escritorios
            .order_by((atualizado_em.desc(), id.asc()))
            .limit(limite)
            .offset(deslocamento)
            .select((id, cnpj, nome, modo_trial_serpro, versao, atualizado_em))
            .load::<(
                Uuid,
                String,
                String,
                bool,
                i64,
                chrono::DateTime<chrono::Utc>,
            )>(&mut conexao)
            .map(|linhas| {
                linhas
                    .into_iter()
                    .map(
                        |(
                            id_linha,
                            cnpj_linha,
                            nome_linha,
                            modo_trial,
                            versao_linha,
                            renovacao,
                        )| {
                            EscritorioResumo {
                                id: id_linha,
                                cnpj: cnpj_linha,
                                nome: nome_linha,
                                modo_trial_serpro: modo_trial,
                                versao: versao_linha,
                                atualizado_em: renovacao,
                            }
                        },
                    )
                    .collect()
            })
            .map_err(|_| RepositorioError::Dados)?;

        Ok((itens, total))
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

    fn buscar_por_email(&self, email_alvo: &str) -> Result<Option<Uuid>, RepositorioError> {
        use self::usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        usuarios
            .filter(lower(email).eq(email_alvo.to_ascii_lowercase()))
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

/// Linha de administrador na ordem do SELECT auxiliar.
type LinhaAdministrador = (
    Uuid,
    String,
    String,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
    Option<chrono::DateTime<chrono::Utc>>,
);

fn administrador_de(linha: LinhaAdministrador) -> AdministradorDto {
    AdministradorDto {
        id: linha.0,
        nome: linha.1,
        email: linha.2,
        cpf: linha.3,
        ativo: linha.4,
        criado_em: linha.5,
        atualizado_em: linha.5,
        ultimo_login_em: linha.6,
    }
}

/// Tuple de colunas padrão; deve ser usado onde
/// `use self::administradores::dsl::*` estiver no escopo.
macro_rules! base_administrador {
    () => {
        (id, nome, email, cpf, ativo, criado_em, ultimo_login_em)
    };
}

impl AdministradorRepositorio for AdminRepositorioDiesel {
    fn criar(
        &self,
        admin: &AdministradorNovo,
        senha_hash_nova: &str,
    ) -> Result<Uuid, RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;
        let administrador_id = Uuid::now_v7();

        diesel::insert_into(administradores)
            .values((
                id.eq(administrador_id),
                nome.eq(&admin.nome),
                email.eq(&admin.email),
                cpf.eq(&admin.cpf),
                senha_hash.eq(senha_hash_nova),
                ativo.eq(true),
                falhas_consecutivas.eq(0),
                criado_em.eq(chrono::Utc::now()),
            ))
            .execute(&mut conexao)
            .map(|_| administrador_id)
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

    fn listar(
        &self,
        paginacao: &Paginacao,
    ) -> Result<(Vec<AdministradorDto>, usize), RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;
        let limite = i64::from(paginacao.por_pagina);
        let deslocamento = i64::from(paginacao.pagina - 1) * limite;

        let total = administradores
            .count()
            .get_result::<i64>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        let linhas = administradores
            .select(base_administrador!())
            .order_by((criado_em.desc(), id.asc()))
            .limit(limite)
            .offset(deslocamento)
            .load::<LinhaAdministrador>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        let itens = linhas.into_iter().map(administrador_de).collect();
        Ok((itens, total as usize))
    }

    fn buscar_por_id(&self, id_alvo: Uuid) -> Result<Option<AdministradorDto>, RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;
        administradores
            .select(base_administrador!())
            .filter(id.eq(id_alvo))
            .first::<LinhaAdministrador>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
            .map(|opcao| opcao.map(administrador_de))
    }

    fn buscar_por_email(
        &self,
        email_alvo: &str,
    ) -> Result<Option<AdministradorDto>, RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;
        administradores
            .select(base_administrador!())
            .filter(lower(email).eq(email_alvo.to_ascii_lowercase()))
            .first::<LinhaAdministrador>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
            .map(|opcao| opcao.map(administrador_de))
    }

    fn atualizar(
        &self,
        id_alvo: Uuid,
        mudancas: &AdministradorAtualizar,
    ) -> Result<Option<AdministradorDto>, RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;
        let alvo = administradores.filter(id.eq(id_alvo));

        // Atribuição condicional: só as colunas presentes no PATCH mudam.
        let atualizado = match (&mudancas.nome, &mudancas.email) {
            (Some(nome_novo), Some(email_novo)) => diesel::update(alvo)
                .set((nome.eq(nome_novo), email.eq(email_novo)))
                .execute(&mut conexao),
            (Some(nome_novo), None) => diesel::update(alvo)
                .set(nome.eq(nome_novo))
                .execute(&mut conexao),
            (None, Some(email_novo)) => diesel::update(alvo)
                .set(email.eq(email_novo))
                .execute(&mut conexao),
            (None, None) => Ok(0),
        };
        atualizado.map_err(|e| {
            if let diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) = e
            {
                RepositorioError::UsuarioExistente
            } else {
                RepositorioError::Dados
            }
        })?;

        administradores
            .select(base_administrador!())
            .filter(id.eq(id_alvo))
            .first::<LinhaAdministrador>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
            .map(|opcao| opcao.map(administrador_de))
    }

    fn alterar_senha(
        &self,
        id_alvo: Uuid,
        senha_hash_nova: &str,
    ) -> Result<Option<AdministradorDto>, RepositorioError> {
        use self::administradores::dsl::*;

        let mut conexao = self.conexao()?;

        diesel::update(administradores.filter(id.eq(id_alvo)))
            .set(senha_hash.eq(senha_hash_nova))
            .execute(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        administradores
            .select(base_administrador!())
            .filter(id.eq(id_alvo))
            .first::<LinhaAdministrador>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
            .map(|opcao| opcao.map(administrador_de))
    }
}
