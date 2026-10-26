//! Persistência da Fase 4: histórico `consultas_fiscais` (registro
//! por tentativa + expurgo de retenção, SPEC §9.11), catálogo de
//! clientes por documento (vínculo do histórico) e CPF do operador.
//! O resolvedor de chave alternativa D2 fica sem fio por enquanto:
//! o teste live decide se 200 vazio ou 404 (DECISOES-PENDENTES D2).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::application::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, RegistroConsultaFiscal,
    RepositorioConsultasFiscais, ResolvedorChaveAlternativa,
};
use crate::application::serpro::ports::RepositorioError;
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{clientes, usuarios};

#[derive(Clone)]
pub struct ConsultasFiscaisRepositorioDiesel {
    pool: DbPool,
}

impl ConsultasFiscaisRepositorioDiesel {
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

impl RepositorioConsultasFiscais for ConsultasFiscaisRepositorioDiesel {
    fn registrar(&self, registro: RegistroConsultaFiscal) -> Result<Uuid, RepositorioError> {
        use crate::infrastructure::persistence::schema::consultas_fiscais::dsl::*;

        let novo_id = Uuid::now_v7();
        let mut conexao = self.conexao()?;
        diesel::insert_into(consultas_fiscais)
            .values((
                id.eq(novo_id),
                escritorio_id.eq(registro.escritorio_id),
                cliente_id.eq(registro.cliente_id),
                produto.eq(registro.produto),
                servico.eq(registro.servico),
                verbo.eq(registro.verbo),
                ambiente.eq(registro.ambiente),
                chave_consulta_hash.eq(registro.chave_consulta_hash),
                chave_consulta_mascarada.eq(registro.chave_consulta_mascarada),
                fonte_consulta.eq(registro.fonte_consulta),
                status_code.eq(registro.status_code),
                estado.eq(registro.estado),
                protocolo.eq(registro.protocolo),
                request_redigido.eq(registro.request_redigido),
                response_redigido.eq(registro.response_redigido),
            ))
            .returning(id)
            .get_result(&mut conexao)
            .map_err(|_| RepositorioError::Dados)
    }

    fn expurgar_antes_de(&self, limite: DateTime<Utc>) -> Result<u64, RepositorioError> {
        use crate::infrastructure::persistence::schema::consultas_fiscais::dsl::*;

        let mut conexao = self.conexao()?;
        diesel::delete(consultas_fiscais.filter(criado_em.lt(limite)))
            .execute(&mut conexao)
            .map(|numero| u64::try_from(numero).unwrap_or(u64::MAX))
            .map_err(|_| RepositorioError::Dados)
    }
}

#[derive(Clone)]
pub struct CatalogoClientesDiesel {
    pool: DbPool,
}

impl CatalogoClientesDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CatalogoClientesPorDocumento for CatalogoClientesDiesel {
    fn buscar_por_documento(
        &self,
        escritorio_id_param: Uuid,
        documento_digitos: &str,
    ) -> Result<Option<Uuid>, RepositorioError> {
        let mut conexao = self
            .pool
            .get()
            .map_err(|_| RepositorioError::Indisponivel)?;
        clientes::dsl::clientes
            .filter(
                clientes::dsl::cpf_cnpj
                    .eq(documento_digitos)
                    .and(clientes::dsl::escritorio_id.eq(escritorio_id_param)),
            )
            .select(clientes::dsl::id)
            .first::<Uuid>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }
}

#[derive(Clone)]
pub struct CatalogoOperadoresDiesel {
    pool: DbPool,
}

impl CatalogoOperadoresDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CatalogoOperadores for CatalogoOperadoresDiesel {
    fn buscar_cpf(&self, usuario_id_param: Uuid) -> Result<Option<String>, RepositorioError> {
        let mut conexao = self
            .pool
            .get()
            .map_err(|_| RepositorioError::Indisponivel)?;
        usuarios::dsl::usuarios
            .filter(usuarios::dsl::id.eq(usuario_id_param))
            .select(usuarios::dsl::cpf)
            .first::<String>(&mut conexao)
            .optional()
            .map_err(|_| RepositorioError::Dados)
    }
}

#[derive(Clone, Default)]
pub struct ResolvedorChaveAlternativaSemCatalogo;

#[async_trait]
impl ResolvedorChaveAlternativa for ResolvedorChaveAlternativaSemCatalogo {
    async fn resolver(
        &self,
        _escritorio_id: Uuid,
        _cliente_id: Option<Uuid>,
        _cpf_cnpj_digitos: &str,
    ) -> Result<Option<String>, RepositorioError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolvedor_sem_catalogo_devolve_none() {
        let resultado = ResolvedorChaveAlternativaSemCatalogo
            .resolver(Uuid::now_v7(), Some(Uuid::nil()), "52998224725")
            .await;
        assert_eq!(resultado, Ok(None));
    }
}
