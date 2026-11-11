//! Fase B — Casos de uso de trabalhos agendados e consultas em lote.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::clientes::ClienteRepositorio;
use crate::application::divida_ativa::use_cases::ConsultarDevedorUseCase;
use crate::application::integra_contador::{
    ComandoValidarProcuracao, ExecutarIntegraContadorUseCase, ValidarProcuracoesUseCase,
};
use crate::application::usuarios::{FiltroUsuarios, UsuarioGestaoRepositorio};
use crate::domain::auth::Papel;
use crate::domain::clientes::{CpfCnpj, FiltroClientes, Paginacao};
use crate::domain::divida_ativa::ComandoConsultaDevedor;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::{
    ComandoRegistrarBloqueio, MotivoBloqueio, ServicoMedicaoSerpro,
};
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::domain::trabalhos::entidades::{
    ConsultaLote, ExecucaoTrabalho, ItemConsultaLote, NovoConsultaLote, NovoItemConsultaLote,
    NovoTrabalhoAgendado, TrabalhoAgendado,
};
use crate::domain::trabalhos::enums::{StatusLote, TipoTrabalho};
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::{
    ExecutorTrabalho, RepositorioConsultasLote, RepositorioExecucoesTrabalho,
    RepositorioItensConsultaLote, RepositorioTrabalhosAgendados,
};

// ============================================================================
// Caso de uso: Criar trabalho agendado
// ============================================================================

pub struct CriarTrabalhoAgendadoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl CriarTrabalhoAgendadoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: NovoTrabalhoAgendado,
    ) -> Result<TrabalhoAgendado, ErroTrabalho> {
        self.repositorio.criar(&comando).await
    }
}

// ============================================================================
// Caso de uso: Listar trabalhos agendados por escritório
// ============================================================================

pub struct ListarTrabalhosAgendadosUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl ListarTrabalhosAgendadosUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Buscar trabalho agendado por ID
// ============================================================================

pub struct BuscarTrabalhoAgendadoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl BuscarTrabalhoAgendadoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, id: Uuid) -> Result<Option<TrabalhoAgendado>, ErroTrabalho> {
        self.repositorio.buscar_por_id(id).await
    }
}

// ============================================================================
// Caso de uso: Habilitar / Desabilitar trabalho agendado
// ============================================================================

pub struct HabilitarDesabilitarTrabalhoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl HabilitarDesabilitarTrabalhoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn habilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        self.repositorio.habilitar(id).await
    }

    pub async fn desabilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        self.repositorio.desabilitar(id).await
    }
}

// ============================================================================
// Caso de uso: Listar execuções de trabalho
// ============================================================================

pub struct ListarExecucoesTrabalhoUseCase {
    repositorio: Arc<dyn RepositorioExecucoesTrabalho>,
}

impl ListarExecucoesTrabalhoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioExecucoesTrabalho>) -> Self {
        Self { repositorio }
    }

    pub async fn executar_por_trabalho(
        &self,
        trabalho_agendado_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        self.repositorio
            .listar_por_trabalho(trabalho_agendado_id)
            .await
    }

    pub async fn executar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Criar consulta em lote
// ============================================================================

pub struct CriarConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
    itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
}

impl CriarConsultaLoteUseCase {
    pub fn novo(
        repositorio: Arc<dyn RepositorioConsultasLote>,
        itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
    ) -> Self {
        Self {
            repositorio,
            itens_repositorio,
        }
    }

    pub async fn executar(
        &self,
        comando: NovoConsultaLote,
        documentos: &[NovoItemConsultaLote],
    ) -> Result<ConsultaLote, ErroTrabalho> {
        let lote = self.repositorio.criar(&comando).await?;
        let _itens = self.itens_repositorio.criar(lote.id, documentos).await?;
        Ok(lote)
    }
}

// ============================================================================
// Caso de uso: Listar consultas em lote por escritório
// ============================================================================

pub struct ListarConsultasLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
}

impl ListarConsultasLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioConsultasLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, escritorio_id: Uuid) -> Result<Vec<ConsultaLote>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Buscar consulta em lote por ID
// ============================================================================

pub struct BuscarConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
}

impl BuscarConsultaLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioConsultasLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, id: Uuid) -> Result<Option<ConsultaLote>, ErroTrabalho> {
        self.repositorio.buscar_por_id(id).await
    }
}

// ============================================================================
// Caso de uso: Listar itens de consulta em lote
// ============================================================================

pub struct ListarItensConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioItensConsultaLote>,
}

impl ListarItensConsultaLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioItensConsultaLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        consulta_lote_id: Uuid,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho> {
        self.repositorio.listar_por_lote(consulta_lote_id).await
    }
}

