use std::collections::HashSet;
use std::str::FromStr;

use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::auth::{
    ClaimsAccess, ClaimsAdminAccess, TokenEmitido, TokenError, TokenService,
};
use crate::domain::auth::{Administrador, Audiencia, Papel, Usuario};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    escritorio_id: String,
    role: String,
    aud: String,
    iat: i64,
    exp: i64,
    jti: String,
}

/// Claims do token do backoffice: no `escritorio_id` claim (ADR-0010, D3).
#[derive(Debug, Serialize, Deserialize)]
struct ClaimsBackoffice {
    sub: String,
    role: String,
    aud: String,
    iat: i64,
    exp: i64,
    jti: String,
}

pub struct JwtService {
    segredo: SecretString,
    access_ttl_secs: u64,
}

impl JwtService {
    pub const fn novo(segredo: SecretString, access_ttl_secs: u64) -> Self {
        Self {
            segredo,
            access_ttl_secs,
        }
    }
}

impl TokenService for JwtService {
    fn emitir(&self, usuario: &Usuario, audiencia: Audiencia) -> Result<TokenEmitido, TokenError> {
        let agora = Utc::now().timestamp();
        let exp = agora
            .checked_add(i64::try_from(self.access_ttl_secs).map_err(|_| TokenError::Invalido)?)
            .ok_or(TokenError::Invalido)?;

        let claims = Claims {
            sub: usuario.id.to_string(),
            escritorio_id: usuario.escritorio_id.to_string(),
            role: usuario.papel.como_str().to_owned(),
            aud: audiencia.como_str().to_owned(),
            iat: agora,
            exp,
            jti: Uuid::now_v7().to_string(),
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.segredo.expose_secret().as_bytes()),
        )
        .map_err(|_| TokenError::Invalido)?;

