use std::sync::Arc;
use crate::domain::divida_ativa::ports::{ClienteDividaAtiva, ConsultasFiscaisRepositorio, ResolvedorChaveAlternativa};
use crate::application::divida_ativa::ports::CpfOperadorRepositorio;
use crate::domain::divida_ativa::{ConsultaDevedor, ResultadoConsultaDevedor, InscricaoDetalhe, RegistroConsulta, FonteConsulta, ErroDividaAtiva};
use crate::domain::serpro::ports::{SerproAmbienteResolver, ResolverEndpointsSerpro};
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::infrastructure::config::SerproConfig;

#[derive(Clone)]
pub struct ConsultarDevedorUseCase {
    ambiente_resolver: Arc<dyn SerproAmbienteResolver>,
    endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
    cliente: Arc<dyn ClienteDividaAtiva>,
    repositorio: Arc<dyn ConsultasFiscaisRepositorio>,
    cpf_operador: Arc<dyn CpfOperadorRepositorio>,
    config: SerproConfig,
}

impl ConsultarDevedorUseCase {
    pub fn new(
        ambiente_resolver: Arc<dyn SerproAmbienteResolver>,
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        cliente: Arc<dyn ClienteDividaAtiva>,
        repositorio: Arc<dyn ConsultasFiscaisRepositorio>,
        cpf_operador: Arc<dyn CpfOperadorRepositorio>,
        config: SerproConfig,
    ) -> Self {
        Self { ambiente_resolver, endpoint_resolver, cliente, repositorio, cpf_operador, config }
    }

    pub async fn executar(&self, comando: ConsultaDevedor, usuario_id: uuid::Uuid) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva> {
        let ambiente = self.ambiente_resolver.resolver_ambiente(comando.escritorio_id).await
            .map_err(|e| ErroDividaAtiva::ErroInterno(e.to_string()))?;
        
        let cpf_op = if self.config.operator_header_enabled {
            self.cpf_operador.buscar_cpf_operador(usuario_id).await
                .map_err(|_| ErroDividaAtiva::Indisponivel)?
        } else {
            None
        };

        let resultado = self.cliente.consultar_devedor(&comando.chave, cpf_op.as_deref()).await?;

        // Registrar auditoria
        self.repositorio.registrar_consulta(&RegistroConsulta {
            escritorio_id: comando.escritorio_id,
            cliente_id: comando.cliente_id,
            produto: "DIVIDA_ATIVA".to_string(),
            servico: "DEVEDOR".to_string(),
            verbo: "GET".to_string(),
            ambiente: ambiente.to_string(),
            chave_hash: "".to_string(),
            chave_mascarada: "***".to_string(),
            fonte: FonteConsulta::CpfCnpj,
            status_code: Some(200),
            estado: "SUCESSO".to_string(),
            protocolo: None,
        }).map_err(|e| ErroDividaAtiva::ErroInterno(e.to_string()))?;

        Ok(resultado)
    }
}

#[derive(Clone)]
pub struct ConsultarInscricaoUseCase {
    ambiente_resolver: Arc<dyn SerproAmbienteResolver>,
    endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
    cliente: Arc<dyn ClienteDividaAtiva>,
    cpf_operador: Arc<dyn CpfOperadorRepositorio>,
    config: SerproConfig,
}

impl ConsultarInscricaoUseCase {
    pub fn new(
        ambiente_resolver: Arc<dyn SerproAmbienteResolver>,
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        cliente: Arc<dyn ClienteDividaAtiva>,
        cpf_operador: Arc<dyn CpfOperadorRepositorio>,
        config: SerproConfig,
    ) -> Self {
        Self { ambiente_resolver, endpoint_resolver, cliente, cpf_operador, config }
    }

    pub async fn executar(&self, numero_inscricao: &str, escritorio_id: uuid::Uuid, usuario_id: uuid::Uuid) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        let ambiente = self.ambiente_resolver.resolver_ambiente(escritorio_id).await
            .map_err(|e| ErroDividaAtiva::ErroInterno(e.to_string()))?;
        
        let cpf_op = if self.config.operator_header_enabled {
            self.cpf_operador.buscar_cpf_operador(usuario_id).await
                .map_err(|_| ErroDividaAtiva::Indisponivel)?
        } else {
            None
        };

        let resultado = self.cliente.consultar_inscricao(numero_inscricao, cpf_op.as_deref()).await?;
        Ok(resultado)
    }
}
