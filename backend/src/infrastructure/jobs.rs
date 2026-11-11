//! Composição do job runner (Fase B.1): monta os repositórios, o executor
//! de rotinas e o processador de lotes a partir do pool e da configuração.
//!
//! Compartilha o mesmo pool e o mesmo `LimitadorPorEscritorio` do processo —
//! a fila e a API disputam o rate limit local do escritório de propósito.

use std::sync::Arc;
use std::time::Duration;

use crate::application::divida_ativa::ports::RepositorioConsultasFiscais;
use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::application::integra_contador::ports::ProvedorCatalogoIntegra;
use crate::application::integra_contador::{
    ExecutarIntegraContadorUseCase, ValidarProcuracoesUseCase,
};
use crate::application::notificacoes::CriarNotificacaoUseCase;
use crate::application::serpro::SerproAmbienteResolverImpl;
use crate::application::serpro::medicao::ServicoMedicaoSerproImpl;
use crate::application::trabalhos::{
    ConfiguracaoRunner, DependenciasRunner, ExecutorTrabalhoImpl, JobRunner, ProcessadorLote,
    SchedulerTrabalhoCron,
};
use crate::domain::serpro::medicao::{OrigemChamada, ServicoMedicaoSerpro};
use crate::domain::serpro::ports::SerproAmbienteResolver;
use crate::domain::trabalhos::ports::{
    ExecutorTrabalho, RepositorioConsultasLote, RepositorioExecucoesTrabalho,
    RepositorioItensConsultaLote, RepositorioTrabalhosAgendados, SchedulerTrabalho,
};
use crate::infrastructure::config::Config;
use crate::infrastructure::divida_ativa::cliente_http::ClienteDividaAtivaHttp;
use crate::infrastructure::divida_ativa::hash::HashChaveSha256;
use crate::infrastructure::divida_ativa::rate::LimitadorPorEscritorio;
use crate::infrastructure::divida_ativa::repositorio::{
    CatalogoClientesDiesel, CatalogoOperadoresDiesel, ConsultasFiscaisRepositorioDiesel,
};
use crate::infrastructure::integra_contador::ClienteIntegraContadorImpl;
use crate::infrastructure::persistence::clientes::ClienteRepositorioDiesel;
use crate::infrastructure::persistence::integra_contador::{
    CatalogoServicosIntegraDiesel, IntegraContadorRepositorioDiesel, ProcuracoesRepositorioDiesel,
    ValidadorProcuracaoDiesel,
};
use crate::infrastructure::persistence::notificacoes::NotificacoesDiesel;
use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::serpro::medicao::RepositorioMedicaoDiesel;
use crate::infrastructure::persistence::trabalhos::{
    ConsultasLoteDiesel, ExecucoesTrabalhoDiesel, ItensConsultaLoteDiesel, TrabalhosAgendadosDiesel,
};
use crate::infrastructure::persistence::usuarios::UsuarioGestaoRepositorioDiesel;
use crate::infrastructure::serpro::mock::{ClienteDividaAtivaMock, ClienteIntegraContadorMock};
use crate::infrastructure::serpro::{SerproEndpointResolverImpl, SerproRepositorioDiesel};

