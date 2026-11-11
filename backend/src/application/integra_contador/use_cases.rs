use std::sync::Arc;
use std::time::Instant;

use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::application::certificados::CertificadoCriptografia;
use crate::application::clientes::ClienteRepositorio;
use crate::application::integra_contador::ports::{
    AuditoriaIntegraContador, AutenticaProcuradorCliente, CatalogoEscritoriosIntegra,
    CertificadoA1Provider, ClienteIntegraContador, ContextoIntegraContador, ContratantePlataforma,
    EntregaIntegraContador, EventoAuditoriaIntegra, EventoChamadaIntegra, LinhaCatalogoServico,
    NovoRegistroProcuracao, ProvedorCatalogoIntegra, RegistroProcuracao, RepositorioIdempotencia,
    RepositorioProcuracoes, ReservaIdempotencia, ResultadoReservaIdempotencia, ServicoResolvido,
    TokensSerproRepositorio, ValidadorProcuracao,
};
use crate::domain::integra_contador::assinatura::{
    AssinadorXml, AtoresTermoAutorizacao, TermoAutorizacao,
};
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, PedidoDados, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::integra_contador::servicos::{
    EstadoCatalogo, bloqueado_por_verbo, resolver_servico,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::{
    ComandoRegistrarChamada, OrigemChamada, ResultadoChamada, ServicoMedicaoSerpro,
};
use crate::domain::serpro::produto::ProdutoSerpro;

#[derive(Debug, Clone)]
pub struct ComandoIntegraContador {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub documento_contribuinte: String,
    pub verbo: VerboIntegraContador,
    pub pedido_dados: PedidoDados,
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
    pub ambiente: AmbienteSerpro,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoIntegraContador {
    pub ambiente: AmbienteSerpro,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub estado: String,
    pub status: u16,
    pub mensagem: &'static str,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
    pub codigos_mensagem: Vec<String>,
    /// Campo `dados` tolerado da resposta SERPRO (string JSON) Ã¢â‚¬â€ fonte do
    /// `payload_redigido` persistido em `procuracoes` (FASE A, P7).
    pub dados_resposta: Option<String>,
}

#[derive(Clone)]
pub struct ExecutarIntegraContadorUseCase {
    contexto: Arc<dyn CatalogoEscritoriosIntegra>,
    cliente: Arc<dyn ClienteIntegraContador>,
    idempotencia: Arc<dyn RepositorioIdempotencia>,
    auditoria: Arc<dyn AuditoriaIntegraContador>,
    procuracoes: Arc<dyn ValidadorProcuracao>,
    operator_header_enabled: bool,
    /// Servico de medicao de uso SERPRO — registra toda chamada de saida
    /// para fins de cobranca, distinguindo TRIAL de PRODUCAO.
    servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    /// Origem da chamada registrada na medição (Fase B.1): consultas
    /// manuais, rotinas agendadas e lotes são distinguidos na cobrança.
    origem: OrigemChamada,
    /// CatÃƒÂ¡logo configurÃƒÂ¡vel (`catalogo_servicos_integra`, ADR-0013).
    /// `None` = catÃƒÂ¡logo estÃƒÂ¡tico do domÃƒÂ­nio (comportamento anterior).
    /// ResoluÃƒÂ§ÃƒÂ£o ÃƒÂ© banco-primeiro; queda no estÃƒÂ¡tico quando nÃƒÂ£o configurado
    /// ou sem linha viÃƒÂ¡vel.
    provedor_catalogo: Option<Arc<dyn ProvedorCatalogoIntegra>>,
}

impl ExecutarIntegraContadorUseCase {
    pub fn novo(
        contexto: Arc<dyn CatalogoEscritoriosIntegra>,
        cliente: Arc<dyn ClienteIntegraContador>,
        idempotencia: Arc<dyn RepositorioIdempotencia>,
        auditoria: Arc<dyn AuditoriaIntegraContador>,
        procuracoes: Arc<dyn ValidadorProcuracao>,
        operator_header_enabled: bool,
        servico_medicao: Arc<dyn ServicoMedicaoSerpro>,
    ) -> Self {
        Self {
            contexto,
            cliente,
            idempotencia,
            auditoria,
            procuracoes,
            operator_header_enabled,
            servico_medicao,
            origem: OrigemChamada::Manual,
            provedor_catalogo: None,
        }
    }

    /// Define a origem da chamada na medição de uso (Fase B.1).
    #[must_use]
    pub const fn com_origem(mut self, origem: OrigemChamada) -> Self {
        self.origem = origem;
        self
    }

    /// Liga o catÃƒÂ¡logo configurÃƒÂ¡vel ao caso de uso (ADR-0013). `None`
    /// preserva o comportamento do catÃƒÂ¡logo estÃƒÂ¡tico (testes antigos).
    #[must_use]
    pub fn com_catalogo(
        mut self,
        provedor: Option<std::sync::Arc<dyn ProvedorCatalogoIntegra>>,
    ) -> Self {
        self.provedor_catalogo = provedor;
        self
    }

    /// ADR-0013: resoluÃƒÂ§ÃƒÂ£o do serviÃƒÂ§o. Com catÃƒÂ¡logo configurÃƒÂ¡vel ligado, o
    /// banco ÃƒÂ© autoritativo Ã¢â‚¬â€ inclusive para recusar, sem queda no estÃƒÂ¡tico
    /// (um admin que desliga o serviÃƒÂ§o nÃƒÂ£o pode ser burlado pelo catÃƒÂ¡logo
    /// de referÃƒÂªncia). Sem catÃƒÂ¡logo configurado (testes e contextos
    /// antigos), aplica o catÃƒÂ¡logo estÃƒÂ¡tico do domÃƒÂ­nio. Em qualquer
    /// caminho o bloqueio por verbo (allowlist D4 + ADR-0013) aplica-se
    /// antes de olhar banco.
    async fn avaliar_servico(
        &self,
        pedido: &PedidoDados,
        verbo: VerboIntegraContador,
    ) -> Result<ServicoResolvido, ErroIntegraContador> {
        if bloqueado_por_verbo(&pedido.id_sistema, &pedido.id_servico, verbo) {
            return Err(ErroIntegraContador::ServicoNaoHabilitado);
        }

        if let Some(catalogo) = &self.provedor_catalogo {
            return catalogo
                .resolver(
                    &pedido.id_sistema,
                    &pedido.id_servico,
                    &pedido.versao_sistema,
                    verbo,
                )
                .await;
        }

        let servico = resolver_servico(
            &pedido.id_sistema,
            &pedido.id_servico,
            &pedido.versao_sistema,
            verbo,
        )?;
        if servico.estado != EstadoCatalogo::Habilitado {
            return Err(ErroIntegraContador::ServicoNaoHabilitado);
        }

        Ok(ServicoResolvido {
            id_sistema: servico.id_sistema.to_owned(),
            id_servico: servico.id_servico.to_owned(),
            versao_sistema: servico
                .versao_sistema
                .unwrap_or(&pedido.versao_sistema)
                .to_owned(),
            apenas_consulta: false,
            requer_procuracao: servico.regra_procuracao.se_aplica(),
            codigos_procuracao: servico.regra_procuracao.codigos_possiveis(),
        })
    }

    pub async fn consultar_procuracoes(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: String,
        ambiente: AmbienteSerpro,
        request_id: String,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        let contexto = self
            .contexto
            .resolver_contexto(escritorio_id, usuario_id, &documento_contribuinte)
            .await?;
        let dados = serde_json::to_string(&serde_json::json!({
            "outorgante": contexto.contribuinte.numero,
            "tipoOutorgante": contexto.contribuinte.tipo.to_string(),
            "outorgado": contexto.autor_pedido_dados.numero,
            "tipoOutorgado": contexto.autor_pedido_dados.tipo.to_string(),
        }))
        .map_err(|_| ErroIntegraContador::PayloadInvalido)?;

        self.executar(ComandoIntegraContador {
            escritorio_id,
            usuario_id,
            documento_contribuinte,
            verbo: VerboIntegraContador::Consultar,
            pedido_dados: PedidoDados {
                id_sistema: "PROCURACOES".to_owned(),
                id_servico: "OBTERPROCURACAO41".to_owned(),
                versao_sistema: "1".to_owned(),
                dados,
            },
            confirmacao_explicita: false,
            chave_idempotencia: None,
            ambiente,
            request_id,
        })
        .await
    }

    pub async fn executar(
        &self,
        comando: ComandoIntegraContador,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        if comando.ambiente == AmbienteSerpro::Producao {
            return Err(ErroIntegraContador::ProducaoBloqueada);
        }

        if comando.verbo.exige_confirmacao() && !comando.confirmacao_explicita {
            return Err(ErroIntegraContador::ConfirmacaoNecessaria);
        }

        let servico = self
            .avaliar_servico(&comando.pedido_dados, comando.verbo)
            .await?;

        let mut contexto = self
            .contexto
            .resolver_contexto(
                comando.escritorio_id,
                comando.usuario_id,
                &comando.documento_contribuinte,
            )
            .await?;
        let payload = validar_dados(&comando.pedido_dados.dados)?;
        validar_payload_procuracoes(&comando.pedido_dados, &payload, &contexto)?;

        if servico.requer_procuracao && contexto.autor_pedido_dados != contexto.contribuinte {
            let codigos = servico.codigos_procuracao.clone();
            let mut autorizada = false;
            for codigo in codigos {
                autorizada |= self
                    .procuracoes
                    .validar(&contexto, &codigo, comando.ambiente)
                    .await?;
            }
            if !autorizada {
                return Err(ErroIntegraContador::ProcuracaoNegada);
            }
        }

        if !self.operator_header_enabled {
            contexto.cpf_operador = None;
        } else if contexto.cpf_operador.as_ref().is_some_and(|cpf| {
            !crate::domain::clientes::CpfCnpj::novo(cpf).is_ok_and(|documento| {
                documento.tipo() == crate::domain::clientes::TipoDocumento::Cpf
            })
        }) {
            return Err(ErroIntegraContador::ConfiguracaoIncompleta);
        }

        let payload_hash = hash_payload(&comando.pedido_dados.dados);
        let reserva = if comando.verbo.exige_confirmacao() {
            Some(criar_reserva(&comando, &payload_hash)?)
        } else {
            None
        };
        if let Some(reserva) = &reserva {
            match self.idempotencia.reservar(reserva).await? {
                ResultadoReservaIdempotencia::Nova => {}
                ResultadoReservaIdempotencia::Concluida { .. } => {
                    return Err(ErroIntegraContador::ConflitoIdempotencia);
                }
            }
            self.auditoria
                .registrar_mutacao(&EventoAuditoriaIntegra {
                    escritorio_id: comando.escritorio_id,
                    usuario_id: comando.usuario_id,
                    acao: "INTEGRA_MUTACAO_SOLICITADA",
                    verbo: comando.verbo,
                    id_sistema: servico.id_sistema.to_owned(),
                    id_servico: servico.id_servico.to_owned(),
                    ambiente: comando.ambiente,
                    estado: "EM_ANDAMENTO".to_owned(),
                    payload_hash: payload_hash.clone(),
                })
                .await?;
        }

        let request = IntegraContadorRequest {
            contratante: contexto.contratante.clone(),
            autor_pedido_dados: contexto.autor_pedido_dados.clone(),
            contribuinte: contexto.contribuinte.clone(),
            pedido_dados: comando.pedido_dados.clone(),
        };
        let inicio = Instant::now();
        let entrega = self
            .cliente
            .executar(
                comando.verbo,
                &request,
                comando.ambiente,
                contexto.cpf_operador.as_deref(),
                &comando.request_id,
            )
            .await;
        let duracao_ms = inicio.elapsed().as_millis() as u64;

        let resultado_medicao = |entrega: &EntregaIntegraContador| -> ResultadoChamada {
            match entrega.http_status {
                200..=299 => ResultadoChamada::Sucesso,
                400..=499 => ResultadoChamada::Erro4xx,
                504 => ResultadoChamada::Timeout,
                500..=599 => ResultadoChamada::Erro5xx,
                _ => ResultadoChamada::Indeterminado,
            }
        };

        match entrega {
            Ok(entrega) => {
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::IntegraContador,
                        verbo: comando.verbo.como_str().to_owned(),
                        servico: Some(servico.id_servico.clone()),
                        ambiente: comando.ambiente,
                        cliente_id: None,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: resultado_medicao(&entrega),
                        status_http: Some(entrega.http_status),
                        duracao_ms,
                        request_id: comando.request_id.clone(),
                    })
                    .await;

                self.finalizar_reserva(
                    reserva.as_ref(),
                    comando.verbo,
                    &entrega,
                    &payload_hash,
                    &comando,
                )
                .await?;
                self.auditoria
                    .registrar_chamada(&EventoChamadaIntegra {
                        escritorio_id: comando.escritorio_id,
                        usuario_id: comando.usuario_id,
                        verbo: comando.verbo,
                        id_sistema: servico.id_sistema.to_owned(),
                        id_servico: servico.id_servico.to_owned(),
                        ambiente: comando.ambiente,
                        estado: entrega.estado.clone(),
                        status_code: Some(i32::from(entrega.http_status)),
                        referencia_resposta: entrega.referencia_resposta.clone(),
                        request_id: comando.request_id,
                        bilhetavel: bilhetavel(comando.verbo, entrega.http_status),
                    })
                    .await?;
                Ok(mapear_resultado(
                    comando.ambiente,
                    comando.verbo,
                    &servico.id_sistema,
                    &servico.id_servico,
                    entrega,
                ))
            }
            Err(erro) => {
                let estado = if comando.verbo.exige_confirmacao() {
                    "INDETERMINADO"
                } else {
                    "FALHA_TRANSPORTE"
                };
                let _ = self
                    .servico_medicao
                    .registrar_chamada(&ComandoRegistrarChamada {
                        escritorio_id: comando.escritorio_id,
                        produto: ProdutoSerpro::IntegraContador,
                        verbo: comando.verbo.como_str().to_owned(),
                        servico: Some(servico.id_servico.clone()),
                        ambiente: comando.ambiente,
                        cliente_id: None,
                        usuario_id: Some(comando.usuario_id),
                        origem: self.origem,
                        resultado: ResultadoChamada::Indeterminado,
                        status_http: None,
                        duracao_ms,
                        request_id: comando.request_id.clone(),
                    })
                    .await;

                if let Some(reserva) = &reserva {
                    self.idempotencia
                        .atualizar(
                            reserva,
                            estado,
                            None,
                            Some(serde_json::json!({"estado": estado})),
                        )
                        .await?;
                    self.auditoria
                        .registrar_mutacao(&EventoAuditoriaIntegra {
                            escritorio_id: comando.escritorio_id,
                            usuario_id: comando.usuario_id,
                            acao: "INTEGRA_MUTACAO_RESULTADO",
                            verbo: comando.verbo,
                            id_sistema: servico.id_sistema.to_owned(),
                            id_servico: servico.id_servico.to_owned(),
                            ambiente: comando.ambiente,
                            estado: estado.to_owned(),
                            payload_hash,
                        })
                        .await?;
                }
                self.auditoria
                    .registrar_chamada(&EventoChamadaIntegra {
                        escritorio_id: comando.escritorio_id,
                        usuario_id: comando.usuario_id,
                        verbo: comando.verbo,
                        id_sistema: servico.id_sistema.to_owned(),
                        id_servico: servico.id_servico.to_owned(),
                        ambiente: comando.ambiente,
                        estado: estado.to_owned(),
                        status_code: None,
                        referencia_resposta: None,
                        request_id: comando.request_id,
                        bilhetavel: false,
                    })
                    .await?;
                Err(erro)
            }
        }
    }

    async fn finalizar_reserva(
        &self,
        reserva: Option<&ReservaIdempotencia>,
        verbo: VerboIntegraContador,
        entrega: &EntregaIntegraContador,
        payload_hash: &str,
        comando: &ComandoIntegraContador,
    ) -> Result<(), ErroIntegraContador> {
        let Some(reserva) = reserva else {
            return Ok(());
        };
        let estado = match entrega.http_status {
            504 => "INDETERMINADO",
            202 | 204 => "PROCESSANDO",
            200..=299 => "CONCLUIDO",
            _ => "REJEITADO",
        };
        self.idempotencia
            .atualizar(
                reserva,
                estado,
                Some(i32::from(entrega.http_status)),
                Some(serde_json::json!({
                    "estado": estado,
                    "referenciaResposta": entrega.referencia_resposta,
                })),
            )
            .await?;
        self.auditoria
            .registrar_mutacao(&EventoAuditoriaIntegra {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                acao: "INTEGRA_MUTACAO_RESULTADO",
                verbo,
                id_sistema: reserva.id_sistema.clone(),
                id_servico: reserva.id_servico.clone(),
                ambiente: comando.ambiente,
                estado: estado.to_owned(),
                payload_hash: payload_hash.to_owned(),
            })
            .await
    }
}

