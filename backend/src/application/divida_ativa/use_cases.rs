//! Use cases consultivos da Dívida Ativa (SPEC §4.4): uma tentativa
//! HTTP por registro de auditoria redigido (`consultas_fiscais`),
//! estados distintos (sem pendências ≠ 404 ≠ indisponível) e o
//! fallback D2 — no máximo **uma** tentativa extra com
//! `fonte_consulta = CODIGO_CONTRIBUINTE`, apenas quando a flag de
//! configuração estiver habilitada e existir porta de chave
//! alternativa ativa.

use std::sync::Arc;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::application::divida_ativa::dto::{
    DevedorPayload, DevedorResponse, InscricaoDetalhePayload, InscricaoResponse,
};
use crate::application::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, ConsultaDividaAtivaHttp, ErroHttpSerpro,
    LimitadorConsultas, RegistroConsultaFiscal, RepositorioConsultasFiscais,
    ResolvedorChaveAlternativa, ServicoDividaSerpro, SolicitacaoSerproHttp,
};
use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::devedor::{InscricaoDevedor, ResultadoDevedor};
use crate::domain::divida_ativa::enums::{EstadoConsulta, FonteConsulta};
use crate::domain::divida_ativa::erros::{ErroConsultaDivida, ErroSerpro};
use crate::domain::divida_ativa::inscricao::InscricaoDetalhe;
use crate::domain::divida_ativa::vo::{
    NumeroInscricao, TipoChave, hash_consulta, horario_consulta, mascarar_chave,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::EndpointSerpro;
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

/// Interpretação de um corpo 200 para o registro de auditoria:
/// estado observado, `response_redigido` e valor de negócio, ou um
/// erro com o status upstream observado (`Some(200)` quando o corpo
/// violou o contrato).
type Interpretacao<R> =
    Result<(EstadoConsulta, Option<Value>, R), (ErroConsultaDivida, Option<i32>)>;

pub struct ConsultaDevedorCommand {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub ambiente: AmbienteSerpro,
    pub cpf_cnpj: String,
}

#[derive(Debug, Clone)]
pub struct ConsultaDevedorResultado {
    pub ambiente: AmbienteSerpro,
    pub estado: EstadoConsulta,
    pub fonte: FonteConsulta,
    pub chave_mascarada: String,
    pub inscricoes: Vec<InscricaoDevedor>,
    pub horario_consulta: String,
    /// 1 tentativa normal, 2 quando o fallback D2 foi tentado.
    pub tentativas: u8,
}

pub struct ConsultaInscricaoCommand {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub ambiente: AmbienteSerpro,
    pub numero_inscricao: String,
}

#[derive(Debug, Clone)]
pub struct ConsultaInscricaoResultado {
    pub ambiente: AmbienteSerpro,
    pub estado: EstadoConsulta,
    pub chave_mascarada: String,
    pub detalhe: InscricaoDetalhe,
    pub horario_consulta: String,
}

/// Dependências compartilhadas dos dois use cases consultivos.
#[derive(Clone)]
struct Executante {
    endpoints: Arc<dyn ResolverEndpointsSerpro>,
    http: Arc<dyn ConsultaDividaAtivaHttp>,
    repositorio: Arc<dyn RepositorioConsultasFiscais>,
    limitador: Arc<dyn LimitadorConsultas>,
    catalogo_operadores: Option<Arc<dyn CatalogoOperadores>>,
    operator_header_enabled: bool,
}

impl Executante {
    async fn resolver_endpoint(
        &self,
        ambiente: AmbienteSerpro,
    ) -> Result<EndpointSerpro, ErroConsultaDivida> {
        self.endpoints
            .resolver(ProdutoSerpro::DividaAtiva, ambiente)
            .await
            .map_err(ErroConsultaDivida::Endpoint)
    }

    /// CPF do operador quando `SERPRO_OPERATOR_HEADER_ENABLED` está
    /// habilitado e o usuário possui CPF (11 dígitos) cadastrado.
    /// Sem catálogo enquanto a flag estiver ligada é fail-closed;
    /// CPF mal cadastrado apenas segue sem cabeçalho.
    async fn cpf_usuario(&self, usuario_id: Uuid) -> Result<Option<String>, ErroConsultaDivida> {
        if !self.operator_header_enabled {
            return Ok(None);
        }
        let Some(catalogo) = &self.catalogo_operadores else {
            return Err(ErroConsultaDivida::PersistenciaIndisponivel);
        };
        let cpf = catalogo
            .buscar_cpf(usuario_id)
            .map_err(|_| ErroConsultaDivida::PersistenciaIndisponivel)?;
        Ok(cpf.map(|valor| valor.trim().to_owned()).filter(|valor| {
            valor.len() == 11 && valor.chars().all(|caractere| caractere.is_ascii_digit())
        }))
    }

    /// Uma tentativa HTTP com registro redigido na trilha
    /// `consultas_fiscais`: interpretação em estado/redação e
    /// persistência fail-closed (falta de trilha vira ERRO_INTERNO,
    /// nunca resposta sem auditoria).
    #[allow(clippy::too_many_arguments)]
    async fn tentativa<R>(
        &self,
        endpoint: &EndpointSerpro,
        servico: ServicoDividaSerpro,
        tipo_chave: &TipoChave,
        chave_normalizada: &str,
        fonte: Option<FonteConsulta>,
        ambiente: AmbienteSerpro,
        escritorio_id: Uuid,
        cliente_id: Option<Uuid>,
        cpf_usuario: Option<String>,
        interpretar: impl Fn(Value) -> Interpretacao<R>,
    ) -> Result<R, ErroConsultaDivida> {
        let request_id = Uuid::now_v7();
        let hash = hash_consulta(chave_normalizada);
        let mascarada = mascarar_chave(tipo_chave, chave_normalizada);
        let request_redigido = json!({
            "chaveMascarada": mascarada,
            "requestId": request_id,
        });

        let solicitacao = SolicitacaoSerproHttp {
            endpoint: endpoint.clone(),
            servico,
            chave_normalizada: chave_normalizada.to_owned(),
            request_id,
            cpf_usuario,
        };

        let (estado, redigido, status_code, rumo) = match self.http.consultar(solicitacao).await {
            Ok(corpo) => match interpretar(corpo) {
                Ok((estado, redigido, rumo)) => (estado, redigido, Some(200), Ok(rumo)),
                Err((erro, upstream)) => (erro.estado_consulta(), None, upstream, Err(erro)),
            },
            Err(ErroHttpSerpro {
                erro,
                status_upstream,
            }) => (erro.estado_consulta(), None, status_upstream, Err(erro)),
        };

        let registro = RegistroConsultaFiscal {
            escritorio_id,
            cliente_id,
            produto: ProdutoSerpro::DividaAtiva.as_str(),
            servico: servico.as_str(),
            verbo: Some("GET"),
            ambiente: ambiente.as_str(),
            chave_consulta_hash: Some(hash),
            chave_consulta_mascarada: Some(mascarada),
            fonte_consulta: fonte.map(|fonte| fonte.as_str()),
            status_code,
            estado: estado.as_str(),
            protocolo: None,
            request_redigido: Some(request_redigido),
            response_redigido: redigido,
        };
        self.repositorio
            .registrar(registro)
            .map_err(|_| ErroConsultaDivida::PersistenciaIndisponivel)?;

        rumo
    }
}

#[derive(Clone)]
pub struct ConsultarDevedorUseCase {
    executante: Executante,
    catalogo_clientes: Option<Arc<dyn CatalogoClientesPorDocumento>>,
    fallback: Option<Arc<dyn ResolvedorChaveAlternativa>>,
}

impl ConsultarDevedorUseCase {
    pub fn novo(
        endpoints: Arc<dyn ResolverEndpointsSerpro>,
        http: Arc<dyn ConsultaDividaAtivaHttp>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        limitador: Arc<dyn LimitadorConsultas>,
    ) -> Self {
        Self {
            executante: Executante {
                endpoints,
                http,
                repositorio,
                limitador,
                catalogo_operadores: None,
                operator_header_enabled: false,
            },
            catalogo_clientes: None,
            fallback: None,
        }
    }

    pub fn com_catalogo_clientes(
        mut self,
        catalogo: Arc<dyn CatalogoClientesPorDocumento>,
    ) -> Self {
        self.catalogo_clientes = Some(catalogo);
        self
    }

    /// Fallback D2: só surte efeito quando `resolver` devolver
    /// código; o caller deve montar a porta condicionada à flag
    /// `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE`.
    pub fn com_fallback(mut self, fallback: Arc<dyn ResolvedorChaveAlternativa>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn com_operadores(mut self, catalogo: Arc<dyn CatalogoOperadores>, enabled: bool) -> Self {
        self.executante.catalogo_operadores = Some(catalogo);
        self.executante.operator_header_enabled = enabled;
        self
    }

    pub async fn executar(
        &self,
        comando: ConsultaDevedorCommand,
    ) -> Result<ConsultaDevedorResultado, ErroConsultaDivida> {
        let documento = CpfCnpj::novo(comando.cpf_cnpj.trim())
            .map_err(|_| ErroConsultaDivida::ParametroInvalido("cpfCnpj"))?;
        self.executante
            .limitador
            .adquirir(comando.escritorio_id)
            .map_err(|_| ErroConsultaDivida::LimiteLocalExcedido)?;
        let endpoint = self.executante.resolver_endpoint(comando.ambiente).await?;

        let cliente_id = match &self.catalogo_clientes {
            Some(catalogo) => catalogo
                .buscar_por_documento(comando.escritorio_id, documento.como_str())
                .map_err(|_| ErroConsultaDivida::PersistenciaIndisponivel)?,
            None => None,
        };
        let cpf_usuario = self.executante.cpf_usuario(comando.usuario_id).await?;

        let tipo = TipoChave::CpfCnpj(documento.clone());
        let chave = documento.como_str();
        let primeira = self
            .executante
            .tentativa(
                &endpoint,
                ServicoDividaSerpro::Devedor,
                &tipo,
                chave,
                Some(FonteConsulta::CpfCnpj),
                comando.ambiente,
                comando.escritorio_id,
                cliente_id,
                cpf_usuario.clone(),
                interpretar_devedor,
            )
            .await;

        let (resultado, fonte, tentativas) = match primeira {
            Ok(rumo) => (rumo, FonteConsulta::CpfCnpj, 1u8),
            Err(ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado)) => {
                let segundo = self
                    .tentativa_fallback(
                        &endpoint,
                        &comando,
                        documento.como_str(),
                        cliente_id,
                        cpf_usuario,
                    )
                    .await;
                match segundo {
                    Ok(rumo) => (rumo, FonteConsulta::CodigoContribuinte, 2u8),
                    Err(erro) => return Err(erro),
                }
            }
            Err(erro) => return Err(erro),
        };

        let (estado, inscricoes) = match resultado {
            ResultadoDevedor::Pendencias { inscricoes } => (EstadoConsulta::Sucesso, inscricoes),
            ResultadoDevedor::SemPendencias => (EstadoConsulta::SemPendencias, Vec::new()),
        };
        Ok(ConsultaDevedorResultado {
            ambiente: comando.ambiente,
            estado,
            fonte,
            chave_mascarada: mascarar_chave(&tipo, chave),
            inscricoes,
            horario_consulta: horario_consulta(chrono::Utc::now()),
            tentativas,
        })
    }

    /// Segunda tentativa por Código de Contribuinte (D2): exige
    /// código saneável do catálogo interno; caso contrário o 404
    /// original é mantido como desfecho.
    async fn tentativa_fallback(
        &self,
        endpoint: &EndpointSerpro,
        comando: &ConsultaDevedorCommand,
        documento_digitos: &str,
        cliente_id: Option<Uuid>,
        cpf_usuario: Option<String>,
    ) -> Result<ResultadoDevedor, ErroConsultaDivida> {
        let Some(fallback) = &self.fallback else {
            return Err(ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado));
        };
        let codigo = fallback
            .resolver(comando.escritorio_id, cliente_id, documento_digitos)
            .await
            .map_err(|_| ErroConsultaDivida::PersistenciaIndisponivel)?
            .map(|bruto| bruto.trim().to_owned())
            .filter(|codigo| !codigo.is_empty() && codigo.chars().count() <= 40);
        let Some(codigo) = codigo else {
            return Err(ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado));
        };

        let tipo = TipoChave::CodigoContribuinte(codigo.clone());
        self.executante
            .tentativa(
                endpoint,
                ServicoDividaSerpro::Devedor,
                &tipo,
                &codigo,
                Some(FonteConsulta::CodigoContribuinte),
                comando.ambiente,
                comando.escritorio_id,
                cliente_id,
                cpf_usuario,
                interpretar_devedor,
            )
            .await
    }
}