/// Monta as dependências do worker e a configuração efetiva do runner.
pub fn montar_dependencias(
    pool: &DbPool,
    config: &Config,
) -> (DependenciasRunner, ConfiguracaoRunner) {
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(config.serpro.http_connect_timeout_secs))
        .timeout(Duration::from_secs(config.serpro.http_request_timeout_secs))
        .build()
        .expect("cliente HTTP do job runner deve inicializar");

    let repositorio_medicao = Arc::new(RepositorioMedicaoDiesel::novo(pool.clone()));
    let servico_medicao: Arc<dyn ServicoMedicaoSerpro> = Arc::new(
        ServicoMedicaoSerproImpl::novo(
            repositorio_medicao.clone(),
            repositorio_medicao.clone(),
            repositorio_medicao.clone(),
            repositorio_medicao.clone(),
        )
        .com_notificador(repositorio_medicao.clone()),
    );

    // ===== Integra Contador =====
    let repositorio_integra = IntegraContadorRepositorioDiesel::novo(pool.clone());
    let procuracoes_integra = Arc::new(ProcuracoesRepositorioDiesel::novo(pool.clone()));
    let catalogo_integra: Arc<dyn ProvedorCatalogoIntegra> =
        Arc::new(CatalogoServicosIntegraDiesel::novo(pool.clone()));
    let endpoints_integra = Arc::new(SerproEndpointResolverImpl::novo(config.serpro.clone()));
    let cliente_integra_http = Arc::new(ClienteIntegraContadorImpl::novo(
        http.clone(),
        endpoints_integra,
        config.serpro.operator_header_enabled,
    ));
    let cliente_integra = Arc::new(ClienteIntegraContadorMock::novo(
        cliente_integra_http,
        config.serpro.trial_mock,
    ));
    let executar_integra_manual = ExecutarIntegraContadorUseCase::novo(
        Arc::new(repositorio_integra.clone()),
        cliente_integra,
        Arc::new(repositorio_integra.clone()),
        Arc::new(repositorio_integra.clone()),
        Arc::new(ValidadorProcuracaoDiesel::novo(
            procuracoes_integra.clone(),
            Some(config.serpro.procuracao_cache_ttl_secs),
        )),
        config.serpro.operator_header_enabled,
        servico_medicao.clone(),
    )
    .com_catalogo(Some(catalogo_integra));
    let executar_integra_rotina = Arc::new(
        executar_integra_manual
            .clone()
            .com_origem(OrigemChamada::Rotina),
    );
    let executar_integra_lote = Arc::new(
        executar_integra_manual
            .clone()
            .com_origem(OrigemChamada::Lote),
    );

    let cliente_repo = Arc::new(ClienteRepositorioDiesel::novo(pool.clone()));
    let cliente_repo_trait: Arc<dyn crate::application::clientes::ClienteRepositorio> =
        cliente_repo.clone();
    let validar_procuracao = Arc::new(ValidarProcuracoesUseCase::novo(
        cliente_repo_trait.clone(),
        Arc::new(repositorio_integra),
        executar_integra_rotina.clone(),
        procuracoes_integra.clone(),
        Some(config.serpro.procuracao_cache_ttl_secs),
    ));

    // ===== Dívida Ativa =====
    let endpoints_divida = Arc::new(SerproEndpointResolverImpl::novo(config.serpro.clone()));
    let consultas_divida: Arc<dyn RepositorioConsultasFiscais> =
        Arc::new(ConsultasFiscaisRepositorioDiesel::novo(pool.clone()));
    let limitador = Arc::new(LimitadorPorEscritorio::novo(
        config.serpro.rate_limit_consultas_por_segundo,
    ));
    let hash = Arc::new(HashChaveSha256::novo(
        config.serpro.consultas_chave_pepper.clone(),
    ));
    let cliente_divida = Arc::new(ClienteDividaAtivaMock::novo(
        Arc::new(ClienteDividaAtivaHttp::novo(http)),
        config.serpro.trial_mock,
    ));
    let catalogo_clientes = Arc::new(CatalogoClientesDiesel::novo(pool.clone()));
    let catalogo_operadores = Arc::new(CatalogoOperadoresDiesel::novo(pool.clone()));

    let consultar_devedor_manual = ConsultarDevedorUseCase::novo(
        endpoints_divida.clone(),
        limitador.clone(),
        consultas_divida.clone(),
        hash.clone(),
        cliente_divida.clone(),
        servico_medicao.clone(),
    )
    .com_catalogo_clientes(catalogo_clientes)
    .com_operadores(
        catalogo_operadores.clone(),
        config.serpro.operator_header_enabled,
    );
    let consultar_devedor_rotina = Arc::new(
        consultar_devedor_manual
            .clone()
            .com_origem(OrigemChamada::Rotina),
    );
    let consultar_devedor_lote = Arc::new(
        consultar_devedor_manual
            .clone()
            .com_origem(OrigemChamada::Lote),
    );

    let consultar_inscricao_manual = ConsultarInscricaoUseCase::novo(
        endpoints_divida,
        limitador,
        consultas_divida,
        hash,
        cliente_divida,
        servico_medicao.clone(),
    )
    .com_operadores(catalogo_operadores, config.serpro.operator_header_enabled);
    let consultar_inscricao_lote = Arc::new(
        consultar_inscricao_manual
            .clone()
            .com_origem(OrigemChamada::Lote),
    );

    let usuarios: Arc<dyn crate::application::usuarios::UsuarioGestaoRepositorio> =
        Arc::new(UsuarioGestaoRepositorioDiesel::novo(pool.clone()));
    let notificacoes = Arc::new(CriarNotificacaoUseCase::novo(Arc::new(
        NotificacoesDiesel::novo(pool.clone()),
    )));

    let configuracao = ConfiguracaoRunner {
        poll_interval: Duration::from_secs(config.jobs.poll_interval_secs),
        max_concorrencia: usize::try_from(config.jobs.max_concorrencia).unwrap_or(4),
        lease: chrono::TimeDelta::try_seconds(i64::try_from(config.jobs.lease_secs).unwrap_or(120))
            .unwrap_or_else(|| chrono::TimeDelta::seconds(120)),
        backoff_base_secs: config.jobs.backoff_base_secs,
        backoff_teto_secs: config.jobs.backoff_teto_secs,
        max_tentativas: i32::try_from(config.jobs.max_tentativas).unwrap_or(5),
    };

    let executor: Arc<dyn ExecutorTrabalho> = Arc::new(ExecutorTrabalhoImpl::novo(
        consultar_devedor_rotina,
        executar_integra_rotina,
        validar_procuracao,
        cliente_repo_trait,
        usuarios.clone(),
        servico_medicao.clone(),
    ));

    let resolver_ambiente: Arc<dyn SerproAmbienteResolver> = Arc::new(
        SerproAmbienteResolverImpl::novo(Arc::new(SerproRepositorioDiesel::novo(pool.clone()))),
    );

    let processador_lote = Arc::new(ProcessadorLote::novo(
        Arc::new(ConsultasLoteDiesel::novo(pool.clone())),
        Arc::new(ItensConsultaLoteDiesel::novo(pool.clone())),
        consultar_devedor_lote,
        consultar_inscricao_lote,
        executar_integra_lote,
        servico_medicao,
        resolver_ambiente,
        usuarios.clone(),
        notificacoes.clone(),
        configuracao.clone(),
    ));

    let trabalhos: Arc<dyn RepositorioTrabalhosAgendados> =
        Arc::new(TrabalhosAgendadosDiesel::novo(pool.clone()));
    let execucoes: Arc<dyn RepositorioExecucoesTrabalho> =
        Arc::new(ExecucoesTrabalhoDiesel::novo(pool.clone()));
    let lotes: Arc<dyn RepositorioConsultasLote> =
        Arc::new(ConsultasLoteDiesel::novo(pool.clone()));
    let itens: Arc<dyn RepositorioItensConsultaLote> =
        Arc::new(ItensConsultaLoteDiesel::novo(pool.clone()));
    let scheduler: Arc<dyn SchedulerTrabalho> = Arc::new(SchedulerTrabalhoCron::novo());

    let dependencias = DependenciasRunner {
        trabalhos,
        execucoes,
        lotes,
        itens,
        executor,
        scheduler,
        processador_lote,
        notificacoes,
        usuarios,
    };

    (dependencias, configuracao)
}

/// Conveniência para o startup: monta e devolve o runner pronto.
pub fn montar_runner(
    pool: &DbPool,
    config: &Config,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> JobRunner {
    let (dependencias, configuracao) = montar_dependencias(pool, config);
    JobRunner::novo(dependencias, configuracao, shutdown)
}