/// Comando para autenticar procurador via AUTENTICAPROCURADOR/ENVIOXMLASSINADO81.
#[derive(Debug, Clone)]
pub struct ComandoAutenticarProcurador {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Uuid,
    pub ambiente: AmbienteSerpro,
    pub request_id: String,
}

/// Resultado da autenticaÃƒÂ§ÃƒÂ£o do procurador.
///
/// `jwt_token` ÃƒÂ© um `SecretString`: nunca ÃƒÂ© serializado em logs/respostas e ÃƒÂ©
/// zeroizado da memÃƒÂ³ria apÃƒÂ³s ser persistido cifrado no banco.
#[derive(Debug, Clone)]
pub struct ResultadoAutenticarProcurador {
    pub jwt_token: SecretString,
    pub expira_em: chrono::DateTime<chrono::Utc>,
}

/// Caso de uso para autenticar o procurador (escritÃƒÂ³rio) junto ao SERPRO
/// via endpoint AUTENTICAPROCURADOR/ENVIOXMLASSINADO81.
///
/// Fluxo (modelo Software House):
/// 1. Busca o cliente (contribuinte) e o certificado A1 ativo do escritÃƒÂ³rio
/// 2. Gera o Termo de AutorizaÃƒÂ§ÃƒÂ£o com 3 atores: Contratante (SaaS, da
///    configuraÃƒÂ§ÃƒÂ£o), AutorPedidoDados (EscritÃƒÂ³rio, via contexto) e
///    Contribuinte (cliente)
/// 3. Assina o XML com o certificado A1 do escritÃƒÂ³rio
/// 4. Envia para SERPRO via `/Apoiar` (AUTENTICAPROCURADOR/ENVIOXMLASSINADO81)
/// 5. Salva o jwt_token retornado vinculado ao escritÃƒÂ³rio e cliente
pub struct AutenticarProcuradorUseCase {
    contexto: Arc<dyn CatalogoEscritoriosIntegra>,
    cliente_repositorio: Arc<dyn ClienteRepositorio>,
    certificado_provider: Arc<dyn CertificadoA1Provider>,
    criptografia: Arc<dyn CertificadoCriptografia>,
    assinador: Arc<dyn AssinadorXml>,
    autentica_cliente: Arc<dyn AutenticaProcuradorCliente>,
    tokens_repositorio: Arc<dyn TokensSerproRepositorio>,
    auditoria: Arc<dyn AuditoriaIntegraContador>,
    /// Identidade da plataforma SaaS (Contratante). `None` quando a
    /// configuraÃƒÂ§ÃƒÂ£o nÃƒÂ£o informa `SERPRO_CONTRATANTE_CNPJ`/`_NOME` Ã¢â‚¬â€ a
    /// operaÃƒÂ§ÃƒÂ£o falha fechada com `ConfiguracaoIncompleta`.
    contratante_plataforma: Option<ContratantePlataforma>,
}

