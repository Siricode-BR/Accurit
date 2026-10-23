use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::str::FromStr;
use uuid::Uuid;

use crate::application::clientes::{
    AtualizarClienteCommand, AtualizarClienteUseCase, CriarClienteCommand, CriarClienteUseCase,
    ExcluirClienteCommand, ExcluirClienteUseCase, ListarClientesCommand, ListarClientesUseCase,
    ObterClienteCommand, ObterClienteUseCase,
};
use crate::domain::auth::Papel;
use crate::presentation::auth::RequestContext;
use crate::presentation::clientes::dto::{
    AtualizarClienteRequest, CriarClienteRequest, ListagemClientesResponse, ListarClientesQuery,
};
use crate::presentation::error::ApiError;

pub async fn criar_cliente(
    State(criar_use_case): State<CriarClienteUseCase>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<CriarClienteRequest>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Operador && context.papel != Papel::Cliente {
        return Err(ApiError::proibido());
    }

    let porte_regime = payload
        .porte_regime
        .as_deref()
        .and_then(|s| crate::domain::clientes::PorteRegime::from_str(s).ok());

    let comando = CriarClienteCommand {
        escritorio_id: context.escritorio_id.ok_or_else(ApiError::proibido)?,
        tipo: payload.tipo,
        porte_regime,
        cpf_cnpj: payload.cpf_cnpj,
        nome_razao_social: payload.nome_razao_social,
        nome_fantasia: payload.nome_fantasia,
        email: payload.email,
        telefone: payload.telefone,
        ip: context.ip,
        usuario_id: context.usuario_id,
    };

    let cliente = criar_use_case.executar(comando).await?;
    Ok((StatusCode::CREATED, Json(cliente)).into_response())
}

pub async fn obter_cliente(
    State(obter_use_case): State<ObterClienteUseCase>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?;

    let comando = ObterClienteCommand { escritorio_id, id };

    let cliente = obter_use_case.executar(comando).await?;
    Ok(Json(cliente).into_response())
}

pub async fn listar_clientes(
    State(listar_use_case): State<ListarClientesUseCase>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ListarClientesQuery>,
) -> Result<Response, ApiError> {
    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?;

    let (filtro, paginacao) = query.para_filtro_e_paginacao()?;

    let comando = ListarClientesCommand {
        escritorio_id,
        filtro,
        paginacao,
    };

    let listagem = listar_use_case.executar(comando).await?;
    Ok(Json(ListagemClientesResponse::from(listagem)).into_response())
}

pub async fn atualizar_cliente(
    State(atualizar_use_case): State<AtualizarClienteUseCase>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AtualizarClienteRequest>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Operador {
        return Err(ApiError::proibido());
    }

    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?;

    let porte_regime = payload
        .porte_regime
        .and_then(|opt| opt.and_then(|s| crate::domain::clientes::PorteRegime::from_str(&s).ok()));

    let comando = AtualizarClienteCommand {
        escritorio_id,
        id,
        tipo: payload.tipo,
        porte_regime,
        cpf_cnpj: payload.cpf_cnpj,
        nome_razao_social: payload.nome_razao_social,
        nome_fantasia: payload.nome_fantasia.flatten(),
        email: payload.email.flatten(),
        telefone: payload.telefone.flatten(),
        ip: context.ip,
        usuario_id: context.usuario_id,
    };

    let cliente = atualizar_use_case.executar(comando).await?;
    Ok(Json(cliente).into_response())
}

pub async fn excluir_cliente(
    State(excluir_use_case): State<ExcluirClienteUseCase>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Operador {
        return Err(ApiError::proibido());
    }

    let escritorio_id = context.escritorio_id.ok_or_else(ApiError::proibido)?;

    let comando = ExcluirClienteCommand {
        escritorio_id,
        id,
        ip: context.ip,
        usuario_id: context.usuario_id,
    };

    excluir_use_case.executar(comando).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn erro_para_resposta(erro: crate::application::clientes::ClienteError) -> Response {
    match erro {
        crate::application::clientes::ClienteError::Dominio(e) => {
            ApiError::validacao(e.to_string()).resposta("")
        }
        crate::application::clientes::ClienteError::CpfCnpj(e) => {
            ApiError::validacao(e.to_string()).resposta("")
        }
        crate::application::clientes::ClienteError::Repositorio(e) => match e {
            crate::application::clientes::RepositorioError::Indisponivel => {
                ApiError::indisponivel().resposta("")
            }
            crate::application::clientes::RepositorioError::Dados => {
                ApiError::erro_interno().resposta("")
            }
            crate::application::clientes::RepositorioError::NaoEncontrado => {
                ApiError::nao_encontrado("cliente").resposta("")
            }
            crate::application::clientes::RepositorioError::Duplicado => {
                ApiError::conflito("CPF/CNPJ já cadastrado neste escritório").resposta("")
            }
        },
        crate::application::clientes::ClienteError::Validacao(msg) => {
            ApiError::validacao(msg).resposta("")
        }
        crate::application::clientes::ClienteError::AcessoNegado => {
            ApiError::proibido().resposta("")
        }
    }
}
