use std::sync::Arc;

use crate::application::admin::{
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, ProvisionarError, ProvisionarTenantCommand, ProvisionarTenantResultado,
    RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
use crate::domain::auth::Papel;
use crate::infrastructure::auth::HashArgon2;

/// Listagem simples de tenants para o backoffice (GET /admin/escritorios):
/// pré-requisito da tela do toggle SERPRO, que seleciona o escritório na
/// lista e navega para o detalhe (remediação mínima da Fase 2B).
#[derive(Clone)]
pub struct ListarEscritoriosUseCase {
    escritorio_repo: Arc<dyn EscritorioRepositorio>,
}

impl ListarEscritoriosUseCase {
    pub const fn novo(escritorio_repo: Arc<dyn EscritorioRepositorio>) -> Self {
        Self { escritorio_repo }
    }

    pub fn executar(&self) -> Result<Vec<EscritorioResumo>, RepositorioError> {
        self.escritorio_repo.listar()
    }
}

#[derive(Clone)]
pub struct ProvisionarTenantUseCase {
    escritorio_repo: Arc<dyn EscritorioRepositorio>,
    usuario_repo: Arc<dyn UsuarioAdminRepositorio>,
    auditoria: Arc<dyn AuditoriaAdminRepositorio>,
    argon2: Arc<HashArgon2>,
}

impl ProvisionarTenantUseCase {
    pub fn novo(
        escritorio_repo: Arc<dyn EscritorioRepositorio>,
        usuario_repo: Arc<dyn UsuarioAdminRepositorio>,
        auditoria: Arc<dyn AuditoriaAdminRepositorio>,
        argon2: Arc<HashArgon2>,
    ) -> Self {
        Self {
            escritorio_repo,
            usuario_repo,
            auditoria,
            argon2,
        }
    }

    pub async fn executar(
        &self,
        comando: ProvisionarTenantCommand,
    ) -> Result<ProvisionarTenantResultado, ProvisionarError> {
        validar_slug(&comando.slug)?;
        validar_nome(&comando.nome)?;
        validar_email(&comando.email_titular)?;
        validar_cpf(&comando.cpf_titular)?;
        validar_senha(&comando.senha_titular)?;

        let escritorio_id = if let Some(id) = self.escritorio_repo.buscar_por_slug(&comando.slug)? {
            id
        } else {
            let escritorio = EscritorioNovo {
                slug: comando.slug.clone(),
                nome: comando.nome.clone(),
            };
            self.escritorio_repo.criar(&escritorio)?
        };

        if self
            .usuario_repo
            .buscar_por_escritorio_e_email(escritorio_id, &comando.email_titular)?
            .is_some()
        {
            return Err(ProvisionarError::Repositorio(
                RepositorioError::UsuarioExistente,
            ));
        }

        let senha_hash = self
            .argon2
            .gerar_hash(&comando.senha_titular)
            .map_err(|_| ProvisionarError::Senha)?;

        let cpf_normalizado: String = comando
            .cpf_titular
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        let titular_id = self.usuario_repo.criar(&UsuarioNovo {
            escritorio_id,
            email: comando.email_titular.trim().to_ascii_lowercase(),
            cpf: cpf_normalizado,
            senha_hash,
            papel: Papel::Cliente,
        })?;

        self.auditoria.registrar(&EventoAuditoriaAdmin {
            escritorio_id: Some(escritorio_id),
            administrador_id: comando.administrador_id,
            acao: "TENANT_PROVISIONADO",
            entidade: "escritorios",
            entidade_id: escritorio_id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({
                "slug": comando.slug,
                "nome": comando.nome,
                "titular_id": titular_id,
                "email_titular": comando.email_titular,
            })),
            ip: comando.ip,
        })?;

        Ok(ProvisionarTenantResultado {
            escritorio_id,
            slug: comando.slug,
            titular_id,
            email_titular: comando.email_titular,
        })
    }
}

fn validar_slug(slug: &str) -> Result<(), ProvisionarError> {
    let slug = slug.trim().to_ascii_lowercase();
    if slug.len() < 3 || slug.len() > 40 {
        return Err(ProvisionarError::Validacao(
            "slug deve ter de 3 a 40 caracteres".into(),
        ));
    }
    if !slug
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(ProvisionarError::Validacao(
            "slug deve conter apenas letras min\u{00fa}sculas, d\u{00ed}gitos e h\u{00ed}fen"
                .into(),
        ));
    }
    Ok(())
}

fn validar_nome(nome: &str) -> Result<(), ProvisionarError> {
    let limpo = nome.trim();
    if limpo.is_empty() || limpo.chars().count() > 200 {
        return Err(ProvisionarError::Validacao(
            "nome deve ter entre 1 e 200 caracteres".into(),
        ));
    }
    Ok(())
}

fn validar_email(email: &str) -> Result<(), ProvisionarError> {
    let email = email.trim().to_ascii_lowercase();
    let mut partes = email.split('@');
    let (Some(local), Some(dominio), None) = (partes.next(), partes.next(), partes.next()) else {
        return Err(ProvisionarError::Validacao("email inv\u{00e1}lido".into()));
    };
    if local.is_empty() || !dominio.contains('.') || email.len() > 254 {
        return Err(ProvisionarError::Validacao("email inv\u{00e1}lido".into()));
    }
    Ok(())
}

fn validar_cpf(cpf: &str) -> Result<(), ProvisionarError> {
    let digitos: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    if digitos.len() != 11 {
        return Err(ProvisionarError::Validacao(
            "CPF deve conter exatamente 11 d\u{00ed}gitos".into(),
        ));
    }
    if digitos.bytes().all(|b| b == digitos.as_bytes()[0]) {
        return Err(ProvisionarError::Validacao("CPF inv\u{00e1}lido".into()));
    }

    let numeros: Vec<u32> = digitos.bytes().map(|b| u32::from(b - b'0')).collect();
    let primeiro = calcular_digito(&numeros[..9], 10);
    let segundo = calcular_digito(&numeros[..10], 11);
    if numeros[9] != primeiro || numeros[10] != segundo {
        return Err(ProvisionarError::Validacao("CPF inv\u{00e1}lido".into()));
    }
    Ok(())
}

fn calcular_digito(digitos: &[u32], peso_inicial: u32) -> u32 {
    let soma: u32 = digitos
        .iter()
        .enumerate()
        .map(|(i, d)| d * (peso_inicial - i as u32))
        .sum();
    let resto = (soma * 10) % 11;
    if resto == 10 { 0 } else { resto }
}

fn validar_senha(senha: &str) -> Result<(), ProvisionarError> {
    if senha.len() < 8 {
        return Err(ProvisionarError::Validacao(
            "senha deve ter pelo menos 8 caracteres".into(),
        ));
    }
    Ok(())
}
