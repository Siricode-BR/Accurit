use axum::Router;
use axum::http::Request;
use axum::middleware as axum_middleware;
use axum::routing::get;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::application::admin::{ListarEscritoriosUseCase, ProvisionarTenantUseCase};
use crate::application::certificados::{
    CriarCertificadoUseCase, ListarCertificadosUseCase, ObterCertificadoUseCase,
    PoliticaCertificados, RevogarCertificadoUseCase,
};
use crate::application::clientes::{
    AtualizarClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase, ListarClientesUseCase,
    ListarConsultasDoClienteUseCase, ObterClienteUseCase, PoliticaClientes,
    SituacaoFiscalDoClienteUseCase,
};
use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::application::ports::SaudeBanco;
use crate::application::serpro::medicao::ServicoMedicaoSerproImpl;
use crate::application::serpro::{
    AlterarModoTrialSerproUseCase, CredencialFallbackGlobal, ObterConfiguracaoSerproUseCase,
};
use crate::infrastructure::auth::HashArgon2;
use crate::infrastructure::config::{BURST_RATE_LIMIT, Config, REQUISICOES_POR_SEGUNDO};
use crate::infrastructure::crypto::criar_criptografia;
use crate::infrastructure::persistence::admin::AdminRepositorioDiesel;
use crate::infrastructure::persistence::certificados::CertificadoRepositorioDiesel;
use crate::infrastructure::persistence::clientes::ClienteRepositorioDiesel;
use crate::infrastructure::persistence::serpro::medicao::RepositorioMedicaoDiesel;
use crate::infrastructure::serpro::{SerproCriptografia, SerproRepositorioDiesel};