#[derive(Clone)]
pub struct ConsultarInscricaoUseCase {
    executante: Executante,
}

impl ConsultarInscricaoUseCase {
    pub fn novo(
        endpoints: Arc<dyn ResolverEndpointsSerpro>,
        http: Arc<dyn ConsultaDividaAtivaHttp>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
        limitador: Arc<dyn LimitadorConsultas>,
    ) -> Self {
        Self {
            executante: Executante {
                endpoints,
                http,
                repositorio,
                limitador,
                catalogo_operadores: None,
                operator_header_enabled: false,
            },
        }
    }

    pub fn com_operadores(mut self, catalogo: Arc<dyn CatalogoOperadores>, enabled: bool) -> Self {
        self.executante.catalogo_operadores = Some(catalogo);
        self.executante.operator_header_enabled = enabled;
        self
    }

    pub async fn executar(
        &self,
        comando: ConsultaInscricaoCommand,
    ) -> Result<ConsultaInscricaoResultado, ErroConsultaDivida> {
        let inscricao = NumeroInscricao::novo(&comando.numero_inscricao)
            .map_err(|_| ErroConsultaDivida::ParametroInvalido("numeroInscricao"))?;
        self.executante
            .limitador
            .adquirir(comando.escritorio_id)
            .map_err(|_| ErroConsultaDivida::LimiteLocalExcedido)?;
        let endpoint = self.executante.resolver_endpoint(comando.ambiente).await?;
        let cpf_usuario = self.executante.cpf_usuario(comando.usuario_id).await?;

        let tipo = TipoChave::Inscricao(inscricao.clone());
        let detalhe = self
            .executante
            .tentativa(
                &endpoint,
                ServicoDividaSerpro::Inscricao,
                &tipo,
                inscricao.como_str(),
                None,
                comando.ambiente,
                comando.escritorio_id,
                None,
                cpf_usuario,
                interpretar_inscricao,
            )
            .await?;

        Ok(ConsultaInscricaoResultado {
            ambiente: comando.ambiente,
            estado: EstadoConsulta::Sucesso,
            chave_mascarada: mascarar_chave(
                &TipoChave::Inscricao(inscricao.clone()),
                inscricao.como_str(),
            ),
            detalhe,
            horario_consulta: horario_consulta(chrono::Utc::now()),
        })
    }
}