impl AutenticarProcuradorUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        contexto: Arc<dyn CatalogoEscritoriosIntegra>,
        cliente_repositorio: Arc<dyn ClienteRepositorio>,
        certificado_provider: Arc<dyn CertificadoA1Provider>,
        criptografia: Arc<dyn CertificadoCriptografia>,
        assinador: Arc<dyn AssinadorXml>,
        autentica_cliente: Arc<dyn AutenticaProcuradorCliente>,
        tokens_repositorio: Arc<dyn TokensSerproRepositorio>,
        auditoria: Arc<dyn AuditoriaIntegraContador>,
        contratante_plataforma: Option<ContratantePlataforma>,
    ) -> Self {
        Self {
            contexto,
            cliente_repositorio,
            certificado_provider,
            criptografia,
            assinador,
            autentica_cliente,
            tokens_repositorio,
            auditoria,
            contratante_plataforma,
        }
    }

    pub async fn executar(
        &self,
        comando: ComandoAutenticarProcurador,
    ) -> Result<ResultadoAutenticarProcurador, ErroIntegraContador> {
        // 0. Identidade da plataforma SaaS (Contratante) Ã¢â‚¬â€ fail-closed: sem
        //    `SERPRO_CONTRATANTE_CNPJ`/`SERPRO_CONTRATANTE_NOME` o Termo de
        //    AutorizaÃƒÂ§ÃƒÂ£o nÃƒÂ£o pode ser montado; nenhuma chamada ao SERPRO ocorre.
        let contratante_plataforma = self
            .contratante_plataforma
            .as_ref()
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 1. Busca o cliente para obter CPF/CNPJ e nome
        let cliente = self
            .cliente_repositorio
            .buscar_por_id(comando.escritorio_id, comando.cliente_id)
            .map_err(|_| ErroIntegraContador::Persistencia)?
            .ok_or(ErroIntegraContador::ClienteNaoEncontrado)?;

        // 2. Resolve o contexto (CNPJ e nome do escritÃƒÂ³rio autor e do contribuinte)
        let contexto = self
            .contexto
            .resolver_contexto(
                comando.escritorio_id,
                comando.usuario_id,
                cliente.documento.como_str(),
            )
            .await?;

        // 3. Busca o certificado A1 ativo do escritÃƒÂ³rio
        let certificado = self
            .certificado_provider
            .obter_ativo(comando.escritorio_id)
            .await?
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 5. ConstrÃƒÂ³i o Termo de AutorizaÃƒÂ§ÃƒÂ£o com os atores oficiais do SERPRO:
        //    destinatario (Contratante = plataforma SaaS) e assinadoPor
        //    (Procurador = escritÃƒÂ³rio, titular do certificado A1 que assina).
        //    O contribuinte vive apenas no envelope JSON, nÃƒÂ£o no Termo.
        let termo = TermoAutorizacao::novo_oficial(AtoresTermoAutorizacao {
            destinatario_numero: contratante_plataforma.numero.clone(),
            destinatario_nome: contratante_plataforma.nome.clone(),
            assinado_por_numero: contexto.autor_pedido_dados.numero.clone(),
            assinado_por_nome: contexto.escritorio_nome.clone(),
        })?;

        // 6. Decifra o PFX A1 e a senha do escritÃƒÂ³rio em memÃƒÂ³ria; as
        //    cÃƒÂ³pias usadas na assinatura sÃƒÂ£o zeroizadas pelo AssinadorXmlImpl
        //    ao final (o trait recebe &mut [u8] para isso).
        let mut pfx_der = self
            .criptografia
            .decifrar(&certificado.conteudo_cifrado)
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
        let mut senha = certificado
            .senha_cifrada
            .as_deref()
            .map(|cifrada| self.criptografia.decifrar(cifrada))
            .transpose()
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?
            .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;

        // 7. Assina o XML (o AssinadorXmlImpl zeroiza pfx_der/senha ao fim)
        let xml_assinado = self
            .assinador
            .assinar(&termo, &mut pfx_der, &mut senha)
            .await?;

        // 8. ConstrÃƒÂ³i a requisiÃƒÂ§ÃƒÂ£o para o endpoint `/Apoiar`
        //    (AUTENTICAPROCURADOR/ENVIOXMLASSINADO81). O envelope declara o
        //    contratante SaaS (modelo C), nÃƒÂ£o o escritÃƒÂ³rio.
        let request = IntegraContadorRequest {
            contratante: Identificacao::nova(&contratante_plataforma.numero)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?,
            autor_pedido_dados: contexto.autor_pedido_dados.clone(),
            contribuinte: contexto.contribuinte.clone(),
            pedido_dados: PedidoDados {
                id_sistema: "AUTENTICAPROCURADOR".to_owned(),
                id_servico: "ENVIOXMLASSINADO81".to_owned(),
                versao_sistema: "1.0".to_owned(),
                dados: serde_json::to_string(&serde_json::json!({
                    "xml": xml_assinado.xml_base64
                }))
                .map_err(|_| ErroIntegraContador::PayloadInvalido)?,
            },
        };

        // 9. Envia para SERPRO. Falhas sÃƒÂ£o auditadas antes de propagar o erro.
        let resultado = match self
            .autentica_cliente
            .enviar_xml_assinado(
                &request,
                &xml_assinado.xml_base64,
                comando.ambiente,
                &comando.request_id,
            )
            .await
        {
            Ok(r) => r,
            Err(erro) => {
                let _ = self
                    .auditoria
                    .registrar_chamada(&EventoChamadaIntegra {
                        escritorio_id: comando.escritorio_id,
                        usuario_id: comando.usuario_id,
                        verbo: VerboIntegraContador::Apoiar,
                        id_sistema: "AUTENTICAPROCURADOR".to_owned(),
                        id_servico: "ENVIOXMLASSINADO81".to_owned(),
                        ambiente: comando.ambiente,
                        estado: "FALHA_AUTENTICACAO".to_owned(),
                        status_code: None,
                        referencia_resposta: None,
                        request_id: comando.request_id.clone(),
                        bilhetavel: false,
                    })
                    .await;
                return Err(erro);
            }
        };

        // 10. Salva o token JWT cifrado no banco (vinculado a escritÃƒÂ³rio+cliente).
        //    Zeroiza a cÃƒÂ³pia em memÃƒÂ³ria apÃƒÂ³s persistir.
        let mut jwt_exposto = resultado.jwt_token.expose_secret().to_owned();
        self.tokens_repositorio
            .salvar_jwt_procurador(
                comando.escritorio_id,
                comando.cliente_id,
                comando.ambiente,
                jwt_exposto.as_str(),
                resultado.expira_em,
            )
            .await?;
        jwt_exposto.zeroize();

        // 11. Auditoria de sucesso
        self.auditoria
            .registrar_chamada(&EventoChamadaIntegra {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                verbo: VerboIntegraContador::Apoiar,
                id_sistema: "AUTENTICAPROCURADOR".to_owned(),
                id_servico: "ENVIOXMLASSINADO81".to_owned(),
                ambiente: comando.ambiente,
                estado: "CONCLUIDO".to_owned(),
                status_code: Some(200),
                referencia_resposta: None,
                request_id: comando.request_id,
                bilhetavel: false,
            })
            .await?;

        Ok(ResultadoAutenticarProcurador {
            jwt_token: resultado.jwt_token,
            expira_em: resultado.expira_em,
        })
    }
}

