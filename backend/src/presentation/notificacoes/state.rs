//! Fase C — Estado de apresentação para notificações.

use std::sync::Arc;

use crate::application::notificacoes::{
    BuscarPreferenciaUseCase, ContarNotificacoesNaoLidasUseCase,
    CriarNotificacaoUseCase, CriarOuAtualizarPreferenciaUseCase,
    ListarNotificacoesUseCase, ListarTemplatesNotificacaoUseCase,
    MarcarNotificacaoLidaUseCase, MarcarTodasNotificacoesLidasUseCase,
};

#[derive(Clone)]
pub struct EstadoNotificacoes {
    pub criar_notificacao: Arc<CriarNotificacaoUseCase>,
    pub listar_notificacoes: Arc<ListarNotificacoesUseCase>,
    pub contar_nao_lidas: Arc<ContarNotificacoesNaoLidasUseCase>,
    pub marcar_como_lida: Arc<MarcarNotificacaoLidaUseCase>,
    pub marcar_todas_como_lidas: Arc<MarcarTodasNotificacoesLidasUseCase>,
    pub buscar_preferencia: Arc<BuscarPreferenciaUseCase>,
    pub criar_ou_atualizar_preferencia: Arc<CriarOuAtualizarPreferenciaUseCase>,
    pub listar_templates: Arc<ListarTemplatesNotificacaoUseCase>,
}

#[allow(clippy::too_many_arguments)]
impl EstadoNotificacoes {
    pub fn novo(
        criar_notificacao: Arc<CriarNotificacaoUseCase>,
        listar_notificacoes: Arc<ListarNotificacoesUseCase>,
        contar_nao_lidas: Arc<ContarNotificacoesNaoLidasUseCase>,
        marcar_como_lida: Arc<MarcarNotificacaoLidaUseCase>,
        marcar_todas_como_lidas: Arc<MarcarTodasNotificacoesLidasUseCase>,
        buscar_preferencia: Arc<BuscarPreferenciaUseCase>,
        criar_ou_atualizar_preferencia: Arc<CriarOuAtualizarPreferenciaUseCase>,
        listar_templates: Arc<ListarTemplatesNotificacaoUseCase>,
    ) -> Self {
        Self {
            criar_notificacao,
            listar_notificacoes,
            contar_nao_lidas,
            marcar_como_lida,
            marcar_todas_como_lidas,
            buscar_preferencia,
            criar_ou_atualizar_preferencia,
            listar_templates,
        }
    }
}