/// Interpretação de `/devedor/{cpfCnpj}`: corpo vazio inequívoco
/// (`null`, array vazio, objeto plano nulo) é SEM_PENDENCIAS (SPEC
/// §4.4 passo 7); itens são saneados pelo domínio — moeda fora do
/// padrão vira RESPOSTA_INVALIDA.
fn interpretar_devedor(corpo: Value) -> Interpretacao<ResultadoDevedor> {
    if corpo.is_null() {
        return Ok((
            EstadoConsulta::SemPendencias,
            Some(json!({"quantidade": 0})),
            ResultadoDevedor::SemPendencias,
        ));
    }
    let resposta: DevedorResponse = match serde_json::from_value(corpo) {
        Ok(resposta) => resposta,
        Err(_) => {
            return Err((
                ErroConsultaDivida::RespostaInvalida(
                    "corpo de /devedor fora do contrato".to_owned(),
                ),
                Some(200),
            ));
        }
    };
    let payloads: Vec<DevedorPayload> = match resposta {
        DevedorResponse::Lista(itens) => itens,
        DevedorResponse::Item(item) => vec![*item],
    };
    if payloads.iter().all(DevedorPayload::e_vazio) {
        return Ok((
            EstadoConsulta::SemPendencias,
            Some(json!({"quantidade": 0})),
            ResultadoDevedor::SemPendencias,
        ));
    }

    let mut inscricoes = Vec::with_capacity(payloads.len());
    for payload in payloads {
        match InscricaoDevedor::nova(
            payload.numero_inscricao,
            payload.numero_processo,
            payload.situacao_inscricao,
            payload.situacao_descricao,
            payload.nome_devedor,
            payload.tipo_devedor,
            payload.valor_total_consolidado_moeda,
            payload.cpf_cnpj,
            payload.codigo_sida,
            payload.nome_unidade,
            payload.codigo_comprot,
            payload.codigo_uorg,
        ) {
            Ok(item) => inscricoes.push(item),
            Err(erro) => return Err((erro, Some(200))),
        }
    }
    let quantidade = inscricoes.len();
    Ok((
        EstadoConsulta::Sucesso,
        Some(json!({"quantidade": quantidade})),
        ResultadoDevedor::Pendencias { inscricoes },
    ))
}