        Ok(TokenEmitido {
            token,
            expires_in: self.access_ttl_secs,
        })
    }

    fn validar(&self, token: &str, audiencia: Audiencia) -> Result<ClaimsAccess, TokenError> {
        let mut validacao = Validation::new(Algorithm::HS256);
        validacao.set_audience(&[audiencia.como_str()]);
        validacao.set_required_spec_claims(&["exp", "aud", "sub"]);
        validacao.validate_exp = true;
        validacao.leeway = 0;

        let dados = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.segredo.expose_secret().as_bytes()),
            &validacao,
        )
        .map_err(|erro| match erro.kind() {
            ErrorKind::ExpiredSignature => TokenError::Expirado,
            ErrorKind::InvalidAudience => TokenError::Audiencia,
            _ => TokenError::Invalido,
        })?;

        let claims = dados.claims;
        let usuario_id = Uuid::parse_str(&claims.sub).map_err(|_| TokenError::Invalido)?;
        let escritorio_id =
            Uuid::parse_str(&claims.escritorio_id).map_err(|_| TokenError::Invalido)?;
        let papel = Papel::from_str(&claims.role).map_err(|_| TokenError::Invalido)?;
        let audiencia_claim = Audiencia::from_str(&claims.aud).map_err(|_| TokenError::Invalido)?;
        let jti = Uuid::parse_str(&claims.jti).map_err(|_| TokenError::Invalido)?;

        Ok(ClaimsAccess {
            usuario_id,
            escritorio_id,
            papel,
            audiencia: audiencia_claim,
            jti,
        })
    }

    fn emitir_admin(
        &self,
        administrador: &Administrador,
        audiencia: Audiencia,
    ) -> Result<TokenEmitido, TokenError> {
        debug_assert_eq!(audiencia, Audiencia::Admin);
        let agora = Utc::now().timestamp();
        let exp = agora
            .checked_add(i64::try_from(self.access_ttl_secs).map_err(|_| TokenError::Invalido)?)
            .ok_or(TokenError::Invalido)?;

        let claims = ClaimsBackoffice {
            sub: administrador.id.to_string(),
            role: "ADMIN".to_owned(),
            aud: audiencia.como_str().to_owned(),
            iat: agora,
            exp,
            jti: Uuid::now_v7().to_string(),
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.segredo.expose_secret().as_bytes()),
        )
        .map_err(|_| TokenError::Invalido)?;

        Ok(TokenEmitido {
            token,
            expires_in: self.access_ttl_secs,
        })
    }

    fn validar_admin(
        &self,
        token: &str,
        audiencia: Audiencia,
    ) -> Result<ClaimsAdminAccess, TokenError> {
        debug_assert_eq!(audiencia, Audiencia::Admin);
        let mut validacao = Validation::new(Algorithm::HS256);
        validacao.set_audience(&[audiencia.como_str()]);
        validacao.set_required_spec_claims(&["exp", "aud", "sub"]);
        validacao.validate_exp = true;
        validacao.leeway = 0;

        let dados = decode::<ClaimsBackoffice>(
            token,
            &DecodingKey::from_secret(self.segredo.expose_secret().as_bytes()),
            &validacao,
        )
        .map_err(|erro| match erro.kind() {
            ErrorKind::ExpiredSignature => TokenError::Expirado,
            ErrorKind::InvalidAudience => TokenError::Audiencia,
            _ => TokenError::Invalido,
        })?;

        let claims = dados.claims;
        let administrador_id = Uuid::parse_str(&claims.sub).map_err(|_| TokenError::Invalido)?;
        let jti = Uuid::parse_str(&claims.jti).map_err(|_| TokenError::Invalido)?;

        Ok(ClaimsAdminAccess {
            administrador_id,
            audiencia,
            jti,
        })
    }

    fn ler_audiencia(&self, token: &str) -> Option<Audiencia> {
        let mut validacao = Validation::new(Algorithm::HS256);
        validacao.validate_exp = false;
        validacao.validate_aud = false;
        validacao.required_spec_claims = HashSet::new();

        let dados = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.segredo.expose_secret().as_bytes()),
            &validacao,
        )
        .ok()?;

        let aud = dados.claims.get("aud")?.as_str()?;
        Audiencia::from_str(aud).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::Usuario;

    fn usuario(papel: Papel) -> Usuario {
        Usuario {
            id: Uuid::now_v7(),
            escritorio_id: Uuid::now_v7(),
            email: "teste@exemplo.test".into(),
            senha_hash: "hash".into(),
            papel,
            ativo: true,
            bloqueado_ate: None,
        }
    }

    fn servico(ttl: u64) -> JwtService {
        JwtService::novo(SecretString::from("x".repeat(32)), ttl)
    }

    fn administrador() -> Administrador {
        Administrador {
            id: Uuid::now_v7(),
            email: "gestor@accurit.test".into(),
            senha_hash: "hash".into(),
            ativo: true,
            bloqueado_ate: None,
        }
    }

    #[test]
    fn emite_e_valida_token_da_audiencia_correta() {
        let usuario = usuario(Papel::Operador);
        let token = servico(900).emitir(&usuario, Audiencia::Client).unwrap();

        let claims = servico(900)
            .validar(&token.token, Audiencia::Client)
            .unwrap();
        assert_eq!(claims.usuario_id, usuario.id);
        assert_eq!(claims.escritorio_id, usuario.escritorio_id);
        assert_eq!(claims.papel, Papel::Operador);
        assert_eq!(claims.audiencia, Audiencia::Client);
        assert_eq!(token.expires_in, 900);
    }

    #[test]
    fn token_admin_nao_tem_escritorio_e_nao_vira_token_portal() {
        let administrador = administrador();
        let token = servico(900)
            .emitir_admin(&administrador, Audiencia::Admin)
            .unwrap();

        let claims = servico(900)
            .validar_admin(&token.token, Audiencia::Admin)
            .unwrap();
        assert_eq!(claims.administrador_id, administrador.id);
        assert_eq!(claims.audiencia, Audiencia::Admin);
        assert_eq!(token.expires_in, 900);

        // Sem o claim `escritorio_id`, o portal nunca consegue validar esse token como usuário.
        assert_eq!(
            servico(900)
                .validar(&token.token, Audiencia::Client)
                .unwrap_err(),
            TokenError::Invalido
        );
    }

    #[test]
    fn token_do_portal_nao_valida_no_backoffice() {
        let token = servico(900)
            .emitir(&usuario(Papel::Cliente), Audiencia::Client)
            .unwrap();
        // A audiência do claim não combina com o esperado pelo backoffice.
        assert_eq!(
            servico(900)
                .validar_admin(&token.token, Audiencia::Admin)
                .unwrap_err(),
            TokenError::Audiencia
        );
    }

    #[test]
    fn sonda_admin_token_no_validar_portal() {
        let token = servico(900)
            .emitir_admin(&administrador(), Audiencia::Admin)
            .unwrap();
        let resultado = servico(900).validar(&token.token, Audiencia::Client);
        eprintln!("SONDA: {resultado:?}");
    }

    #[test]
    fn le_audiencia_de_tokens_de_ambas_as_formas() {
        let portal = servico(900)
            .emitir(&usuario(Papel::Cliente), Audiencia::Client)
            .unwrap();
        let backoffice = servico(900)
            .emitir_admin(&administrador(), Audiencia::Admin)
            .unwrap();

        assert_eq!(
            servico(900).ler_audiencia(&portal.token),
            Some(Audiencia::Client)
        );
        assert_eq!(
            servico(900).ler_audiencia(&backoffice.token),
            Some(Audiencia::Admin)
        );
        assert_eq!(servico(900).ler_audiencia("token-fake"), None);
    }

    #[test]
    fn rejeita_token_de_outra_audiencia() {
        let token = servico(900)
            .emitir(&usuario(Papel::Cliente), Audiencia::Client)
            .unwrap();
        assert_eq!(
            servico(900)
                .validar(&token.token, Audiencia::Admin)
                .unwrap_err(),
            TokenError::Audiencia
        );
    }

    #[test]
    fn rejeita_token_expirado() {
        let token = servico(0)
            .emitir(&usuario(Papel::Operador), Audiencia::Client)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1_100));

        assert_eq!(
            servico(0)
                .validar(&token.token, Audiencia::Client)
                .unwrap_err(),
            TokenError::Expirado
        );
    }
}
