use std::str::FromStr;

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use ipnet::IpNet;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::application::clientes::{
    AuditoriaClienteRepositorio, ClienteRepositorio, EventoAuditoriaCliente, ListagemClientes,
    RepositorioError,
};
use crate::domain::clientes::{
    Cliente, CpfCnpj, FiltroClientes, Paginacao, PorteRegime, TipoCliente,
};
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::clientes;

#[derive(Clone)]
pub struct ClienteRepositorioDiesel {
    pool: DbPool,
}

impl ClienteRepositorioDiesel {
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

impl ClienteRepositorio for ClienteRepositorioDiesel {
    fn criar(&self, escritorio_id_param: Uuid, cliente: &Cliente) -> Result<(), RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::insert_into(clientes)
            .values((
                id.eq(cliente.id),
                escritorio_id.eq(escritorio_id_param),
                tipo.eq(cliente.tipo.como_str()),
                porte_regime.eq(cliente.porte_regime.map(|p| p.como_str())),
                cpf_cnpj.eq(cliente.documento.como_str()),
                nome_razao_social.eq(&cliente.nome_razao_social),
                nome_fantasia.eq(&cliente.nome_fantasia),
                email.eq(&cliente.email),
                telefone.eq(&cliente.telefone),
                criado_em.eq(cliente.criado_em),
                atualizado_em.eq(cliente.atualizado_em),
            ))
            .execute(&mut conexao)
            .map(|_| ())
            .map_err(|_| RepositorioError::Dados)
    }

    fn buscar_por_id(
        &self,
        escritorio_id_alvo: Uuid,
        id_alvo: Uuid,
    ) -> Result<Option<Cliente>, RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;
        clientes
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(id.eq(id_alvo))
            .select(ClienteRow::as_select())
            .first(&mut conexao)
            .optional()
            .map(|opt| opt.map(para_cliente))
            .map_err(|_| RepositorioError::Dados)
    }

    fn buscar_por_cpf_cnpj(
        &self,
        escritorio_id_alvo: Uuid,
        cpf_cnpj_alvo: &str,
    ) -> Result<Option<Cliente>, RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;
        clientes
            .filter(escritorio_id.eq(escritorio_id_alvo))
            .filter(cpf_cnpj.eq(cpf_cnpj_alvo))
            .select(ClienteRow::as_select())
            .first(&mut conexao)
            .optional()
            .map(|opt| opt.map(para_cliente))
            .map_err(|_| RepositorioError::Dados)
    }

    fn listar(
        &self,
        escritorio_id_alvo: Uuid,
        filtro: FiltroClientes,
        paginacao: Paginacao,
    ) -> Result<ListagemClientes, RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;

        let base_query = || {
            crate::infrastructure::persistence::schema::clientes::table
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .select(ClienteRow::as_select())
                .into_boxed()
        };

        let query_for_count = match filtro {
            FiltroClientes::Todos => base_query(),
            FiltroClientes::Cpf => base_query().filter(tipo.eq("PF")),
            FiltroClientes::CnpjMei => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("MEI")),
            FiltroClientes::CnpjSimplesNacional => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("SIMPLES_NACIONAL")),
            FiltroClientes::CnpjLucroPresumido => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("LUCRO_PRESUMIDO")),
            FiltroClientes::CnpjLucroReal => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("LUCRO_REAL")),
        };

        let total: i64 = query_for_count
            .select(diesel::dsl::count(id))
            .first(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        let query_for_items = match filtro {
            FiltroClientes::Todos => base_query(),
            FiltroClientes::Cpf => base_query().filter(tipo.eq("PF")),
            FiltroClientes::CnpjMei => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("MEI")),
            FiltroClientes::CnpjSimplesNacional => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("SIMPLES_NACIONAL")),
            FiltroClientes::CnpjLucroPresumido => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("LUCRO_PRESUMIDO")),
            FiltroClientes::CnpjLucroReal => base_query()
                .filter(tipo.eq("PJ"))
                .filter(porte_regime.eq("LUCRO_REAL")),
        };

        let itens = query_for_items
            .order(criado_em.desc())
            .limit(paginacao.tamanho)
            .offset(paginacao.deslocamento())
            .load(&mut conexao)
            .map_err(|_| RepositorioError::Dados)?;

        Ok(ListagemClientes {
            itens: itens
                .into_iter()
                .map(para_cliente)
                .map(|c| (&c).into())
                .collect(),
            total,
            pagina: paginacao.pagina,
            tamanho: paginacao.tamanho,
        })
    }

    fn atualizar(
        &self,
        escritorio_id_alvo: Uuid,
        cliente: &Cliente,
    ) -> Result<(), RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::update(
            crate::infrastructure::persistence::schema::clientes::table
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(id.eq(cliente.id)),
        )
        .set((
            tipo.eq(cliente.tipo.como_str()),
            porte_regime.eq(cliente.porte_regime.map(|p| p.como_str())),
            cpf_cnpj.eq(cliente.documento.como_str()),
            nome_razao_social.eq(&cliente.nome_razao_social),
            nome_fantasia.eq(&cliente.nome_fantasia),
            email.eq(&cliente.email),
            telefone.eq(&cliente.telefone),
            atualizado_em.eq(cliente.atualizado_em),
        ))
        .execute(&mut conexao)
        .map(|_| ())
        .map_err(|_| RepositorioError::Dados)
    }

    fn excluir(&self, escritorio_id_alvo: Uuid, id_alvo: Uuid) -> Result<bool, RepositorioError> {
        use crate::infrastructure::persistence::schema::clientes::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::delete(
            crate::infrastructure::persistence::schema::clientes::table
                .filter(escritorio_id.eq(escritorio_id_alvo))
                .filter(id.eq(id_alvo)),
        )
        .execute(&mut conexao)
        .map(|linhas| linhas > 0)
        .map_err(|_| RepositorioError::Dados)
    }
}

impl AuditoriaClienteRepositorio for ClienteRepositorioDiesel {
    fn registrar(&self, evento: &EventoAuditoriaCliente) -> Result<(), RepositorioError> {
        use crate::infrastructure::persistence::schema::auditoria_eventos::dsl::*;

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
                escritorio_id.eq(Some(evento.escritorio_id)),
                usuario_id.eq(Some(evento.usuario_id)),
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

#[derive(diesel::Selectable, diesel::Queryable)]
#[diesel(table_name = clientes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ClienteRow {
    id: Uuid,
    escritorio_id: Uuid,
    tipo: String,
    porte_regime: Option<String>,
    cpf_cnpj: String,
    nome_razao_social: String,
    nome_fantasia: Option<String>,
    email: Option<String>,
    telefone: Option<String>,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

fn para_cliente(row: ClienteRow) -> Cliente {
    Cliente {
        id: row.id,
        escritorio_id: row.escritorio_id,
        tipo: TipoCliente::from_str(&row.tipo).unwrap_or(TipoCliente::Pf),
        porte_regime: row
            .porte_regime
            .and_then(|s| PorteRegime::from_str(&s).ok()),
        documento: CpfCnpj::novo(&row.cpf_cnpj).expect("CPF/CNPJ válido no banco"),
        nome_razao_social: row.nome_razao_social,
        nome_fantasia: row.nome_fantasia,
        email: row.email,
        telefone: row.telefone,
        criado_em: row.criado_em,
        atualizado_em: row.atualizado_em,
    }
}