// ============================================================================
// Caso de uso: Reexecutar consulta em lote (manual, Fase B.1)
// ============================================================================

/// Devolve à fila os itens `falhou`/`bloqueado` de um lote já finalizado,
/// permitindo a reexecução manual pela tela de consultas em lote. O worker
/// interno processa o lote no próximo poll.
pub struct ReexecutarConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
    itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
}

impl ReexecutarConsultaLoteUseCase {
    pub fn novo(
        repositorio: Arc<dyn RepositorioConsultasLote>,
        itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
    ) -> Self {
        Self {
            repositorio,
            itens_repositorio,
        }
    }

    pub async fn executar(&self, lote_id: Uuid) -> Result<(), ErroTrabalho> {
        let lote = self
            .repositorio
            .buscar_por_id(lote_id)
            .await?
            .ok_or(ErroTrabalho::ConsultaLoteNaoEncontrada)?;
        if lote.status == StatusLote::Executando {
            return Err(ErroTrabalho::ExecucaoEmAndamento);
        }

        let resetados = self
            .itens_repositorio
            .resetar_para_reexecucao(lote_id)
            .await?;
        if resetados == 0 {
            return Ok(());
        }

        self.repositorio
            .atualizar_status(
                lote_id,
                StatusLote::Pendente,
                lote.iniciado_em,
                None,
                lote.total_itens,
                0,
                0,
            )
            .await?;
        Ok(())
    }
}

// ============================================================================
// Executor de trabalho: executa uma tarefa fiscal agendada
// ============================================================================

/// Executa uma rotina fiscal pelos casos de uso existentes — portanto
/// passando pelo decorator de medição (origem `rotina`), pelo rate limit
/// local e pelo limite do plano do escritório.
pub struct ExecutorTrabalhoImpl {
    consultar_devedor: Arc<ConsultarDevedorUseCase>,
    executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
    validar_procuracao: Arc<ValidarProcuracoesUseCase>,
    clientes: Arc<dyn ClienteRepositorio>,
    usuarios: Arc<dyn UsuarioGestaoRepositorio>,
    servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
}