/// Interpretação de `/inscricao/{numeroInscricao}`: apenas objeto
/// único (ou array de um elemento) é contrato válido; vazio/outros
/// formatos são RESPOSTA_INVALIDA (ausência real upstream vem como
/// 404, já mapeado pelo transporte).
fn interpretar_inscricao(corpo: Value) -> Interpretacao<InscricaoDetalhe> {
    if corpo.is_null() {
        return Err((
            ErroConsultaDivida::RespostaInvalida("corpo de /inscricao vazio".to_owned()),
            Some(200),
        ));
    }
    let resposta: InscricaoResponse = match serde_json::from_value(corpo) {
        Ok(resposta) => resposta,
        Err(_) => {
            return Err((
                ErroConsultaDivida::RespostaInvalida(
                    "corpo de /inscricao fora do contrato".to_owned(),
                ),
                Some(200),
            ));
        }
    };
    let payload: InscricaoDetalhePayload = match resposta {
        InscricaoResponse::Lista(itens) if itens.len() == 1 => {
            itens.into_iter().next().expect("único")
        }
        InscricaoResponse::Lista(_) => {
            return Err((
                ErroConsultaDivida::RespostaInvalida(
                    "número inesperado de detalhes de inscrição".to_owned(),
                ),
                Some(200),
            ));
        }
        InscricaoResponse::Item(item) => *item,
    };

    let inscricao_bruta = payload.inscricao.clone().unwrap_or_default();
    let inscricao_digitos = inscricao_bruta
        .chars()
        .filter(|caractere| caractere.is_ascii_digit())
        .collect::<String>();
    let detalhe = match InscricaoDetalhe::nova(
        inscricao_bruta,
        inscricao_digitos,
        payload.processo_administrativo,
        payload.situacao,
        payload.data_inscricao,
        payload.numero_pfn_responsavel,
        payload.nome_pfn_responsavel,
        payload.numero_pfn_inscricao,
        payload.nome_pfn_inscricao,
        payload.codigo_receita_principal,
        payload.nome_receita,
        payload.indicador_prescricao_sv8,
        payload.valor_total_inscrito_moeda,
        payload.valor_total_consolidado_moeda,
        payload.valor_remanescente_moeda,
        payload.nome_devedor,
        payload.tipo_devedor,
        payload.nome_unidade,
        payload.codigo_sistema_origem,
        payload.descricao_sistema_origem,
    ) {
        Ok(detalhe) => detalhe,
        Err(erro) => return Err((erro, Some(200))),
    };

    Ok((
        EstadoConsulta::Sucesso,
        Some(json!({"quantidade": 1})),
        detalhe,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use secrecy::SecretString;
    use url::Url;

    use super::*;
    use crate::application::serpro::ports::RepositorioError;
    use crate::domain::serpro::endpoints::EstrategiaAutenticacaoSerpro;
    use crate::domain::serpro::erros::ErroEndpoint;

    // -------------------- mocks --------------------

    struct HttpMock {
        pilha: Mutex<VecDeque<Result<Value, ErroHttpSerpro>>>,
        solicitacoes: Mutex<Vec<SolicitacaoSerproHttp>>,
    }

    impl HttpMock {
        fn com_corpos(corpos: Vec<Result<Value, ErroHttpSerpro>>) -> Arc<Self> {
            Arc::new(Self {
                pilha: Mutex::new(corpos.into()),
                solicitacoes: Mutex::new(Vec::new()),
            })
        }
    }

    #[async_trait]
    impl ConsultaDividaAtivaHttp for HttpMock {
        async fn consultar(
            &self,
            solicitacao: SolicitacaoSerproHttp,
        ) -> Result<Value, ErroHttpSerpro> {
            self.solicitacoes
                .lock()
                .expect("solicitações")
                .push(solicitacao);
            self.pilha
                .lock()
                .expect("pilha")
                .pop_front()
                .expect("corpo programado para a tentativa")
        }
    }

    struct TrilhaMock {
        linhas: Mutex<Vec<RegistroConsultaFiscal>>,
    }

    impl TrilhaMock {
        fn novo() -> Arc<Self> {
            Arc::new(Self {
                linhas: Mutex::new(Vec::new()),
            })
        }
    }

    impl RepositorioConsultasFiscais for TrilhaMock {
        fn registrar(&self, registro: RegistroConsultaFiscal) -> Result<Uuid, RepositorioError> {
            self.linhas.lock().expect("linhas").push(registro);
            Ok(Uuid::nil())
        }

        fn expurgar_antes_de(
            &self,
            _limite: chrono::DateTime<chrono::Utc>,
        ) -> Result<u64, RepositorioError> {
            Ok(0)
        }
    }

    struct EndpointsMock {
        resultado: Result<EndpointSerpro, ErroEndpoint>,
    }

    #[async_trait]
    impl ResolverEndpointsSerpro for EndpointsMock {
        async fn resolver(
            &self,
            _produto: ProdutoSerpro,
            _ambiente: crate::domain::serpro::ambiente::AmbienteSerpro,
        ) -> Result<EndpointSerpro, ErroEndpoint> {
            match &self.resultado {
                Ok(_) => Ok(endpoint()),
                Err(erro) => Err(erro.clone()),
            }
        }
    }

    struct LimitadorMock {
        permitir: bool,
    }

    impl LimitadorConsultas for LimitadorMock {
        fn adquirir(&self, _escritorio_id: Uuid) -> Result<(), ()> {
            if self.permitir { Ok(()) } else { Err(()) }
        }
    }

    struct ClientesMock {
        cliente_id: Uuid,
    }

    impl CatalogoClientesPorDocumento for ClientesMock {
        fn buscar_por_documento(
            &self,
            _escritorio_id: Uuid,
            _documento_digitos: &str,
        ) -> Result<Option<Uuid>, RepositorioError> {
            Ok(Some(self.cliente_id))
        }
    }

    struct OperadoresMock;

    impl CatalogoOperadores for OperadoresMock {
        fn buscar_cpf(&self, _usuario_id: Uuid) -> Result<Option<String>, RepositorioError> {
            Ok(Some("11122233344".to_owned()))
        }
    }

    struct FallbackMock {
        codigo: Option<String>,
    }

    #[async_trait]
    impl ResolvedorChaveAlternativa for FallbackMock {
        async fn resolver(
            &self,
            _escritorio_id: Uuid,
            _cliente_id: Option<Uuid>,
            _cpf_cnpj_digitos: &str,
        ) -> Result<Option<String>, RepositorioError> {
            Ok(self.codigo.clone())
        }
    }

    fn endpoint() -> EndpointSerpro {
        EndpointSerpro {
            produto: ProdutoSerpro::DividaAtiva,
            ambiente: crate::domain::serpro::ambiente::AmbienteSerpro::Trial,
            url_base: Url::parse("https://mock.invalid/consulta-divida-ativa-trial/api/v1")
                .expect("url do mock"),
            autenticacao: EstrategiaAutenticacaoSerpro::BearerFixo(SecretString::from(
                "token-publico-de-demo",
            )),
        }
    }

    fn use_case_devedor(
        http: Arc<HttpMock>,
        trilha: Arc<TrilhaMock>,
        limitador: Arc<LimitadorMock>,
    ) -> ConsultarDevedorUseCase {
        ConsultarDevedorUseCase::novo(
            Arc::new(EndpointsMock {
                resultado: Ok(endpoint()),
            }),
            http.clone(),
            trilha.clone(),
            limitador.clone(),
        )
    }

    fn comando_devedor() -> ConsultaDevedorCommand {
        ConsultaDevedorCommand {
            escritorio_id: Uuid::nil(),
            usuario_id: Uuid::nil(),
            ambiente: crate::domain::serpro::ambiente::AmbienteSerpro::Trial,
            cpf_cnpj: "529.982.247-25".to_owned(),
        }
    }

    fn erro_http(erro: ErroConsultaDivida, status: Option<i32>) -> Result<Value, ErroHttpSerpro> {
        Err(ErroHttpSerpro {
            erro,
            status_upstream: status,
        })
    }

    fn corpo_devedor_amostra() -> Value {
        json!([
            {
                "numero_inscricao": "10 5 03 003880-14",
                "numero_processo": "10166 677771/2014-12",
                "situacao_inscricao": "121105",
                "situacao_descricao": "ATIVA NAO PRIORIZADA PARA AJUIZAMENTO",
                "nome_devedor": "DEVEDOR DA SILVA",
                "tipo_devedor": "PRINCIPAL",
                "valor_total_consolidado_moeda": "4.457,56",
                "cpf_cnpj": "529.982.247-25",
                "codigo_sida": "1000",
                "nome_unidade": "PROCURADORIA REGIONAL DA FAZENDA NACIONAL DA 1A REGIAO",
                "codigo_comprot": "1101951",
                "codigo_uorg": "0005786"
            }
        ])
    }

    // -------------------- devedor --------------------

    #[tokio::test]
    async fn devedor_con_inscricoes_responde_sucesso_e_audita() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(corpo_devedor_amostra())]);
        let use_case = use_case_devedor(
            http.clone(),
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        )
        .com_catalogo_clientes(Arc::new(ClientesMock {
            cliente_id: Uuid::nil(),
        }));

        let resultado = use_case.executar(comando_devedor()).await.expect("sucesso");

        assert_eq!(resultado.estado, EstadoConsulta::Sucesso);
        assert_eq!(resultado.fonte, FonteConsulta::CpfCnpj);
        assert_eq!(resultado.inscricoes.len(), 1);
        assert_eq!(
            resultado.inscricoes[0].numero_inscricao.as_deref(),
            Some("10 5 03 003880-14")
        );
        assert_eq!(
            resultado.inscricoes[0].valor_total_consolidado,
            Some(Decimal::new(445756, 2))
        );
        assert_eq!(resultado.chave_mascarada, "***.***.247-25");
        assert_eq!(resultado.tentativas, 1);

        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas.len(), 1);
        let linha = &linhas[0];
        assert_eq!(linha.produto, "DIVIDA_ATIVA");
        assert_eq!(linha.servico, "DEVEDOR");
        assert_eq!(linha.verbo, Some("GET"));
        assert_eq!(linha.ambiente, "TRIAL");
        assert_eq!(linha.fonte_consulta, Some("CPF_CNPJ"));
        assert_eq!(linha.status_code, Some(200));
        assert_eq!(linha.estado, "SUCESSO");
        assert!(linha.cliente_id.is_some());
        let redigido = linha.request_redigido.as_ref().expect("request redigido");
        assert_eq!(redigido["chaveMascarada"], "***.***.247-25");
        assert!(linha.chave_consulta_hash.as_deref().map(str::len) == Some(64));
        let resposta = linha.response_redigido.as_ref().expect("response redigido");
        assert_eq!(resposta["quantidade"], 1);

        let solicitacoes = http.solicitacoes.lock().expect("solicitações");
        assert_eq!(solicitacoes.len(), 1);
        assert_eq!(solicitacoes[0].chave_normalizada, "52998224725");
        assert!(solicitacoes[0].cpf_usuario.is_none());
    }

    #[tokio::test]
    async fn devedor_corpo_vazio_fica_sem_pendencias() {
        for corpo in [Value::Null, json!([]), json!({})] {
            let trilha = TrilhaMock::novo();
            let http = HttpMock::com_corpos(vec![Ok(corpo)]);
            let use_case = use_case_devedor(
                http,
                trilha.clone(),
                Arc::new(LimitadorMock { permitir: true }),
            );

            let resultado = use_case
                .executar(comando_devedor())
                .await
                .expect("consultado");
            assert_eq!(resultado.estado, EstadoConsulta::SemPendencias);
            assert!(resultado.inscricoes.is_empty());

            let linhas = trilha.linhas.lock().expect("linhas");
            assert_eq!(linhas.len(), 1);
            assert_eq!(linhas[0].estado, "SEM_PENDENCIAS");
            let resposta = linhas[0].response_redigido.as_ref().expect("response");
            assert_eq!(resposta["quantidade"], 0);
        }
    }

    #[tokio::test]
    async fn moeda_ilegivel_vira_resposta_invalida() {
        let corpo = json!([{ "valor_total_consolidado_moeda": "preço??", "nome_devedor": "X" }]);
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(corpo)]);
        let use_case = use_case_devedor(
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        );

        let erro = use_case
            .executar(comando_devedor())
            .await
            .expect_err("erro de contrato");
        assert_eq!(erro.codigo(), "SERPRO_RESPOSTA_INVALIDA");
        assert_eq!(erro.status_http(), 502);

        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas[0].estado, "RESPOSTA_INVALIDA");
        assert_eq!(linhas[0].status_code, Some(200));
        assert!(linhas[0].response_redigido.is_none());
    }

    #[tokio::test]
    async fn quarenta_e_quatro_sem_fallback_propaga_e_audita() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![erro_http(
            ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado),
            Some(404),
        )]);
        let use_case = use_case_devedor(
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        );

        let erro = use_case.executar(comando_devedor()).await.expect_err("404");
        assert_eq!(erro.codigo(), "SERPRO_NAO_ENCONTRADO");
        assert_eq!(erro.status_http(), 404);

        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas.len(), 1);
        assert_eq!(linhas[0].estado, "NAO_ENCONTRADO");
        assert_eq!(linhas[0].status_code, Some(404));
    }

    #[tokio::test]
    async fn fallback_busca_por_codigo_contribuinte() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![
            erro_http(
                ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado),
                Some(404),
            ),
            Ok(corpo_devedor_amostra()),
        ]);
        let use_case = use_case_devedor(
            http.clone(),
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        )
        .com_fallback(Arc::new(FallbackMock {
            codigo: Some("  ABC12345  ".to_owned()),
        }));

        let resultado = use_case
            .executar(comando_devedor())
            .await
            .expect("sucesso na 2a");
        assert_eq!(resultado.estado, EstadoConsulta::Sucesso);
        assert_eq!(resultado.fonte, FonteConsulta::CodigoContribuinte);
        assert_eq!(resultado.tentativas, 2);

        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas.len(), 2);
        assert_eq!(linhas[0].estado, "NAO_ENCONTRADO");
        assert_eq!(linhas[0].fonte_consulta, Some("CPF_CNPJ"));
        assert_eq!(linhas[1].estado, "SUCESSO");
        assert_eq!(linhas[1].fonte_consulta, Some("CODIGO_CONTRIBUINTE"));
        assert_eq!(
            linhas[1].chave_consulta_mascarada.as_deref(),
            Some("AB****45")
        );

        let solicitacoes = http.solicitacoes.lock().expect("solicitações");
        assert_eq!(solicitacoes.len(), 2);
        assert_eq!(solicitacoes[1].chave_normalizada, "ABC12345");
    }

    #[tokio::test]
    async fn fallback_sem_codigo_permanece_nao_encontrado() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![erro_http(
            ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado),
            Some(404),
        )]);
        let use_case = use_case_devedor(
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        )
        .com_fallback(Arc::new(FallbackMock { codigo: None }));

        let erro = use_case.executar(comando_devedor()).await.expect_err("404");
        assert_eq!(erro.codigo(), "SERPRO_NAO_ENCONTRADO");
        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas.len(), 1);
    }

    #[tokio::test]
    async fn teto_de_429_ecoa_retry_after() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![erro_http(
            ErroConsultaDivida::Serpro(ErroSerpro::LimiteExcedido {
                retry_after: Some(7),
            }),
            Some(429),
        )]);
        let use_case = use_case_devedor(
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        );

        let erro = use_case.executar(comando_devedor()).await.expect_err("429");
        assert_eq!(erro.codigo(), "SERPRO_LIMITE_EXCEDIDO");
        assert_eq!(erro.retry_after_segundos(), Some(7));
        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas[0].estado, "LIMITE_EXCEDIDO");
    }

    #[tokio::test]
    async fn limite_local_falha_rapido_e_nao_registra() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![]);
        let use_case = use_case_devedor(
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: false }),
        );

        let erro = use_case
            .executar(comando_devedor())
            .await
            .expect_err("429 local");
        assert_eq!(erro.codigo(), "LIMITE_REQUISICOES");
        assert!(trilha.linhas.lock().expect("linhas").is_empty());
    }

    #[tokio::test]
    async fn producao_bloqueada_nao_audita_e_responde_503() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![]);
        let use_case = ConsultarDevedorUseCase::novo(
            Arc::new(EndpointsMock {
                resultado: Err(ErroEndpoint::DividaAtivaProducaoBloqueada),
            }),
            http,
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        );
        let mut comando = comando_devedor();
        comando.ambiente = crate::domain::serpro::ambiente::AmbienteSerpro::Producao;

        let erro = use_case.executar(comando).await.expect_err("bloqueado");
        assert_eq!(erro.codigo(), "SERPRO_PROD_NOT_CONFIGURED");
        assert_eq!(erro.status_http(), 503);
        assert!(trilha.linhas.lock().expect("linhas").is_empty());
    }

    #[tokio::test]
    async fn operador_acionado_somente_quando_habilitado() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(corpo_devedor_amostra())]);
        let use_case = use_case_devedor(
            http.clone(),
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        )
        .com_operadores(Arc::new(OperadoresMock), true);

        use_case.executar(comando_devedor()).await.expect("sucesso");
        let solicitacoes = http.solicitacoes.lock().expect("solicitações");
        assert_eq!(solicitacoes[0].cpf_usuario.as_deref(), Some("11122233344"));
    }

    #[tokio::test]
    async fn operador_desabilitado_nao_injeta_header() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(corpo_devedor_amostra())]);
        let use_case = use_case_devedor(
            http.clone(),
            trilha,
            Arc::new(LimitadorMock { permitir: true }),
        )
        .com_operadores(Arc::new(OperadoresMock), false);

        use_case.executar(comando_devedor()).await.expect("sucesso");
        let solicitacoes = http.solicitacoes.lock().expect("solicitações");
        assert!(solicitacoes[0].cpf_usuario.is_none());
    }

    // -------------------- inscrição --------------------

    fn use_case_inscricao(
        http: Arc<HttpMock>,
        trilha: Arc<TrilhaMock>,
    ) -> ConsultarInscricaoUseCase {
        ConsultarInscricaoUseCase::novo(
            Arc::new(EndpointsMock {
                resultado: Ok(endpoint()),
            }),
            http.clone(),
            trilha.clone(),
            Arc::new(LimitadorMock { permitir: true }),
        )
    }

    fn comando_inscricao() -> ConsultaInscricaoCommand {
        ConsultaInscricaoCommand {
            escritorio_id: Uuid::nil(),
            usuario_id: Uuid::nil(),
            ambiente: crate::domain::serpro::ambiente::AmbienteSerpro::Trial,
            numero_inscricao: "90 6 17 000964-19".to_owned(),
        }
    }

    fn corpo_detalhe_amostra() -> Value {
        json!({
            "inscricao": "90 6 17 000964-19",
            "processoAdministrativo": "46206 099222/99-01",
            "situacao": "ATIVA NAO PRIORIZADA PARA AJUIZAMENTO",
            "dataInscricao": "16/12/2003",
            "numeroPFNResponsavel": "1000",
            "nomePFNResponsavel": "PRIMEIRA REGIAO",
            "numeroPFNInscricao": "1000",
            "nomePFNInscricao": "PRIMEIRA REGIAO",
            "codigoReceitaPrincipal": "3623",
            "nomeReceita": "DIV.ATIVA-CLT",
            "indicadorPrescricaoSV8": "NAO",
            "valorTotalInscritoMoeda": "805,06",
            "valorTotalConsolidadoMoeda": "2.403,59",
            "valorRemanescenteMoeda": "805,06",
            "codigoSistemaOrigem": "M",
            "descricaoSistemaOrigem": "Manual"
        })
    }

    #[tokio::test]
    async fn inscricao_detalhe_da_amostra_oficial() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(corpo_detalhe_amostra())]);
        let use_case = use_case_inscricao(http, trilha.clone());

        let resultado = use_case
            .executar(comando_inscricao())
            .await
            .expect("detalhe");
        assert_eq!(resultado.estado, EstadoConsulta::Sucesso);
        assert_eq!(resultado.chave_mascarada, "906*******419");
        assert_eq!(resultado.detalhe.prescrito, Some(false));
        assert_eq!(
            resultado.detalhe.valor_inscrito,
            Some(Decimal::new(80506, 2))
        );
        assert_eq!(
            resultado.detalhe.valor_consolidado,
            Some(Decimal::new(240359, 2))
        );
        let unidade = resultado.detalhe.unidade_responsavel.expect("unidade");
        assert_eq!(unidade.numero, "1000");

        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas.len(), 1);
        assert_eq!(linhas[0].servico, "INSCRICAO");
        assert_eq!(linhas[0].fonte_consulta, None);
        assert_eq!(linhas[0].estado, "SUCESSO");
        assert!(linhas[0].cliente_id.is_none());
        let resposta = linhas[0].response_redigido.as_ref().expect("response");
        assert_eq!(resposta["quantidade"], 1);
    }

    #[tokio::test]
    async fn inscricao_aceita_array_de_um_elemento() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![Ok(json!([corpo_detalhe_amostra()]))]);
        let use_case = use_case_inscricao(http, trilha);

        let resultado = use_case
            .executar(comando_inscricao())
            .await
            .expect("detalhe");
        assert_eq!(resultado.estado, EstadoConsulta::Sucesso);
    }

    #[tokio::test]
    async fn inscricao_404_propaga_e_audita() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![erro_http(
            ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado),
            Some(404),
        )]);
        let use_case = use_case_inscricao(http, trilha.clone());

        let erro = use_case
            .executar(comando_inscricao())
            .await
            .expect_err("404");
        assert_eq!(erro.codigo(), "SERPRO_NAO_ENCONTRADO");
        let linhas = trilha.linhas.lock().expect("linhas");
        assert_eq!(linhas[0].estado, "NAO_ENCONTRADO");
    }

    #[tokio::test]
    async fn inscricao_invalida_erro_400_sem_tocar_rede() {
        let trilha = TrilhaMock::novo();
        let http = HttpMock::com_corpos(vec![]);
        let use_case = use_case_inscricao(http, trilha);
        let mut comando = comando_inscricao();
        comando.numero_inscricao = "12 ".to_owned();

        let erro = use_case.executar(comando).await.expect_err("inválido");
        assert_eq!(erro.codigo(), "PARAMETRO_INVALIDO");
        assert_eq!(erro.status_http(), 400);
    }

    #[tokio::test]
    async fn falha_de_persistencia_e_fail_closed() {
        struct TrilhaQuebrada;
        impl RepositorioConsultasFiscais for TrilhaQuebrada {
            fn registrar(
                &self,
                _registro: RegistroConsultaFiscal,
            ) -> Result<Uuid, RepositorioError> {
                Err(RepositorioError::Indisponivel)
            }
            fn expurgar_antes_de(
                &self,
                _limite: chrono::DateTime<chrono::Utc>,
            ) -> Result<u64, RepositorioError> {
                Err(RepositorioError::Indisponivel)
            }
        }
        let http = HttpMock::com_corpos(vec![Ok(corpo_devedor_amostra())]);
        let use_case = ConsultarDevedorUseCase::novo(
            Arc::new(EndpointsMock {
                resultado: Ok(endpoint()),
            }),
            http,
            Arc::new(TrilhaQuebrada),
            Arc::new(LimitadorMock { permitir: true }),
        );

        let erro = use_case
            .executar(comando_devedor())
            .await
            .expect_err("fail closed");
        assert_eq!(erro.codigo(), "ERRO_INTERNO");
    }
}
