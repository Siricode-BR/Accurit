//! Fase B.1 — Job runner interno: worker tokio que reivindica trabalhos
//! agendados e lotes da fila persistida, executa pelos casos de uso
//! existentes (medição com origem `rotina`/`lote`), aplica backoff
//! exponencial com teto, renova leases e notifica falhas/conclusões.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{Semaphore, watch};
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::application::integra_contador::ExecutarIntegraContadorUseCase;
use crate::application::notificacoes::CriarNotificacaoUseCase;
use crate::application::usuarios::{FiltroUsuarios, UsuarioGestaoRepositorio};
use crate::domain::auth::Papel;
use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::{
    ComandoConsultaDevedor, ComandoConsultaInscricao, ErroDividaAtiva,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::notificacoes::entidades::NovoNotificacao;
use crate::domain::notificacoes::enums::{CanalNotificacao, TipoNotificacao};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::{
    ComandoRegistrarBloqueio, MotivoBloqueio, ServicoMedicaoSerpro,
};
use crate::domain::serpro::ports::SerproAmbienteResolver;
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::domain::trabalhos::entidades::{ConsultaLote, ItemConsultaLote, TrabalhoAgendado};
use crate::domain::trabalhos::enums::{StatusExecucao, StatusLote};
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::{
    ExecutorTrabalho, RepositorioConsultasLote, RepositorioExecucoesTrabalho,
    RepositorioItensConsultaLote, RepositorioTrabalhosAgendados, SchedulerTrabalho,
};

/// Quantidade de itens reivindicados por lote em cada rodada do claim.
const ITENS_POR_CLAIM: usize = 50;

/// Configuração do runner no vocabulário da aplicação (o startup converte a
/// configuração de ambiente/infraestrutura para este struct).
#[derive(Debug, Clone)]
pub struct ConfiguracaoRunner {
    pub poll_interval: Duration,
    pub max_concorrencia: usize,
    pub lease: chrono::TimeDelta,
    pub backoff_base_secs: u64,
    pub backoff_teto_secs: u64,
    pub max_tentativas: i32,
}

impl Default for ConfiguracaoRunner {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(60),
            max_concorrencia: 4,
            lease: chrono::TimeDelta::seconds(120),
            backoff_base_secs: 30,
            backoff_teto_secs: 3_600,
            max_tentativas: 5,
        }
    }
}

/// Tipo de falha de um item de lote, que decide entre retry com backoff e
/// falha definitiva.
enum FalhaItem {
    Transitoria(String),
    Permanente(String),
}

fn falha_divida_ativa(erro: ErroDividaAtiva) -> FalhaItem {
    match erro {
        ErroDividaAtiva::LimiteLocalAtingido
        | ErroDividaAtiva::LimiteSerproAtingido
        | ErroDividaAtiva::Indeterminado
        | ErroDividaAtiva::ServicoIndisponivel => FalhaItem::Transitoria(erro.codigo().to_owned()),
        _ => FalhaItem::Permanente(erro.codigo().to_owned()),
    }
}

fn falha_integra_contador(erro: ErroIntegraContador) -> FalhaItem {
    match erro {
        ErroIntegraContador::Indisponivel
        | ErroIntegraContador::Indeterminado
        | ErroIntegraContador::LimiteExcedido => {
            FalhaItem::Transitoria("SERPRO_INDISPONIVEL".into())
        }
        ErroIntegraContador::ServicoNaoHabilitado => {
            FalhaItem::Permanente("SERVICO_NAO_HABILITADO".into())
        }
        ErroIntegraContador::ProducaoBloqueada => {
            FalhaItem::Permanente("SERPRO_PROD_BLOQUEADA".into())
        }
        ErroIntegraContador::ProcuracaoNegada => {
            FalhaItem::Permanente("PROCURACAO_NAO_AUTORIZADA".into())
        }
        _ => FalhaItem::Permanente("INTEGRA_CONTADOR_FALHOU".into()),
    }
}

