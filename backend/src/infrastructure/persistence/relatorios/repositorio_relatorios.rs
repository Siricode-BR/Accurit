//! Fase F — Implementação Diesel de `RepositorioRelatorios` e
//! `RepositorioLinksCompartilhamento`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::relatorios::entidades::{
    LinkCompartilhamento, NovoLinkCompartilhamento, NovoRelatorio, Relatorio,
};
use crate::domain::relatorios::enums::{FormatoRelatorio, StatusRelatorio, TipoRelatorio};
use crate::domain::relatorios::erros::ErroRelatorio;
use crate::domain::relatorios::ports::{RepositorioLinksCompartilhamento, RepositorioRelatorios};
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{links_compartilhamento_relatorios, relatorios};

#[derive(Clone)]
pub struct RelatoriosDiesel {
    pool: DbPool,
}

impl RelatoriosDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroRelatorio,
    > {
        self.pool
            .get()
            .map_err(|_| ErroRelatorio::Persistencia("pool indisponível".into()))
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = relatorios)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RelatorioRow {
    id: Uuid,
    escritorio_id: Uuid,
    usuario_id: Uuid,
    cliente_id: Option<Uuid>,
    tipo_relatorio: String,
    formato: String,
    status: String,
    conteudo: Option<Vec<u8>>,
    nome_arquivo: Option<String>,
    competencia: Option<String>,
    expira_em: DateTime<Utc>,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

impl RelatorioRow {
    fn para_entidade(self) -> Result<Relatorio, ErroRelatorio> {
        let tipo_relatorio = match self.tipo_relatorio.as_str() {
            "situacao_fiscal" => TipoRelatorio::SituacaoFiscal,
            "historico_consultas" => TipoRelatorio::HistoricoConsultas,
            "pendencias_carteira" => TipoRelatorio::PendenciasCarteira,
            _ => {
                return Err(ErroRelatorio::Persistencia(
                    "tipo de relatório inválido".into(),
                ));
            }
        };
        let formato = match self.formato.as_str() {
            "PDF" => FormatoRelatorio::Pdf,
            "XLSX" => FormatoRelatorio::Xlsx,
            _ => return Err(ErroRelatorio::Persistencia("formato inválido".into())),
        };
        let status = match self.status.as_str() {
            "gerando" => StatusRelatorio::Gerando,
            "concluido" => StatusRelatorio::Concluido,
            "falhou" => StatusRelatorio::Falhou,
            _ => return Err(ErroRelatorio::Persistencia("status inválido".into())),
        };
        Ok(Relatorio {
            id: self.id,
            escritorio_id: self.escritorio_id,
            usuario_id: self.usuario_id,
            cliente_id: self.cliente_id,
            tipo_relatorio,
            formato,
            status,
            conteudo: self.conteudo,
            nome_arquivo: self.nome_arquivo,
            competencia: self.competencia,
            expira_em: self.expira_em,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        })
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = links_compartilhamento_relatorios)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LinkCompartilhamentoRow {
    id: Uuid,
    relatorio_id: Uuid,
    token: String,
    expira_em: DateTime<Utc>,
    acessado_em: Option<DateTime<Utc>>,
    criado_em: DateTime<Utc>,
}

impl LinkCompartilhamentoRow {
    fn para_entidade(self) -> LinkCompartilhamento {
        LinkCompartilhamento {
            id: self.id,
            relatorio_id: self.relatorio_id,
            token: self.token,
            expira_em: self.expira_em,
            acessado_em: self.acessado_em,
            criado_em: self.criado_em,
        }
    }
}

#[async_trait]
impl RepositorioRelatorios for RelatoriosDiesel {
    async fn criar(&self, relatorio: &NovoRelatorio) -> Result<Relatorio, ErroRelatorio> {
        let pool = self.pool.clone();
        let novo = relatorio.clone();
        tokio::task::spawn_blocking(move || -> Result<Relatorio, ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let linha: Relatorio = diesel::insert_into(relatorios::table)
                .values((
                    relatorios::id.eq(Uuid::now_v7()),
                    relatorios::escritorio_id.eq(novo.escritorio_id),
                    relatorios::usuario_id.eq(novo.usuario_id),
                    relatorios::cliente_id.eq(novo.cliente_id),
                    relatorios::tipo_relatorio.eq(novo.tipo_relatorio.as_str()),
                    relatorios::formato.eq(novo.formato.as_str()),
                    relatorios::status.eq(StatusRelatorio::Gerando.as_str()),
                    relatorios::nome_arquivo.eq::<Option<String>>(None),
                    relatorios::competencia.eq(novo.competencia),
                ))
                .get_result::<RelatorioRow>(&mut conexao)
                .map_err(|_| ErroRelatorio::Persistencia("inserir relatório".into()))?
                .para_entidade()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<Relatorio>, ErroRelatorio> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<Relatorio>, ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let linha = relatorios::table
                .filter(relatorios::id.eq(id))
                .select(RelatorioRow::as_select())
                .first(&mut conexao)
                .optional()
                .map_err(|_| ErroRelatorio::Persistencia("buscar relatório".into()))?;
            let linha = linha.map(|row| row.para_entidade()).transpose()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<Relatorio>, ErroRelatorio> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<Relatorio>, ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let linhas = relatorios::table
                .filter(relatorios::escritorio_id.eq(escritorio_id))
                .order(relatorios::criado_em.desc())
                .limit(limite)
                .select(RelatorioRow::as_select())
                .load(&mut conexao)
                .map_err(|_| ErroRelatorio::Persistencia("listar relatórios".into()))?
                .into_iter()
                .map(|row| row.para_entidade())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn atualizar_status_e_conteudo(
        &self,
        id: Uuid,
        status: StatusRelatorio,
        conteudo: Option<Vec<u8>>,
        nome_arquivo: Option<String>,
    ) -> Result<(), ErroRelatorio> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let atualizados = diesel::update(relatorios::table.filter(relatorios::id.eq(id)))
                .set((
                    relatorios::status.eq(status.as_str()),
                    relatorios::conteudo.eq(conteudo),
                    relatorios::nome_arquivo.eq(nome_arquivo),
                    relatorios::atualizado_em.eq(Utc::now()),
                ))
                .execute(&mut conexao)
                .map_err(|_| ErroRelatorio::Persistencia("atualizar relatório".into()))?;
            if atualizados != 1 {
                return Err(ErroRelatorio::RelatorioNaoEncontrado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }
}

#[async_trait]
impl RepositorioLinksCompartilhamento for RelatoriosDiesel {
    async fn criar(
        &self,
        link: &NovoLinkCompartilhamento,
    ) -> Result<LinkCompartilhamento, ErroRelatorio> {
        let pool = self.pool.clone();
        let novo = link.clone();
        tokio::task::spawn_blocking(move || -> Result<LinkCompartilhamento, ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let linha = diesel::insert_into(links_compartilhamento_relatorios::table)
                .values((
                    links_compartilhamento_relatorios::id.eq(Uuid::now_v7()),
                    links_compartilhamento_relatorios::relatorio_id.eq(novo.relatorio_id),
                    links_compartilhamento_relatorios::token.eq(Uuid::now_v7().to_string()),
                    links_compartilhamento_relatorios::expira_em.eq(novo.expira_em),
                ))
                .get_result::<LinkCompartilhamentoRow>(&mut conexao)
                .map_err(|_| ErroRelatorio::Persistencia("inserir link".into()))?
                .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_token(
        &self,
        token: &str,
    ) -> Result<Option<LinkCompartilhamento>, ErroRelatorio> {
        let pool = self.pool.clone();
        let token_owned = token.to_owned();
        tokio::task::spawn_blocking(
            move || -> Result<Option<LinkCompartilhamento>, ErroRelatorio> {
                let mut conexao = pool
                    .get()
                    .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
                let linha = links_compartilhamento_relatorios::table
                    .filter(links_compartilhamento_relatorios::token.eq(token_owned))
                    .select(LinkCompartilhamentoRow::as_select())
                    .first(&mut conexao)
                    .optional()
                    .map_err(|_| ErroRelatorio::Persistencia("buscar link".into()))?;
                let linha = linha.map(|row: LinkCompartilhamentoRow| row.para_entidade());
                Ok(linha)
            },
        )
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn registrar_acesso(&self, id: Uuid) -> Result<(), ErroRelatorio> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ErroRelatorio> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
            let atualizados = diesel::update(
                links_compartilhamento_relatorios::table
                    .filter(links_compartilhamento_relatorios::id.eq(id)),
            )
            .set(links_compartilhamento_relatorios::acessado_em.eq(Utc::now()))
            .execute(&mut conexao)
            .map_err(|_| ErroRelatorio::Persistencia("registrar acesso".into()))?;
            if atualizados != 1 {
                return Err(ErroRelatorio::LinkNaoEncontrado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_expirados(
        &self,
        agora: DateTime<Utc>,
    ) -> Result<Vec<LinkCompartilhamento>, ErroRelatorio> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(
            move || -> Result<Vec<LinkCompartilhamento>, ErroRelatorio> {
                let mut conexao = pool
                    .get()
                    .map_err(|_| ErroRelatorio::Persistencia("pool".into()))?;
                let linhas = links_compartilhamento_relatorios::table
                    .filter(links_compartilhamento_relatorios::expira_em.lt(agora))
                    .load(&mut conexao)
                    .map_err(|_| ErroRelatorio::Persistencia("listar links expirados".into()))?
                    .into_iter()
                    .map(|row: LinkCompartilhamentoRow| row.para_entidade())
                    .collect();
                Ok(linhas)
            },
        )
        .await
        .map_err(|_| ErroRelatorio::Persistencia("spawn_blocking".into()))?
    }
}
