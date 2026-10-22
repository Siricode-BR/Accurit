use std::str::FromStr;

use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_types::{Nullable, Text, Uuid as SqlUuid};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::{Cpf, Papel};
use crate::infrastructure::auth::HashArgon2;
use crate::infrastructure::config::{Config, ConfigError};
use crate::infrastructure::persistence::migrations::{
    MIGRATIONS, MigrationError, aplicar_migrations_com_lock, conectar_com_retry,
};
use crate::infrastructure::persistence::pool::{DbPool, PoolIndisponivel, criar_pool};

const VARIAVEL_SENHA_ADMIN: &str = "BOOTSTRAP_ADMIN_PASSWORD";
const VARIAVEL_SENHA_TENANT: &str = "BOOTSTRAP_USER_PASSWORD";
const SENHA_MINIMA: usize = 8;

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Configuracao(#[from] ConfigError),
    #[error(transparent)]
    Migracao(#[from] MigrationError),
    #[error(transparent)]
    Pool(#[from] PoolIndisponivel),
    #[error("{0}")]
    Opcoes(String),
    #[error("não foi possível gerar o hash da senha")]
    Senha,
    #[error("falha ao gravar dados de inicialização")]
    Banco,
    #[error("tarefa de inicialização falhou")]
    Tarefa,
    #[error("já existe um administrador com este e-mail")]
    AdministradorExistente,
    #[error("escritório já possui usuário com este e-mail")]
    UsuarioExistente,
}

pub enum Alvo {
    Administrador(AdministradorOpcoes),
    Tenant(TenantOpcoes),
}

pub struct AdministradorOpcoes {
    email: String,
    cpf: Cpf,
    senha: String,
}

pub struct TenantOpcoes {
    escritorio_slug: String,
    escritorio_nome: String,
    email: String,
    cpf: Cpf,
    papel: Papel,
    senha: String,
}

impl std::fmt::Debug for AdministradorOpcoes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdministradorOpcoes")
            .field("email", &self.email)
            .field("cpf", &self.cpf)
            .field("senha", &"REDACTED")
            .finish()
    }
}

impl std::fmt::Debug for TenantOpcoes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TenantOpcoes")
            .field("escritorio_slug", &self.escritorio_slug)
            .field("escritorio_nome", &self.escritorio_nome)
            .field("email", &self.email)
            .field("cpf", &self.cpf)
            .field("papel", &self.papel)
            .field("senha", &"REDACTED")
            .finish()
    }
}

impl std::fmt::Debug for Alvo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Alvo::Administrador(opcoes) => f
                .debug_struct("bootstrap-admin")
                .field("email", &opcoes.email)
                .field("cpf", &opcoes.cpf)
                .field("senha", &"REDACTED")
                .finish(),
            Alvo::Tenant(opcoes) => f
                .debug_struct("bootstrap-tenant")
                .field("escritorio_slug", &opcoes.escritorio_slug)
                .field("escritorio_nome", &opcoes.escritorio_nome)
                .field("email", &opcoes.email)
                .field("cpf", &opcoes.cpf)
                .field("papel", &opcoes.papel)
                .field("senha", &"REDACTED")
                .finish(),
        }
    }
}

pub const AJUDA: &str = "\
Uso:
  bootstrap-admin --email <email> --cpf <cpf>
      Cria o primeiro gestor de tenants (backoffice /admin).
      A senha é lida da variável BOOTSTRAP_ADMIN_PASSWORD.

  bootstrap-tenant --escritorio-slug <slug> --escritorio-nome <nome> \
--email <email> --cpf <cpf> --papel <CLIENTE|OPERADOR>
      Cria o escritório (quando inexistente) e um usuário da plataforma do \
escritório (caminho de reserva; o provisionamento normal é feito no /admin).
      A senha é lida da variável BOOTSTRAP_USER_PASSWORD.";

fn valor(args: &[String], indice: usize) -> Result<&str, BootstrapError> {
    args.get(indice)
        .map(String::as_str)
        .filter(|valor| !valor.starts_with("--"))
        .ok_or_else(|| BootstrapError::Opcoes(format!("valor ausente para {}", args[indice - 1])))
}