// ===== FASE A Ã¢â‚¬â€ validaÃƒÂ§ÃƒÂ£o e listagem de procuraÃƒÂ§ÃƒÂµes persistentes =====

/// Comando de validaÃƒÂ§ÃƒÂ£o de procuraÃƒÂ§ÃƒÂ£o de um cliente (`validar` ou
/// `revalidar`). Com `forcar`, ignora o TTL e consulta o SERPRO sempre.
#[derive(Debug, Clone)]
pub struct ComandoValidarProcuracao {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Uuid,
    pub ambiente: AmbienteSerpro,
    pub request_id: String,
    /// `revalidar`: forÃƒÂ§a a consulta ignorando a verificaÃƒÂ§ÃƒÂ£o recente (TTL).
    pub forcar: bool,
}

/// Resultado da validaÃƒÂ§ÃƒÂ£o: o registro persistido e a fonte.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoValidarProcuracao {
    /// `CACHE` = verificaÃƒÂ§ÃƒÂ£o recente ainda dentro do TTL (nenhuma chamada
    /// SERPRO); `SERPRO` = consulta nova.
    pub fonte: &'static str,
    pub registro: RegistroProcuracao,
}

/// Estado por serviÃƒÂ§o para a rotina fiscal (FASE A).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstadoProcuracaoServico {
    pub id_sistema: String,
    pub id_servico: String,
    pub verbo: VerboIntegraContador,
    /// Linha habilitada no catÃƒÂ¡logo vigente (configurÃƒÂ¡vel ou estÃƒÂ¡tico).
    pub habilitado: bool,
    /// `NAO_EXIGIDA` | `AUTORIZADA` | `CONSULTADA` | `PENDENTE`.
    pub estado: String,
}