/// Backoff exponencial com teto: `base * 2^(tentativas - 1)`, limitado a
/// `teto`.
fn atraso_backoff(tentativas: i32, base_secs: u64, teto_secs: u64) -> u64 {
    let expoente = tentativas.clamp(1, 21) as u32 - 1;
    base_secs
        .saturating_mul(1_u64 << expoente)
        .min(teto_secs.max(base_secs))
}

/// Usuário ativo do escritório que recebe as notificações do runner.
async fn usuario_notificavel(
    usuarios: &Arc<dyn UsuarioGestaoRepositorio>,
    escritorio_id: Uuid,
) -> Option<Uuid> {
    let usuarios = usuarios.clone();
    tokio::task::spawn_blocking(move || {
        let filtro = FiltroUsuarios {
            escritorio_id: Some(escritorio_id),
            ativo: Some(true),
            ..FiltroUsuarios::default()
        };
        usuarios.listar(&filtro).ok().and_then(|lista| {
            lista
                .iter()
                .find(|usuario| usuario.papel == Papel::Cliente)
                .or_else(|| lista.first())
                .map(|usuario| usuario.id)
        })
    })
    .await
    .ok()
    .flatten()
}

// ============================================================================
// Processador de consultas em lote
// ============================================================================

/// Processa os itens de um lote reivindicado, um a um, com claim por item,
/// backoff e finalização calculada por contagem.
pub struct ProcessadorLote {
    lotes: Arc<dyn RepositorioConsultasLote>,
    itens: Arc<dyn RepositorioItensConsultaLote>,
    consultar_devedor: Arc<ConsultarDevedorUseCase>,
    consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
    executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
    servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    resolver_ambiente: Arc<dyn SerproAmbienteResolver>,
    usuarios: Arc<dyn UsuarioGestaoRepositorio>,
    notificacoes: Arc<CriarNotificacaoUseCase>,
    config: ConfiguracaoRunner,
}

