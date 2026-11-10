//! Etapa 0.2 — caso de uso de gestão de administradores do backoffice.
//!
//! Métodos de leitura/escrita delegam ao `AdministradorRepositorio`; a senha
//! nunca passa em texto plano para além desta camada (hash com Argon2).
//!
//! Regras de negócio aplicadas aqui (não no handler):
//! - senha com no mínimo 8 caracteres;
//! - e-mail único na plataforma (normalizado em minúsculas);
//! - um administrador não pode desativar a si mesmo;
//! - não é possível desativar o último administrador ativo;
//! - desativar revoga as sessões do backoffice e é auditado.

use std::net::IpAddr;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::admin::{
    AdministradorAtualizar, AdministradorDto, AdministradorNovo, AdministradorRepositorio,
    AuditoriaAdminRepositorio, EventoAuditoriaAdmin, Paginacao, RepositorioError,
};
use crate::infrastructure::auth::HashArgon2;

#[derive(Clone)]
pub struct GestaoAdministradoresUseCase {
    repositorio: Arc<dyn AdministradorRepositorio>,
    auditoria: Arc<dyn AuditoriaAdminRepositorio>,
    argon2: Arc<HashArgon2>,
}

impl GestaoAdministradoresUseCase {
    pub fn novo(
        repositorio: Arc<dyn AdministradorRepositorio>,
        auditoria: Arc<dyn AuditoriaAdminRepositorio>,
        argon2: Arc<HashArgon2>,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
            argon2,
        }
    }

    /// Listar administradores com paginação e busca opcional.
    pub async fn listar(
        &self,
        pagina: usize,
        por_pagina: usize,
        busca: Option<&str>,
    ) -> Result<(Vec<AdministradorDto>, usize), RepositorioError> {
        let pagina_u32 = u32::try_from(pagina).unwrap_or(1);
        let por_pagina_u32 = u32::try_from(por_pagina).unwrap_or(Paginacao::POR_PAGINA_PADRAO);
        let termo = busca.map(str::trim).filter(|termo| !termo.is_empty());
        self.repositorio
            .listar(&Paginacao::nova(pagina_u32, por_pagina_u32), termo)
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
    pub async fn criar(
        &self,
        novo: &AdministradorNovo,
        ator_id: Uuid,
        ip: Option<IpAddr>,
    ) -> Result<AdministradorDto, RepositorioError> {
        let nome = novo.nome.trim();
        if nome.is_empty() || nome.chars().count() > 200 {
            return Err(RepositorioError::Validacao(
                "nome deve ter entre 1 e 200 caracteres".into(),
            ));
        }
        if novo.senha.len() < 8 {
            return Err(RepositorioError::Validacao(
                "senha deve ter pelo menos 8 caracteres".into(),
            ));
        }
        let email = novo.email.trim().to_ascii_lowercase();
        let cpf = normalizar_digitos(&novo.cpf);
        if cpf.len() != 11 {
            return Err(RepositorioError::Validacao(
                "CPF deve conter exatamente 11 dígitos".into(),
            ));
        }
        if self.repositorio.buscar_por_email(&email)?.is_some() {
            return Err(RepositorioError::UsuarioExistente);
        }

        let senha_hash = self
            .argon2
            .gerar_hash(&novo.senha)
            .map_err(|_| RepositorioError::Dados)?;

        let limpo = AdministradorNovo {
            nome: nome.to_owned(),
            email: email.clone(),
            cpf,
            senha: String::new(),
        };
        let id = self.repositorio.criar(&limpo, &senha_hash)?;

        let dto = self
            .repositorio
            .buscar_por_id(id)?
            .ok_or(RepositorioError::Dados)?;

        self.auditar(
            ator_id,
            "ADMINISTRADOR_CRIADO",
            id,
            None,
            Some(serde_json::json!({ "nome": dto.nome, "email": dto.email })),
            ip,
        )?;

        Ok(dto)
    }

    /// Atualizar administrador (parcial), com as proteções de desativação.
    pub async fn atualizar(
        &self,
        id: &str,
        payload: &AdministradorAtualizar,
        ator_id: Uuid,
        ip: Option<IpAddr>,
    ) -> Result<AdministradorDto, RepositorioError> {
        let valor = Uuid::parse_str(id.trim())
            .map_err(|_| RepositorioError::Validacao("id inválido".into()))?;

        let anterior = self
            .repositorio
            .buscar_por_id(valor)?
            .ok_or_else(|| RepositorioError::Validacao("administrador não encontrado".into()))?;

        let mut limpo = payload.clone();
        if let Some(nome) = limpo.nome.as_ref() {
            if nome.trim().is_empty() || nome.chars().count() > 200 {
                return Err(RepositorioError::Validacao(
                    "nome deve ter entre 1 e 200 caracteres".into(),
                ));
            }
            limpo.nome = Some(nome.trim().to_owned());
        }
        if let Some(email) = limpo.email.as_ref() {
            let email = email.trim().to_ascii_lowercase();
            if !email_valido(&email) {
                return Err(RepositorioError::Validacao("email inválido".into()));
            }
            if let Some(outro) = self.repositorio.buscar_por_email(&email)?
                && outro.id != valor
            {
                return Err(RepositorioError::UsuarioExistente);
            }
            limpo.email = Some(email);
        }

        if limpo.ativo == Some(false) {
            if valor == ator_id {
                return Err(RepositorioError::AutoDesativacao);
            }
            if anterior.ativo && self.repositorio.contar_ativos()? <= 1 {
                return Err(RepositorioError::UltimoAdministradorAtivo);
            }
        }

        let dto = self
            .repositorio
            .atualizar(valor, &limpo)?
            .ok_or(RepositorioError::Dados)?;

        if anterior.ativo && limpo.ativo == Some(false) {
            self.repositorio.revogar_sessoes(valor)?;
        }

        self.auditar(
            ator_id,
            "ADMINISTRADOR_ATUALIZADO",
            valor,
            Some(serde_json::json!({
                "nome": anterior.nome,
                "email": anterior.email,
                "ativo": anterior.ativo,
            })),
            Some(serde_json::json!({
                "nome": dto.nome,
                "email": dto.email,
                "ativo": dto.ativo,
            })),
            ip,
        )?;

        Ok(dto)
    }

    /// Alterar senha do administrador.
    pub async fn alterar_senha(
        &self,
        id: &str,
        senha: &str,
        ator_id: Uuid,
        ip: Option<IpAddr>,
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
        let dto = self
            .repositorio
            .alterar_senha(valor, &senha_hash)?
            .ok_or(RepositorioError::Dados)?;

        self.auditar(
            ator_id,
            "ADMINISTRADOR_SENHA_REDEFINIDA",
            valor,
            None,
            None,
            ip,
        )?;

        Ok(dto)
    }

    #[allow(clippy::too_many_arguments)]
    fn auditar(
        &self,
        ator_id: Uuid,
        acao: &'static str,
        entidade_id: Uuid,
        valor_anterior: Option<serde_json::Value>,
        valor_novo: Option<serde_json::Value>,
        ip: Option<IpAddr>,
    ) -> Result<(), RepositorioError> {
        self.auditoria.registrar(&EventoAuditoriaAdmin {
            escritorio_id: None,
            administrador_id: ator_id,
            acao,
            entidade: "administradores",
            entidade_id,
            valor_anterior,
            valor_novo,
            ip,
        })
    }
}

fn normalizar_digitos(valor: &str) -> String {
    valor.chars().filter(char::is_ascii_digit).collect()
}

fn email_valido(email: &str) -> bool {
    let mut partes = email.split('@');
    let (Some(local), Some(dominio), None) = (partes.next(), partes.next(), partes.next()) else {
        return false;
    };
    !local.is_empty() && dominio.contains('.') && email.len() <= 254
}