/// Estado consolidado da procuraÃƒÂ§ÃƒÂ£o de um cliente, para o painel de rotina.
#[derive(Debug, Clone, PartialEq)]
pub struct EstadoProcuracaoCliente {
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    /// Status do ÃƒÂºltimo registro agregado (OBTERPROCURACAO41), quando existir.
    pub status: Option<String>,
    pub verificado_em: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub servicos: Vec<EstadoProcuracaoServico>,
}

/// Comando de listagem do estado de procuraÃƒÂ§ÃƒÂ£o de um cliente.
#[derive(Debug, Clone)]
pub struct ComandoListarProcuracao {
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub ambiente: AmbienteSerpro,
}

/// Caso de uso: valida (ou revalida) a procuraÃƒÂ§ÃƒÂ£o do cliente consultando o
/// SERPRO via OBTERPROCURACAO41 e persistindo o registro agregado em
/// `procuracoes` (FASE A). O formato de resposta nÃƒÂ£o estÃƒÂ¡ documentado (P7),
/// entÃƒÂ£o o registro fica agregado (`codigo_servico NULL`) com
/// `payload_redigido` tolerado.
pub struct ValidarProcuracoesUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    contexto: Arc<dyn CatalogoEscritoriosIntegra>,
    executar: std::sync::Arc<ExecutarIntegraContadorUseCase>,
    repositorio: Arc<dyn RepositorioProcuracoes>,
    /// Idade mÃƒÂ¡xima da verificaÃƒÂ§ÃƒÂ£o para a consulta pular o SERPRO (`CACHE`).
    ttl_verificacao: Option<chrono::TimeDelta>,
}