impl ProcessadorLote {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        lotes: Arc<dyn RepositorioConsultasLote>,
        itens: Arc<dyn RepositorioItensConsultaLote>,
        consultar_devedor: Arc<ConsultarDevedorUseCase>,
        consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
        executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
        servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
        resolver_ambiente: Arc<dyn SerproAmbienteResolver>,
        usuarios: Arc<dyn UsuarioGestaoRepositorio>,
        notificacoes: Arc<CriarNotificacaoUseCase>,
        config: ConfiguracaoRunner,
    ) -> Self {
        Self {
            lotes,
            itens,
            consultar_devedor,
            consultar_inscricao,
            executar_integra_contador,
            servico_medicao,
            resolver_ambiente,
            usuarios,
            notificacoes,
            config,
        }
    }

    /// Processa o lote reivindicado até finalizar ou esgotar os itens
    /// prontos (backoff em andamento devolve o lote à fila).
    pub async fn processar(&self, lote: ConsultaLote) -> Result<(), ErroTrabalho> {
        let ambiente = match self
            .resolver_ambiente
            .resolver_ambiente(lote.escritorio_id)
            .await
        {
            Ok(ambiente) => ambiente,
            Err(erro) => {
                tracing::warn!(erro = %erro, lote = %lote.id, "ambiente SERPRO do lote indisponível");
                self.lotes
                    .atualizar_status(
                        lote.id,
                        StatusLote::Falhou,
                        lote.iniciado_em,
                        Some(Utc::now()),
                        lote.total_itens,
                        lote.itens_concluidos,
                        lote.itens_com_erro,
                    )
                    .await?;
                self.notificar_falha_lote(&lote, "ambiente SERPRO do escritório indisponível")
                    .await;
                return Ok(());
            }
        };
        let usuario_id = match usuario_notificavel(&self.usuarios, lote.escritorio_id).await {
            Some(usuario_id) => usuario_id,
            None => {
                tracing::warn!(lote = %lote.id, "escritório sem usuário ativo para o lote");
                self.lotes
                    .atualizar_status(
                        lote.id,
                        StatusLote::Bloqueado,
                        lote.iniciado_em,
                        Some(Utc::now()),
                        lote.total_itens,
                        lote.itens_concluidos,
                        lote.itens_com_erro,
                    )
                    .await?;
                return Ok(());
            }
        };
        let produto = lote
            .produto
            .parse::<ProdutoSerpro>()
            .unwrap_or(ProdutoSerpro::DividaAtiva);

        // Itens presos por crash de um worker anterior voltam à fila.
        let _ = self.itens.recuperar_presos(Utc::now()).await;

        loop {
            let agora = Utc::now();
            let itens = self
                .itens
                .reivindicar_pendentes(lote.id, agora, agora + self.config.lease, ITENS_POR_CLAIM)
                .await?;
            if itens.is_empty() {
                break;
            }
            for item in itens {
                self.processar_item(&lote, &item, ambiente, usuario_id, produto)
                    .await?;
                // Heartbeat do lote durante o processamento.
                self.lotes
                    .renovar_lease(lote.id, Utc::now() + self.config.lease)
                    .await?;
            }
        }

        let contagem = self.itens.contar_por_status(lote.id, Utc::now()).await?;
        let total = contagem.pendente
            + contagem.executando
            + contagem.concluido
            + contagem.falhou
            + contagem.bloqueado;

        if contagem.pendente > 0 || contagem.executando > 0 {
            // Itens aguardando backoff: devolve o lote à fila e tenta no
            // próximo poll (sem prender o worker dormindo).
            self.lotes
                .atualizar_status(
                    lote.id,
                    StatusLote::Pendente,
                    lote.iniciado_em,
                    None,
                    i32::try_from(total).unwrap_or(i32::MAX),
                    i32::try_from(contagem.concluido).unwrap_or(i32::MAX),
                    i32::try_from(contagem.falhou).unwrap_or(i32::MAX),
                )
                .await?;
            return Ok(());
        }

        let status = if contagem.falhou == 0 && contagem.bloqueado == 0 {
            StatusLote::Concluido
        } else if contagem.concluido > 0 {
            StatusLote::Parcial
        } else if contagem.falhou == 0 {
            StatusLote::Bloqueado
        } else {
            StatusLote::Falhou
        };

        self.lotes
            .atualizar_status(
                lote.id,
                status,
                lote.iniciado_em,
                Some(Utc::now()),
                i32::try_from(total).unwrap_or(i32::MAX),
                i32::try_from(contagem.concluido).unwrap_or(i32::MAX),
                i32::try_from(contagem.falhou).unwrap_or(i32::MAX),
            )
            .await?;

        self.notificar_conclusao(&lote, status, &contagem).await;
        Ok(())
    }

    async fn processar_item(
        &self,
        lote: &ConsultaLote,
        item: &ItemConsultaLote,
        ambiente: AmbienteSerpro,
        usuario_id: Uuid,
        produto: ProdutoSerpro,
    ) -> Result<(), ErroTrabalho> {
        let pode_chamar = self
            .servico_medicao
            .pode_fazer_chamada(lote.escritorio_id)
            .await
            .map_err(|erro| {
                tracing::warn!(erro = %erro, "falha ao verificar o limite do plano");
                ErroTrabalho::Transitorio(
                    "falha ao verificar o limite do plano do escritório".into(),
                    0,
                )
            })?;

        if !pode_chamar {
            let comando = ComandoRegistrarBloqueio {
                escritorio_id: lote.escritorio_id,
                produto,
                motivo: MotivoBloqueio::LimitePlano,
                contexto: Some(format!("lote {} item {}", lote.id, item.id)),
            };
            if let Err(erro) = self.servico_medicao.registrar_bloqueio(&comando).await {
                tracing::warn!(erro = %erro, "não foi possível registrar o bloqueio do item");
            }
            return self
                .itens
                .atualizar_item(
                    item.id,
                    "bloqueado".to_owned(),
                    None,
                    Some("Limite do plano atingido".to_owned()),
                    Some(Utc::now()),
                )
                .await;
        }

        let request_id = format!("lote-{}-{}", lote.id, Uuid::now_v7());
        let resultado = match lote.servico.as_str() {
            "DEVEDOR" => match CpfCnpj::novo(&item.documento) {
                Ok(chave) => match self
                    .consultar_devedor
                    .executar(
                        ComandoConsultaDevedor {
                            escritorio_id: lote.escritorio_id,
                            usuario_id,
                            chave,
                        },
                        ambiente,
                        &request_id,
                    )
                    .await
                {
                    Ok(resultado) => Ok(serde_json::json!({
                        "quantidade": resultado.inscricoes.len()
                    })),
                    Err(erro) => Err(falha_divida_ativa(erro)),
                },
                Err(_) => Err(FalhaItem::Permanente("VALIDACAO".into())),
            },
            "INSCRICAO" => match self
                .consultar_inscricao
                .executar(
                    ComandoConsultaInscricao {
                        escritorio_id: lote.escritorio_id,
                        usuario_id,
                        numero_inscricao: item.documento.clone(),
                    },
                    ambiente,
                    &request_id,
                )
                .await
            {
                Ok(detalhe) => Ok(serde_json::json!({ "estado": detalhe.estado.as_str() })),
                Err(erro) => Err(falha_divida_ativa(erro)),
            },
            "OBTERPROCURACAO41" => match self
                .executar_integra_contador
                .consultar_procuracoes(
                    lote.escritorio_id,
                    usuario_id,
                    item.documento.clone(),
                    ambiente,
                    request_id,
                )
                .await
            {
                Ok(resultado) => Ok(serde_json::json!({ "estado": resultado.estado })),
                Err(erro) => Err(falha_integra_contador(erro)),
            },
            _ => Err(FalhaItem::Permanente("SERVICO_NAO_HABILITADO".into())),
        };

        let agora = Utc::now();
        match resultado {
            Ok(resumo) => {
                self.itens
                    .atualizar_item(
                        item.id,
                        "concluido".to_owned(),
                        Some(resumo),
                        None,
                        Some(agora),
                    )
                    .await
            }
            Err(FalhaItem::Permanente(codigo)) => {
                self.itens
                    .atualizar_item(
                        item.id,
                        "falhou".to_owned(),
                        None,
                        Some(codigo),
                        Some(agora),
                    )
                    .await
            }
            Err(FalhaItem::Transitoria(codigo)) => {
                if item.tentativas >= self.config.max_tentativas {
                    return self
                        .itens
                        .atualizar_item(
                            item.id,
                            "falhou".to_owned(),
                            None,
                            Some(codigo),
                            Some(agora),
                        )
                        .await;
                }
                let atraso = atraso_backoff(
                    item.tentativas,
                    self.config.backoff_base_secs,
                    self.config.backoff_teto_secs,
                );
                self.itens
                    .agendar_reexecucao(
                        item.id,
                        codigo,
                        Some(
                            agora
                                + chrono::TimeDelta::try_seconds(atraso as i64).unwrap_or_default(),
                        ),
                    )
                    .await
            }
        }
    }

    async fn notificar_conclusao(
        &self,
        lote: &ConsultaLote,
        status: StatusLote,
        contagem: &crate::domain::trabalhos::ports::ContagemItensLote,
    ) {
        let contexto = format!(
            "{} concluídos, {} falhas, {} bloqueados",
            contagem.concluido, contagem.falhou, contagem.bloqueado
        );
        let corpo = format!("O lote '{}' foi finalizado ({}).", lote.nome, contexto);
        self.notificar(
            lote.escritorio_id,
            TipoNotificacao::LoteConcluido,
            format!("Consulta em lote finalizada: {}", status.as_str()),
            corpo,
            "consulta_lote",
            lote.id,
        )
        .await;
    }

    async fn notificar_falha_lote(&self, lote: &ConsultaLote, motivo: &str) {
        self.notificar(
            lote.escritorio_id,
            TipoNotificacao::FalhaRotina,
            "Falha na consulta em lote".to_owned(),
            format!("O lote '{}' falhou: {motivo}.", lote.nome),
            "consulta_lote",
            lote.id,
        )
        .await;
    }

    async fn notificar(
        &self,
        escritorio_id: Uuid,
        tipo: TipoNotificacao,
        titulo: String,
        corpo: String,
        referencia_tipo: &str,
        referencia_id: Uuid,
    ) {
        let Some(usuario_id) = usuario_notificavel(&self.usuarios, escritorio_id).await else {
            return;
        };
        let comando = NovoNotificacao {
            usuario_id,
            escritorio_id,
            canal: CanalNotificacao::InApp,
            tipo,
            titulo,
            corpo,
            referencia_tipo: Some(referencia_tipo.to_owned()),
            referencia_id: Some(referencia_id),
        };
        if let Err(erro) = self.notificacoes.executar(comando).await {
            tracing::warn!(erro = %erro, "não foi possível criar a notificação do job runner");
        }
    }
}

