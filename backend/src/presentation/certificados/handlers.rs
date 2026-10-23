use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;

use crate::application::certificados::{
    CriarCertificadoCommand, CriarCertificadoUseCase, ListarCertificadosCommand,
    ListarCertificadosUseCase, ObterCertificadoCommand, ObterCertificadoUseCase,
    RevogarCertificadoCommand, RevogarCertificadoUseCase,
};
use crate::domain::auth::Papel;
use crate::presentation::auth::RequestContext;
use crate::presentation::certificados::dto::{
    CertificadoResponse, CriarCertificadoRequest, ListagemCertificadosResponse,
    ListarCertificadosQuery,
};
use crate::presentation::error::ApiError;

pub async fn criar_certificado(
    State(criar_use_case): State<CriarCertificadoUseCase>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<CriarCertificadoRequest>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Operador && context.papel != Papel::Cliente {
        return Err(ApiError::proibido());
    }

    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?.into();

    let conteudo = general_purpose::STANDARD
        .decode(&payload.conteudo_base64)
        .map_err(|_| ApiError::validacao("conteudo_base64 inválido"))?;

    let algoritmo = payload
        .algoritmo
        .unwrap_or_else(|| "AES-256-GCM".to_string());

    let comando = CriarCertificadoCommand {
        escritorio_id,
        tipo: payload
            .tipo
            .parse()
            .map_err(|_| ApiError::validacao("tipo inválido"))?,
        titular_cpf_cnpj: payload.titular_cpf_cnpj,
        conteudo,
        senha: payload.senha,
        algoritmo,
        fingerprint: payload.fingerprint,
        valido_de: payload.valido_de,
        valido_ate: payload.valido_ate,
        ip: context.ip,
        usuario_id: context.usuario_id,
    };

    let certificado = criar_use_case.executar(comando).await?;
    Ok((
        StatusCode::CREATED,
        Json(CertificadoResponse::from(certificado)),
    )
        .into_response())
}

pub async fn listar_certificados(
    State(listar_use_case): State<ListarCertificadosUseCase>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ListarCertificadosQuery>,
) -> Result<Response, ApiError> {
    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?.into();

    let (apenas_validos, pagina, tamanho) = query.para_comando()?;

    let comando = ListarCertificadosCommand {
        escritorio_id,
        apenas_validos,
        pagina,
        tamanho,
    };

    let listagem = listar_use_case.executar(comando).await?;
    Ok(Json(ListagemCertificadosResponse::from(listagem)).into_response())
}

pub async fn obter_certificado(
    State(obter_use_case): State<ObterCertificadoUseCase>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?.into();

    let comando = ObterCertificadoCommand { escritorio_id, id };

    let certificado = obter_use_case.executar(comando).await?;
    Ok(Json(CertificadoResponse::from(certificado)).into_response())
}

pub async fn revogar_certificado(
    State(revogar_use_case): State<RevogarCertificadoUseCase>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Operador {
        return Err(ApiError::proibido());
    }

    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?.into();

    let comando = RevogarCertificadoCommand {
        escritorio_id,
        id,
        ip: context.ip,
        usuario_id: context.usuario_id,
    };

    revogar_use_case.executar(comando).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn erro_para_resposta(erro: crate::application::certificados::CertificadoError) -> Response {
    match erro {
        crate::application::certificados::CertificadoError::Dominio(e) => {
            ApiError::validacao(e.to_string()).resposta("")
        }
        crate::application::certificados::CertificadoError::Repositorio(e) => match e {
            crate::application::certificados::RepositorioError::Indisponivel => {
                ApiError::indisponivel().resposta("")
            }
            crate::application::certificados::RepositorioError::Dados => {
                ApiError::erro_interno().resposta("")
            }
            crate::application::certificados::RepositorioError::NaoEncontrado => {
                ApiError::nao_encontrado("certificado").resposta("")
            }
            crate::application::certificados::RepositorioError::FingerprintDuplicado => {
                ApiError::conflito("Certificado com este fingerprint já existe").resposta("")
            }
        },
        crate::application::certificados::CertificadoError::Criptografia => {
            ApiError::erro_interno().resposta("")
        }
        crate::application::certificados::CertificadoError::AcessoNegado => {
            ApiError::proibido().resposta("")
        }
        crate::application::certificados::CertificadoError::Fingerprint(e) => {
            ApiError::validacao(e.to_string()).resposta("")
        }
    }
}
