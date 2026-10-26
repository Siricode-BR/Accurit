use std::collections::HashMap;
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use secrecy::{SecretSlice, SecretString};
use thiserror::Error;
use url::Url;

use super::app_env::AppEnv;

pub const REQUISICOES_POR_SEGUNDO: u32 = 30;
pub const BURST_RATE_LIMIT: u32 = 60;

const PORTA_PADRAO: u16 = 8080;
const JWT_SEGREDO_MINIMO_BYTES: usize = 32;
const CERTIFICADO_CHAVE_BYTES: usize = 32;

pub const AUTH_ACCESS_TTL_SEGUNDOS: u64 = 900;
pub const AUTH_REFRESH_TTL_ADMIN_SEGUNDOS: u64 = 28_800;
pub const AUTH_REFRESH_TTL_CLIENT_SEGUNDOS: u64 = 604_800;
pub const AUTH_MAX_TENTATIVAS: u32 = 5;
pub const AUTH_BLOQUEIO_SEGUNDOS: u64 = 900;
pub const ARGON2_MEMORIA_KIB: u32 = 19_456;
pub const ARGON2_ITERACOES: u32 = 2;
pub const ARGON2_PARALELISMO: u32 = 1;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: SecretString,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub access_ttl_secs: u64,
    pub refresh_ttl_admin_secs: u64,
    pub refresh_ttl_client_secs: u64,
    pub max_failed_attempts: u32,
    pub lockout_secs: u64,
    pub cookie_secure: bool,
    pub argon2_memory_kib: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