fn email_valido(email: &str) -> bool {
    let mut partes = email.split('@');
    let (Some(local), Some(dominio), None) = (partes.next(), partes.next(), partes.next()) else {
        return false;
    };
    !local.is_empty() && dominio.contains('.') && email.len() <= 254
}

fn slug_valido(slug: &str) -> bool {
    let tamanho = slug.len();
    (3..=40).contains(&tamanho)
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn parse_email(args: &[String]) -> Result<String, BootstrapError> {
    let mut email = None;
    let mut indice = 0;
    while indice < args.len() {
        match args[indice].as_str() {
            "--email" => {
                email = Some(valor(args, indice + 1)?.to_owned());
                indice += 2;
            }
            _ => indice += 1,
        }
    }
    let email = email
        .ok_or_else(|| BootstrapError::Opcoes("--email é obrigatório".into()))?
        .trim()
        .to_ascii_lowercase();
    if !email_valido(&email) {
        return Err(BootstrapError::Opcoes("--email inválido".into()));
    }
    Ok(email)
}

fn parse_cpf(args: &[String], label: &str) -> Result<Cpf, BootstrapError> {
    let mut bruto = None;
    let mut indice = 0;
    while indice < args.len() {
        match args[indice].as_str() {
            "--cpf" => {
                bruto = Some(valor(args, indice + 1)?.to_owned());
                indice += 2;
            }
            _ => indice += 1,
        }
    }

    let valor_cpf =
        bruto.ok_or_else(|| BootstrapError::Opcoes(format!("{label} é obrigatório")))?;
    Cpf::novo(&valor_cpf).map_err(|_| BootstrapError::Opcoes(format!("{label} inválido")))
}

fn validar_senha(senha: &str, variavel: &str) -> Result<(), BootstrapError> {
    if senha.len() < SENHA_MINIMA {
        return Err(BootstrapError::Opcoes(format!(
            "{variavel} deve ter pelo menos {SENHA_MINIMA} caracteres"
        )));
    }
    Ok(())
}

impl AdministradorOpcoes {
    fn parse(args: &[String], senha: String) -> Result<Self, BootstrapError> {
        validar_senha(&senha, VARIAVEL_SENHA_ADMIN)?;

        Ok(Self {
            email: parse_email(args)?,
            cpf: parse_cpf(args, "--cpf")?,
            senha,
        })
    }
}

impl TenantOpcoes {
    fn parse(args: &[String], senha: String) -> Result<Self, BootstrapError> {
        validar_senha(&senha, VARIAVEL_SENHA_TENANT)?;

        let mut escritorio_slug = None;
        let mut escritorio_nome = None;
        let mut papel = None;

        let mut indice = 0;
        while indice < args.len() {
            match args[indice].as_str() {
                "--escritorio-slug" => {
                    escritorio_slug = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                "--escritorio-nome" => {
                    escritorio_nome = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                "--papel" => {
                    papel = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                _ => indice += 1,
            }
        }

        let escritorio_slug = escritorio_slug
            .ok_or_else(|| BootstrapError::Opcoes("--escritorio-slug é obrigatório".into()))?
            .trim()
            .to_ascii_lowercase();
        if !slug_valido(&escritorio_slug) {
            return Err(BootstrapError::Opcoes(
                "--escritorio-slug deve ter de 3 a 40 caracteres [a-z0-9-]".into(),
            ));
        }

        let escritorio_nome = escritorio_nome
            .ok_or_else(|| BootstrapError::Opcoes("--escritorio-nome é obrigatório".into()))?
            .trim()
            .to_owned();
        if escritorio_nome.is_empty() || escritorio_nome.chars().count() > 200 {
            return Err(BootstrapError::Opcoes("--escritorio-nome inválido".into()));
        }

        let papel = papel.ok_or_else(|| BootstrapError::Opcoes("--papel é obrigatório".into()))?;
        let papel = Papel::from_str(&papel.to_ascii_uppercase())
            .map_err(|_| BootstrapError::Opcoes("--papel deve ser CLIENTE ou OPERADOR".into()))?;
        if papel == Papel::Admin {
            return Err(BootstrapError::Opcoes(
                "--papel ADMIN é exclusivo do backoffice; use bootstrap-admin".into(),
            ));
        }

        Ok(Self {
            escritorio_slug,
            escritorio_nome,
            email: parse_email(args)?,
            cpf: parse_cpf(args, "--cpf")?,
            papel,
            senha,
        })
    }
}

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

pub async fn executar(alvo: &str, args: &[String]) -> Result<(), BootstrapError> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{AJUDA}");
        return Ok(());
    }

    if alvo == "bootstrap-admin" {
        let senha = std::env::var(VARIAVEL_SENHA_ADMIN)
            .map_err(|_| BootstrapError::Opcoes(format!("{VARIAVEL_SENHA_ADMIN} é obrigatória")))?;
        let opcoes = AdministradorOpcoes::parse(args, senha)?;
        let config = Config::from_env()?;
        let database_url = secrecy::ExposeSecret::expose_secret(&config.database.url).to_owned();
        let argon2 = HashArgon2::novo(&config.auth).map_err(|_| BootstrapError::Senha)?;

        return tokio::task::spawn_blocking(move || {
            let conexao = conectar_com_retry(&database_url)?;
            aplicar_migrations_com_lock(conexao, MIGRATIONS)?;

            let pool = criar_pool(&database_url)?;
            bootstrap_administrador(&pool, &argon2, opcoes)
        })
        .await
        .map_err(|_| BootstrapError::Tarefa)?;
    }

    let senha = std::env::var(VARIAVEL_SENHA_TENANT)
        .map_err(|_| BootstrapError::Opcoes(format!("{VARIAVEL_SENHA_TENANT} é obrigatória")))?;
    let opcoes = TenantOpcoes::parse(args, senha)?;
    let config = Config::from_env()?;
    let database_url = secrecy::ExposeSecret::expose_secret(&config.database.url).to_owned();
    let argon2 = HashArgon2::novo(&config.auth).map_err(|_| BootstrapError::Senha)?;

    tokio::task::spawn_blocking(move || {
        let conexao = conectar_com_retry(&database_url)?;
        aplicar_migrations_com_lock(conexao, MIGRATIONS)?;

        let pool = criar_pool(&database_url)?;
        bootstrap_tenant(&pool, &argon2, opcoes)
    })
    .await
    .map_err(|_| BootstrapError::Tarefa)?
}

fn bootstrap_administrador(
    pool: &DbPool,
    argon2: &HashArgon2,
    opcoes: AdministradorOpcoes,
) -> Result<(), BootstrapError> {
    let mut conexao = pool.get().map_err(|_| BootstrapError::Banco)?;

    let existente =
        diesel::sql_query("SELECT id FROM administradores WHERE lower(email) = lower($1)")
            .bind::<Text, _>(&opcoes.email)
            .get_result::<IdRow>(&mut conexao);

    match existente {
        Ok(_) => return Err(BootstrapError::AdministradorExistente),
        Err(diesel::result::Error::NotFound) => {}
        Err(_) => return Err(BootstrapError::Banco),
    }

    let senha_hash = argon2
        .gerar_hash(&opcoes.senha)
        .map_err(|_| BootstrapError::Senha)?;

    diesel::sql_query(
        "INSERT INTO administradores (id, email, cpf, senha_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(&opcoes.email)
    .bind::<Text, _>(opcoes.cpf.como_str())
    .bind::<Text, _>(senha_hash)
    .execute(&mut conexao)
    .map_err(|_| BootstrapError::Banco)?;

    tracing::info!("administrador de inicialização criado");

    Ok(())
}

fn bootstrap_tenant(
    pool: &DbPool,
    argon2: &HashArgon2,
    opcoes: TenantOpcoes,
) -> Result<(), BootstrapError> {
    let mut conexao = pool.get().map_err(|_| BootstrapError::Banco)?;

    let escritorio_id = match diesel::sql_query("SELECT id FROM escritorios WHERE slug = $1")
        .bind::<Text, _>(&opcoes.escritorio_slug)
        .get_result::<IdRow>(&mut conexao)
    {
        Ok(row) => row.id,
        Err(diesel::result::Error::NotFound) => {
            let id = Uuid::now_v7();
            diesel::sql_query("INSERT INTO escritorios (id, slug, nome) VALUES ($1, $2, $3)")
                .bind::<SqlUuid, _>(id)
                .bind::<Text, _>(&opcoes.escritorio_slug)
                .bind::<Text, _>(&opcoes.escritorio_nome)
                .execute(&mut conexao)
                .map_err(|_| BootstrapError::Banco)?;
            id
        }
        Err(_) => return Err(BootstrapError::Banco),
    };

    let existente = diesel::sql_query(
        "SELECT id FROM usuarios WHERE escritorio_id = $1 AND lower(email) = lower($2)",
    )
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(&opcoes.email)
    .get_result::<IdRow>(&mut conexao);

    match existente {
        Ok(_) => return Err(BootstrapError::UsuarioExistente),
        Err(diesel::result::Error::NotFound) => {}
        Err(_) => return Err(BootstrapError::Banco),
    }

    let senha_hash = argon2
        .gerar_hash(&opcoes.senha)
        .map_err(|_| BootstrapError::Senha)?;

    diesel::sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(&opcoes.email)
    .bind::<Nullable<Text>, _>(opcoes.cpf.como_str())
    .bind::<Text, _>(senha_hash)
    .bind::<Text, _>(opcoes.papel.como_str())
    .execute(&mut conexao)
    .map_err(|_| BootstrapError::Banco)?;

    tracing::info!(
        escritorio = %opcoes.escritorio_slug,
        papel = %opcoes.papel,
        "usuário de inicialização criado"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(valores: &[&str]) -> Vec<String> {
        valores.iter().map(|valor| (*valor).to_owned()).collect()
    }

    #[test]
    fn admin_exige_email_cpf_e_senha_forte() {
        let erro =
            AdministradorOpcoes::parse(&args(&["--email", "gestor@accurit.test"]), "curta".into())
                .expect_err("deve exigir senha forte");
        assert!(erro.to_string().contains(VARIAVEL_SENHA_ADMIN));

        let erro = AdministradorOpcoes::parse(&args(&[]), "senha-forte-123".into())
            .expect_err("deve exigir email");
        assert!(erro.to_string().contains("--email"));

        let opcoes = AdministradorOpcoes::parse(
            &args(&["--email", "gestor@accurit.test", "--cpf", "52998224725"]),
            "senha-forte-123".into(),
        )
        .expect("opções válidas");
        assert_eq!(opcoes.email, "gestor@accurit.test");
    }

    #[test]
    fn tenant_exige_cpf_para_todos_os_papeis() {
        let erro = TenantOpcoes::parse(
            &args(&[
                "--escritorio-slug",
                "demo",
                "--escritorio-nome",
                "Demo",
                "--email",
                "cliente@demo.test",
                "--papel",
                "CLIENTE",
            ]),
            "senha-forte-123".into(),
        )
        .expect_err("deve exigir cpf");
        assert!(erro.to_string().contains("--cpf"));

        let erro_admin = TenantOpcoes::parse(
            &args(&[
                "--escritorio-slug",
                "demo",
                "--escritorio-nome",
                "Demo",
                "--email",
                "admin@demo.test",
                "--cpf",
                "52998224725",
                "--papel",
                "ADMIN",
            ]),
            "senha-forte-123".into(),
        )
        .expect_err("ADMIN é exclusivo do backoffice");
        assert!(erro_admin.to_string().contains("bootstrap-admin"));
    }

    #[test]
    fn tenant_aceita_operador_e_papel_minusculo() {
        let opcoes = TenantOpcoes::parse(
            &args(&[
                "--escritorio-slug",
                "demo",
                "--escritorio-nome",
                "Demo",
                "--email",
                "operador@demo.test",
                "--cpf",
                "52998224725",
                "--papel",
                "operador",
            ]),
            "senha-forte-123".into(),
        )
        .expect("opções válidas");
        assert_eq!(opcoes.escritorio_slug, "demo");
        assert_eq!(opcoes.papel, Papel::Operador);
    }

    #[test]
    fn valida_slug_e_email() {
        assert!(slug_valido("escritorio-1"));
        assert!(!slug_valido("ab"));
        assert!(!slug_valido("Escritorio"));
        assert!(email_valido("a@b.co"));
        assert!(!email_valido("a@b"));
        assert!(!email_valido("sem-arroba"));
    }
}