// ============================================================================
// Job runner
// ============================================================================

/// Dependências do worker. Montadas no startup (e pelos testes de integração)
/// com os mesmos casos de uso usados pelas rotas HTTP.
pub struct DependenciasRunner {
    pub trabalhos: Arc<dyn RepositorioTrabalhosAgendados>,
    pub execucoes: Arc<dyn RepositorioExecucoesTrabalho>,
    pub lotes: Arc<dyn RepositorioConsultasLote>,
    pub itens: Arc<dyn RepositorioItensConsultaLote>,
    pub executor: Arc<dyn ExecutorTrabalho>,
    pub scheduler: Arc<dyn SchedulerTrabalho>,
    pub processador_lote: Arc<ProcessadorLote>,
    pub notificacoes: Arc<CriarNotificacaoUseCase>,
    pub usuarios: Arc<dyn UsuarioGestaoRepositorio>,
}

#[derive(Clone)]
pub struct JobRunner {
    deps: Arc<DependenciasRunner>,
    config: ConfiguracaoRunner,
    shutdown: watch::Receiver<bool>,
}

impl JobRunner {
    pub fn novo(
        deps: DependenciasRunner,
        config: ConfiguracaoRunner,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        Self {
            deps: Arc::new(deps),
            config,
            shutdown,
        }
    }

