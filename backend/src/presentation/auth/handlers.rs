use std::net::IpAddr;

use axum::Extension;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::application::auth::{
    AuthError, LoginCommand, LogoutCommand, RenovarCommand, RepositorioError, SessaoEmitida,
    UsuarioAtualCommand,
};
use crate::domain::auth::Audiencia;
use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;

use super::dto::{LoginRequest, LoginResponse, UsuarioAtualResponse, UsuarioResponse};
use super::middleware::RequestContext;
use super::state::AuthState;

const HEADER_CSRF: &str = "x-requested-with";
const VALOR_CSRF: &str = "accurit";

pub async fn login_admin(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
    corpo: Result<Json<LoginRequest>, JsonRejection>,
) -> Response {
    login(state.0, request_id, cabecalhos, corpo, Audiencia::Admin).await
}

pub async fn login_client(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
    corpo: Result<Json<LoginRequest>, JsonRejection>,
) -> Response {
    login(state.0, request_id, cabecalhos, corpo, Audiencia::Client).await
}

async fn login(
    state: AuthState,
    request_id: RequestId,
    cabecalhos: HeaderMap,
    corpo: Result<Json<LoginRequest>, JsonRejection>,
    audiencia: Audiencia,
) -> Response {
    let Json(corpo) = match corpo {
        Ok(corpo) => corpo,
        Err(_) => return ApiError::invalid_json().resposta(request_id.como_str()),
    };

    let comando = LoginCommand {
        escritorio: corpo.escritorio,
        email: corpo.email,
        senha: corpo.senha,
        audiencia,
        ip: ip_da_requisicao(&cabecalhos),
        user_agent: user_agent(&cabecalhos),
    };

    let login = state.login.clone();
    let resultado = tokio::task::spawn_blocking(move || login.executar(comando)).await;

    match resultado {
        Ok(Ok(sessao)) => resposta_sessao(state, audiencia, sessao),
        Ok(Err(erro)) => erro_auth(erro).resposta(request_id.como_str()),
        Err(_) => ApiError::erro_interno().resposta(request_id.como_str()),
    }
}

pub async fn refresh_admin(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
) -> Response {
    refresh(state.0, request_id, cabecalhos, Audiencia::Admin).await
}

pub async fn refresh_client(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
) -> Response {
    refresh(state.0, request_id, cabecalhos, Audiencia::Client).await
}

async fn refresh(
    state: AuthState,
    request_id: RequestId,
    cabecalhos: HeaderMap,
    audiencia: Audiencia,
) -> Response {
    if let Err(erro) = validar_csrf(&cabecalhos) {
        return erro.resposta(request_id.como_str());
    }

    let Some(refresh_token) = super::cookie::CookieSettings::extrair(audiencia, &cabecalhos) else {
        return com_cookie_limpo(
            ApiError::sessao_expirada().resposta(request_id.como_str()),
            state.cookie,
            audiencia,
        );
    };

    let comando = RenovarCommand {
        refresh_token,
        audiencia,
        ip: ip_da_requisicao(&cabecalhos),
        user_agent: user_agent(&cabecalhos),
    };

    let renovar = state.renovar.clone();
    let resultado = tokio::task::spawn_blocking(move || renovar.executar(comando)).await;

    match resultado {
        Ok(Ok(sessao)) => resposta_sessao(state, audiencia, sessao),
        Ok(Err(erro)) => com_cookie_limpo(
            erro_auth(erro).resposta(request_id.como_str()),
            state.cookie,
            audiencia,
        ),
        Err(_) => ApiError::erro_interno().resposta(request_id.como_str()),
    }
}

pub async fn logout_admin(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
) -> Response {
    logout(state.0, request_id, cabecalhos, Audiencia::Admin).await
}

pub async fn logout_client(
    state: State<AuthState>,
    request_id: RequestId,
    cabecalhos: HeaderMap,
) -> Response {
    logout(state.0, request_id, cabecalhos, Audiencia::Client).await
}

async fn logout(
    state: AuthState,
    request_id: RequestId,
    cabecalhos: HeaderMap,
    audiencia: Audiencia,
) -> Response {
    if let Err(erro) = validar_csrf(&cabecalhos) {
        return erro.resposta(request_id.como_str());
    }

    if let Some(refresh_token) = super::cookie::CookieSettings::extrair(audiencia, &cabecalhos) {
        let comando = LogoutCommand {
            refresh_token,
            audiencia,
            ip: ip_da_requisicao(&cabecalhos),
        };
        let logout = state.logout.clone();
        let resultado = tokio::task::spawn_blocking(move || logout.executar(comando)).await;
        match resultado {
            Ok(Ok(())) => {}
            Ok(Err(erro)) => {
                return com_cookie_limpo(
                    erro_auth(erro).resposta(request_id.como_str()),
                    state.cookie,
                    audiencia,
                );
            }
            Err(_) => return ApiError::erro_interno().resposta(request_id.como_str()),
        }
    }

    com_cookie_limpo(
        StatusCode::NO_CONTENT.into_response(),
        state.cookie,
        audiencia,
    )
}

