use std::sync::Arc;

use crate::application::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, ClienteDividaAtiva, HashDeChaveConsulta,
    LimitadorConsultas, RepositorioConsultasFiscais, ResolvedorChaveAlternativa,
};
use crate::domain::divida_ativa::{
    ComandoConsultaDevedor, ComandoConsultaInscricao, ErroDividaAtiva, FonteConsulta,
    InscricaoDetalhe, RegistroConsultaFiscal, ResultadoConsultaDevedor, mascarar_documento,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

#[derive(Clone)]
pub struct ConsultarDevedorUseCase {
    endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
    limitador: Arc<dyn LimitadorConsultas>,
    repositorio: Arc<dyn RepositorioConsultasFiscais>,
    hash: Arc<dyn HashDeChaveConsulta>,
    cliente: Arc<dyn ClienteDividaAtiva>,
    catalogo_clientes: Option<Arc<dyn CatalogoClientesPorDocumento>>,
    catalogo_operadores: Option<Arc<dyn CatalogoOperadores>>,
    fallback: Option<Arc<dyn ResolvedorChaveAlternativa>>,
    operador_header_habilitado: bool,
}

impl ConsultarDevedorUseCase {
    pub fn novo(
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        limitador: Arc<dyn LimitadorConsultas>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        hash: Arc<dyn HashDeChaveConsulta>,
        cliente: Arc<dyn ClienteDividaAtiva>,
    ) -> Self {
        Self {
            endpoint_resolver,
            limitador,
            repositorio,
            hash,
            cliente,
            catalogo_clientes: None,
            catalogo_operadores: None,
            fallback: None,
            operador_header_habilitado: false,
        }
    }

    pub fn com_catalogo_clientes(
        mut self,
        catalogo: Arc<dyn CatalogoClientesPorDocumento>,
    ) -> Self {
        self.catalogo_clientes = Some(catalogo);
        self
    }

    pub fn com_operadores(
        mut self,
        catalogo: Arc<dyn CatalogoOperadores>,
        habilitado: bool,
    ) -> Self {
        self.catalogo_operadores = Some(catalogo);
        self.operador_header_habilitado = habilitado;
        self
    }

    pub fn com_fallback(mut self, fallback: Arc<dyn ResolvedorChaveAlternativa>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub async fn executar(
        &self,
        comando: ComandoConsultaDevedor,
        ambiente: AmbienteSerpro,
    ) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva> {
        self.limitador.permitir(comando.escritorio_id)?;

        let endpoint = self
            .endpoint_resolver
            .resolver(ProdutoSerpro::DividaAtiva, ambiente)
            .await
            .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?;

        let cpf_operador = if self.operador_header_habilitado {
            self.catalogo_operadores
                .as_ref()
                .and_then(|catalogo| catalogo.buscar_cpf(comando.usuario_id).ok().flatten())
        } else {
            None
        };

        let resultado = self
            .cliente
            .consultar_devedor(&endpoint, &comando.chave, cpf_operador)
            .await?;

        let chave_hash = self
            .hash
            .hash(comando.chave.como_str())
            .map(Some)
            .unwrap_or(None);
        let cliente_id = self.catalogo_clientes.as_ref().and_then(|catalogo| {
            catalogo
                .buscar_por_documento(comando.escritorio_id, comando.chave.como_str())
                .ok()
                .flatten()
        });

        let registro = RegistroConsultaFiscal {
            escritorio_id: comando.escritorio_id,
            cliente_id,
            produto: "DIVIDA_ATIVA".to_owned(),
            servico: "DEVEDOR".to_owned(),
            verbo: "GET".to_owned(),
            ambiente: ambiente.as_str().to_owned(),
            chave_consulta_hash: chave_hash,
            chave_consulta_mascarada: Some(mascarar_documento(comando.chave.como_str())),
            fonte_consulta: Some(FonteConsulta::CpfCnpj.as_str().to_owned()),
            status_code: Some(200),
            estado: resultado.estado.as_str().to_owned(),
            protocolo: None,
            request_redigido: None,
            response_redigido: None,
        };

        self.repositorio
            .registrar(&registro)
            .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?;

        Ok(resultado)
    }
}

#[derive(Clone)]
pub struct ConsultarInscricaoUseCase {
    endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
    limitador: Arc<dyn LimitadorConsultas>,
    repositorio: Arc<dyn RepositorioConsultasFiscais>,
    hash: Arc<dyn HashDeChaveConsulta>,
    cliente: Arc<dyn ClienteDividaAtiva>,
    catalogo_operadores: Option<Arc<dyn CatalogoOperadores>>,
    operador_header_habilitado: bool,
}

impl ConsultarInscricaoUseCase {
    pub fn novo(
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        limitador: Arc<dyn LimitadorConsultas>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        hash: Arc<dyn HashDeChaveConsulta>,
        cliente: Arc<dyn ClienteDividaAtiva>,
    ) -> Self {
        Self {
            endpoint_resolver,
            limitador,
            repositorio,
            hash,
            cliente,
            catalogo_operadores: None,
            operador_header_habilitado: false,
        }
    }

    pub fn com_operadores(
        mut self,
        catalogo: Arc<dyn CatalogoOperadores>,
        habilitado: bool,
    ) -> Self {
        self.catalogo_operadores = Some(catalogo);
        self.operador_header_habilitado = habilitado;
        self
    }

    pub async fn executar(
        &self,
        comando: ComandoConsultaInscricao,
        ambiente: AmbienteSerpro,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        self.limitador.permitir(comando.escritorio_id)?;

        let endpoint = self
            .endpoint_resolver
            .resolver(ProdutoSerpro::DividaAtiva, ambiente)
            .await
            .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?;

        let cpf_operador = if self.operador_header_habilitado {
            self.catalogo_operadores
                .as_ref()
                .and_then(|catalogo| catalogo.buscar_cpf(comando.usuario_id).ok().flatten())
        } else {
            None
        };

        let detalhe = self
            .cliente
            .consultar_inscricao(&endpoint, &comando.numero_inscricao, cpf_operador)
            .await?;

        let registro = RegistroConsultaFiscal {
            escritorio_id: comando.escritorio_id,
            cliente_id: None,
            produto: "DIVIDA_ATIVA".to_owned(),
            servico: "INSCRICAO".to_owned(),
            verbo: "GET".to_owned(),
            ambiente: ambiente.as_str().to_owned(),
            chave_consulta_hash: self
                .hash
                .hash(&comando.numero_inscricao)
                .map(Some)
                .unwrap_or(None),
            chave_consulta_mascarada: Some(mascarar_documento(&comando.numero_inscricao)),
            fonte_consulta: Some(FonteConsulta::CpfCnpj.as_str().to_owned()),
            status_code: Some(200),
            estado: detalhe.estado.as_str().to_owned(),
            protocolo: None,
            request_redigido: None,
            response_redigido: None,
        };

        self.repositorio
            .registrar(&registro)
            .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?;

        Ok(detalhe)
    }
}