    /// Executa uma rodada completa (recuperação + claims + processamento dos
    /// itens reivindicados) e aguarda as tarefas terminarem. Exposta para os
    /// testes de integração e chamada pelo loop principal.
    pub async fn rodada(&self) {
        let agora = Utc::now();

        if let Err(erro) = self.deps.execucoes.recuperar_presos(agora).await {
            tracing::warn!(erro = %erro, "falha ao recuperar execuções presas");
        }
        if let Err(erro) = self.deps.lotes.recuperar_presos(agora).await {
            tracing::warn!(erro = %erro, "falha ao recuperar lotes presos");
        }
        if let Err(erro) = self.deps.itens.recuperar_presos(agora).await {
            tracing::warn!(erro = %erro, "falha ao recuperar itens presos");
        }

        let lease = agora + self.config.lease;
        let max = self.config.max_concorrencia.max(1);

        let trabalhos = match self
            .deps
            .trabalhos
            .reivindicar_devidos(agora, lease, max)
            .await
        {
            Ok(trabalhos) => trabalhos,
            Err(erro) => {
                tracing::warn!(erro = %erro, "falha ao reivindicar trabalhos");
                Vec::new()
            }
        };
        let lotes = match self
            .deps
            .lotes
            .reivindicar_pendentes(agora, lease, max)
            .await
        {
            Ok(lotes) => lotes,
            Err(erro) => {
                tracing::warn!(erro = %erro, "falha ao reivindicar lotes");
                Vec::new()
            }
        };
        let execucoes_retry = match self
            .deps
            .execucoes
            .reivindicar_pendentes(agora, lease, max)
            .await
        {
            Ok(execucoes) => execucoes,
            Err(erro) => {
                tracing::warn!(erro = %erro, "falha ao reivindicar execuções em retry");
                Vec::new()
            }
        };

        if trabalhos.is_empty() && lotes.is_empty() && execucoes_retry.is_empty() {
            return;
        }

        let semaforo = Arc::new(Semaphore::new(max));
        let mut tarefas = tokio::task::JoinSet::new();

        for (trabalho, execucao_id) in trabalhos {
            let runner = self.clone();
            let semaforo = semaforo.clone();
            tarefas.spawn(async move {
                let _permita = semaforo.acquire_owned().await;
                runner.processar_trabalho(trabalho, execucao_id).await;
            });
        }
        for executacao in execucoes_retry {
            let runner = self.clone();
            let semaforo = semaforo.clone();
            tarefas.spawn(async move {
                let _permita = semaforo.acquire_owned().await;
                match runner
                    .deps
                    .trabalhos
                    .buscar_por_id(executacao.trabalho_agendado_id)
                    .await
                {
                    Ok(Some(trabalho)) => {
                        runner.processar_trabalho(trabalho, executacao.id).await;
                    }
                    Ok(None) => {
                        tracing::warn!(
                            execucao = %executacao.id,
                            "execução em retry sem trabalho correspondente"
                        );
                    }
                    Err(erro) => {
                        tracing::warn!(erro = %erro, "falha ao carregar trabalho da execução em retry");
                    }
                }
            });
        }
        for lote in lotes {
            let runner = self.clone();
            let semaforo = semaforo.clone();
            tarefas.spawn(async move {
                let _permita = semaforo.acquire_owned().await;
                if let Err(erro) = runner.deps.processador_lote.processar(lote.clone()).await {
                    tracing::warn!(erro = %erro, lote = %lote.id, "processamento do lote falhou");
                }
            });
        }

        while tarefas.join_next().await.is_some() {}
    }

