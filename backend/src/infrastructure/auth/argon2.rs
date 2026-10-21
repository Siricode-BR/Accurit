use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};

use crate::application::auth::{SenhaError, SenhaService};
use crate::infrastructure::config::AuthConfig;

const SENHA_FICTICIA: &str = "accurit-senha-ficticia-sem-uso";

pub struct HashArgon2 {
    argon2: Argon2<'static>,
    hash_ficticio: String,
}

impl HashArgon2 {
    pub fn novo(config: &AuthConfig) -> Result<Self, SenhaError> {
        let params = Params::new(
            config.argon2_memory_kib,
            config.argon2_iterations,
            config.argon2_parallelism,
            None,
        )
        .map_err(|_| SenhaError)?;

        let mut servico = Self {
            argon2: Argon2::new(Algorithm::Argon2id, Version::V0x13, params),
            hash_ficticio: String::new(),
        };
        servico.hash_ficticio = servico.gerar_hash(SENHA_FICTICIA)?;
        Ok(servico)
    }

    pub fn gerar_hash(&self, senha: &str) -> Result<String, SenhaError> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(senha.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| SenhaError)
    }
}

impl SenhaService for HashArgon2 {
    fn verificar(&self, senha: &str, hash: &str) -> bool {
        let Ok(hash) = PasswordHash::new(hash) else {
            return false;
        };
        self.argon2.verify_password(senha.as_bytes(), &hash).is_ok()
    }

    fn hash_ficticio(&self) -> &str {
        &self.hash_ficticio
    }

    fn gerar_hash(&self, senha: &str) -> Result<String, SenhaError> {
        HashArgon2::gerar_hash(self, senha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AuthConfig {
        AuthConfig {
            access_ttl_secs: 900,
            refresh_ttl_admin_secs: 28_800,
            refresh_ttl_client_secs: 604_800,
            max_failed_attempts: 5,
            lockout_secs: 900,
            cookie_secure: false,
            argon2_memory_kib: 8,
            argon2_iterations: 1,
            argon2_parallelism: 1,
        }
    }

    #[test]
    fn gera_e_verifica_hash_argon2id() {
        let servico = HashArgon2::novo(&config()).expect("hash deve inicializar");
        let hash = servico.gerar_hash("Senha-forte-123").expect("hash");
        assert!(hash.starts_with("$argon2id$"));
        assert!(servico.verificar("Senha-forte-123", &hash));
        assert!(!servico.verificar("Senha-errada", &hash));
    }

    #[test]
    fn hash_ficticio_e_valido_e_verificavel() {
        let servico = HashArgon2::novo(&config()).expect("hash deve inicializar");
        assert!(servico.verificar(SENHA_FICTICIA, servico.hash_ficticio()));
    }
}