#[derive(Debug, Clone)]
pub struct SerproConfig {
    pub token_url: Url,
    pub sapi_auth_url: Url,
    pub integra_trial_url: Url,
    pub integra_prod_url: Url,
    pub divida_ativa_trial_url: Url,
    pub divida_ativa_prod_url: Option<Url>,
    pub trial_bearer_token: Option<SecretString>,
    pub consumer_key: Option<SecretString>,
    pub consumer_secret: Option<SecretString>,
    pub operator_header_enabled: bool,
    pub divida_ativa_fallback_codigo_contribuinte: bool,
    /// Rate limit local de consultas da Dívida Ativa por escritório
    /// (SPEC §3.3): `SERPRO_RATE_LIMIT_CONSULTAS_POR_SEGUNDO`.
    pub rate_limit_consultas_por_segundo: u32,
    pub http_connect_timeout_secs: u64,
    pub http_request_timeout_secs: u64,
    pub procuracao_cache_ttl_secs: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: AppEnv,
    pub port: u16,
    pub database: DatabaseConfig,
    pub jwt_secret: SecretString,
    pub certificate_master_key: SecretSlice<u8>,
    pub cors_allowed_origins: Vec<String>,
    pub auth: AuthConfig,
    pub serpro: SerproConfig,
    /// Retenção do histórico `consultas_fiscais`, em dias (SPEC
    /// §9.11): `CONSULTAS_RETENCAO_DIAS`.
    pub consultas_retencao_dias: u64,
    pub log_filter: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("variável {0} é obrigatória")]
    Obrigatoria(&'static str),
    #[error("variável {0} inválida: {1}")]
    Invalida(&'static str, &'static str),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(std::env::vars())
    }

    pub fn from_pairs<I>(pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let mut mapa: HashMap<String, String> = HashMap::new();
        for (chave, valor) in pairs {
            mapa.insert(chave, valor);
        }

        let app_env = AppEnv::from_str(&obrigatoria(&mapa, "APP_ENV")?).map_err(|_| {
            ConfigError::Invalida("APP_ENV", "use development, staging ou production")
        })?;

        let port = match opcional(&mapa, "PORT") {
            None => PORTA_PADRAO,
            Some(valor) => {
                let porta = valor.parse::<u16>().map_err(|_| {
                    ConfigError::Invalida("PORT", "deve ser um número entre 1 e 65535")
                })?;
                if porta == 0 {
                    return Err(ConfigError::Invalida(
                        "PORT",
                        "deve ser um número entre 1 e 65535",
                    ));
                }
                porta
            }
        };

        let database_url = obrigatoria(&mapa, "DATABASE_URL")?;
        let database = DatabaseConfig {
            url: validar_database_url(database_url)?,
        };

        let jwt_secret = obrigatoria(&mapa, "JWT_SECRET")?;
        if jwt_secret.len() < JWT_SEGREDO_MINIMO_BYTES {
            return Err(ConfigError::Invalida(
                "JWT_SECRET",
                "deve ter pelo menos 32 bytes",
            ));
        }

        let certificate_master_key =
            validar_chave_mestre(&obrigatoria(&mapa, "CERTIFICATE_MASTER_KEY")?)?;

        let cors_allowed_origins = validar_origens_cors(opcional(&mapa, "CORS_ALLOWED_ORIGINS"))?;

        let cookie_secure = booleano(&mapa, "AUTH_COOKIE_SECURE", true)?;
        if !cookie_secure && app_env != AppEnv::Development {
            return Err(ConfigError::Invalida(
                "AUTH_COOKIE_SECURE",
                "só pode ser false em development",
            ));
        }

        let auth = AuthConfig {
            access_ttl_secs: inteiro_positivo(
                &mapa,
                "AUTH_ACCESS_TTL_SECS",
                AUTH_ACCESS_TTL_SEGUNDOS,
            )?,
            refresh_ttl_admin_secs: inteiro_positivo(
                &mapa,
                "AUTH_REFRESH_TTL_ADMIN_SECS",
                AUTH_REFRESH_TTL_ADMIN_SEGUNDOS,
            )?,
            refresh_ttl_client_secs: inteiro_positivo(
                &mapa,
                "AUTH_REFRESH_TTL_CLIENT_SECS",
                AUTH_REFRESH_TTL_CLIENT_SEGUNDOS,
            )?,
            max_failed_attempts: inteiro_positivo_u32(
                &mapa,
                "AUTH_MAX_FAILED_ATTEMPTS",
                AUTH_MAX_TENTATIVAS,
            )?,
            lockout_secs: inteiro_positivo(&mapa, "AUTH_LOCKOUT_SECS", AUTH_BLOQUEIO_SEGUNDOS)?,
            cookie_secure,
            argon2_memory_kib: inteiro_positivo_u32(
                &mapa,
                "AUTH_ARGON2_MEMORY_KIB",
                ARGON2_MEMORIA_KIB,
            )?,
            argon2_iterations: inteiro_positivo_u32(
                &mapa,
                "AUTH_ARGON2_ITERATIONS",
                ARGON2_ITERACOES,
            )?,
            argon2_parallelism: inteiro_positivo_u32(
                &mapa,
                "AUTH_ARGON2_PARALLELISM",
                ARGON2_PARALELISMO,
            )?,
        };

        let serpro = SerproConfig {
            token_url: url_https_obrigatoria(&mapa, "SERPRO_TOKEN_URL")?,
            sapi_auth_url: url_https_obrigatoria(&mapa, "SERPRO_SAPI_AUTH_URL")?,
            integra_trial_url: url_https_obrigatoria(&mapa, "SERPRO_INTEGRA_TRIAL_URL")?,
            integra_prod_url: url_https_obrigatoria(&mapa, "SERPRO_INTEGRA_PROD_URL")?,
            divida_ativa_trial_url: url_https_obrigatoria(&mapa, "SERPRO_DIVIDA_ATIVA_TRIAL_URL")?,
            divida_ativa_prod_url: url_https_opcional(&mapa, "SERPRO_DIVIDA_ATIVA_PROD_URL")?,
            trial_bearer_token: segredo_opcional(&mapa, "SERPRO_TRIAL_BEARER_TOKEN"),
            consumer_key: segredo_opcional(&mapa, "SERPRO_CONSUMER_KEY"),
            consumer_secret: segredo_opcional(&mapa, "SERPRO_CONSUMER_SECRET"),
            operator_header_enabled: booleano(&mapa, "SERPRO_OPERATOR_HEADER_ENABLED", true)?,
            divida_ativa_fallback_codigo_contribuinte: booleano(
                &mapa,
                "DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE",
                false,
            )?,
            rate_limit_consultas_por_segundo: inteiro_positivo_u32(
                &mapa,
                "SERPRO_RATE_LIMIT_CONSULTAS_POR_SEGUNDO",
                4,
            )?,
            http_connect_timeout_secs: inteiro_positivo(
                &mapa,
                "SERPRO_HTTP_CONNECT_TIMEOUT_SECS",
                5,
            )?,
            http_request_timeout_secs: inteiro_positivo(
                &mapa,
                "SERPRO_HTTP_REQUEST_TIMEOUT_SECS",
                30,
            )?,
            procuracao_cache_ttl_secs: inteiro_positivo(
                &mapa,
                "SERPRO_PROCURACAO_CACHE_TTL_SECS",
                300,
            )?,
        };

        let log_filter = opcional(&mapa, "RUST_LOG").unwrap_or_else(|| "info".to_owned());

        let consultas_retencao_dias = inteiro_positivo(&mapa, "CONSULTAS_RETENCAO_DIAS", 90)?;

        Ok(Self {
            app_env,
            port,
            database,
            jwt_secret: SecretString::from(jwt_secret),
            certificate_master_key,
            cors_allowed_origins,
            auth,
            serpro,
            consultas_retencao_dias,
            log_filter,
        })
    }
}

