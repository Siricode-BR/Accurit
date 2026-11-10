use std::sync::Arc;

use crate::application::admin::{
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, Paginacao, ProvisionarError, ProvisionarTenantCommand,
    ProvisionarTenantResultado, RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
use crate::domain::auth::Papel;
use crate::infrastructure::auth::HashArgon2;

/// Listagem paginada de tenants para o backoffice (GET /admin/escritorios):
/// fonte da tela de tenants (lista com o toggle SERPRO por linha).
#[derive(Clone)]
pub struct ListarEscritoriosUseCase {
    escritorio_repo: Arc<dyn EscritorioRepositorio>,
}

impl ListarEscritoriosUseCase {
    pub const fn novo(escritorio_repo: Arc<dyn EscritorioRepositorio>) -> Self {
        Self { escritorio_repo }
    }

    pub fn executar(
        &self,
        paginacao: &Paginacao,
    ) -> Result<(Vec<EscritorioResumo>, i64), RepositorioError> {
        self.escritorio_repo.listar(paginacao)
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
        validar_cnpj(&comando.cnpj)?;
        validar_nome(&comando.nome)?;
        validar_email(&comando.email_titular)?;
        validar_cpf(&comando.cpf_titular)?;
        validar_senha(&comando.senha_titular)?;

        // O CNPJ é armazenado apenas com dígitos (o value object normaliza a
        // máscara recebida do formulário).
        let cnpj_normalizado = crate::domain::tenancy::Cnpj::novo(&comando.cnpj)
            .map_err(|_| ProvisionarError::Validacao("CNPJ inv\u{00e1}lido".into()))?
            .como_str()
            .to_owned();

        // O e-mail identifica o usuário no login, então precisa ser único na plataforma.
        // Verifica antes de criar o escritório para não deixar um tenant órfão.
        let email_titular = comando.email_titular.trim().to_ascii_lowercase();
        if self
            .usuario_repo
            .buscar_por_email(&email_titular)?
            .is_some()
        {
            return Err(ProvisionarError::Repositorio(
                RepositorioError::UsuarioExistente,
            ));
        }

        let escritorio_id =
            if let Some(id) = self.escritorio_repo.buscar_por_cnpj(&cnpj_normalizado)? {
                id
            } else {
                let escritorio = EscritorioNovo {
                    cnpj: cnpj_normalizado.clone(),
                    nome: comando.nome.clone(),
                };
                self.escritorio_repo.criar(&escritorio)?
            };

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
            email: email_titular,
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
                "cnpj": cnpj_normalizado,
                "nome": comando.nome,
                "titular_id": titular_id,
                "email_titular": comando.email_titular,
            })),
            ip: comando.ip,
        })?;

        Ok(ProvisionarTenantResultado {
            escritorio_id,
            cnpj: cnpj_normalizado,
            titular_id,
            email_titular: comando.email_titular,
        })
    }
}

pub(crate) fn validar_cnpj(cnpj: &str) -> Result<(), ProvisionarError> {
    match crate::domain::tenancy::Cnpj::novo(cnpj) {
        Ok(_) => Ok(()),
        Err(_) => Err(ProvisionarError::Validacao("CNPJ inv\u{00e1}lido".into())),
    }
}

pub(crate) fn validar_nome(nome: &str) -> Result<(), ProvisionarError> {
    let limpo = nome.trim();
    if limpo.is_empty() || limpo.chars().count() > 200 {
        return Err(ProvisionarError::Validacao(
            "nome deve ter entre 1 e 200 caracteres".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validar_email(email: &str) -> Result<(), ProvisionarError> {
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

pub(crate) fn validar_cpf(cpf: &str) -> Result<(), ProvisionarError> {
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

pub(crate) fn validar_senha(senha: &str) -> Result<(), ProvisionarError> {
    if senha.len() < 8 {
        return Err(ProvisionarError::Validacao(
            "senha deve ter pelo menos 8 caracteres".into(),
        ));
    }
    Ok(())
}
