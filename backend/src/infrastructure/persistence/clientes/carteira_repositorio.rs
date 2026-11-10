//! Fase E — Implementação Diesel de `RepositorioCarteiraClientes` e
//! `RepositorioAcessoUsuarios` (sócios/contatos, filiais, tags, carteira e
//! funções de acesso). Segue o estilo do `repositorio.rs`: glob de colunas no
//! corpo da função e a tabela sempre pelo caminho completo (as constantes
//! `SOCIO_CONTATOS` etc. evitam a ambiguidade do nome do módulo).

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use crate::application::clientes::{
    RepositorioAcessoUsuarios, RepositorioCarteiraClientes, RepositorioError,
};
use crate::domain::auth::Papel;
use crate::domain::clientes::{
    AtualizarCarteiraCliente, CarteiraCliente, ClienteFilial, ClienteTag, CpfCnpj, FuncaoUsuario,
    NovaClienteTag, NovoClienteFilial, NovoSocioContato, PapelSocioContato, SocioContato,
};

use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema;

const SOCIO_CONTATOS: schema::socio_contatos::table = schema::socio_contatos::table;
const CLIENTE_FILIAIS: schema::cliente_filiais::table = schema::cliente_filiais::table;
const CLIENTE_TAGS: schema::cliente_tags::table = schema::cliente_tags::table;
const CLIENTE_TAGS_VINCULOS: schema::cliente_tags_vinculos::table =
    schema::cliente_tags_vinculos::table;
const CARTEIRA_CLIENTES: schema::carteira_clientes::table = schema::carteira_clientes::table;
const CARTEIRA_USUARIOS: schema::carteira_usuarios::table = schema::carteira_usuarios::table;
const FUNCOES_USUARIOS: schema::funcoes_usuarios::table = schema::funcoes_usuarios::table;
const CLIENTES: schema::clientes::table = schema::clientes::table;

#[derive(Clone)]
pub struct CarteiraClientesRepositorioDiesel {
    pool: DbPool,
}