fn obrigatoria(mapa: &HashMap<String, String>, nome: &'static str) -> Result<String, ConfigError> {
    match mapa.get(nome) {
        Some(valor) if !valor.trim().is_empty() => Ok(valor.trim().to_owned()),
        _ => Err(ConfigError::Obrigatoria(nome)),
    }
}

fn opcional(mapa: &HashMap<String, String>, nome: &str) -> Option<String> {
    mapa.get(nome)
        .map(|valor| valor.trim())
        .filter(|valor| !valor.is_empty())
        .map(ToOwned::to_owned)
}

fn segredo_opcional(mapa: &HashMap<String, String>, nome: &str) -> Option<SecretString> {
    opcional(mapa, nome).map(SecretString::from)
}

fn validar_database_url(valor: String) -> Result<SecretString, ConfigError> {
    let url = Url::parse(&valor)
        .map_err(|_| ConfigError::Invalida("DATABASE_URL", "deve ser uma URL PostgreSQL válida"))?;
    if !matches!(url.scheme(), "postgres" | "postgresql") {
        return Err(ConfigError::Invalida(
            "DATABASE_URL",
            "deve usar o esquema postgres:// ou postgresql://",
        ));
    }
    if url.host_str().is_none() || url.path().trim_matches('/').is_empty() {
        return Err(ConfigError::Invalida(
            "DATABASE_URL",
            "deve informar host e nome do banco",
        ));
    }
    Ok(SecretString::from(valor))
}

fn validar_chave_mestre(valor: &str) -> Result<SecretSlice<u8>, ConfigError> {
    let bytes = STANDARD
        .decode(valor)
        .map_err(|_| ConfigError::Invalida("CERTIFICATE_MASTER_KEY", "deve ser base64 válido"))?;
    if bytes.len() != CERTIFICADO_CHAVE_BYTES {
        return Err(ConfigError::Invalida(
            "CERTIFICATE_MASTER_KEY",
            "deve decodificar exatamente 32 bytes",
        ));
    }
    Ok(SecretSlice::from(bytes))
}

fn url_https_obrigatoria(
    mapa: &HashMap<String, String>,
    nome: &'static str,
) -> Result<Url, ConfigError> {
    let valor = obrigatoria(mapa, nome)?;
    validar_url_https(nome, &valor)
}

fn url_https_opcional(
    mapa: &HashMap<String, String>,
    nome: &'static str,
) -> Result<Option<Url>, ConfigError> {
    match opcional(mapa, nome) {
        Some(valor) => validar_url_https(nome, &valor).map(Some),
        None => Ok(None),
    }
}

fn validar_url_https(nome: &'static str, valor: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(valor)
        .map_err(|_| ConfigError::Invalida(nome, "deve ser uma URL https válida"))?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(ConfigError::Invalida(nome, "deve ser uma URL https válida"));
    }
    Ok(url)
}

fn validar_origens_cors(valor: Option<String>) -> Result<Vec<String>, ConfigError> {
    let Some(valor) = valor else {
        return Ok(Vec::new());
    };

    valor
        .split(',')
        .map(str::trim)
        .filter(|origem| !origem.is_empty())
        .map(|origem| {
            let url = Url::parse(origem)
                .map_err(|_| ConfigError::Invalida("CORS_ALLOWED_ORIGINS", "origem inválida"))?;
            if !matches!(url.scheme(), "http" | "https")
                || url.host_str().is_none()
                || url.username() != ""
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
                || url.path() != "/"
            {
                return Err(ConfigError::Invalida(
                    "CORS_ALLOWED_ORIGINS",
                    "cada origem deve ser http(s)://host[:porta], sem caminho",
                ));
            }
            Ok(url.origin().ascii_serialization())
        })
        .collect()
}

fn booleano(
    mapa: &HashMap<String, String>,
    nome: &'static str,
    padrao: bool,
) -> Result<bool, ConfigError> {
    match opcional(mapa, nome) {
        None => Ok(padrao),
        Some(valor) => match valor.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ConfigError::Invalida(nome, "use true ou false")),
        },
    }
}

