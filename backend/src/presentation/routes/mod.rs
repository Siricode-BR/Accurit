use axum::Router;
use axum::http::Request;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::application::admin::ProvisionarTenantUseCase;
use crate::application::certificados::{
    CriarCertificadoUseCase, ListarCertificadosUseCase, ObterCertificadoUseCase,
    PoliticaCertificados, RevogarCertificadoUseCase,
};
use crate::application::clientes::{
    AtualizarClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase, ListarClientesUseCase,
    ObterClienteUseCase, PoliticaClientes,
};
use crate::application::ports::SaudeBanco;
use crate::infrastructure::auth::HashArgon2;
use crate::infrastructure::config::{BURST_RATE_LIMIT, Config, REQUISICOES_POR_SEGUNDO};
use crate::infrastructure::crypto::criar_criptografia;
use crate::infrastructure::persistence::admin::AdminRepositorioDiesel;
use crate::infrastructure::persistence::certificados::CertificadoRepositorioDiesel;
use crate::infrastructure::persistence::clientes::ClienteRepositorioDiesel;
use crate::presentation::admin::handlers::provisionar_tenant;
use crate::presentation::auth::{AuthState, ExigirAudiencia, construir as construir_auth, exigir};
use crate::presentation::certificados::construir_certificados;
use crate::presentation::clientes::router::construir_clientes;
use crate::presentation::handlers;
use crate::presentation::middleware::{cors, rate_limit, request_id, security_headers};
use crate::presentation::state::AppState;
use secrecy::ExposeSecret;

pub fn construir<S>(estado: AppState<S>, auth: Option<AuthState>, config: &Config) -> Router
where
    S: SaudeBanco + 'static,
{
    let limite = rate_limit::LimiteRequisicoes::novo(REQUISICOES_POR_SEGUNDO, BURST_RATE_LIMIT);

    let pool = estado.pool.clone();
    let cliente_repo = ClienteRepositorioDiesel::novo(pool.clone());
    let admin_repo = AdminRepositorioDiesel::novo(pool.clone());
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");

    let criar_clientes = CriarClienteUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        std::sync::Arc::new(cliente_repo.clone()),
        PoliticaClientes::default(),
    );
    let obter_clientes = ObterClienteUseCase::novo(std::sync::Arc::new(cliente_repo.clone()));
    let listar_clientes = ListarClientesUseCase::novo(std::sync::Arc::new(cliente_repo.clone()));
    let atualizar_clientes = AtualizarClienteUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        std::sync::Arc::new(cliente_repo.clone()),
        PoliticaClientes::default(),
    );
    let excluir_clientes = ExcluirClienteUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        std::sync::Arc::new(cliente_repo.clone()),
    );

    let provisionar_tenant_uc = ProvisionarTenantUseCase::novo(
        std::sync::Arc::new(admin_repo.clone()),
        std::sync::Arc::new(admin_repo.clone()),
        std::sync::Arc::new(admin_repo.clone()),
        std::sync::Arc::new(argon2),
    );

    let certificado_repo = CertificadoRepositorioDiesel::novo(std::sync::Arc::new(pool.clone()));
    let criptografia = criar_criptografia(config.certificate_master_key.expose_secret())
        .expect("criptografia deve inicializar");
    let auditoria_certificado = certificado_repo.clone();

    let criar_certificados = CriarCertificadoUseCase::novo(
        std::sync::Arc::new(certificado_repo.clone()),
        std::sync::Arc::new(auditoria_certificado.clone()),
        criptografia,
        PoliticaCertificados::nova(),
    );
    let listar_certificados =
        ListarCertificadosUseCase::novo(std::sync::Arc::new(certificado_repo.clone()));
    let obter_certificados =
        ObterCertificadoUseCase::novo(std::sync::Arc::new(certificado_repo.clone()));
    let revogar_certificados = RevogarCertificadoUseCase::novo(
        std::sync::Arc::new(certificado_repo.clone()),
        std::sync::Arc::new(auditoria_certificado.clone()),
    );

    let mut api = Router::new()
        .route("/health", get(handlers::saude::<S>))
        .with_state(estado);

    if let Some(auth_state) = auth.clone() {
        api = api.nest("/auth", construir_auth(auth_state.clone()));

        api = api.nest(
            "/clientes",
            construir_clientes(
                criar_clientes,
                obter_clientes,
                listar_clientes,
                atualizar_clientes,
                excluir_clientes,
                auth_state.clone(),
            ),
        );

        api = api.nest(
            "/admin",
            Router::new()
                .route("/escritorios", post(provisionar_tenant))
                .route_layer(axum_middleware::from_fn_with_state(
                    ExigirAudiencia::novo(
                        auth_state.tokens.clone(),
                        crate::domain::auth::Audiencia::Admin,
                    ),
                    exigir,
                ))
                .with_state(provisionar_tenant_uc),
        );

        api = api.nest(
            "/certificados",
            construir_certificados(
                criar_certificados,
                listar_certificados,
                obter_certificados,
                revogar_certificados,
                auth_state.clone(),
            ),
        );
    }

    let api = api
        .fallback(handlers::nao_encontrado)
        .method_not_allowed_fallback(handlers::metodo_nao_permitido)
        .layer(axum_middleware::from_fn_with_state(
            limite,
            rate_limit::middleware,
        ))
        .layer(cors::camada(&config.cors_allowed_origins))
        .layer(axum_middleware::from_fn(security_headers::middleware));

    Router::new()
        .nest("/api/v1", api)
        .fallback(handlers::nao_encontrado)
        .method_not_allowed_fallback(handlers::metodo_nao_permitido)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|requisicao: &Request<_>| {
                    let request_id = requisicao
                        .extensions()
                        .get::<request_id::RequestId>()
                        .map(request_id::RequestId::como_str)
                        .unwrap_or("-");

                    tracing::info_span!(
                        "http_request",
                        method = %requisicao.method(),
                        path = %requisicao.uri().path(),
                        request_id = %request_id
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(axum_middleware::from_fn(request_id::middleware))
}