impl CarteiraClientesRepositorioDiesel {
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

// ===== Rows =====

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = schema::socio_contatos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SocioContatoRow {
    id: Uuid,
    escritorio_id: Uuid,
    cliente_id: Uuid,
    papel: String,
    nome: String,
    cpf_cnpj: Option<String>,
    email: Option<String>,
    telefone: Option<String>,
    representante_principal: bool,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

fn para_socio_contato(row: SocioContatoRow) -> SocioContato {
    SocioContato {
        id: row.id,
        escritorio_id: row.escritorio_id,
        cliente_id: row.cliente_id,
        papel: PapelSocioContato::de_str(&row.papel).unwrap_or(PapelSocioContato::Contato),
        nome: row.nome,
        documento: row
            .cpf_cnpj
            .as_ref()
            .map(|d| CpfCnpj::novo(d).expect("documento válido no banco")),
        email: row.email,
        telefone: row.telefone,
        representante_principal: row.representante_principal,
        criado_em: row.criado_em,
        atualizado_em: row.atualizado_em,
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = schema::cliente_filiais)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct FilialRow {
    id: Uuid,
    escritorio_id: Uuid,
    cliente_id: Uuid,
    cnpj: String,
    nome_unidade: Option<String>,
    endereco: Option<String>,
    telefone: Option<String>,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

fn para_filial(row: FilialRow) -> ClienteFilial {
    ClienteFilial {
        id: row.id,
        escritorio_id: row.escritorio_id,
        cliente_id: row.cliente_id,
        cnpj: CpfCnpj::novo(&row.cnpj).expect("CNPJ válido no banco"),
        nome_unidade: row.nome_unidade,
        endereco: row.endereco,
        telefone: row.telefone,
        criado_em: row.criado_em,
        atualizado_em: row.atualizado_em,
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = schema::cliente_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TagRow {
    id: Uuid,
    escritorio_id: Uuid,
    nome: String,
    cor: Option<String>,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

fn para_tag(row: TagRow) -> ClienteTag {
    ClienteTag {
        id: row.id,
        escritorio_id: row.escritorio_id,
        nome: row.nome,
        cor: row.cor,
        criado_em: row.criado_em,
        atualizado_em: row.atualizado_em,
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = schema::carteira_clientes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CarteiraClienteRow {
    cliente_id: Uuid,
    escritorio_id: Uuid,
    responsavel_usuario_id: Option<Uuid>,
    observacoes: Option<String>,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

fn para_carteira(row: CarteiraClienteRow) -> CarteiraCliente {
    CarteiraCliente {
        cliente_id: row.cliente_id,
        escritorio_id: row.escritorio_id,
        responsavel_usuario_id: row.responsavel_usuario_id,
        observacoes: row.observacoes,
        criado_em: row.criado_em,
        atualizado_em: row.atualizado_em,
    }
}

// ===== RepositorioCarteiraClientes =====

impl RepositorioCarteiraClientes for CarteiraClientesRepositorioDiesel {
    fn criar_socio_contato(
        &self,
        novo: &NovoSocioContato,
    ) -> Result<SocioContato, RepositorioError> {
        use schema::socio_contatos::dsl::*;

        let mut conexao = self.conexao()?;
        let agora = Utc::now();
        let id_novo = Uuid::now_v7();
        // O representante principal é único por cliente: demove os outros
        // ANTES de inserir (índice parcial impõe a unicidade).
        if novo.representante_principal {
            diesel::update(SOCIO_CONTATOS.filter(cliente_id.eq(novo.cliente_id)))
                .set(representante_principal.eq(false))
                .execute(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;
        }
        diesel::insert_into(SOCIO_CONTATOS)
            .values((
                id.eq(id_novo),
                escritorio_id.eq(novo.escritorio_id),
                cliente_id.eq(novo.cliente_id),
                papel.eq(novo.papel.como_str()),
                nome.eq(novo.nome.trim()),
                cpf_cnpj.eq(novo.documento.as_ref().map(|d| d.como_str())),
                email.eq(novo
                    .email
                    .as_deref()
                    .map(str::trim)
                    .filter(|e| !e.is_empty())),
                telefone.eq(novo
                    .telefone
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())),
                representante_principal.eq(novo.representante_principal),
                criado_em.eq(agora),
                atualizado_em.eq(agora),
            ))
            .execute(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;
        Ok(SocioContato {
            id: id_novo,
            escritorio_id: novo.escritorio_id,
            cliente_id: novo.cliente_id,
            papel: novo.papel,
            nome: novo.nome.trim().to_owned(),
            documento: novo.documento.clone(),
            email: novo.email.clone(),
            telefone: novo.telefone.clone(),
            representante_principal: novo.representante_principal,
            criado_em: agora,
            atualizado_em: agora,
        })
    }

    fn listar_socios_contatos(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
    ) -> Result<Vec<SocioContato>, RepositorioError> {
        use schema::socio_contatos::dsl::*;

        let mut conexao = self.conexao()?;
        SOCIO_CONTATOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cliente_id.eq(cliente_id_alvo))
            .order(criado_em)
            .select(SocioContatoRow::as_select())
            .load(&mut conexao)
            .map(|rows| rows.into_iter().map(para_socio_contato).collect())
            .map_err(|_| RepositorioError::Dados)
    }

    fn remover_socio_contato(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
        id_alvo: Uuid,
    ) -> Result<bool, RepositorioError> {
        use schema::socio_contatos::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::delete(
            SOCIO_CONTATOS
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(cliente_id.eq(cliente_id_alvo))
                .filter(id.eq(id_alvo)),
        )
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }

    fn criar_filial(&self, novo: &NovoClienteFilial) -> Result<ClienteFilial, RepositorioError> {
        use schema::cliente_filiais::dsl::*;

        let mut conexao = self.conexao()?;
        let agora = Utc::now();
        let id_novo = Uuid::now_v7();
        // CNPJ único dentro do cliente.
        let duplicado = CLIENTE_FILIAIS
            .filter(cliente_id.eq(novo.cliente_id))
            .filter(cnpj.eq(novo.cnpj.como_str()))
            .first::<FilialRow>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?
            .is_some();
        if duplicado {
            return Err(RepositorioError::Duplicado);
        }
        diesel::insert_into(CLIENTE_FILIAIS)
            .values((
                id.eq(id_novo),
                escritorio_id.eq(novo.escritorio_id),
                cliente_id.eq(novo.cliente_id),
                cnpj.eq(novo.cnpj.como_str()),
                nome_unidade.eq(novo
                    .nome_unidade
                    .as_deref()
                    .map(str::trim)
                    .filter(|n| !n.is_empty())),
                endereco.eq(novo
                    .endereco
                    .as_deref()
                    .map(str::trim)
                    .filter(|e| !e.is_empty())),
                telefone.eq(novo
                    .telefone
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())),
                criado_em.eq(agora),
                atualizado_em.eq(agora),
            ))
            .execute(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;
        Ok(ClienteFilial {
            id: id_novo,
            escritorio_id: novo.escritorio_id,
            cliente_id: novo.cliente_id,
            cnpj: novo.cnpj.clone(),
            nome_unidade: novo.nome_unidade.clone(),
            endereco: novo.endereco.clone(),
            telefone: novo.telefone.clone(),
            criado_em: agora,
            atualizado_em: agora,
        })
    }

    fn listar_filiais(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
    ) -> Result<Vec<ClienteFilial>, RepositorioError> {
        use schema::cliente_filiais::dsl::*;

        let mut conexao = self.conexao()?;
        CLIENTE_FILIAIS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cliente_id.eq(cliente_id_alvo))
            .order(criado_em)
            .select(FilialRow::as_select())
            .load(&mut conexao)
            .map(|rows| rows.into_iter().map(para_filial).collect())
            .map_err(|_| RepositorioError::Dados)
    }

    fn remover_filial(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
        id_alvo: Uuid,
    ) -> Result<bool, RepositorioError> {
        use schema::cliente_filiais::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::delete(
            CLIENTE_FILIAIS
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(cliente_id.eq(cliente_id_alvo))
                .filter(id.eq(id_alvo)),
        )
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }

    fn criar_tag(&self, nova: &NovaClienteTag) -> Result<ClienteTag, RepositorioError> {
        use schema::cliente_tags::dsl::*;

        let mut conexao = self.conexao()?;
        let agora = Utc::now();
        let id_novo = Uuid::now_v7();
        let nome_norm = nova.nome.trim().to_ascii_lowercase();
        // Nome único no escritório (case-insensitive).
        let nomes: Vec<String> = CLIENTE_TAGS
            .filter(escritorio_id.eq(nova.escritorio_id))
            .select(nome)
            .load(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;
        if nomes
            .iter()
            .any(|n| n.trim().to_ascii_lowercase() == nome_norm)
        {
            return Err(RepositorioError::Duplicado);
        }
        diesel::insert_into(CLIENTE_TAGS)
            .values((
                id.eq(id_novo),
                escritorio_id.eq(nova.escritorio_id),
                nome.eq(nova.nome.trim()),
                cor.eq(nova.cor.as_deref().map(str::trim).filter(|c| !c.is_empty())),
                criado_em.eq(agora),
                atualizado_em.eq(agora),
            ))
            .execute(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;
        Ok(ClienteTag {
            id: id_novo,
            escritorio_id: nova.escritorio_id,
            nome: nova.nome.trim().to_owned(),
            cor: nova.cor.clone(),
            criado_em: agora,
            atualizado_em: agora,
        })
    }

    fn listar_tags(&self, escritorio_id_alvo: Uuid) -> Result<Vec<ClienteTag>, RepositorioError> {
        use schema::cliente_tags::dsl::*;

        let mut conexao = self.conexao()?;
        CLIENTE_TAGS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .order(criado_em)
            .select(TagRow::as_select())
            .load(&mut conexao)
            .map(|rows| rows.into_iter().map(para_tag).collect())
            .map_err(|_| RepositorioError::Dados)
    }

    fn tags_do_cliente(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
    ) -> Result<Vec<ClienteTag>, RepositorioError> {
        use schema::cliente_tags_vinculos::dsl::*;

        let mut conexao = self.conexao()?;
        let vinculadas: Vec<Uuid> = CLIENTE_TAGS_VINCULOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cliente_id.eq(cliente_id_alvo))
            .select(tag_id)
            .load(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;
        if vinculadas.is_empty() {
            return Ok(Vec::new());
        }
        self.buscar_tags_por_ids(escritorio_id_alvo, &vinculadas)
    }

    fn vincular_tag(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
        tag_id_alvo: Uuid,
    ) -> Result<(), RepositorioError> {
        use schema::cliente_tags::dsl as tag_cols;
        use schema::cliente_tags_vinculos::dsl::*;

        let mut conexao = self.conexao()?;
        // Só vincula se a tag pertencer ao mesmo escritório.
        let pertence = CLIENTE_TAGS
            .filter(tag_cols::id.eq(tag_id_alvo))
            .filter(tag_cols::escritorio_id.eq(escritorio_id_alvo))
            .select(tag_cols::id)
            .first::<Uuid>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?
            .is_some();
        if !pertence {
            return Err(RepositorioError::NaoEncontrado);
        }
        diesel::insert_into(CLIENTE_TAGS_VINCULOS)
            .values((
                id.eq(Uuid::now_v7()),
                escritorio_id.eq(escritorio_id_alvo),
                tag_id.eq(tag_id_alvo),
                cliente_id.eq(cliente_id_alvo),
                criado_em.eq(Utc::now()),
            ))
            .on_conflict_do_nothing()
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
    }

    fn desvincular_tag(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
        tag_id_alvo: Uuid,
    ) -> Result<bool, RepositorioError> {
        use schema::cliente_tags_vinculos::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::delete(
            CLIENTE_TAGS_VINCULOS
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(cliente_id.eq(cliente_id_alvo))
                .filter(tag_id.eq(tag_id_alvo)),
        )
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }

    fn clientes_com_tag(
        &self,
        escritorio_id_alvo: Uuid,
        tag_id_alvo: Uuid,
    ) -> Result<Vec<Uuid>, RepositorioError> {
        use schema::cliente_tags_vinculos::dsl::*;

        let mut conexao = self.conexao()?;
        CLIENTE_TAGS_VINCULOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(tag_id.eq(tag_id_alvo))
            .select(cliente_id)
            .load(&mut conexao)
            .map_err(|_| RepositorioError::Dados)
    }

    fn carteira_do_cliente(
        &self,
        escritorio_id_alvo: Uuid,
        cliente_id_alvo: Uuid,
    ) -> Result<Option<CarteiraCliente>, RepositorioError> {
        use schema::carteira_clientes::dsl::*;

        let mut conexao = self.conexao()?;
        CARTEIRA_CLIENTES
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cliente_id.eq(cliente_id_alvo))
            .select(CarteiraClienteRow::as_select())
            .first(&mut conexao)
            .optional()
            .map(|opt| opt.map(para_carteira))
            .map_err(|_| RepositorioError::Dados)
    }

    fn definir_carteira(
        &self,
        cliente_id_alvo: Uuid,
        escritorio_id_alvo: Uuid,
        atualizacao: &AtualizarCarteiraCliente,
    ) -> Result<CarteiraCliente, RepositorioError> {
        use schema::carteira_clientes::dsl::*;

        let mut conexao = self.conexao()?;
        let existente: Option<CarteiraClienteRow> = CARTEIRA_CLIENTES
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cliente_id.eq(cliente_id_alvo))
            .select(CarteiraClienteRow::as_select())
            .first(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?;

        let novo_responsavel = if atualizacao.limpar_responsavel {
            None
        } else {
            match atualizacao.responsavel_usuario_id {
                Some(id) => Some(id),
                None => existente.as_ref().and_then(|e| e.responsavel_usuario_id),
            }
        };
        let novas_observacoes = if atualizacao.limpar_observacoes {
            None
        } else {
            match atualizacao.observacoes.as_ref() {
                Some(obs) => Some(obs.clone()),
                None => existente.as_ref().and_then(|e| e.observacoes.clone()),
            }
        };

        match existente {
            Some(_) => {
                diesel::update(
                    CARTEIRA_CLIENTES
                        .filter(escritorio_id.eq(escritorio_id_alvo))
                        .filter(cliente_id.eq(cliente_id_alvo)),
                )
                .set((
                    responsavel_usuario_id.eq(novo_responsavel),
                    observacoes.eq(novas_observacoes),
                    atualizado_em.eq(Utc::now()),
                ))
                .execute(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;
            }
            None => {
                diesel::insert_into(CARTEIRA_CLIENTES)
                    .values((
                        cliente_id.eq(cliente_id_alvo),
                        escritorio_id.eq(escritorio_id_alvo),
                        responsavel_usuario_id.eq(novo_responsavel),
                        observacoes.eq(novas_observacoes),
                        criado_em.eq(Utc::now()),
                        atualizado_em.eq(Utc::now()),
                    ))
                    .execute(&mut conexao)
                    .map_err(|_| RepositorioError::Dados)?;
            }
        }

        self.carteira_do_cliente(escritorio_id_alvo, cliente_id_alvo)?
            .ok_or(RepositorioError::Dados)
    }
}

impl CarteiraClientesRepositorioDiesel {
    fn buscar_tags_por_ids(
        &self,
        escritorio_alvo: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<ClienteTag>, RepositorioError> {
        use schema::cliente_tags::dsl::*;

        let mut conexao = self.conexao()?;
        CLIENTE_TAGS
            .filter(escritorio_id.eq(escritorio_alvo))
            .filter(id.eq_any(ids.to_vec()))
            .order(criado_em)
            .select(TagRow::as_select())
            .load(&mut conexao)
            .map(|rows| rows.into_iter().map(para_tag).collect())
            .map_err(|_| RepositorioError::Dados)
    }
}

// ===== RepositorioAcessoUsuarios =====

impl RepositorioAcessoUsuarios for CarteiraClientesRepositorioDiesel {
    fn funcao_efetiva(
        &self,
        escritorio_id_alvo: Uuid,
        usuario_id_alvo: Uuid,
        papel: Papel,
    ) -> Result<FuncaoUsuario, RepositorioError> {
        Ok(self
            .funcao_declarada(escritorio_id_alvo, usuario_id_alvo)?
            .unwrap_or(match papel {
                Papel::Cliente => FuncaoUsuario::Gestor,
                // Papéis fora do portal tratam-se como leitura (menor privilégio).
                Papel::Operador | Papel::Admin => FuncaoUsuario::AnalistaFiscal,
            }))
    }

    fn funcao_declarada(
        &self,
        escritorio_id_alvo: Uuid,
        usuario_id_alvo: Uuid,
    ) -> Result<Option<FuncaoUsuario>, RepositorioError> {
        use schema::funcoes_usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        FUNCOES_USUARIOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(usuario_id.eq(usuario_id_alvo))
            .select(funcao)
            .first::<String>(&mut conexao)
            .optional()
            .map(|opt| opt.and_then(|f| FuncaoUsuario::de_str(&f)))
            .map_err(|_| RepositorioError::Dados)
    }

    fn carteira_do_usuario(
        &self,
        escritorio_id_alvo: Uuid,
        usuario_id_alvo: Uuid,
    ) -> Result<Vec<Uuid>, RepositorioError> {
        use schema::carteira_usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        CARTEIRA_USUARIOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(usuario_id.eq(usuario_id_alvo))
            .select(cliente_id)
            .load(&mut conexao)
            .map_err(|_| RepositorioError::Dados)
    }

    fn definir_funcao(
        &self,
        escritorio_id_alvo: Uuid,
        usuario_id_alvo: Uuid,
        nova_funcao: FuncaoUsuario,
    ) -> Result<(), RepositorioError> {
        use schema::funcoes_usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        let existente = FUNCOES_USUARIOS
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(usuario_id.eq(usuario_id_alvo))
            .select(usuario_id)
            .first::<Uuid>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)?
            .is_some();
        if existente {
            diesel::update(
                FUNCOES_USUARIOS
                    .filter(escritorio_id.eq(escritorio_id_alvo))
                    .filter(usuario_id.eq(usuario_id_alvo)),
            )
            .set((
                funcao.eq(nova_funcao.como_str()),
                atualizado_em.eq(Utc::now()),
            ))
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
        } else {
            diesel::insert_into(FUNCOES_USUARIOS)
                .values((
                    usuario_id.eq(usuario_id_alvo),
                    escritorio_id.eq(escritorio_id_alvo),
                    funcao.eq(nova_funcao.como_str()),
                    criado_em.eq(Utc::now()),
                    atualizado_em.eq(Utc::now()),
                ))
                .execute(&mut conexao)
                .map(|_| ())
                .map_err(|_| RepositorioError::Dados)
        }
    }

    fn definir_carteira_usuario(
        &self,
        escritorio_id_alvo: Uuid,
        usuario_id_alvo: Uuid,
        ids: &[Uuid],
    ) -> Result<(), RepositorioError> {
        use schema::carteira_usuarios::dsl::*;

        let mut conexao = self.conexao()?;
        // Todos os clientes informados devem pertencer ao mesmo escritório.
        let distintos: Vec<Uuid> = ids
            .iter()
            .copied()
            .collect::<std::collections::HashSet<Uuid>>()
            .into_iter()
            .collect();
        if !distintos.is_empty() {
            use schema::clientes::dsl as cliente_cols;
            let validos: Vec<Uuid> = CLIENTES
                .filter(cliente_cols::escritorio_id.eq(escritorio_id_alvo))
                .filter(cliente_cols::id.eq_any(distintos.clone()))
                .select(cliente_cols::id)
                .load(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;
            if validos.len() != distintos.len() {
                return Err(RepositorioError::Dados);
            }
        }
        diesel::delete(
            CARTEIRA_USUARIOS
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(usuario_id.eq(usuario_id_alvo)),
        )
        .execute(&mut conexao)
        .map_err(|_| RepositorioError::Dados)?;
        for id_cliente in &distintos {
            diesel::insert_into(CARTEIRA_USUARIOS)
                .values((
                    id.eq(Uuid::now_v7()),
                    escritorio_id.eq(escritorio_id_alvo),
                    usuario_id.eq(usuario_id_alvo),
                    cliente_id.eq(*id_cliente),
                    criado_em.eq(Utc::now()),
                ))
                .execute(&mut conexao)
                .map_err(|_| RepositorioError::Dados)?;
        }
        Ok(())
    }
}
