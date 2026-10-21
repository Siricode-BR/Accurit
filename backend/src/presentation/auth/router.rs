use axum::Router;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};

use crate::domain::auth::Audiencia;

use super::handlers;
use super::middleware::ExigirAudiencia;
use super::rate_limit;
use super::state::AuthState;

pub fn construir(estado: AuthState) -> Router {
    let limiter = estado.limite_login.clone();
    let login = Router::new()
        .route("/admin/login", post(handlers::login_admin))
        .route("/client/login", post(handlers::login_client))
        .route_layer(axum_middleware::from_fn_with_state(
            limiter,
            rate_limit::middleware,
        ));

    let admin = Router::new()
        .route("/admin/me", get(handlers::me_admin))
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(estado.tokens.clone(), Audiencia::Admin),
            super::middleware::exigir,
        ));

    let client = Router::new()
        .route("/client/me", get(handlers::me_client))
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(estado.tokens.clone(), Audiencia::Client),
            super::middleware::exigir,
        ));

    let sessao = Router::new()
        .route("/admin/refresh", post(handlers::refresh_admin))
        .route("/admin/logout", post(handlers::logout_admin))
        .route("/client/refresh", post(handlers::refresh_client))
        .route("/client/logout", post(handlers::logout_client));

    Router::new()
        .merge(login)
        .merge(sessao)
        .merge(admin)
        .merge(client)
        .with_state(estado)
}
