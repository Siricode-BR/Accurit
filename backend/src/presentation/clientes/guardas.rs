//! Fase E — Guardas de acesso à carteira nas rotas de clientes.
//!
//! O `RequestContext` carrega papel e escopos do token, mas função e carteira
//! vivem no banco (`funcoes_usuarios`/`carteira_usuarios`) e são lidas por
//! requisição. Escrever exige função com permissão; ver um cliente específico
//! exige que ele esteja dentro da carteira do usuário (404 se fora — não vaza
//! existência, mesmo padrão do IDOR do SPEC).

use std::sync::Arc;

use uuid::Uuid;

use crate::application::clientes::VerificarAcessoUseCase;
use crate::domain::clientes::AcessoCarteira;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;

/// Acesso do usuário do contexto, lido por requisição.
pub fn acesso_do_contexto(
    verificar: &Arc<VerificarAcessoUseCase>,
    contexto: &RequestContext,
) -> Result<AcessoCarteira, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;
    verificar
        .executar(escritorio_id, contexto.usuario_id, contexto.papel)
        .map_err(ApiError::from)
}

/// Exige permissão de escrita da função e, se há cliente, visibilidade.
pub fn exigir_escrita(
    acesso: &AcessoCarteira,
    cliente_id: Option<Uuid>,
) -> Result<(), ApiError> {
    if !acesso.funcao.pode_escrever_clientes() {
        return Err(ApiError::proibido());
    }
    exigir_ver(acesso, cliente_id)
}

/// Exige visibilidade; cliente fora da carteira responde 404.
pub fn exigir_ver(acesso: &AcessoCarteira, cliente_id: Option<Uuid>) -> Result<(), ApiError> {
    if let Some(cliente_id) = cliente_id && !acesso.pode_ver(cliente_id) {
        return Err(ApiError::nao_encontrado("cliente"));
    }
    Ok(())
}
