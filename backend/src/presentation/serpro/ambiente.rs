use axum::http::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::serpro::{ObterConfiguracaoSerproCommand, ObterConfiguracaoSerproUseCase};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::presentation::auth::RequestContext;

/// Snapshot imutável do ambiente efetivo do escritório, resolvido **uma
/// única vez** no início da requisição (SPEC §8: "o backend é a autoridade:
/// lê a flag no início de cada requisição"). Fica em `Extension`.
///
/// Requisições já em andamento mantêm a cópia local: trocas de modo não as
/// afetam (SPEC §8, "não trocar de ambiente no meio da requisição").
#[derive(Debug, Clone, Copy)]
pub struct AmbienteResolvido {
    pub ambiente: AmbienteSerpro,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

impl AmbienteResolvido {
    pub fn novo(modo_trial_serpro: bool, atualizado_em: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            ambiente: if modo_trial_serpro {
                AmbienteSerpro::Trial
            } else {
                AmbienteSerpro::Producao
            },
            atualizado_em,
        }
    }
}

/// Estado do middleware de snapshot: reusa o use case de leitura de
/// configuração (mesma fonte de verdade do PATCH e do use case do handler).
#[derive(Clone)]
pub struct EstadoAmbiente {
    pub configuracao: Arc<ObterConfiguracaoSerproUseCase>,
}

/// Middleware `from_fn_with_state`: exige `RequestContext` (rotas autenticadas
/// do portal), resolve o ambiente do escritório do token e insere
/// `AmbienteResolvido` antes do handler. Falha de repositório vira 503;
/// escritório ausente vira 404.
pub async fn resolver_ambiente(
    estado: axum::extract::State<EstadoAmbiente>,
    mut requisicao: Request<axum::body::Body>,
    proximo: axum::middleware::Next,
) -> Response {
    let contexto = requisicao.extensions().get::<RequestContext>().cloned();
    let Some(contexto) = contexto else {
        return erro_interno("contexto do portal ausente na requisição");
    };
    let Some(escritorio_id): Option<Uuid> = contexto.escritorio_id else {
        return erro_interno("escritorio_id ausente no token do portal");
    };

    match estado
        .configuracao
        .executar(ObterConfiguracaoSerproCommand { escritorio_id })
        .await
    {
        Ok(resultado) => {
            let snapshot =
                AmbienteResolvido::novo(resultado.modo_trial_serpro, resultado.atualizado_em);
            requisicao.extensions_mut().insert(snapshot);
            proximo.run(requisicao).await
        }
        Err(erro) => super::handlers::map_obter_para_api(erro).into_response(),
    }
}

fn erro_interno(detalhe: &'static str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, detalhe).into_response()
}