use crate::application::relatorios::use_cases::{
    AcessarRelatorioPorLinkUseCase, BuscarRelatorioUseCase, CriarLinkCompartilhamentoUseCase,
    GerarRelatorioUseCase, ListarRelatoriosUseCase, ValidarLinkCompartilhamentoUseCase,
};
use crate::domain::relatorios::ports::{RepositorioLinksCompartilhamento, RepositorioRelatorios};
use crate::infrastructure::persistence::relatorios::RelatoriosDiesel;
use crate::presentation::admin::state::AdminState;
use crate::presentation::auth::{AuthState, ExigirAudiencia, construir as construir_auth, exigir};
use crate::presentation::certificados::construir_certificados;
use crate::presentation::clientes::router::construir_clientes;
use crate::presentation::handlers;
use crate::presentation::middleware::{cors, rate_limit, request_id, security_headers};
use crate::presentation::relatorios::router::{
    construir as construir_relatorios, construir_publico,
};
use crate::presentation::relatorios::state::EstadoRelatorios;
use crate::presentation::serpro::ambiente::{EstadoAmbiente, resolver_ambiente};
use crate::presentation::serpro::handlers::obter_ambiente;
use crate::presentation::serpro::state::SerproState;
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
    let argon2 =
        std::sync::Arc::new(HashArgon2::novo(&config.auth).expect("argon2 deve inicializar"));

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

    // Histórico e situação fiscal por cliente (leitura, SPEC §9.3).
    let consultas_cliente_repo = std::sync::Arc::new(
        crate::infrastructure::persistence::clientes::ConsultasDoClienteRepositorioDiesel::novo(
            pool.clone(),
        ),
    );
    let listar_consultas_cliente = ListarConsultasDoClienteUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        consultas_cliente_repo.clone(),
    );
    let situacao_fiscal_cliente = SituacaoFiscalDoClienteUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        consultas_cliente_repo,
    );

    // ===== Fase E — gestão de carteira =====
    let carteira_repo = std::sync::Arc::new(
        crate::infrastructure::persistence::clientes::CarteiraClientesRepositorioDiesel::novo(
            pool.clone(),
        ),
    );
    let cliente_repo_trait: std::sync::Arc<dyn crate::application::clientes::ClienteRepositorio> =
        std::sync::Arc::new(cliente_repo.clone());
    let auditoria_clientes: std::sync::Arc<
        dyn crate::application::clientes::AuditoriaClienteRepositorio,
    > = std::sync::Arc::new(cliente_repo.clone());
    let verificar_acesso =
        crate::application::clientes::VerificarAcessoUseCase::novo(carteira_repo.clone()
            as std::sync::Arc<dyn crate::application::clientes::RepositorioAcessoUsuarios>);
    let gestao_socios = crate::application::clientes::GestaoSociosContatosUseCase::novo(
        cliente_repo_trait.clone(),
        carteira_repo.clone(),
        auditoria_clientes.clone(),
    );
    let gestao_filiais = crate::application::clientes::GestaoFiliaisUseCase::novo(
        cliente_repo_trait.clone(),
        carteira_repo.clone(),
        auditoria_clientes.clone(),
    );
    let gestao_tags = crate::application::clientes::GestaoClienteTagsUseCase::novo(
        carteira_repo.clone(),
        auditoria_clientes.clone(),
    );
    let gestao_carteira_cliente = crate::application::clientes::CarteiraClienteUseCase::novo(
        cliente_repo_trait.clone(),
        carteira_repo.clone(),
        auditoria_clientes,
    );
    let importar_clientes = crate::application::clientes::ImportarClientesUseCase::novo(
        std::sync::Arc::new(cliente_repo.clone()),
        std::sync::Arc::new(cliente_repo.clone()),
    );
    let _gestao_acesso_usuarios =
        crate::application::clientes::GestaoAcessoUsuariosUseCase::novo(carteira_repo.clone()
            as std::sync::Arc<dyn crate::application::clientes::RepositorioAcessoUsuarios>);

    let provisionar_tenant_uc = ProvisionarTenantUseCase::novo(
        std::sync::Arc::new(admin_repo.clone()),
        std::sync::Arc::new(admin_repo.clone()),
        std::sync::Arc::new(admin_repo.clone()),
        argon2.clone(),
    );
    let listar_tenant_uc = ListarEscritoriosUseCase::novo(std::sync::Arc::new(admin_repo.clone()));

    let certificado_repo = CertificadoRepositorioDiesel::novo(std::sync::Arc::new(pool.clone()));
    let criptografia_certificados =
        criar_criptografia(config.certificate_master_key.expose_secret())
            .expect("criptografia deve inicializar");
    let auditoria_certificado = certificado_repo.clone();

    let criar_certificados = CriarCertificadoUseCase::novo(
        std::sync::Arc::new(certificado_repo.clone()),
        std::sync::Arc::new(auditoria_certificado.clone()),
        criptografia_certificados,
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

    // ===== SERPRO (Fase 3) =====
    let serpro_repo = SerproRepositorioDiesel::novo(pool.clone());
    // Medição de uso SERPRO (Fase G/H): repositorio único implementa todas
    // as portas de medição (chamadas, agregados, planos, assinaturas,
    // bloqueios).
    let repositorio_medicao = std::sync::Arc::new(RepositorioMedicaoDiesel::novo(pool.clone()));
    let servico_medicao = std::sync::Arc::new(ServicoMedicaoSerproImpl::novo(
        repositorio_medicao.clone(),
        repositorio_medicao.clone(),
        repositorio_medicao.clone(),
        repositorio_medicao.clone(),
    ));
    let obter_config_serpro =
        ObterConfiguracaoSerproUseCase::novo(std::sync::Arc::new(serpro_repo.clone()));
    // O mesmo use case alimenta o middleware do snapshot de ambiente (SPEC
    // §8: o ambiente é lido uma única vez no início da requisição).
    let estado_ambiente = EstadoAmbiente {
        configuracao: std::sync::Arc::new(obter_config_serpro.clone()),
    };
    let alterar_modo_serpro = AlterarModoTrialSerproUseCase::novo(
        std::sync::Arc::new(serpro_repo.clone()),
        std::sync::Arc::new(serpro_repo.clone()),
        std::sync::Arc::new(serpro_repo.clone()),
        std::sync::Arc::new(serpro_repo.clone()),
    );
    let serpro_state = SerproState::novo(
        std::sync::Arc::new(obter_config_serpro),
        std::sync::Arc::new(alterar_modo_serpro),
    );

    // Provider de credenciais com fallback DA decisão D6: globais de
    // configuração só desmobilizam dados de TRIAL quando APP_ENV=development.
    let fallback_trial_dev = if config.app_env != crate::infrastructure::config::AppEnv::Development
    {
        None
    } else {
        match (
            config.serpro.consumer_key.as_ref(),
            config.serpro.consumer_secret.as_ref(),
        ) {
            (Some(chave), Some(secreto)) => Some(CredencialFallbackGlobal {
                consumer_key: chave.clone(),
                consumer_secret: secreto.clone(),
            }),
            _ => None,
        }
    };
    let criptografia_serpro =
        SerproCriptografia::novo(config.certificate_master_key.expose_secret())
            .expect("criptografia SERPRO deve inicializar");
    let _provider_credenciais_serpro =
        crate::application::serpro::SerproCredentialProviderImpl::novo(
            std::sync::Arc::new(serpro_repo.clone()),
            std::sync::Arc::new(criptografia_serpro),
            fallback_trial_dev,
        );

    let admin_state = AdminState::novo(
        std::sync::Arc::new(provisionar_tenant_uc),
        std::sync::Arc::new(listar_tenant_uc),
        serpro_state.clone(),
    );

    let usuarios_gestao_repo = std::sync::Arc::new(
        crate::infrastructure::persistence::usuarios::UsuarioGestaoRepositorioDiesel::novo(
            pool.clone(),
        ),
    );
    let _gestao_usuarios = crate::application::usuarios::GestaoUsuariosUseCase::novo(
        usuarios_gestao_repo,
        argon2.clone(),
    );

    let gestao_administradores = crate::application::admin::GestaoAdministradoresUseCase::novo(
        std::sync::Arc::new(admin_repo.clone()),
        argon2.clone(),
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
                listar_consultas_cliente,
                situacao_fiscal_cliente,
                verificar_acesso.clone(),
                gestao_socios,
                gestao_filiais,
                gestao_tags,
                gestao_carteira_cliente,
                importar_clientes,
                auth_state.clone(),
            ),
        );

        let administradores_router =
            crate::presentation::administradores::construir_administradores(
                gestao_administradores.clone(),
                auth_state.clone(),
            );
        api = api.nest("/administradores", administradores_router.clone());
        api = api.nest("/admin/administradores", administradores_router);

        // Etapa 0.2/0.4 — rotas de gestão de escritórios do backoffice SaaS.
        // (provisionamento de tenant + alternância do modo Trial SERPRO)
        api = api.nest(
            "/admin/escritorios",
            axum::Router::new()
                .route(
                    "/",
                    axum::routing::get(crate::presentation::admin::handlers::listar_escritorios)
                        .post(crate::presentation::admin::handlers::provisionar_tenant),
                )
                .route(
                    "/{id}/serpro/trial",
                    axum::routing::patch(
                        crate::presentation::admin::serpro::handlers::alterar_modo_trial_serpro,
                    ),
                )
                .with_state(admin_state.clone()),
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

        // Portal (client): leitura enxuta do ambiente ativo para o selo
        // (SPEC §8, D8). O middleware `resolver_ambiente` roda **após** a
        // exigência de audiência e insere o snapshot `AmbienteResolvido`
        // antes do handler — ambiente resolvido uma vez por requisição.
        api = api.nest(
            "/serpro",
            Router::new()
                .route("/ambiente", get(obter_ambiente))
                // Ordem dos route_layer: o último adicionado é o mais externo.
                // `exigir` (audiência client) deve rodar primeiro para inserir
                // o RequestContext; o snapshot entra logo depois, mais interno.
                .route_layer(axum_middleware::from_fn_with_state(
                    estado_ambiente.clone(),
                    resolver_ambiente,
                ))
                .route_layer(axum_middleware::from_fn_with_state(
                    ExigirAudiencia::novo(
                        auth_state.tokens.clone(),
                        crate::domain::auth::Audiencia::Client,
                    ),
                    exigir,
                ))
                .with_state(serpro_state),
        );

        let http_integra_contador = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(
                config.serpro.http_connect_timeout_secs,
            ))
            .timeout(std::time::Duration::from_secs(
                config.serpro.http_request_timeout_secs,
            ))
            .build()
            .expect("cliente HTTP Integra Contador deve inicializar");
        let endpoints_integra_contador = std::sync::Arc::new(
            crate::infrastructure::serpro::SerproEndpointResolverImpl::novo(config.serpro.clone()),
        );
        let repositorio_integra_contador =
            crate::infrastructure::persistence::integra_contador::IntegraContadorRepositorioDiesel::novo(
                pool.clone(),
            );
        // ===== FASE A — procurações persistentes e catálogo configurável =====
        let cliente_repositorio_integra = std::sync::Arc::new(cliente_repo.clone());
        let procuracoes_repositorio_integra = std::sync::Arc::new(
            crate::infrastructure::persistence::integra_contador::ProcuracoesRepositorioDiesel::novo(
                pool.clone(),
            ),
        );
        let catalogo_integra: std::sync::Arc<
            dyn crate::application::integra_contador::ports::ProvedorCatalogoIntegra,
        > = std::sync::Arc::new(
            crate::infrastructure::persistence::integra_contador::CatalogoServicosIntegraDiesel::novo(
                pool.clone(),
            ),
        );
        let cliente_integra_contador = std::sync::Arc::new(
            crate::infrastructure::integra_contador::ClienteIntegraContadorImpl::novo(
                http_integra_contador,
                endpoints_integra_contador,
                config.serpro.operator_header_enabled,
            ),
        );
        let executar_integra_contador = std::sync::Arc::new(
            crate::application::integra_contador::ExecutarIntegraContadorUseCase::novo(
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                cliente_integra_contador,
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                std::sync::Arc::new(
                    crate::infrastructure::persistence::integra_contador::ValidadorProcuracaoDiesel::novo(
                        procuracoes_repositorio_integra.clone(),
                        Some(config.serpro.procuracao_cache_ttl_secs),
                    ),
                ),
                config.serpro.operator_header_enabled,
                servico_medicao.clone(),
            )
            .com_catalogo(Some(catalogo_integra.clone())),
        );
        let validar_procuracao_integra = std::sync::Arc::new(
            crate::application::integra_contador::ValidarProcuracoesUseCase::novo(
                cliente_repositorio_integra.clone(),
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                executar_integra_contador.clone(),
                procuracoes_repositorio_integra.clone(),
                Some(config.serpro.procuracao_cache_ttl_secs),
            ),
        );
        let listar_procuracao_integra = std::sync::Arc::new(
            crate::application::integra_contador::ListarProcuracoesUseCase::novo(
                cliente_repositorio_integra.clone(),
                procuracoes_repositorio_integra.clone(),
                Some(catalogo_integra.clone()),
                Some(config.serpro.procuracao_cache_ttl_secs),
            ),
        );

        // ===== AUTENTICAPROCURADOR (modelo Software House) =====
        let criptografia_integra =
            criar_criptografia(config.certificate_master_key.expose_secret())
                .expect("criptografia Integra deve inicializar");
        let autentica_procurador_http_cliente = std::sync::Arc::new(
            crate::infrastructure::integra_contador::AutenticaProcuradorClienteImpl::novo(
                reqwest::Client::builder()
                    .connect_timeout(std::time::Duration::from_secs(
                        config.serpro.http_connect_timeout_secs,
                    ))
                    .timeout(std::time::Duration::from_secs(
                        config.serpro.http_request_timeout_secs,
                    ))
                    .build()
                    .expect("cliente HTTP AUTENTICAPROCURADOR deve inicializar"),
                std::sync::Arc::new(
                    crate::infrastructure::serpro::SerproEndpointResolverImpl::novo(
                        config.serpro.clone(),
                    ),
                ),
            ),
        );
        let certificado_provider_integra = std::sync::Arc::new(
            crate::infrastructure::persistence::integra_contador::CertificadoA1ProviderDiesel::novo(
                std::sync::Arc::new(certificado_repo.clone()),
            ),
        );
        // O token jwt_procurador é cifrado com o mesmo formato AES-256-GCM
        // (nonce||ciphertext) do ecossistema tokens_serpro/credenciais_serpro.
        let tokens_serpro_repositorio = std::sync::Arc::new(
            crate::infrastructure::persistence::integra_contador::TokensSerproRepositorioDiesel::novo(
                pool.clone(),
                std::sync::Arc::new(
                    SerproCriptografia::novo(config.certificate_master_key.expose_secret())
                        .expect("criptografia de tokens SERPRO deve inicializar"),
                ),
            ),
        );
        // Identidade da Software House (Contratante no modelo C), vinda da
        // configuração. Sem ela o fluxo AUTENTICAPROCURADOR falha fechado.
        let contratante_plataforma = match (
            config.serpro.contratante_cnpj.clone(),
            config.serpro.contratante_nome.clone(),
        ) {
            (Some(numero), Some(nome)) => {
                Some(crate::application::integra_contador::ContratantePlataforma { numero, nome })
            }
            _ => {
                tracing::warn!(
                    "SERPRO_CONTRATANTE_CNPJ/SERPRO_CONTRATANTE_NOME ausentes; \
                     AUTENTICAPROCURADOR falhará com 503 até serem configurados"
                );
                None
            }
        };
        let autenticar_procurador_use_case = std::sync::Arc::new(
            crate::application::integra_contador::AutenticarProcuradorUseCase::novo(
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                std::sync::Arc::new(cliente_repo.clone()),
                certificado_provider_integra,
                criptografia_integra,
                std::sync::Arc::new(
                    crate::infrastructure::integra_contador::AssinadorXmlImpl::novo(),
                ),
                autentica_procurador_http_cliente,
                tokens_serpro_repositorio,
                std::sync::Arc::new(repositorio_integra_contador.clone()),
                contratante_plataforma,
            ),
        );
        let integra_contador_state =
            crate::presentation::integra_contador::state::EstadoIntegraContador::novo(
                executar_integra_contador,
                autenticar_procurador_use_case,
                validar_procuracao_integra,
                listar_procuracao_integra,
                catalogo_integra,
            );
        api = api.nest(
            "/serpro/integra-contador",
            crate::presentation::integra_contador::construir(
                integra_contador_state.clone(),
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
                estado_ambiente.clone(),
            ),
        );
        api = api.nest(
            "/admin/integra-contador",
            crate::presentation::integra_contador::construir_admin(
                integra_contador_state,
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Admin,
                ),
            ),
        );

        // ===== Dívida Ativa (Fase 4, SPEC §4.3–§4.5) =====
        let http_serpro = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(
                config.serpro.http_connect_timeout_secs,
            ))
            .timeout(std::time::Duration::from_secs(
                config.serpro.http_request_timeout_secs,
            ))
            .build()
            .expect("cliente HTTP SERPRO deve inicializar");

        let endpoints_serpro = std::sync::Arc::new(
            crate::infrastructure::serpro::SerproEndpointResolverImpl::novo(config.serpro.clone()),
        );
        let consultas_repo = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::repositorio::ConsultasFiscaisRepositorioDiesel::novo(
                pool.clone(),
            ),
        );
        let limitador_consultas = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::rate::LimitadorPorEscritorio::novo(
                config.serpro.rate_limit_consultas_por_segundo,
            ),
        );
        let hash_chave_consultas = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::hash::HashChaveSha256::novo(
                config.serpro.consultas_chave_pepper.clone(),
            ),
        );
        let consultas_repositorio_trait = consultas_repo.clone();
        let cliente_http_divida = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::cliente_http::ClienteDividaAtivaHttp::novo(
                http_serpro,
            ),
        );
        let catalogo_clientes_consulta = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::repositorio::CatalogoClientesDiesel::novo(
                pool.clone(),
            ),
        );
        let catalogo_operadores_consulta = std::sync::Arc::new(
            crate::infrastructure::divida_ativa::repositorio::CatalogoOperadoresDiesel::novo(
                pool.clone(),
            ),
        );

        // Ambiente: o snapshot de ambiente entra via middleware no
        // router (`EstadoAmbiente` compartilhado com `/serpro`).

        // Fallback D2: só no fio quando a flag estiver habilitada; sem
        // catálogo interno, a segunda tentativa nunca acontece (D2).
        let mut consultar_devedor_uc = ConsultarDevedorUseCase::novo(
            endpoints_serpro.clone(),
            limitador_consultas.clone(),
            consultas_repositorio_trait.clone(),
            hash_chave_consultas.clone(),
            cliente_http_divida.clone(),
            servico_medicao.clone(),
        )
        .com_catalogo_clientes(catalogo_clientes_consulta)
        .com_operadores(
            catalogo_operadores_consulta.clone(),
            config.serpro.operator_header_enabled,
        );
        if config.serpro.divida_ativa_fallback_codigo_contribuinte {
            consultar_devedor_uc = consultar_devedor_uc.com_fallback(std::sync::Arc::new(
                crate::infrastructure::divida_ativa::repositorio::ResolvedorChaveAlternativaSemCatalogo,
            ));
        }
        let consultar_inscricao_uc = ConsultarInscricaoUseCase::novo(
            endpoints_serpro,
            limitador_consultas,
            consultas_repositorio_trait,
            hash_chave_consultas,
            cliente_http_divida,
            servico_medicao,
        )
        .com_operadores(
            catalogo_operadores_consulta,
            config.serpro.operator_header_enabled,
        );

        api = api.nest(
            "/divida-ativa",
            crate::presentation::divida_ativa::router::construir(
                std::sync::Arc::new(consultar_devedor_uc),
                std::sync::Arc::new(consultar_inscricao_uc),
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
                estado_ambiente,
            ),
        );

        // ===== Fase B — Trabalhos agendados e consultas em lote =====
        let trabalhos_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::trabalhos::TrabalhosAgendadosDiesel::novo(
                pool.clone(),
            ),
        );
        let execucoes_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::trabalhos::ExecucoesTrabalhoDiesel::novo(
                pool.clone(),
            ),
        );
        let lotes_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::trabalhos::ConsultasLoteDiesel::novo(pool.clone()),
        );
        let itens_lote_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::trabalhos::ItensConsultaLoteDiesel::novo(
                pool.clone(),
            ),
        );

        let criar_trabalho_uc = std::sync::Arc::new(
            crate::application::trabalhos::CriarTrabalhoAgendadoUseCase::novo(
                trabalhos_repo.clone(),
            ),
        );
        let listar_trabalhos_uc = std::sync::Arc::new(
            crate::application::trabalhos::ListarTrabalhosAgendadosUseCase::novo(
                trabalhos_repo.clone(),
            ),
        );
        let buscar_trabalho_uc = std::sync::Arc::new(
            crate::application::trabalhos::BuscarTrabalhoAgendadoUseCase::novo(
                trabalhos_repo.clone(),
            ),
        );
        let habilitar_desabilitar_trabalho_uc = std::sync::Arc::new(
            crate::application::trabalhos::HabilitarDesabilitarTrabalhoUseCase::novo(
                trabalhos_repo.clone(),
            ),
        );
        let listar_execucoes_uc = std::sync::Arc::new(
            crate::application::trabalhos::ListarExecucoesTrabalhoUseCase::novo(
                execucoes_repo.clone(),
            ),
        );
        let criar_lote_uc = std::sync::Arc::new(
            crate::application::trabalhos::CriarConsultaLoteUseCase::novo(
                lotes_repo.clone(),
                itens_lote_repo.clone(),
            ),
        );
        let listar_lotes_uc = std::sync::Arc::new(
            crate::application::trabalhos::ListarConsultasLoteUseCase::novo(lotes_repo.clone()),
        );
        let buscar_lote_uc = std::sync::Arc::new(
            crate::application::trabalhos::BuscarConsultaLoteUseCase::novo(lotes_repo.clone()),
        );
        let listar_itens_lote_uc = std::sync::Arc::new(
            crate::application::trabalhos::ListarItensConsultaLoteUseCase::novo(
                itens_lote_repo.clone(),
            ),
        );

        let estado_trabalhos = crate::presentation::trabalhos::state::EstadoTrabalhos::novo(
            criar_trabalho_uc,
            listar_trabalhos_uc,
            buscar_trabalho_uc,
            habilitar_desabilitar_trabalho_uc,
            listar_execucoes_uc,
            criar_lote_uc,
            listar_lotes_uc,
            buscar_lote_uc,
            listar_itens_lote_uc,
        );

        api = api.nest(
            "/trabalhos",
            crate::presentation::trabalhos::construir(
                estado_trabalhos,
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
            ),
        );

        // ===== Fase C — Notificações =====
        let notificacoes_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::notificacoes::NotificacoesDiesel::novo(
                pool.clone(),
            ),
        );
        let preferencias_notificacao_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::notificacoes::PreferenciasNotificacaoDiesel::novo(
                pool.clone(),
            ),
        );
        let templates_notificacao_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::notificacoes::TemplatesNotificacaoDiesel::novo(
                pool.clone(),
            ),
        );

        let criar_notificacao_uc = std::sync::Arc::new(
            crate::application::notificacoes::CriarNotificacaoUseCase::novo(
                notificacoes_repo.clone(),
            ),
        );
        let listar_notificacoes_uc = std::sync::Arc::new(
            crate::application::notificacoes::ListarNotificacoesUseCase::novo(
                notificacoes_repo.clone(),
            ),
        );
        let contar_nao_lidas_uc = std::sync::Arc::new(
            crate::application::notificacoes::ContarNotificacoesNaoLidasUseCase::novo(
                notificacoes_repo.clone(),
            ),
        );
        let marcar_como_lida_uc = std::sync::Arc::new(
            crate::application::notificacoes::MarcarNotificacaoLidaUseCase::novo(
                notificacoes_repo.clone(),
            ),
        );
        let marcar_todas_como_lidas_uc = std::sync::Arc::new(
            crate::application::notificacoes::MarcarTodasNotificacoesLidasUseCase::novo(
                notificacoes_repo.clone(),
            ),
        );
        let buscar_preferencia_uc = std::sync::Arc::new(
            crate::application::notificacoes::BuscarPreferenciaUseCase::novo(
                preferencias_notificacao_repo.clone(),
            ),
        );
        let criar_ou_atualizar_preferencia_uc = std::sync::Arc::new(
            crate::application::notificacoes::CriarOuAtualizarPreferenciaUseCase::novo(
                preferencias_notificacao_repo.clone(),
            ),
        );
        let listar_templates_uc = std::sync::Arc::new(
            crate::application::notificacoes::ListarTemplatesNotificacaoUseCase::novo(
                templates_notificacao_repo.clone(),
            ),
        );

        let estado_notificacoes =
            crate::presentation::notificacoes::state::EstadoNotificacoes::novo(
                criar_notificacao_uc,
                listar_notificacoes_uc,
                contar_nao_lidas_uc,
                marcar_como_lida_uc,
                marcar_todas_como_lidas_uc,
                buscar_preferencia_uc,
                criar_ou_atualizar_preferencia_uc,
                listar_templates_uc,
            );

        api = api.nest(
            "/notificacoes",
            crate::presentation::notificacoes::construir(
                estado_notificacoes,
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
            ),
        );

        // ===== Fase D — Emissão de guias =====
        let guias_emissao_repo = std::sync::Arc::new(
            crate::infrastructure::persistence::guias::GuiasEmissaoDiesel::novo(pool.clone()),
        );

        let emitir_guia_uc =
            std::sync::Arc::new(crate::application::guias::EmitirGuiaUseCase::novo(
                guias_emissao_repo.clone(),
                std::sync::Arc::new(
                    crate::infrastructure::guias::ProvedorEmissaoGuiaNaoImplementado,
                ),
            ));
        let listar_guias_uc = std::sync::Arc::new(
            crate::application::guias::ListarGuiasUseCase::novo(guias_emissao_repo.clone()),
        );
        let buscar_guia_uc = std::sync::Arc::new(
            crate::application::guias::BuscarGuiaUseCase::novo(guias_emissao_repo.clone()),
        );
        let baixar_guia_uc = std::sync::Arc::new(
            crate::application::guias::BaixarGuiaUseCase::novo(guias_emissao_repo.clone()),
        );

        let estado_guias = crate::presentation::guias::state::EstadoGuias::novo(
            emitir_guia_uc,
            listar_guias_uc,
            buscar_guia_uc,
            baixar_guia_uc,
        );

        api = api.nest(
            "/guias",
            crate::presentation::guias::construir(
                estado_guias,
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
            ),
        );

        // ===== Fase F — Relatórios =====
        let relatorios_repo = std::sync::Arc::new(RelatoriosDiesel::novo(pool.clone()));
        let gerar_relatorio_uc = std::sync::Arc::new(GerarRelatorioUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioRelatorios>,
            cliente_repo_trait.clone(),
        ));
        let listar_relatorios_uc = std::sync::Arc::new(ListarRelatoriosUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioRelatorios>,
        ));
        let buscar_relatorio_uc = std::sync::Arc::new(BuscarRelatorioUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioRelatorios>,
        ));
        let criar_link_uc = std::sync::Arc::new(CriarLinkCompartilhamentoUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioLinksCompartilhamento>,
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioRelatorios>,
        ));
        let validar_link_uc = std::sync::Arc::new(ValidarLinkCompartilhamentoUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioLinksCompartilhamento>,
        ));
        let acessar_por_link_uc = std::sync::Arc::new(AcessarRelatorioPorLinkUseCase::novo(
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioLinksCompartilhamento>,
            relatorios_repo.clone() as std::sync::Arc<dyn RepositorioRelatorios>,
        ));
        let estado_relatorios = EstadoRelatorios::novo(
            gerar_relatorio_uc,
            listar_relatorios_uc,
            buscar_relatorio_uc,
            criar_link_uc,
            validar_link_uc,
            acessar_por_link_uc,
        );

        api = api.nest(
            "/relatorios",
            construir_relatorios(
                estado_relatorios.clone(),
                ExigirAudiencia::novo(
                    auth_state.tokens.clone(),
                    crate::domain::auth::Audiencia::Client,
                ),
            ),
        );

        // Rota pública de link de compartilhamento (sem autenticação, somente leitura).
        api = api.merge(construir_publico(estado_relatorios));
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