fn inteiro_positivo_u32(
    mapa: &HashMap<String, String>,
    nome: &'static str,
    padrao: u32,
) -> Result<u32, ConfigError> {
    let valor = inteiro_positivo(mapa, nome, u64::from(padrao))?;
    u32::try_from(valor)
        .map_err(|_| ConfigError::Invalida(nome, "deve caber em um inteiro de 32 bits"))
}

fn inteiro_positivo(
    mapa: &HashMap<String, String>,
    nome: &'static str,
    padrao: u64,
) -> Result<u64, ConfigError> {
    match opcional(mapa, nome) {
        None => Ok(padrao),
        Some(valor) => match valor.parse::<u64>() {
            Ok(numero) if numero > 0 => Ok(numero),
            _ => Err(ConfigError::Invalida(
                nome,
                "deve ser um número inteiro positivo",
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HashMap<String, String> {
        HashMap::from([
            ("APP_ENV".into(), "development".into()),
            (
                "DATABASE_URL".into(),
                "postgres://app:senha@localhost:5432/accurit".into(),
            ),
            ("JWT_SECRET".into(), "x".repeat(32)),
            ("CERTIFICATE_MASTER_KEY".into(), STANDARD.encode([7_u8; 32])),
            (
                "SERPRO_TOKEN_URL".into(),
                "https://exemplo.test/token".into(),
            ),
            (
                "SERPRO_SAPI_AUTH_URL".into(),
                "https://exemplo.test/auth".into(),
            ),
            (
                "SERPRO_INTEGRA_TRIAL_URL".into(),
                "https://exemplo.test/integra".into(),
            ),
            (
                "SERPRO_INTEGRA_PROD_URL".into(),
                "https://exemplo.test/integra-prod".into(),
            ),
            (
                "SERPRO_DIVIDA_ATIVA_TRIAL_URL".into(),
                "https://exemplo.test/divida".into(),
            ),
            (
                "SERPRO_TRIAL_BEARER_TOKEN".into(),
                "token-super-secreto".into(),
            ),
        ])
    }

    fn config(pairs: HashMap<String, String>) -> Result<Config, ConfigError> {
        Config::from_pairs(pairs)
    }

    #[test]
    fn carrega_configuracao_minima_com_padroes() {
        let cfg = config(base()).expect("config válida");
        assert_eq!(cfg.app_env, AppEnv::Development);
        assert_eq!(cfg.port, PORTA_PADRAO);
        assert!(cfg.cors_allowed_origins.is_empty());
        assert_eq!(cfg.serpro.http_request_timeout_secs, 30);
        assert!(cfg.serpro.operator_header_enabled);
        assert!(cfg.serpro.divida_ativa_prod_url.is_none());
        assert_eq!(cfg.auth.access_ttl_secs, AUTH_ACCESS_TTL_SEGUNDOS);
        assert_eq!(
            cfg.auth.refresh_ttl_admin_secs,
            AUTH_REFRESH_TTL_ADMIN_SEGUNDOS
        );
        assert_eq!(
            cfg.auth.refresh_ttl_client_secs,
            AUTH_REFRESH_TTL_CLIENT_SEGUNDOS
        );
        assert!(cfg.auth.cookie_secure);
    }

    #[test]
    fn rejeita_cookie_inseguro_fora_de_development() {
        let mut mapa = base();
        mapa.insert("APP_ENV".into(), "production".into());
        mapa.insert("AUTH_COOKIE_SECURE".into(), "false".into());
        assert_eq!(
            config(mapa).expect_err("deve falhar"),
            ConfigError::Invalida("AUTH_COOKIE_SECURE", "só pode ser false em development")
        );
    }

    #[test]
    fn aceita_cookie_inseguro_em_development() {
        let mut mapa = base();
        mapa.insert("AUTH_COOKIE_SECURE".into(), "false".into());
        assert!(!config(mapa).expect("config válida").auth.cookie_secure);
    }

    #[test]
    fn rejeita_variavel_obrigatoria_ausente() {
        let mut mapa = base();
        mapa.remove("JWT_SECRET");
        assert_eq!(
            config(mapa).expect_err("deve falhar"),
            ConfigError::Obrigatoria("JWT_SECRET")
        );
    }

    #[test]
    fn rejeita_placeholder_de_jwt_curto() {
        let mut mapa = base();
        mapa.insert("JWT_SECRET".into(), "change-me".into());
        assert_eq!(
            config(mapa).expect_err("deve falhar"),
            ConfigError::Invalida("JWT_SECRET", "deve ter pelo menos 32 bytes")
        );
    }

    #[test]
    fn rejeita_chave_mestre_com_tamanho_errado() {
        let mut mapa = base();
        mapa.insert("CERTIFICATE_MASTER_KEY".into(), STANDARD.encode([7_u8; 16]));
        assert_eq!(
            config(mapa).expect_err("deve falhar"),
            ConfigError::Invalida(
                "CERTIFICATE_MASTER_KEY",
                "deve decodificar exatamente 32 bytes"
            )
        );
    }

    #[test]
    fn rejeita_chave_mestre_base64_invalido() {
        let mut mapa = base();
        mapa.insert("CERTIFICATE_MASTER_KEY".into(), "change-me".into());
        assert_eq!(
            config(mapa).expect_err("deve falhar"),
            ConfigError::Invalida("CERTIFICATE_MASTER_KEY", "deve ser base64 válido")
        );
    }

    #[test]
    fn rejeita_database_url_nao_postgres() {
        let mut mapa = base();
        mapa.insert("DATABASE_URL".into(), "mysql://localhost/db".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("DATABASE_URL", _))
        ));
    }

    #[test]
    fn rejeita_porta_zero() {
        let mut mapa = base();
        mapa.insert("PORT".into(), "0".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("PORT", _))
        ));
    }

    #[test]
    fn rejeita_serpro_sem_https() {
        let mut mapa = base();
        mapa.insert(
            "SERPRO_TOKEN_URL".into(),
            "http://exemplo.test/token".into(),
        );
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("SERPRO_TOKEN_URL", _))
        ));
    }

    #[test]
    fn valida_e_normaliza_origens_cors() {
        let mut mapa = base();
        mapa.insert(
            "CORS_ALLOWED_ORIGINS".into(),
            " http://localhost:4200 , https://app.exemplo.test/ ".into(),
        );
        let cfg = config(mapa).expect("config válida");
        assert_eq!(
            cfg.cors_allowed_origins,
            vec![
                "http://localhost:4200".to_owned(),
                "https://app.exemplo.test".to_owned()
            ]
        );
    }

    #[test]
    fn rejeita_origem_cors_com_caminho() {
        let mut mapa = base();
        mapa.insert(
            "CORS_ALLOWED_ORIGINS".into(),
            "https://app.exemplo.test/admin".into(),
        );
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("CORS_ALLOWED_ORIGINS", _))
        ));
    }

    #[test]
    fn rejeita_booleano_invalido() {
        let mut mapa = base();
        mapa.insert("SERPRO_OPERATOR_HEADER_ENABLED".into(), "sim".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("SERPRO_OPERATOR_HEADER_ENABLED", _))
        ));
    }

    #[test]
    fn rejeita_timeout_zero() {
        let mut mapa = base();
        mapa.insert("SERPRO_HTTP_REQUEST_TIMEOUT_SECS".into(), "0".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("SERPRO_HTTP_REQUEST_TIMEOUT_SECS", _))
        ));
    }

    #[test]
    fn padrao_consultas_divida_ativa() {
        let cfg = config(base()).expect("config válida");
        assert_eq!(cfg.serpro.rate_limit_consultas_por_segundo, 4);
        assert_eq!(cfg.consultas_retencao_dias, 90);
        assert!(!cfg.serpro.divida_ativa_fallback_codigo_contribuinte);
    }

    #[test]
    fn rejeita_rate_limit_consultas_zero() {
        let mut mapa = base();
        mapa.insert("SERPRO_RATE_LIMIT_CONSULTAS_POR_SEGUNDO".into(), "0".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida(
                "SERPRO_RATE_LIMIT_CONSULTAS_POR_SEGUNDO",
                _
            ))
        ));
    }

    #[test]
    fn rejeita_retenciao_zero() {
        let mut mapa = base();
        mapa.insert("CONSULTAS_RETENCAO_DIAS".into(), "0".into());
        assert!(matches!(
            config(mapa),
            Err(ConfigError::Invalida("CONSULTAS_RETENCAO_DIAS", _))
        ));
    }

    #[test]
    fn fallback_divida_ativa_aceita_fio() {
        let mut mapa = base();
        mapa.insert(
            "DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE".into(),
            "true".into(),
        );
        let cfg = config(mapa).expect("config válida");
        assert!(cfg.serpro.divida_ativa_fallback_codigo_contribuinte);
    }

    #[test]
    fn debug_nao_expoe_segredos() {
        let cfg = config(base()).expect("config válida");
        let texto = format!("{cfg:?}");
        assert!(!texto.contains("token-super-secreto"));
        assert!(!texto.contains("senha"));
        assert!(!texto.contains(&STANDARD.encode([7_u8; 32])));
        assert!(texto.contains("REDACTED"));
    }
}