pub async fn me_admin(
    state: State<AuthState>,
    request_id: RequestId,
    Extension(contexto): Extension<RequestContext>,
) -> Response {
    usuario_atual(state.0, request_id, contexto).await
}

pub async fn me_client(
    state: State<AuthState>,
    request_id: RequestId,
    Extension(contexto): Extension<RequestContext>,
) -> Response {
    usuario_atual(state.0, request_id, contexto).await
}

async fn usuario_atual(
    state: AuthState,
    request_id: RequestId,
    contexto: RequestContext,
) -> Response {
    let comando = UsuarioAtualCommand {
        escritorio_id: contexto.escritorio_id,
        usuario_id: contexto.usuario_id,
        audiencia: contexto.audiencia,
    };

    let usuario_atual = state.usuario_atual.clone();
    let resultado = tokio::task::spawn_blocking(move || usuario_atual.executar(comando)).await;

    match resultado {
        Ok(Ok(usuario)) => (
            StatusCode::OK,
            Json(UsuarioAtualResponse {
                usuario: UsuarioResponse::from(usuario),
            }),
        )
            .into_response(),
        Ok(Err(erro)) => erro_auth(erro).resposta(request_id.como_str()),
        Err(_) => ApiError::erro_interno().resposta(request_id.como_str()),
    }
}

fn resposta_sessao(state: AuthState, audiencia: Audiencia, sessao: SessaoEmitida) -> Response {
    let corpo = LoginResponse {
        access_token: sessao.access_token,
        expires_in: sessao.expires_in,
        usuario: UsuarioResponse::from(sessao.usuario),
    };

    let Ok(cookie) = HeaderValue::from_str(&state.cookie.criar(
        audiencia,
        &sessao.refresh_token,
        sessao.refresh_ttl_secs,
    )) else {
        tracing::error!("não foi possível montar o cookie de sessão");
        return ApiError::erro_interno().resposta("desconhecido");
    };

    let mut resposta = (StatusCode::OK, Json(corpo)).into_response();
    resposta.headers_mut().append(header::SET_COOKIE, cookie);
    resposta
}

fn com_cookie_limpo(
    mut resposta: Response,
    cookie: super::cookie::CookieSettings,
    audiencia: Audiencia,
) -> Response {
    if let Ok(valor) = HeaderValue::from_str(&cookie.limpar(audiencia)) {
        resposta.headers_mut().append(header::SET_COOKIE, valor);
    }
    resposta
}

fn validar_csrf(cabecalhos: &HeaderMap) -> Result<(), ApiError> {
    let content_type = cabecalhos
        .get(header::CONTENT_TYPE)
        .and_then(|valor| valor.to_str().ok())
        .unwrap_or("");
    if !content_type
        .split(';')
        .next()
        .is_some_and(|valor| valor.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(ApiError::csrf_ausente());
    }

    let solicitado = cabecalhos
        .get(HEADER_CSRF)
        .and_then(|valor| valor.to_str().ok())
        .unwrap_or("");
    if solicitado != VALOR_CSRF {
        return Err(ApiError::csrf_ausente());
    }

    Ok(())
}

fn erro_auth(erro: AuthError) -> ApiError {
    match erro {
        AuthError::CredenciaisInvalidas => ApiError::credenciais_invalidas(),
        AuthError::SessaoExpirada => ApiError::sessao_expirada(),
        AuthError::AudienciaIncompativel => ApiError::audiencia_incompativel(),
        AuthError::Repositorio(RepositorioError::Indisponivel) => ApiError::banco_indisponivel(),
        AuthError::Repositorio(RepositorioError::Dados)
        | AuthError::Senha(_)
        | AuthError::Token(_) => ApiError::erro_interno(),
    }
}

fn ip_da_requisicao(cabecalhos: &HeaderMap) -> Option<IpAddr> {
    cabecalhos
        .get("x-real-ip")
        .and_then(|valor| valor.to_str().ok())
        .map(str::trim)
        .and_then(|valor| valor.parse::<IpAddr>().ok())
}

fn user_agent(cabecalhos: &HeaderMap) -> Option<String> {
    cabecalhos
        .get(header::USER_AGENT)
        .and_then(|valor| valor.to_str().ok())
        .map(str::to_owned)
}