impl ValidarProcuracoesUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        contexto: Arc<dyn CatalogoEscritoriosIntegra>,
        executar: std::sync::Arc<ExecutarIntegraContadorUseCase>,
        repositorio: Arc<dyn RepositorioProcuracoes>,
        ttl_verificacao_secs: Option<u64>,
    ) -> Self {
        Self {
            clientes,
            contexto,
            executar,
            repositorio,
            ttl_verificacao: ttl_verificacao_secs
                .filter(|segundos| *segundos > 0)
                .and_then(|segundos| chrono::TimeDelta::try_seconds(segundos as i64)),
        }
    }

    pub async fn executar(
        &self,
        comando: ComandoValidarProcuracao,
    ) -> Result<ResultadoValidarProcuracao, ErroIntegraContador> {
        let cliente = self
            .clientes
            .buscar_por_id(comando.escritorio_id, comando.cliente_id)
            .map_err(|_| ErroIntegraContador::Persistencia)?
            .ok_or(ErroIntegraContador::ClienteNaoEncontrado)?;

        let contexto = self
            .contexto
            .resolver_contexto(
                comando.escritorio_id,
                comando.usuario_id,
                cliente.documento.como_str(),
            )
            .await?;
        let contribuinte = contexto.contribuinte.numero.clone();
        let procurador = contexto.autor_pedido_dados.numero.clone();

        // Fresh-check: verificaÃƒÂ§ÃƒÂ£o recente dentro do TTL responde do cache
        // (nenhuma chamada SERPRO). `revalidar` (forcar) ignora o TTL.
        if !comando.forcar
            && let Some(ttl) = self.ttl_verificacao
        {
            let agregado = self
                .repositorio
                .buscar_agregado_recente(
                    comando.escritorio_id,
                    comando.ambiente,
                    &contribuinte,
                    &procurador,
                )
                .await?;
            if let Some(agregado) = agregado.filter(|a| a.verificado_em + ttl > chrono::Utc::now())
            {
                return Ok(ResultadoValidarProcuracao {
                    fonte: "CACHE",
                    registro: agregado,
                });
            }
        }

        // Consulta o SERPRO. `ConfiguracaoIncompleta` (contexto do escritÃƒÂ³rio)
        // e erros de configuraÃƒÂ§ÃƒÂ£o propagam SEM persistir tentativa.
        match self
            .executar
            .consultar_procuracoes(
                comando.escritorio_id,
                comando.usuario_id,
                cliente.documento.como_str().to_owned(),
                comando.ambiente,
                comando.request_id.clone(),
            )
            .await
        {
            Ok(resultado) => {
                let registro = self
                    .repositorio
                    .salvar(&NovoRegistroProcuracao {
                        escritorio_id: comando.escritorio_id,
                        cliente_id: Some(comando.cliente_id),
                        ambiente: comando.ambiente,
                        contribuinte_cpf_cnpj: contribuinte.clone(),
                        procurador_cpf_cnpj: procurador.clone(),
                        codigo_servico: None,
                        status: status_de_verificacao(&resultado.estado).to_owned(),
                        valido_de: None,
                        valido_ate: None,
                        payload_redigido: payload_redigido_de(resultado.dados_resposta.as_deref()),
                        origem: "OBTERPROCURACAO41".to_owned(),
                    })
                    .await?;
                Ok(ResultadoValidarProcuracao {
                    fonte: "SERPRO",
                    registro,
                })
            }
            Err(ErroIntegraContador::ConfiguracaoIncompleta) => {
                Err(ErroIntegraContador::ConfiguracaoIncompleta)
            }
            Err(
                erro @ (ErroIntegraContador::Indisponivel | ErroIntegraContador::Indeterminado),
            ) => {
                // Falha de transporte/timeout: persiste a tentativa como
                // `INDETERMINADA` (estado da resposta fica indeterminado Ã¢â‚¬â€
                // o proprio SERPRO deixa claro que nao se pode concluir).
                self.repositorio
                    .salvar(&NovoRegistroProcuracao {
                        escritorio_id: comando.escritorio_id,
                        cliente_id: Some(comando.cliente_id),
                        ambiente: comando.ambiente,
                        contribuinte_cpf_cnpj: contribuinte.clone(),
                        procurador_cpf_cnpj: procurador.clone(),
                        codigo_servico: None,
                        status: "INDETERMINADA".to_owned(),
                        valido_de: None,
                        valido_ate: None,
                        payload_redigido: None,
                        origem: "OBTERPROCURACAO41".to_owned(),
                    })
                    .await?;
                Err(erro)
            }
            Err(erro) => Err(erro),
        }
    }
}

/// Caso de uso: lista o estado consolidado da procuraÃƒÂ§ÃƒÂ£o de um cliente Ã¢â‚¬â€
/// registro agregado + estado por serviÃƒÂ§o (`NAO_EXIGIDA`/`AUTORIZADA`/
/// `CONSULTADA`/`PENDENTE`) calculado fail-closed (P7).
pub struct ListarProcuracoesUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    repositorio: Arc<dyn RepositorioProcuracoes>,
    catalogo: Option<Arc<dyn ProvedorCatalogoIntegra>>,
    ttl_verificacao: Option<chrono::TimeDelta>,
}

impl ListarProcuracoesUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        repositorio: Arc<dyn RepositorioProcuracoes>,
        catalogo: Option<Arc<dyn ProvedorCatalogoIntegra>>,
        ttl_verificacao_secs: Option<u64>,
    ) -> Self {
        Self {
            clientes,
            repositorio,
            catalogo,
            ttl_verificacao: ttl_verificacao_secs
                .filter(|segundos| *segundos > 0)
                .and_then(|segundos| chrono::TimeDelta::try_seconds(segundos as i64)),
        }
    }

    pub async fn executar(
        &self,
        comando: ComandoListarProcuracao,
    ) -> Result<EstadoProcuracaoCliente, ErroIntegraContador> {
        let cliente = self
            .clientes
            .buscar_por_id(comando.escritorio_id, comando.cliente_id)
            .map_err(|_| ErroIntegraContador::Persistencia)?
            .ok_or(ErroIntegraContador::ClienteNaoEncontrado)?;

        let registros = self
            .repositorio
            .listar_por_cliente(comando.escritorio_id, comando.cliente_id)
            .await?
            .into_iter()
            .filter(|registro| registro.ambiente == comando.ambiente)
            .collect::<Vec<_>>();

        let agregado = registros
            .iter()
            .find(|registro| registro.codigo_servico.is_none())
            .cloned();

        // Estado por serviÃƒÂ§o: catÃƒÂ¡logo configurÃƒÂ¡vel (banco) com queda no
        // catÃƒÂ¡logo estÃƒÂ¡tico do domÃƒÂ­nio quando nÃƒÂ£o configurado.
        let linhas = match &self.catalogo {
            Some(catalogo) => catalogo.listar().await?,
            None => linhas_catalogo_estaticas(),
        };
        let servicos = linhas
            .iter()
            .map(|linha| EstadoProcuracaoServico {
                id_sistema: linha.id_sistema.clone(),
                id_servico: linha.id_servico.clone(),
                verbo: linha.verbo,
                habilitado: linha.habilitado && linha.versao_confirmada,
                estado: estado_por_servico(
                    linha,
                    &registros,
                    agregado.as_ref(),
                    self.ttl_verificacao,
                )
                .to_owned(),
            })
            .collect();

        Ok(EstadoProcuracaoCliente {
            contribuinte_cpf_cnpj: cliente.documento.como_str().to_owned(),
            procurador_cpf_cnpj: registros
                .first()
                .map(|registro| registro.procurador_cpf_cnpj.clone())
                .unwrap_or_default(),
            status: agregado.as_ref().map(|registro| registro.status.clone()),
            verificado_em: agregado.as_ref().map(|registro| registro.verificado_em),
            valido_de: agregado.as_ref().and_then(|registro| registro.valido_de),
            valido_ate: agregado.as_ref().and_then(|registro| registro.valido_ate),
            servicos,
        })
    }
}