    async fn processar_trabalho(&self, trabalho: TrabalhoAgendado, execucao_id: Uuid) {
        // Avança a próxima execução imediatamente: uma queda após o claim não
        // pode deixar a rotina "vencida" e reexecutando em loop.
        let agora = Utc::now();
        let proxima = self
            .deps
            .scheduler
            .proxima_execucao(&trabalho.cron_expression, agora);
        if let Err(erro) = self
            .deps
            .trabalhos
            .atualizar_proxima_execucao(trabalho.id, proxima, Some(agora))
            .await
        {
            tracing::warn!(erro = %erro, trabalho = %trabalho.id, "falha ao agendar próxima execução");
        }

        let resultado = self.executar_com_heartbeat(&trabalho, execucao_id).await;
        let finalizado = Utc::now();

        match resultado {
            Ok((processados, com_erro)) => {
                let status = if processados == 0 && com_erro > 0 {
                    StatusExecucao::Falhou
                } else {
                    StatusExecucao::Concluido
                };
                if let Err(erro) = self
                    .deps
                    .execucoes
                    .atualizar_status(
                        execucao_id,
                        status,
                        Some(finalizado),
                        None,
                        processados,
                        com_erro,
                    )
                    .await
                {
                    tracing::warn!(erro = %erro, "falha ao finalizar execução do trabalho");
                }
                if status == StatusExecucao::Falhou {
                    self.notificar_falha_rotina(&trabalho, &format!("{com_erro} falhas"))
                        .await;
                }
            }
            Err(ErroTrabalho::Bloqueado(motivo)) => {
                if let Err(erro) = self
                    .deps
                    .execucoes
                    .atualizar_status(
                        execucao_id,
                        StatusExecucao::Bloqueado,
                        Some(finalizado),
                        Some(motivo),
                        0,
                        0,
                    )
                    .await
                {
                    tracing::warn!(erro = %erro, "falha ao marcar execução como bloqueada");
                }
            }
            Err(ErroTrabalho::Transitorio(motivo, atraso_sugerido)) => {
                let tentativas = self
                    .deps
                    .execucoes
                    .buscar_por_id(execucao_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|execucao| execucao.tentativas)
                    .unwrap_or(self.config.max_tentativas);

                if tentativas >= self.config.max_tentativas {
                    if let Err(erro) = self
                        .deps
                        .execucoes
                        .atualizar_status(
                            execucao_id,
                            StatusExecucao::Falhou,
                            Some(finalizado),
                            Some(motivo.clone()),
                            0,
                            0,
                        )
                        .await
                    {
                        tracing::warn!(erro = %erro, "falha ao marcar execução como falha");
                    }
                    self.notificar_falha_rotina(&trabalho, &motivo).await;
                } else {
                    let base = if atraso_sugerido > 0 {
                        atraso_sugerido
                    } else {
                        atraso_backoff(
                            tentativas,
                            self.config.backoff_base_secs,
                            self.config.backoff_teto_secs,
                        )
                    };
                    let proxima_tentativa = finalizado
                        + chrono::TimeDelta::try_seconds(base as i64).unwrap_or_default();
                    if let Err(erro) = self
                        .deps
                        .execucoes
                        .agendar_reexecucao(execucao_id, motivo, Some(proxima_tentativa), 0, 0)
                        .await
                    {
                        tracing::warn!(erro = %erro, "falha ao agendar reexecução do trabalho");
                    }
                }
            }
            Err(erro) => {
                let motivo = erro.to_string();
                if let Err(falha) = self
                    .deps
                    .execucoes
                    .atualizar_status(
                        execucao_id,
                        StatusExecucao::Falhou,
                        Some(finalizado),
                        Some(motivo.clone()),
                        0,
                        0,
                    )
                    .await
                {
                    tracing::warn!(erro = %falha, "falha ao marcar execução como falha");
                }
                self.notificar_falha_rotina(&trabalho, &motivo).await;
            }
        }
    }

