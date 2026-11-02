//! Handlers enxutos da Dívida Ativa: extraem contexto/snapshot, chamam
//! o use case e mapeiam o resultado para DTOs. Nada de lógica de
//! integração aqui — tudo de dia use case (SPEC §4.3–§4.5).

use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::{ComandoConsultaDevedor, ComandoConsultaInscricao};
use crate::presentation::auth::RequestContext;
use crate::presentation::divida_ativa::dto::{
    RespostaDevedor, RespostaInscricao, mapear_erro_divida,
};
use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;
use crate::presentation::serpro::ambiente::AmbienteResolvido;

/// Handlers recebem o use case já composto — só extração e resposta.
#[derive(Clone)]
pub struct EstadoDividaAtiva {
    pub consultar_devedor: Arc<ConsultarDevedorUseCase>,
    pub consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
}

fn contexto_do_escritorio(contexto: &RequestContext) -> Result<Uuid, ApiError> {
    contexto.escritorio_id.ok_or_else(|| {
        tracing::warn!(
            usuario_id = %contexto.usuario_id,
            "audiência client sem escritorio_id no token"
        );
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token do portal sem escritório associado",
        )
    })
}

/// `GET /divida-ativa/devedores/{cpfCnpj}` — pesquisa por CPF/CNPJ.
pub async fn consultar_devedor(
    State(estado): State<EstadoDividaAtiva>,
    Extension(contexto): Extension<RequestContext>,
    Extension(snapshot): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Path(chave): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let cpf_cnpj = CpfCnpj::novo(&chave)
        .map_err(|_| ApiError::validacao("CPF/CNPJ inválido: informe 11 ou 14 dígitos válidos"))?;
    let escritorio_id = contexto_do_escritorio(&contexto)?;

    let comando = ComandoConsultaDevedor {
        escritorio_id,
        usuario_id: contexto.usuario_id,
        chave: cpf_cnpj,
    };

    let resultado = estado
        .consultar_devedor
        .executar(comando, snapshot.ambiente, request_id.como_str())
        .await
        .map_err(mapear_erro_divida)?;

    Ok(Json(RespostaDevedor::de(&resultado)))
}

/// `GET /divida-ativa/inscricoes/{numero}` — detalhe da inscrição.
pub async fn consultar_inscricao(
    State(estado): State<EstadoDividaAtiva>,
    Extension(contexto): Extension<RequestContext>,
    Extension(snapshot): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Path(numero): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let escritorio_id = contexto_do_escritorio(&contexto)?;
    let comando = ComandoConsultaInscricao::novo(escritorio_id, contexto.usuario_id, &numero)
        .map_err(mapear_erro_divida)?;

    let detalhe = estado
        .consultar_inscricao
        .executar(comando, snapshot.ambiente, request_id.como_str())
        .await
        .map_err(mapear_erro_divida)?;

    Ok(Json(RespostaInscricao::de(&detalhe)))
}
