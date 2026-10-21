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

const VARIAVEL_SENHA: &str = "BOOTSTRAP_USER_PASSWORD";
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
    #[error("escritório já possui usuário com este e-mail")]
    UsuarioExistente,
}

pub struct Opcoes {
    escritorio_slug: String,
    escritorio_nome: String,
    email: String,
    cpf: Option<Cpf>,
    papel: Papel,
    senha: String,
}

impl std::fmt::Debug for Opcoes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Opcoes")
            .field("escritorio_slug", &self.escritorio_slug)
            .field("escritorio_nome", &self.escritorio_nome)
            .field("email", &self.email)
            .field("cpf", &self.cpf)
            .field("papel", &self.papel)
            .field("senha", &"REDACTED")
            .finish()
    }
}

pub const AJUDA: &str = "\
Uso: bootstrap-user --escritorio-slug <slug> --escritorio-nome <nome> \
--email <email> --papel <ADMIN|OPERADOR|CLIENTE> [--cpf <cpf>]

Cria o escritório (quando inexistente) e o primeiro usuário.
A senha é lida da variável de ambiente BOOTSTRAP_USER_PASSWORD.";

fn valor(args: &[String], indice: usize) -> Result<&str, BootstrapError> {
    args.get(indice)
        .map(String::as_str)
        .filter(|valor| !valor.starts_with("--"))
        .ok_or_else(|| BootstrapError::Opcoes(format!("valor ausente para {}", args[indice - 1])))
}

impl Opcoes {
    pub fn parse(args: &[String], senha: String) -> Result<Self, BootstrapError> {
        let mut escritorio_slug = None;
        let mut escritorio_nome = None;
        let mut email = None;
        let mut cpf = None;
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
                "--email" => {
                    email = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                "--cpf" => {
                    cpf = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                "--papel" => {
                    papel = Some(valor(args, indice + 1)?.to_owned());
                    indice += 2;
                }
                outro => {
                    return Err(BootstrapError::Opcoes(format!(
                        "opção desconhecida: {outro}"
                    )));
                }
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

        let email = email
            .ok_or_else(|| BootstrapError::Opcoes("--email é obrigatório".into()))?
            .trim()
            .to_ascii_lowercase();
        if !email_valido(&email) {
            return Err(BootstrapError::Opcoes("--email inválido".into()));
        }

        let papel = papel.ok_or_else(|| BootstrapError::Opcoes("--papel é obrigatório".into()))?;
        let papel = Papel::from_str(&papel.to_ascii_uppercase()).map_err(|_| {
            BootstrapError::Opcoes("--papel deve ser ADMIN, OPERADOR ou CLIENTE".into())
        })?;

        let cpf = match cpf {
            Some(valor) => Some(
                Cpf::novo(&valor).map_err(|_| BootstrapError::Opcoes("--cpf inválido".into()))?,
            ),
            None => None,
        };
        if papel != Papel::Cliente && cpf.is_none() {
            return Err(BootstrapError::Opcoes(
                "--cpf é obrigatório para ADMIN e OPERADOR".into(),
            ));
        }

        if senha.len() < SENHA_MINIMA {
            return Err(BootstrapError::Opcoes(format!(
                "{VARIAVEL_SENHA} deve ter pelo menos {SENHA_MINIMA} caracteres"
            )));
        }

        Ok(Self {
            escritorio_slug,
            escritorio_nome,
            email,
            cpf,
            papel,
            senha,
        })
    }
}

fn slug_valido(slug: &str) -> bool {
    let tamanho = slug.len();
    (3..=40).contains(&tamanho)
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn email_valido(email: &str) -> bool {
    let mut partes = email.split('@');
    let (Some(local), Some(dominio), None) = (partes.next(), partes.next(), partes.next()) else {
        return false;
    };
    !local.is_empty() && dominio.contains('.') && email.len() <= 254
}

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

pub async fn executar(args: &[String]) -> Result<(), BootstrapError> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{AJUDA}");
        return Ok(());
    }

    let senha = std::env::var(VARIAVEL_SENHA)
        .map_err(|_| BootstrapError::Opcoes(format!("{VARIAVEL_SENHA} é obrigatória")))?;
    let opcoes = Opcoes::parse(args, senha)?;
    let config = Config::from_env()?;
    let database_url = secrecy::ExposeSecret::expose_secret(&config.database.url).to_owned();
    let argon2 = HashArgon2::novo(&config.auth).map_err(|_| BootstrapError::Senha)?;

    tokio::task::spawn_blocking(move || {
        let conexao = conectar_com_retry(&database_url)?;
        aplicar_migrations_com_lock(conexao, MIGRATIONS)?;

        let pool = criar_pool(&database_url)?;
        bootstrap_usuario(&pool, &argon2, opcoes)
    })
    .await
    .map_err(|_| BootstrapError::Tarefa)?
}

fn bootstrap_usuario(
    pool: &DbPool,
    argon2: &HashArgon2,
    opcoes: Opcoes,
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
    let cpf = opcoes.cpf.as_ref().map(|cpf| cpf.como_str().to_owned());

    diesel::sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(&opcoes.email)
    .bind::<Nullable<Text>, _>(cpf.as_deref())
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
    fn exige_cpf_para_papeis_do_escritorio() {
        let erro = Opcoes::parse(
            &args(&[
                "--escritorio-slug",
                "demo",
                "--escritorio-nome",
                "Demo",
                "--email",
                "admin@demo.test",
                "--papel",
                "ADMIN",
            ]),
            "senha-forte-123".into(),
        )
        .expect_err("deve exigir cpf");
        assert!(erro.to_string().contains("--cpf"));
    }

    #[test]
    fn aceita_cliente_sem_cpf() {
        let opcoes = Opcoes::parse(
            &args(&[
                "--escritorio-slug",
                "demo",
                "--escritorio-nome",
                "Demo",
                "--email",
                "cliente@demo.test",
                "--papel",
                "cliente",
            ]),
            "senha-forte-123".into(),
        )
        .expect("opções válidas");
        assert_eq!(opcoes.escritorio_slug, "demo");
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