impl ExecutorTrabalhoImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        consultar_devedor: Arc<ConsultarDevedorUseCase>,
        executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
        validar_procuracao: Arc<ValidarProcuracoesUseCase>,
        clientes: Arc<dyn ClienteRepositorio>,
        usuarios: Arc<dyn UsuarioGestaoRepositorio>,
        servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    ) -> Self {
        Self {
            consultar_devedor,
            executar_integra_contador,
            validar_procuracao,
            clientes,
            usuarios,
            servico_medicao,
        }
    }

    /// Clientes do escritório, com o documento já normalizado.
    async fn clientes_do_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<(Uuid, String)>, ErroTrabalho> {
        let clientes = self.clientes.clone();
        tokio::task::spawn_blocking(move || {
            let mut pagina = 1_i64;
            let mut encontrados = Vec::new();
            loop {
                let paginacao = Paginacao::validada(pagina, 100)
                    .map_err(|_| ErroTrabalho::Interno("paginação de clientes".into()))?;
                let listagem = clientes
                    .listar(escritorio_id, FiltroClientes::Todos, paginacao, None)
                    .map_err(|_| ErroTrabalho::Persistencia("listar clientes".into()))?;
                let total = listagem.total;
                encontrados.extend(
                    listagem
                        .itens
                        .into_iter()
                        .map(|cliente| (cliente.id, cliente.cpf_cnpj)),
                );
                if encontrados.len() as i64 >= total || listagem_total_vazia(total) {
                    break;
                }
                pagina += 1;
            }
            Ok(encontrados)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    /// Usuário ativo do escritório usado como operador nas chamadas SERPRO
    /// (preferência para o papel `CLIENTE`, o titular).
    async fn usuario_operador(&self, escritorio_id: Uuid) -> Result<Uuid, ErroTrabalho> {
        let usuarios = self.usuarios.clone();
        tokio::task::spawn_blocking(move || {
            let filtro = FiltroUsuarios {
                escritorio_id: Some(escritorio_id),
                ativo: Some(true),
                ..FiltroUsuarios::default()
            };
            let lista = usuarios
                .listar(&filtro)
                .map_err(|_| ErroTrabalho::Persistencia("listar usuários".into()))?;
            lista
                .iter()
                .find(|usuario| usuario.papel == Papel::Cliente)
                .or_else(|| lista.first())
                .map(|usuario| usuario.id)
                .ok_or_else(|| {
                    ErroTrabalho::Bloqueado(
                        "escritório sem usuário ativo para executar a rotina".into(),
                    )
                })
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn pode_fazer_chamada(&self, escritorio_id: Uuid) -> Result<bool, ErroTrabalho> {
        self.servico_medicao
            .pode_fazer_chamada(escritorio_id)
            .await
            .map_err(|erro| {
                tracing::warn!(erro = %erro, "falha ao verificar o limite do plano");
                ErroTrabalho::Transitorio(
                    "falha ao verificar o limite do plano do escritório".into(),
                    0,
                )
            })
    }

    async fn registrar_bloqueio_limite(
        &self,
        escritorio_id: Uuid,
        produto: ProdutoSerpro,
        contexto: String,
    ) {
        let comando = ComandoRegistrarBloqueio {
            escritorio_id,
            produto,
            motivo: MotivoBloqueio::LimitePlano,
            contexto: Some(contexto),
        };
        if let Err(erro) = self.servico_medicao.registrar_bloqueio(&comando).await {
            tracing::warn!(erro = %erro, "não foi possível registrar o bloqueio do limite do plano");
        }
    }
}

/// Lista vazia com total zero: evita uma segunda página em escritório sem
/// clientes (defensivo contra repositórios que devolvem total negativo).
const fn listagem_total_vazia(total: i64) -> bool {
    total <= 0
}

#[async_trait]
impl ExecutorTrabalho for ExecutorTrabalhoImpl {
    async fn executar(&self, trabalho: &TrabalhoAgendado) -> Result<(i32, i32), ErroTrabalho> {
        let ambiente = match trabalho.ambiente.as_str() {
            "TRIAL" => AmbienteSerpro::Trial,
            "PRODUCAO" => AmbienteSerpro::Producao,
            _ => return Err(ErroTrabalho::Interno("ambiente inválido".into())),
        };
        let produto = trabalho
            .produto
            .parse::<ProdutoSerpro>()
            .unwrap_or(ProdutoSerpro::DividaAtiva);

        if !self.pode_fazer_chamada(trabalho.escritorio_id).await? {
            self.registrar_bloqueio_limite(
                trabalho.escritorio_id,
                produto,
                format!("trabalho {}", trabalho.id),
            )
            .await;
            return Err(ErroTrabalho::Bloqueado(
                "limite de chamadas do plano atingido".into(),
            ));
        }

        let usuario_id = self.usuario_operador(trabalho.escritorio_id).await?;
        let clientes = self.clientes_do_escritorio(trabalho.escritorio_id).await?;

        let mut processados = 0_i32;
        let mut com_erro = 0_i32;
        for (cliente_id, documento) in clientes {
            if !self.pode_fazer_chamada(trabalho.escritorio_id).await? {
                self.registrar_bloqueio_limite(
                    trabalho.escritorio_id,
                    produto,
                    format!("trabalho {} após {processados} consultas", trabalho.id),
                )
                .await;
                return Err(ErroTrabalho::Bloqueado(format!(
                    "limite do plano atingido após {processados} consultas"
                )));
            }

            let request_id = format!("rotina-{}-{}", trabalho.id, Uuid::now_v7());
            let sucesso = match trabalho.tipo {
                TipoTrabalho::ConsultaDividaAtiva => {
                    let chave = match CpfCnpj::novo(&documento) {
                        Ok(chave) => chave,
                        Err(_) => {
                            com_erro += 1;
                            continue;
                        }
                    };
                    match self
                        .consultar_devedor
                        .executar(
                            ComandoConsultaDevedor {
                                escritorio_id: trabalho.escritorio_id,
                                usuario_id,
                                chave,
                            },
                            ambiente,
                            &request_id,
                        )
                        .await
                    {
                        Ok(_) => true,
                        Err(erro) => {
                            tracing::info!(
                                codigo = erro.codigo(),
                                "consulta da rotina de Dívida Ativa falhou"
                            );
                            false
                        }
                    }
                }
                TipoTrabalho::ValidarProcuracao => self
                    .validar_procuracao
                    .executar(ComandoValidarProcuracao {
                        escritorio_id: trabalho.escritorio_id,
                        usuario_id,
                        cliente_id,
                        ambiente,
                        request_id,
                        forcar: false,
                    })
                    .await
                    .is_ok(),
                TipoTrabalho::ConsultaIntegraContador => self
                    .executar_integra_contador
                    .consultar_procuracoes(
                        trabalho.escritorio_id,
                        usuario_id,
                        documento,
                        ambiente,
                        request_id,
                    )
                    .await
                    .is_ok(),
            };

            if sucesso {
                processados += 1;
            } else {
                com_erro += 1;
            }
        }

        Ok((processados, com_erro))
    }
}
