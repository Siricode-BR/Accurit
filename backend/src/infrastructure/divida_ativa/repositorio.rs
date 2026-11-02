//! Persistência da Fase 4: histórico `consultas_fiscais` (registro por
//! tentativa + expurgo de retenção, SPEC §9.11), catálogo de clientes
//! por documento (vínculo do histórico) e CPF do operador (D3).
//! O resolvedor de chave alternativa D2 fica sem fio por enquanto:
//! o teste live decide se 200 vazio ou 404 (DECISOES-PENDENTES D2).

use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::application::serpro::ports::RepositorioError;
use crate::domain::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, RepositorioConsultasFiscais,
    ResolvedorChaveAlternativa,
};
use crate::domain::divida_ativa::{RegistroConsultaFiscal, mascarar_documento};
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::consultas_fiscais::dsl as dsl_consultas;
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
    fn registrar(&self, registro: &RegistroConsultaFiscal) -> Result<Uuid, RepositorioError> {
        use dsl_consultas::*;

        let novo_id = Uuid::now_v7();
        let mut conexao = self.conexao()?;
        diesel::insert_into(consultas_fiscais)
            .values((
                id.eq(novo_id),
                escritorio_id.eq(registro.escritorio_id),
                cliente_id.eq(registro.cliente_id),
                produto.eq(registro.produto.as_str()),
                servico.eq(registro.servico.as_str()),
                verbo.eq(registro.verbo.as_str()),
                ambiente.eq(registro.ambiente.as_str()),
                chave_consulta_hash.eq(registro.chave_consulta_hash.as_deref()),
                chave_consulta_mascarada.eq(registro.chave_consulta_mascarada.as_deref()),
                fonte_consulta.eq(registro.fonte_consulta.as_deref()),
                status_code.eq(registro.status_code),
                estado.eq(registro.estado.as_str()),
                protocolo.eq(registro.protocolo.as_deref()),
                request_redigido.eq(&registro.request_redigido),
                response_redigido.eq(&registro.response_redigido),
            ))
            .returning(id)
            .get_result(&mut conexao)
            .map_err(|_| RepositorioError::Dados)
    }

    fn expurgar_antes_de(
        &self,
        limite: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, RepositorioError> {
        use dsl_consultas::*;

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

/// D2 sem catálogo: a segunda tentativa nunca acontece quando a flag
/// está habilitada mas não foi conectado um resolvedor real.
#[derive(Clone, Default)]
pub struct ResolvedorChaveAlternativaSemCatalogo;

impl ResolvedorChaveAlternativa for ResolvedorChaveAlternativaSemCatalogo {
    fn resolver(&self, _chave: &crate::domain::clientes::CpfCnpj) -> Option<String> {
        None
    }
}

/// Reexport da máscara do domínio para os testes de infraestrutura.
pub fn mascarar_chave_resultado(digitos: &str) -> String {
    mascarar_documento(digitos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolvedor_sem_catalogo_devolve_none() {
        let chave = crate::domain::clientes::CpfCnpj::novo("52998224725").expect("CPF válido");
        assert_eq!(ResolvedorChaveAlternativaSemCatalogo.resolver(&chave), None);
    }

    #[test]
    fn mascara_reusa_utilitario_do_dominio() {
        assert_eq!(mascarar_chave_resultado("52998224725"), "***.***.247-**");
    }
}
