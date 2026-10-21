use std::str::FromStr;

use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::auth::{ClaimsAccess, TokenEmitido, TokenError, TokenService};
use crate::domain::auth::{Audiencia, Papel, Usuario};

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

    #[test]
    fn emite_e_valida_token_da_audiencia_correta() {
        let usuario = usuario(Papel::Admin);
        let token = servico(900).emitir(&usuario, Audiencia::Admin).unwrap();

        let claims = servico(900)
            .validar(&token.token, Audiencia::Admin)
            .unwrap();
        assert_eq!(claims.usuario_id, usuario.id);
        assert_eq!(claims.escritorio_id, usuario.escritorio_id);
        assert_eq!(claims.papel, Papel::Admin);
        assert_eq!(claims.audiencia, Audiencia::Admin);
        assert_eq!(token.expires_in, 900);
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
            .emitir(&usuario(Papel::Admin), Audiencia::Admin)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1_100));

        assert_eq!(
            servico(0)
                .validar(&token.token, Audiencia::Admin)
                .unwrap_err(),
            TokenError::Expirado
        );
    }
}
