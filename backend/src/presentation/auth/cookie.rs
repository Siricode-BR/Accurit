use axum::http::{HeaderMap, header};
use cookie::time::Duration as CookieDuration;
use cookie::{Cookie, SameSite};

use crate::domain::auth::Audiencia;

pub const NOME_COOKIE_ADMIN: &str = "accurit_admin_rt";
pub const NOME_COOKIE_CLIENT: &str = "accurit_client_rt";
pub const CAMINHO_COOKIE_ADMIN: &str = "/api/v1/auth/admin";
pub const CAMINHO_COOKIE_CLIENT: &str = "/api/v1/auth/client";

#[derive(Debug, Clone, Copy)]
pub struct CookieSettings {
    pub secure: bool,
}

impl CookieSettings {
    pub const fn nome(audiencia: Audiencia) -> &'static str {
        match audiencia {
            Audiencia::Admin => NOME_COOKIE_ADMIN,
            Audiencia::Client => NOME_COOKIE_CLIENT,
        }
    }

    pub const fn caminho(audiencia: Audiencia) -> &'static str {
        match audiencia {
            Audiencia::Admin => CAMINHO_COOKIE_ADMIN,
            Audiencia::Client => CAMINHO_COOKIE_CLIENT,
        }
    }

    pub fn criar(self, audiencia: Audiencia, valor: &str, max_age_secs: u64) -> String {
        Cookie::build((Self::nome(audiencia), valor))
            .path(Self::caminho(audiencia))
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(self.secure)
            .max_age(CookieDuration::seconds(
                i64::try_from(max_age_secs).unwrap_or(i64::MAX),
            ))
            .build()
            .to_string()
    }

    pub fn limpar(self, audiencia: Audiencia) -> String {
        Cookie::build((Self::nome(audiencia), ""))
            .path(Self::caminho(audiencia))
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(self.secure)
            .max_age(CookieDuration::ZERO)
            .build()
            .to_string()
    }

    pub fn extrair(audiencia: Audiencia, cabecalhos: &HeaderMap) -> Option<String> {
        let cabecalho = cabecalhos.get(header::COOKIE)?.to_str().ok()?;
        Cookie::split_parse(cabecalho)
            .filter_map(Result::ok)
            .find(|cookie| cookie.name() == Self::nome(audiencia))
            .map(|cookie| cookie.value().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_tem_atributos_de_seguranca() {
        let settings = CookieSettings { secure: true };
        let cookie = settings.criar(Audiencia::Admin, "token", 900);
        assert!(cookie.contains("accurit_admin_rt=token"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("Path=/api/v1/auth/admin"));
        assert!(cookie.contains("Max-Age=900"));
    }

    #[test]
    fn extrai_cookie_da_audiencia_correta() {
        let mut cabecalhos = HeaderMap::new();
        cabecalhos.insert(
            header::COOKIE,
            "accurit_admin_rt=abc; accurit_client_rt=def"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            CookieSettings::extrair(Audiencia::Admin, &cabecalhos),
            Some("abc".to_owned())
        );
        assert_eq!(
            CookieSettings::extrair(Audiencia::Client, &cabecalhos),
            Some("def".to_owned())
        );
    }
}