/// Linhas do catÃƒÂ¡logo estÃƒÂ¡tico do domÃƒÂ­nio, na forma do catÃƒÂ¡logo
/// configurÃƒÂ¡vel Ã¢â‚¬â€ usado quando nenhum provedor de banco estÃƒÂ¡ ligado.
fn linhas_catalogo_estaticas() -> Vec<LinhaCatalogoServico> {
    crate::domain::integra_contador::servicos::catalogo_de_referencia()
        .into_iter()
        .map(|servico| LinhaCatalogoServico {
            id: Uuid::nil(),
            id_sistema: servico.id_sistema.to_owned(),
            id_servico: servico.id_servico.to_owned(),
            verbo: servico.verbo,
            versao_sistema: servico.versao_sistema.map(str::to_owned),
            // No catÃƒÂ¡logo estÃƒÂ¡tico, confirmado Ã¢â€¡â€ "1" jÃƒÂ¡ gravado na origem.
            versao_confirmada: servico.versao_sistema.is_some(),
            habilitado: servico.estado == EstadoCatalogo::Habilitado,
            apenas_consulta: false,
            requer_procuracao: servico.regra_procuracao.se_aplica(),
            codigos_procuracao: servico.regra_procuracao.codigos_possiveis(),
            descricao: String::new(),
            atualizado_em: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        })
        .collect()
}

/// Status do registro indica validade dentro da janela (fail-closed).
fn registro_esta_valido(
    registro: &RegistroProcuracao,
    agora: chrono::DateTime<chrono::Utc>,
) -> bool {
    if registro.status != "VALIDA" && registro.status != "ATIVA" {
        return false;
    }
    if registro
        .valido_de
        .is_some_and(|valido_de| agora < valido_de)
    {
        return false;
    }
    if registro
        .valido_ate
        .is_some_and(|valido_ate| agora > valido_ate)
    {
        return false;
    }
    true
}

/// Estado por serviÃƒÂ§o (fail-closed, P7):
/// - `NAO_EXIGIDA`: a regra de procuraÃƒÂ§ÃƒÂ£o nÃƒÂ£o se aplica;
/// - `AUTORIZADA`: existe registro per-serviÃƒÂ§o vÃƒÂ¡lido para um dos cÃƒÂ³digos
///   da regra (possÃƒÂ­vel sÃƒÂ³ apÃƒÂ³s o mapeamento do corpo da resposta Ã¢â‚¬â€ P7);
/// - `CONSULTADA`: procuraÃƒÂ§ÃƒÂ£o consultada/agregada recentemente com status
///   vÃƒÂ¡lido, mas sem correspondÃƒÂªncia por serviÃƒÂ§o;
/// - `PENDENTE`: caso contrÃƒÂ¡rio.
fn estado_por_servico(
    linha: &LinhaCatalogoServico,
    registros: &[RegistroProcuracao],
    agregado: Option<&RegistroProcuracao>,
    ttl_verificacao: Option<chrono::TimeDelta>,
) -> &'static str {
    if !linha.requer_procuracao {
        return "NAO_EXIGIDA";
    }

    let agora = chrono::Utc::now();
    let autorizada_por_servico = !linha.codigos_procuracao.is_empty()
        && registros.iter().any(|registro| {
            registro
                .codigo_servico
                .as_deref()
                .is_some_and(|codigo| linha.codigos_procuracao.contains(&codigo.to_owned()))
                && registro_esta_valido(registro, agora)
        });
    if autorizada_por_servico {
        return "AUTORIZADA";
    }

    let agregado_recente = agregado
        .map(|registro| {
            registro_esta_valido(registro, agora)
                && ttl_verificacao
                    .map(|ttl| registro.verificado_em + ttl > agora)
                    .unwrap_or(true)
        })
        .unwrap_or(false);
    if agregado_recente {
        return "CONSULTADA";
    }

    "PENDENTE"
}

fn status_de_verificacao(estado: &str) -> &'static str {
    match estado {
        "CONCLUIDO" => "VALIDA",
        "FALHA_AUTORIZACAO" => "NAO_AUTORIZADA",
        "INDETERMINADO" | "FALHA_TRANSPORTE" => "INDETERMINADA",
        _ => "INDETERMINADA",
    }
}

/// Converte o `dados` tolerado da resposta em `payload_redigido`: JSON
/// estruturado quando parseÃƒÂ¡vel; senÃƒÂ£o, string crua para inspeÃƒÂ§ÃƒÂ£o (P7).
fn payload_redigido_de(dados: Option<&str>) -> Option<Value> {
    let dados = dados?;
    match serde_json::from_str::<Value>(dados) {
        Ok(valor) => Some(valor),
        Err(_) => Some(Value::String(dados.to_owned())),
    }
}

fn criar_reserva(
    comando: &ComandoIntegraContador,
    payload_hash: &str,
) -> Result<ReservaIdempotencia, ErroIntegraContador> {
    let chave = comando
        .chave_idempotencia
        .as_ref()
        .filter(|chave| {
            (16..=64).contains(&chave.len())
                && chave
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        })
        .ok_or(ErroIntegraContador::IdempotenciaNecessaria)?;

    Ok(ReservaIdempotencia {
        escritorio_id: comando.escritorio_id,
        chave: chave.clone(),
        verbo: comando.verbo,
        id_sistema: comando.pedido_dados.id_sistema.clone(),
        id_servico: comando.pedido_dados.id_servico.clone(),
        versao_sistema: comando.pedido_dados.versao_sistema.clone(),
        ambiente: comando.ambiente,
        payload_hash: payload_hash.to_owned(),
    })
}

