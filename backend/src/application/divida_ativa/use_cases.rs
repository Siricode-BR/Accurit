use std::sync::Arc;
use std::time::Instant;

use crate::application::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, ClienteDividaAtiva, HashDeChaveConsulta,
    LimitadorConsultas, RepositorioConsultasFiscais, ResolvedorChaveAlternativa,
};
use crate::domain::divida_ativa::{
    ComandoConsultaDevedor, ComandoConsultaInscricao, ErroDividaAtiva, FonteConsulta,
    InscricaoDetalhe, NumeroInscricao, RegistroConsultaFiscal, ResultadoConsultaDevedor, TipoChave,
    mascarar_chave, mascarar_documento,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::erros::ErroEndpoint;
use crate::domain::serpro::medicao::{
    ComandoRegistrarChamada, OrigemChamada, ResultadoChamada, ServicoMedicaoSerpro,
};
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

fn erro_endpoint(erro: ErroEndpoint, ambiente: AmbienteSerpro) -> ErroDividaAtiva {
    match erro {
        ErroEndpoint::DividaAtivaProducaoBloqueada => ErroDividaAtiva::ProducaoNaoConfigurada,
        ErroEndpoint::Indefinido(_) if ambiente == AmbienteSerpro::Trial => {
            ErroDividaAtiva::TrialNaoConfigurado
        }
        ErroEndpoint::Indefinido(_) => ErroDividaAtiva::ProducaoNaoConfigurada,
        erro @ ErroEndpoint::Invalida { .. } => {
            tracing::error!(erro = %erro, "configuração do endpoint da Dívida Ativa inválida");
            ErroDividaAtiva::ErroInterno(erro.to_string())
        }
    }
}

fn registro_base(
    escritorio_id: uuid::Uuid,
    servico: &str,
    ambiente: AmbienteSerpro,
    chave_mascarada: String,
    fonte_consulta: Option<&str>,
) -> RegistroConsultaFiscal {
    RegistroConsultaFiscal {
        escritorio_id,
        cliente_id: None,
        produto: "DIVIDA_ATIVA".to_owned(),
        servico: servico.to_owned(),
        verbo: "GET".to_owned(),
        ambiente: ambiente.as_str().to_owned(),
        chave_consulta_hash: None,
        chave_consulta_mascarada: Some(chave_mascarada.clone()),
        fonte_consulta: fonte_consulta.map(str::to_owned),
        status_code: None,
        estado: "EM_ANDAMENTO".to_owned(),
        protocolo: None,
        request_redigido: Some(serde_json::json!({ "chaveMascarada": chave_mascarada })),
        response_redigido: None,
    }
}

fn status_historico(erro: &ErroDividaAtiva) -> (i32, &'static str) {
    match erro {
        ErroDividaAtiva::ChaveInvalida => (400, "CHAVE_INVALIDA"),
        ErroDividaAtiva::LimiteLocalAtingido | ErroDividaAtiva::LimiteSerproAtingido => {
            (429, "LIMITE_EXCEDIDO")
        }
        ErroDividaAtiva::TrialNaoConfigurado
        | ErroDividaAtiva::ProducaoNaoConfigurada
        | ErroDividaAtiva::ServicoIndisponivel => (503, "INDISPONIVEL"),
        ErroDividaAtiva::Indeterminado => (504, "INDETERMINADO"),
        ErroDividaAtiva::NaoAutorizado | ErroDividaAtiva::ProxyNegado => (502, "NAO_AUTORIZADO"),
        ErroDividaAtiva::RespostaInvalida | ErroDividaAtiva::RespostaInvalidaDetalhada(_) => {
            (502, "RESPOSTA_INVALIDA")
        }
        ErroDividaAtiva::NaoEncontrado => (404, "NAO_ENCONTRADO"),
        ErroDividaAtiva::ErroInterno(_) => (500, "ERRO_INTERNO"),
    }
}

fn registrar_falha(
    repositorio: &dyn RepositorioConsultasFiscais,
    mut registro: RegistroConsultaFiscal,
    erro: ErroDividaAtiva,
) -> ErroDividaAtiva {
    let (status, estado) = status_historico(&erro);
    registro.status_code = Some(status);
    registro.estado = estado.to_owned();
    registro.response_redigido = Some(serde_json::json!({ "codigo": erro.codigo() }));

    match repositorio.registrar(&registro) {
        Ok(_) => erro,
        Err(falha_persistencia) => {
            tracing::error!(
                erro = %falha_persistencia,
                "não foi possível persistir a falha da consulta de Dívida Ativa"
            );
            ErroDividaAtiva::ErroInterno("falha ao registrar consulta fiscal".to_owned())
        }
    }
}

fn cpf_operador(
    catalogo: &Option<Arc<dyn CatalogoOperadores>>,
    habilitado: bool,
    usuario_id: uuid::Uuid,
) -> Result<Option<String>, ErroDividaAtiva> {
    if !habilitado {
        return Ok(None);
    }

    let catalogo = catalogo.as_ref().ok_or_else(|| {
        ErroDividaAtiva::ErroInterno("catálogo de operadores não configurado".to_owned())
    })?;
    catalogo
        .buscar_cpf(usuario_id)
        .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?
        .map(Some)
        .ok_or_else(|| ErroDividaAtiva::ErroInterno("CPF do operador não encontrado".to_owned()))
}

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
    /// Servico de medicao de uso SERPRO — registra toda chamada de saida
    /// para fins de cobranca, distinguindo TRIAL de PRODUCAO.
    servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    /// Origem da chamada registrada na medição (Fase B.1): consultas
    /// manuais, rotinas agendadas e lotes são distinguidos na cobrança.
    origem: OrigemChamada,
}

