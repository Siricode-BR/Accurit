use chrono::{DateTime, Utc};
use diesel::prelude::*;
use ipnet::IpNet;
use uuid::Uuid;

use crate::application::usuarios::{
    AlteracaoUsuario, EventoAuditoriaUsuario, FiltroUsuarios, ListagemUsuarios, OrigemAtor,
    RepositorioError, UsuarioDetalhe, UsuarioGestaoNovo, UsuarioGestaoRepositorio,
};
use crate::domain::auth::Papel;
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{
    auditoria_eventos, escritorios, sessoes, usuarios,
};

#[derive(Clone)]
pub struct UsuarioGestaoRepositorioDiesel {
    pool: DbPool,
}

type Linha = (
    Uuid,
    Uuid,
    String,
    String,
    String,
    String,
    String,
    String,
    bool,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
);

impl UsuarioGestaoRepositorioDiesel {
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

fn para_detalhe(l: Linha) -> Result<UsuarioDetalhe, RepositorioError> {
    let papel = match l.7.as_str() {
        "CLIENTE" => Papel::Cliente,
        "OPERADOR" => Papel::Operador,
        _ => return Err(RepositorioError::Dados),
    };
    Ok(UsuarioDetalhe {
        id: l.0,
        escritorio_id: l.1,
        escritorio_nome: l.2,
        escritorio_slug: l.3,
        nome: l.4,
        email: l.5,
        cpf: l.6,
        papel,
        ativo: l.8,
        ultimo_login_em: l.9,
        criado_em: l.10,
    })
}

macro_rules! base {
    () => {
        usuarios::table.inner_join(escritorios::table).select((
            usuarios::id,
            usuarios::escritorio_id,
            escritorios::nome,
            escritorios::slug,
            usuarios::nome,
            usuarios::email,
            usuarios::cpf,
            usuarios::papel,
            usuarios::ativo,
            usuarios::ultimo_login_em,
            usuarios::criado_em,
        ))
    };
}

fn escapar_like(termo: &str) -> String {
    termo
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

impl UsuarioGestaoRepositorio for UsuarioGestaoRepositorioDiesel {
    fn listar(&self, filtro: &FiltroUsuarios) -> Result<Vec<UsuarioDetalhe>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let mut consulta = base!().into_boxed();
        if let Some(esc) = filtro.escritorio_id {
            consulta = consulta.filter(usuarios::escritorio_id.eq(esc));
        }
        if let Some(ativo) = filtro.ativo {
            consulta = consulta.filter(usuarios::ativo.eq(ativo));
        }
        if let Some(busca) = &filtro.busca {
            let padrao = format!("%{}%", escapar_like(busca));
            consulta = consulta.filter(
                usuarios::nome
                    .ilike(padrao.clone())
                    .or(usuarios::email.ilike(padrao.clone()))
                    .or(usuarios::cpf.like(padrao)),
            );
        }
        consulta
            .order((usuarios::nome.asc(), usuarios::email.asc()))
            .load::<Linha>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?
            .into_iter()
            .map(para_detalhe)
            .collect()
    }

    fn listar_paginado(
        &self,
        filtro: &FiltroUsuarios,
    ) -> Result<ListagemUsuarios, RepositorioError> {
        let mut conexao = self.conexao()?;
        let mut consulta = base!().into_boxed();
        if let Some(esc) = filtro.escritorio_id {
            consulta = consulta.filter(usuarios::escritorio_id.eq(esc));
        }
        if let Some(ativo) = filtro.ativo {
            consulta = consulta.filter(usuarios::ativo.eq(ativo));
        }
        if let Some(busca) = &filtro.busca {
            let padrao = format!("%{}%", escapar_like(busca));
            consulta = consulta.filter(
                usuarios::nome
                    .ilike(padrao.clone())
                    .or(usuarios::email.ilike(padrao.clone()))
                    .or(usuarios::cpf.like(padrao)),
            );
        }

        // Total de linhas no mesmo filtro (JOIN removeu, conta direto na tabela).
        let mut contagem = usuarios::table.into_boxed();
        if let Some(esc) = filtro.escritorio_id {
            contagem = contagem.filter(usuarios::escritorio_id.eq(esc));
        }
        if let Some(ativo) = filtro.ativo {
            contagem = contagem.filter(usuarios::ativo.eq(ativo));
        }
        if let Some(busca) = &filtro.busca {
            let padrao = format!("%{}%", escapar_like(busca));
            contagem = contagem.filter(
                usuarios::nome
                    .ilike(padrao.clone())
                    .or(usuarios::email.ilike(padrao.clone()))
                    .or(usuarios::cpf.like(padrao)),
            );
        }
        let total: i64 = contagem
            .count()
            .first(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        let tamanho = filtro.tamanho.unwrap_or(20).clamp(1, 100);
        let pagina = filtro.pagina.unwrap_or(1).max(1);
        let offset = (pagina - 1) * tamanho;

        let itens: Vec<UsuarioDetalhe> = consulta
            .order((usuarios::nome.asc(), usuarios::email.asc()))
            .limit(tamanho)
            .offset(offset)
            .load::<Linha>(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?
            .into_iter()
            .map(para_detalhe)
            .collect::<Result<Vec<UsuarioDetalhe>, RepositorioError>>()?;

        Ok(ListagemUsuarios {
            itens,
            total,
            pagina,
            tamanho,
        })
    }

    fn obter(
        &self,
        id: Uuid,
        escritorio_id: Option<Uuid>,
    ) -> Result<Option<UsuarioDetalhe>, RepositorioError> {
        let mut conexao = self.conexao()?;
        let mut consulta = base!().filter(usuarios::id.eq(id)).into_boxed();
        if let Some(esc) = escritorio_id {
            consulta = consulta.filter(usuarios::escritorio_id.eq(esc));
        }
        consulta
            .first::<Linha>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?
            .map(para_detalhe)
            .transpose()
    }

    fn criar(&self, novo: &UsuarioGestaoNovo) -> Result<Uuid, RepositorioError> {
        use diesel::result::{DatabaseErrorKind, Error};
        let mut conexao = self.conexao()?;
        let id = Uuid::now_v7();
        diesel::insert_into(usuarios::table)
            .values((
                usuarios::id.eq(id),
                usuarios::escritorio_id.eq(novo.escritorio_id),
                usuarios::nome.eq(&novo.nome),
                usuarios::email.eq(&novo.email),
                usuarios::cpf.eq(&novo.cpf),
                usuarios::senha_hash.eq(&novo.senha_hash),
                usuarios::papel.eq(novo.papel.como_str()),
                usuarios::ativo.eq(true),
            ))
            .execute(&mut conexao)
            .map(|_| id)
            .map_err(|e| match e {
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    if info.constraint_name().is_some_and(|c| c.contains("email")) {
                        RepositorioError::EmailExistente
                    } else {
                        RepositorioError::CpfExistente
                    }
                }
                Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                    RepositorioError::EscritorioInexistente
                }
                _ => RepositorioError::Dados,
            })
    }

    fn atualizar(&self, id: Uuid, alt: &AlteracaoUsuario) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        conexao
            .transaction::<_, diesel::result::Error, _>(|c| {
                let alvo = usuarios::table.filter(usuarios::id.eq(id));
                if let Some(nome) = &alt.nome {
                    diesel::update(alvo)
                        .set(usuarios::nome.eq(nome))
                        .execute(c)?;
                }
                if let Some(papel) = alt.papel {
                    diesel::update(alvo)
                        .set(usuarios::papel.eq(papel.como_str()))
                        .execute(c)?;
                }
                if let Some(ativo) = alt.ativo {
                    diesel::update(alvo)
                        .set(usuarios::ativo.eq(ativo))
                        .execute(c)?;
                    if ativo {
                        diesel::update(alvo)
                            .set((
                                usuarios::falhas_consecutivas.eq(0),
                                usuarios::bloqueado_ate.eq(None::<DateTime<Utc>>),
                            ))
                            .execute(c)?;
                    }
                }
                Ok(())
            })
            .map_err(|_| RepositorioError::Dados)
    }