fn validar_dados(dados: &str) -> Result<serde_json::Value, ErroIntegraContador> {
    if dados.len() > 262_144 {
        return Err(ErroIntegraContador::PayloadInvalido);
    }
    serde_json::from_str(dados).map_err(|_| ErroIntegraContador::PayloadInvalido)
}

fn validar_payload_procuracoes(
    pedido: &PedidoDados,
    payload: &serde_json::Value,
    contexto: &ContextoIntegraContador,
) -> Result<(), ErroIntegraContador> {
    if pedido.id_sistema != "PROCURACOES" || pedido.id_servico != "OBTERPROCURACAO41" {
        return Ok(());
    }
    let tipo_outorgante = contexto.contribuinte.tipo.to_string();
    let tipo_outorgado = contexto.autor_pedido_dados.tipo.to_string();
    let valido = payload
        .get("outorgante")
        .and_then(serde_json::Value::as_str)
        == Some(contexto.contribuinte.numero.as_str())
        && payload
            .get("tipoOutorgante")
            .and_then(serde_json::Value::as_str)
            == Some(tipo_outorgante.as_str())
        && payload.get("outorgado").and_then(serde_json::Value::as_str)
            == Some(contexto.autor_pedido_dados.numero.as_str())
        && payload
            .get("tipoOutorgado")
            .and_then(serde_json::Value::as_str)
            == Some(tipo_outorgado.as_str());
    if !valido {
        return Err(ErroIntegraContador::PayloadInvalido);
    }
    Ok(())
}

fn hash_payload(dados: &str) -> String {
    hex::encode(Sha256::digest(dados.as_bytes()))
}

fn bilhetavel(verbo: VerboIntegraContador, status: u16) -> bool {
    !matches!(
        (verbo, status),
        (VerboIntegraContador::Apoiar, _)
            | (VerboIntegraContador::Monitorar, _)
            | (_, 204 | 304 | 400 | 401 | 404 | 429 | 500 | 503 | 504)
    )
}

fn mapear_resultado(
    ambiente: AmbienteSerpro,
    verbo: VerboIntegraContador,
    id_sistema: &str,
    id_servico: &str,
    entrega: EntregaIntegraContador,
) -> ResultadoIntegraContador {
    let mensagem = match entrega.http_status {
        202 => "A operacao esta em processamento.",
        204 => "O processamento continua; consulte o estado antes de repetir.",
        304 => "O recurso nao foi alterado.",
        504 => "Estado indeterminado; nao reenvie a operacao.",
        200..=299 => "Operacao concluida.",
        _ => "A SERPRO rejeitou ou nao conseguiu processar a operacao.",
    };
    let (codigos_mensagem, dados_resposta) = match entrega.resposta {
        Some(resposta) => (
            resposta
                .mensagens
                .into_iter()
                .filter(|mensagem| !mensagem.codigo.is_empty())
                .map(|mensagem| mensagem.codigo)
                .collect(),
            resposta.dados,
        ),
        None => (Vec::new(), None),
    };
    ResultadoIntegraContador {
        ambiente,
        verbo,
        id_sistema: id_sistema.to_owned(),
        id_servico: id_servico.to_owned(),
        estado: entrega.estado,
        status: entrega.http_status,
        mensagem,
        referencia_resposta: entrega.referencia_resposta,
        tempo_espera_segundos: entrega.tempo_espera_segundos,
        etag: entrega.etag,
        expires: entrega.expires,
        codigos_mensagem,
        dados_resposta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calcula_hash_estavel_do_payload() {
        assert_eq!(
            hash_payload(r#"{"x":1}"#),
            "5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22"
        );
    }

    #[test]
    fn rechaca_chave_de_idempotencia_curta() {
        let comando = ComandoIntegraContador {
            escritorio_id: Uuid::nil(),
            usuario_id: Uuid::nil(),
            documento_contribuinte: "52998224725".to_owned(),
            verbo: VerboIntegraContador::Declarar,
            pedido_dados: PedidoDados {
                id_sistema: "PGDASD".to_owned(),
                id_servico: "TRANSDECLARACAO11".to_owned(),
                versao_sistema: "1".to_owned(),
                dados: "{}".to_owned(),
            },
            confirmacao_explicita: true,
            chave_idempotencia: Some("short".to_owned()),
            ambiente: AmbienteSerpro::Trial,
            request_id: "request".to_owned(),
        };

        assert_eq!(
            criar_reserva(&comando, "hash"),
            Err(ErroIntegraContador::IdempotenciaNecessaria)
        );
    }

    #[test]
    fn status_sem_conteudo_nao_e_bilhetavel() {
        assert!(!bilhetavel(VerboIntegraContador::Consultar, 204));
        assert!(!bilhetavel(VerboIntegraContador::Apoiar, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 202));
    }

    #[test]
    fn regras_bilhetamento_cinco_verbos() {
        // Apoiar e Monitorar nunca sÃƒÂ£o bilhetados
        assert!(!bilhetavel(VerboIntegraContador::Apoiar, 200));
        assert!(!bilhetavel(VerboIntegraContador::Monitorar, 200));

        // Consultar, Declarar, Emitir sÃƒÂ£o bilhetados em 200 e 202
        assert!(bilhetavel(VerboIntegraContador::Consultar, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 200));
        assert!(bilhetavel(VerboIntegraContador::Emitir, 200));
        assert!(bilhetavel(VerboIntegraContador::Declarar, 202));
        assert!(bilhetavel(VerboIntegraContador::Emitir, 202));

        // CÃƒÂ³digos nÃƒÂ£o bilhetados em nenhum verbo
        for status in [204, 304, 400, 401, 404, 429, 500, 503, 504] {
            assert!(!bilhetavel(VerboIntegraContador::Consultar, status));
            assert!(!bilhetavel(VerboIntegraContador::Declarar, status));
            assert!(!bilhetavel(VerboIntegraContador::Emitir, status));
        }
    }
}