impl ConsultarDevedorUseCase {
    pub fn novo(
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        limitador: Arc<dyn LimitadorConsultas>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        hash: Arc<dyn HashDeChaveConsulta>,
        cliente: Arc<dyn ClienteDividaAtiva>,
        servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
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
            servico_medicao,
            origem: OrigemChamada::Manual,
        }
    }

    /// Define a origem da chamada na medição de uso (Fase B.1).
    #[must_use]
    pub const fn com_origem(mut self, origem: OrigemChamada) -> Self {
        self.origem = origem;
        self
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
        request_id: &str,
    ) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva> {
        let mut registro = registro_base(
            comando.escritorio_id,
            "DEVEDOR",
            ambiente,
            mascarar_documento(comando.chave.como_str()),
            Some(FonteConsulta::CpfCnpj.as_str()),
        );
        registro.chave_consulta_hash = self.hash.hash(comando.chave.como_str());

        if let Err(erro) = self.limitador.permitir(comando.escritorio_id) {
            return Err(registrar_falha(&*self.repositorio, registro, erro));
        }

        let endpoint = match self
            .endpoint_resolver
            .resolver(ProdutoSerpro::DividaAtiva, ambiente)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(erro) => {
                return Err(registrar_falha(
                    &*self.repositorio,
                    registro,
                    erro_endpoint(erro, ambiente),
                ));
            }
        };

        let operador = match cpf_operador(
            &self.catalogo_operadores,
            self.operador_header_habilitado,
            comando.usuario_id,
        ) {
            Ok(cpf) => cpf,
            Err(erro) => return Err(registrar_falha(&*self.repositorio, registro, erro)),
        };

        let inicio = Instant::now();
        let resultado = match self
            .cliente
            .consultar_devedor(&endpoint, &comando.chave, operador, request_id)
            .await
        {
            Ok(resultado) => {
                let duracao_ms = inicio.elapsed().as_millis() as u64;
                let resultado_medicao = match resultado.ambiente {
                    AmbienteSerpro::Trial => match registro.status_code {
                        Some(200..=299) => ResultadoChamada::Sucesso,
                        Some(400..=499) => ResultadoChamada::Erro4xx,
                        Some(500..=599) => ResultadoChamada::Erro5xx,
                        _ => ResultadoChamada::Indeterminado,
                    },
                    AmbienteSerpro::Producao => match registro.status_code {
                        Some(200..=299) => ResultadoChamada::Sucesso,
                        Some(400..=499) => ResultadoChamada::Erro4xx,
                        Some(500..=599) => ResultadoChamada::Erro5xx,
                        _ => ResultadoChamada::Indeterminado,
                    },
                };
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::DividaAtiva,
                        verbo: "consultar_devedor".to_owned(),
                        servico: Some("devedor".to_owned()),
                        ambiente: resultado.ambiente,
                        cliente_id: registro.cliente_id,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: resultado_medicao,
                        status_http: Some(200),
                        duracao_ms,
                        request_id: request_id.to_owned(),
                    })
                    .await;
                resultado
            }
            Err(erro) => {
                let duracao_ms = inicio.elapsed().as_millis() as u64;
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::DividaAtiva,
                        verbo: "consultar_devedor".to_owned(),
                        servico: Some("devedor".to_owned()),
                        ambiente,
                        cliente_id: None,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: ResultadoChamada::Indeterminado,
                        status_http: None,
                        duracao_ms,
                        request_id: request_id.to_owned(),
                    })
                    .await;
                return Err(registrar_falha(&*self.repositorio, registro, erro));
            }
        };

        if let Some(catalogo) = &self.catalogo_clientes {
            match catalogo.buscar_por_documento(comando.escritorio_id, comando.chave.como_str()) {
                Ok(cliente_id) => registro.cliente_id = cliente_id,
                Err(erro) => tracing::warn!(
                    erro = %erro,
                    "não foi possível associar a consulta ao cliente do escritório"
                ),
            }
        }

        registro.status_code = Some(200);
        registro.estado = resultado.estado.as_str().to_owned();
        registro.response_redigido = Some(serde_json::json!({
            "quantidade": resultado.inscricoes.len()
        }));
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
    /// Servico de medicao de uso SERPRO — registra toda chamada de saida
    /// para fins de cobranca, distinguindo TRIAL de PRODUCAO.
    servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    /// Origem da chamada registrada na medição (Fase B.1).
    origem: OrigemChamada,
}