    /// Executa o trabalho renovando o lease enquanto ele está em andamento.
    async fn executar_com_heartbeat(
        &self,
        trabalho: &TrabalhoAgendado,
        execucao_id: Uuid,
    ) -> Result<(i32, i32), ErroTrabalho> {
        let lease_secs = self.config.lease.num_seconds().max(1) as u64;
        let mut heartbeat = tokio::time::interval(Duration::from_secs((lease_secs / 2).max(1)));
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut futuro = Box::pin(self.deps.executor.executar(trabalho));
        loop {
            tokio::select! {
                resultado = &mut futuro => return resultado,
                _ = heartbeat.tick() => {
                    if let Err(erro) = self
                        .deps
                        .execucoes
                        .renovar_lease(execucao_id, Utc::now() + self.config.lease)
                        .await
                    {
                        tracing::warn!(execucao = %execucao_id, erro = %erro, "lease da execução não renovado");
                    }
                }
            }
        }
    }

    async fn notificar_falha_rotina(&self, trabalho: &TrabalhoAgendado, motivo: &str) {
        let Some(usuario_id) =
            usuario_notificavel(&self.deps.usuarios, trabalho.escritorio_id).await
        else {
            return;
        };
        let comando = NovoNotificacao {
            usuario_id,
            escritorio_id: trabalho.escritorio_id,
            canal: CanalNotificacao::InApp,
            tipo: TipoNotificacao::FalhaRotina,
            titulo: "Rotina fiscal falhou".to_owned(),
            corpo: format!("A rotina '{}' falhou: {motivo}.", trabalho.nome),
            referencia_tipo: Some("trabalho_agendado".to_owned()),
            referencia_id: Some(trabalho.id),
        };
        if let Err(erro) = self.deps.notificacoes.executar(comando).await {
            tracing::warn!(erro = %erro, "não foi possível notificar a falha da rotina");
        }
    }

    /// Loop principal do worker: poll da fila até o sinal de desligamento.
    pub async fn executar(mut self) {
        let mut ticker = tokio::time::interval(self.config.poll_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        tracing::info!(
            intervalo_segundos = self.config.poll_interval.as_secs(),
            concorrencia = self.config.max_concorrencia,
            "job runner iniciado"
        );

        loop {
            tokio::select! {
                _ = self.shutdown.changed() => break,
                _ = ticker.tick() => {
                    if *self.shutdown.borrow() {
                        break;
                    }
                    self.rodada().await;
                }
            }
        }

        tracing::info!("job runner encerrado");
    }
}