    fn definir_senha(&self, id: Uuid, senha_hash: &str) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::update(usuarios::table.filter(usuarios::id.eq(id)))
            .set((
                usuarios::senha_hash.eq(senha_hash),
                usuarios::falhas_consecutivas.eq(0),
                usuarios::bloqueado_ate.eq(None::<DateTime<Utc>>),
            ))
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
    }

    fn revogar_sessoes(&self, id: Uuid) -> Result<u64, RepositorioError> {
        let mut conexao = self.conexao()?;
        diesel::update(
            sessoes::table
                .filter(sessoes::usuario_id.eq(id))
                .filter(sessoes::revogada_em.is_null()),
        )
        .set(sessoes::revogada_em.eq(Some(Utc::now())))
        .execute(&mut conexao)
        .map(|n| n as u64)
        .map_err(|_| RepositorioError::Dados)
    }

    fn contar_titulares_ativos(
        &self,
        escritorio_id: Uuid,
        excluindo: Uuid,
    ) -> Result<i64, RepositorioError> {
        let mut conexao = self.conexao()?;
        usuarios::table
            .filter(usuarios::escritorio_id.eq(escritorio_id))
            .filter(usuarios::papel.eq("CLIENTE"))
            .filter(usuarios::ativo.eq(true))
            .filter(usuarios::id.ne(excluindo))
            .count()
            .get_result(&mut conexao)
            .map_err(|_| RepositorioError::Dados)
    }

    fn registrar_auditoria(&self, e: &EventoAuditoriaUsuario) -> Result<(), RepositorioError> {
        let mut conexao = self.conexao()?;
        let (usuario, admin) = match e.ator.origem {
            OrigemAtor::Escritorio(_) => (Some(e.ator.id), None),
            OrigemAtor::Backoffice => (None, Some(e.ator.id)),
        };
        diesel::insert_into(auditoria_eventos::table)
            .values((
                auditoria_eventos::id.eq(Uuid::now_v7()),
                auditoria_eventos::escritorio_id.eq(e.escritorio_id),
                auditoria_eventos::usuario_id.eq(usuario),
                auditoria_eventos::administrador_id.eq(admin),
                auditoria_eventos::acao.eq(e.acao),
                auditoria_eventos::entidade.eq(Some("usuario")),
                auditoria_eventos::entidade_id.eq(Some(e.usuario_alvo_id)),
                auditoria_eventos::valor_anterior.eq(e.valor_anterior.clone()),
                auditoria_eventos::valor_novo.eq(e.valor_novo.clone()),
                auditoria_eventos::ip.eq(e.ator.ip.map(IpNet::from)),
            ))
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
    }
}
