//! Etapa 0.2 — caso de uso de gestão de administradores do backoffice.
//!
//! Métodos de leitura/escrita delegam ao `AdministradorRepositorio`; a senha
//! nunca passa em texto plano para além desta camada (hash com Argon2).

use std::sync::Arc;

use uuid::Uuid;

use crate::application::admin::{
    AdministradorAtualizar, AdministradorDto, AdministradorNovo, AdministradorRepositorio,
    Paginacao, RepositorioError,
};
use crate::infrastructure::auth::HashArgon2;

#[derive(Clone)]
pub struct GestaoAdministradoresUseCase {
    repositorio: Arc<dyn AdministradorRepositorio>,
    argon2: Arc<HashArgon2>,
}

impl GestaoAdministradoresUseCase {
    pub fn novo(repositorio: Arc<dyn AdministradorRepositorio>, argon2: Arc<HashArgon2>) -> Self {
        Self {
            repositorio,
            argon2,
        }
    }

    /// Listar administradores com paginação.
    pub async fn listar(
        &self,
        pagina: usize,
        por_pagina: usize,
    ) -> Result<(Vec<AdministradorDto>, usize), RepositorioError> {
        let pagina_u32 = u32::try_from(pagina).unwrap_or(1);
        let por_pagina_u32 = u32::try_from(por_pagina).unwrap_or(Paginacao::POR_PAGINA_PADRAO);
        self.repositorio
            .listar(&Paginacao::nova(pagina_u32, por_pagina_u32))
    }

    /// Buscar administrador por ID.
    pub async fn buscar_por_id(
        &self,
        id: &str,
    ) -> Result<Option<AdministradorDto>, RepositorioError> {
        let valor = Uuid::parse_str(id.trim())
            .map_err(|_| RepositorioError::Validacao("id inválido".into()))?;
        self.repositorio.buscar_por_id(valor)
    }

    /// Buscar administrador por e-mail (verificação de unicidade).
    pub async fn buscar_por_email(
        &self,
        email: &str,
    ) -> Result<Option<AdministradorDto>, RepositorioError> {
        self.repositorio
            .buscar_por_email(email.trim().to_ascii_lowercase().as_str())
    }

    /// Criar um administrador; a senha sai desta função apenas em hash.
    pub async fn criar(&self, novo: &AdministradorNovo) -> Result<Uuid, RepositorioError> {
        if novo.senha.len() < 8 {
            return Err(RepositorioError::Validacao(
                "senha deve ter pelo menos 8 caracteres".into(),
            ));
        }
        let senha_hash = self
            .argon2
            .gerar_hash(&novo.senha)
            .map_err(|_| RepositorioError::Dados)?;

        let mut limpo = novo.clone();
        limpo.senha = String::new();
        self.repositorio.criar(&limpo, &senha_hash)
    }

    /// Atualizar administrador (parcial).
    pub async fn atualizar(
        &self,
        id: &str,
        payload: &AdministradorAtualizar,
    ) -> Result<AdministradorDto, RepositorioError> {
        let valor = Uuid::parse_str(id.trim())
            .map_err(|_| RepositorioError::Validacao("id inválido".into()))?;
        self.repositorio
            .atualizar(valor, payload)?
            .ok_or(RepositorioError::Dados)
    }

    /// Alterar senha do administrador.
    pub async fn alterar_senha(
        &self,
        id: &str,
        senha: &str,
    ) -> Result<AdministradorDto, RepositorioError> {
        if senha.len() < 8 {
            return Err(RepositorioError::Validacao(
                "senha deve ter pelo menos 8 caracteres".into(),
            ));
        }
        let valor = Uuid::parse_str(id.trim())
            .map_err(|_| RepositorioError::Validacao("id inválido".into()))?;
        let senha_hash = self
            .argon2
            .gerar_hash(senha)
            .map_err(|_| RepositorioError::Dados)?;
        self.repositorio
            .alterar_senha(valor, &senha_hash)?
            .ok_or(RepositorioError::Dados)
    }
}