impl ConsultarInscricaoUseCase {
    pub fn novo(
        endpoint_resolver: Arc<dyn ResolverEndpointsSerpro>,
        limitador: Arc<dyn LimitadorConsultas>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        hash: Arc<dyn HashDeChaveConsulta>,
        cliente: Arc<dyn ClienteDividaAtiva>,
        servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    ) -> Self {
        Self {
            endpoint_resolver,
            limitador,
            repositorio,
            hash,
            cliente,
            catalogo_operadores: None,
            operador_header_habilitado: false,
            servico_medicao,
            origem: OrigemChamada::Manual,
        }
    }

    /// Define a origem da chamada na medição de uso (Fase B.1).
    #[must_use]
    pub const fn com_origem(mut self, origem: OrigemChamada) -> Self {
        self.origem = origem;
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

    pub async fn executar(
        &self,
        comando: ComandoConsultaInscricao,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva> {
        let numero = NumeroInscricao::novo(&comando.numero_inscricao)
            .map_err(|_| ErroDividaAtiva::ChaveInvalida)?;
        let chave_mascarada =
            mascarar_chave(&TipoChave::Inscricao(numero), &comando.numero_inscricao);
        let mut registro = registro_base(
            comando.escritorio_id,
            "INSCRICAO",
            ambiente,
            chave_mascarada,
            None,
        );
        registro.chave_consulta_hash = self.hash.hash(&comando.numero_inscricao);

        if let Err(erro) = self.limitador.permitir(comando.escritorio_id) {
            return Err(registrar_falha(&*self.repositorio, registro, erro));
        }

        let endpoint = match self
            .endpoint_resolver
            .resolver(ProdutoSerpro::DividaAtiva, ambiente)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(erro) => {
                return Err(registrar_falha(
                    &*self.repositorio,
                    registro,
                    erro_endpoint(erro, ambiente),
                ));
            }
        };

        let operador = match cpf_operador(
            &self.catalogo_operadores,
            self.operador_header_habilitado,
            comando.usuario_id,
        ) {
            Ok(cpf) => cpf,
            Err(erro) => return Err(registrar_falha(&*self.repositorio, registro, erro)),
        };

        let inicio = Instant::now();
        let detalhe = match self
            .cliente
            .consultar_inscricao(&endpoint, &comando.numero_inscricao, operador, request_id)
            .await
        {
            Ok(detalhe) => {
                let duracao_ms = inicio.elapsed().as_millis() as u64;
                let resultado_medicao = match detalhe.ambiente {
                    AmbienteSerpro::Trial => match 200u16 {
                        200..=299 => ResultadoChamada::Sucesso,
                        400..=499 => ResultadoChamada::Erro4xx,
                        500..=599 => ResultadoChamada::Erro5xx,
                        _ => ResultadoChamada::Indeterminado,
                    },
                    AmbienteSerpro::Producao => match 200u16 {
                        200..=299 => ResultadoChamada::Sucesso,
                        400..=499 => ResultadoChamada::Erro4xx,
                        500..=599 => ResultadoChamada::Erro5xx,
                        _ => ResultadoChamada::Indeterminado,
                    },
                };
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::DividaAtiva,
                        verbo: "consultar_inscricao".to_owned(),
                        servico: Some("inscricao".to_owned()),
                        ambiente: detalhe.ambiente,
                        cliente_id: None,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: resultado_medicao,
                        status_http: Some(200),
                        duracao_ms,
                        request_id: request_id.to_owned(),
                    })
                    .await;
                detalhe
            }
            Err(erro) => {
                let duracao_ms = inicio.elapsed().as_millis() as u64;
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::DividaAtiva,
                        verbo: "consultar_inscricao".to_owned(),
                        servico: Some("inscricao".to_owned()),
                        ambiente,
                        cliente_id: None,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: ResultadoChamada::Indeterminado,
                        status_http: None,
                        duracao_ms,
                        request_id: request_id.to_owned(),
                    })
                    .await;
                return Err(registrar_falha(&*self.repositorio, registro, erro));
            }
        };

        registro.status_code = Some(200);
        registro.estado = detalhe.estado.as_str().to_owned();
        registro.response_redigido = Some(serde_json::json!({
            "estado": detalhe.estado.as_str()
        }));
        self.repositorio
            .registrar(&registro)
            .map_err(|erro| ErroDividaAtiva::ErroInterno(erro.to_string()))?;

        Ok(detalhe)
    }
}
