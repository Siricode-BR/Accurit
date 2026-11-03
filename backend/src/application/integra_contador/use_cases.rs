//! Caso de uso principal do Integra Contador (SPEC §5–§7).
//!
//! Fluxo, em ordem fixa (fail-closed da esquerda para a direita):
//! 1. verificação no catálogo (`validar_servico`) — D4/P4;
//! 2. gates de idempotência para verbos mutáveis (Declarar/Emitir),
//!    sempre ANTES de qualquer chamada externa;
//! 3. `contratante` resolvido do tenant (nunca do corpo);
//! 4. limite local por escritório (SPEC §3.3);
//! 5. procuração e-CAC quando `autorPedidoDados ≠ contribuinte`
//!    (SPEC §6), com token de procurador obtido via
//!    `ENVIOXMLASSINADO81` antes da chamada real;
//! 6. replay/conflito de idempotência e reserva;
//! 7. chamada transportada com interpretação de 202/204/304/504;
//! 8. idempotência finalizada com resultado pré-computado;
//! 9. bilhetagem em `consultas_fiscais` (`INTEGRA_CONTADOR`) — sem
//!    registro para `Apoiar`/`Monitorar` nem para 204/304/400/401/404/
//!    429/500/503/504.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::application::divida_ativa::ports::{
    CatalogoClientesPorDocumento, CatalogoOperadores, LimitadorConsultas,
    RepositorioConsultasFiscais,
};
use crate::application::integra_contador::ports::{
    AutenticadorProcurador, CatalogoEscritoriosIntegra, ClienteIntegraContador,
    EntregaIntegraContador, RepositorioIdempotencia, ReservaIdempotencia, ValidadorProcuracao,
};
use crate::domain::divida_ativa::RegistroConsultaFiscal;
use crate::domain::divida_ativa::mascarar_documento;
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, PedidoDados,
    VerboIntegraContador,
};
use crate::domain::integra_contador::erros::{ErroIntegraContador, ErroProcuracao};
use crate::domain::integra_contador::regras_procuracao::{
    RegraProcuracao, regra_procuracao_para_servico,
};
use crate::domain::integra_contador::servicos::{ServicoReferencia, validar_servico};
use crate::domain::serpro::ambiente::AmbienteSerpro;

pub const PRODUTO_INTEGRA_CONTADOR: &str = "INTEGRA_CONTADOR";

/// Estados de resultado que NÃO vão para `consultas_fiscais`: estados de
/// transporte da regra de bilhetagem da SPEC (204, 304, 400, 401, 404,
/// 429, 500, 503, 504). `Apoiar`/`Monitorar` nem chegam a ser bilhetados
/// (verbo), independente do resultado.
const ESTADOS_NAO_BILHETADOS: [&str; 9] = [
    "EM_PROCESSAMENTO_COM_ETAG", // 204
    "TOKEN_PROCURADOR_VALIDO",   // 304
    "PAYLOAD_INVALIDO",          // 400
    "NAO_AUTORIZADO",            // 401
    "NAO_ENCONTRADO",            // 404
    "LIMITE_EXCEDIDO",           // 429
    "ERRO_SERPRO",               // 500
    "INDISPONIVEL",              // 503
    "INDETERMINADO",             // 504
];

/// Comando de execução de uma operação Integra Contador.
#[derive(Debug, Clone)]
pub struct ComandoIntegraContador {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    /// Versão informada; reconciliada com o catálogo (nunca valor global).
    pub versao_sistema: Option<String>,
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    /// JSON específico do serviço (string, SPEC §5.2).
    pub dados: String,
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
}

/// Resultado normalizado para a presentation. Para 202/204/429 os
/// detalhes seguem encapsulados no erro correspondente que este tipo não
/// cobre (uso de `interpretar_entrega` no fluxo).
pub struct ResultadoIntegraContador {
    /// Estado canônico interno (SUCESSO, PROCESSAMENTO_ASSINCRONO, …).
    pub estado: String,
    pub resposta: Option<IntegraContadorResponse>,
    pub protocolo: Option<String>,
}

fn codigo_de_erro(erro: &ErroIntegraContador) -> &'static str {
    match erro {
        ErroIntegraContador::Catalogo(_) => "SERPRO_CATALOGO",
        ErroIntegraContador::PayloadInvalido { .. } => "SERPRO_PAYLOAD_INVALIDO",
        ErroIntegraContador::ProcuracaoNecessaria { .. } => "SERPRO_PROCURACAO_NECESSARIA",
        ErroIntegraContador::Procuracao(_) => "SERPRO_PROCURACAO",
        ErroIntegraContador::Assinatura(_) => "SERPRO_ASSINATURA",
        ErroIntegraContador::TokenExpirado => "SERPRO_TOKEN_EXPIRADO",
        ErroIntegraContador::AutenticacaoSapiFalhou { .. } => "SERPRO_SAPI",
        ErroIntegraContador::RequisicaoFalhou { .. } => "SERPRO_REQUISICAO",
        ErroIntegraContador::RespostaInvalida { .. } => "SERPRO_RESPOSTA_INVALIDA",
        ErroIntegraContador::EstadoIndeterminado => "SERPRO_INDETERMINADO",
        ErroIntegraContador::ProcessamentoAssincrono { .. } => "SERPRO_PROCESSANDO",
        ErroIntegraContador::ProcessamentoNaoConcluido => "SERPRO_PROCESSANDO",
        ErroIntegraContador::TokenProcuradorValido => "SERPRO_TOKEN_PROCURADOR_VALIDO",
        ErroIntegraContador::AcessoNegadoSerpro { .. } => "SERPRO_FORBIDDEN",
        ErroIntegraContador::RateLimitExcedido { .. } => "SERPRO_RATE_LIMITED",
        ErroIntegraContador::ServicoIndisponivel { .. } => "SERPRO_INDISPONIVEL",
        ErroIntegraContador::ConfirmacaoExplicitaNecessaria => "SERPRO_CONFIRMACAO_NECESSARIA",
        ErroIntegraContador::ChaveIdempotenciaNecessaria => "SERPRO_IDEMPOTENCIA_NECESSARIA",
        ErroIntegraContador::IdempotenciaConflito => "SERPRO_IDEMPOTENCIA_CONFLITO",
        ErroIntegraContador::ErroInterno(_) => "SERPRO_ERRO_INTERNO",
    }
}

/// Registra a falha no bilhete (quando a regra permite) e devolve o erro
/// original. Persistir o erro original jamais vence o retorno do erro.
fn registrar_falha(
    repositorio: &dyn RepositorioConsultasFiscais,
    mut registro: RegistroConsultaFiscal,
    erro: &ErroIntegraContador,
) -> ErroIntegraContador {
    registro.response_redigido = Some(serde_json::json!({ "codigo": codigo_de_erro(erro) }));
    registro.estado = estado_persistencia_de_erro(erro).to_owned();
    match repositorio.registrar(&registro) {
        Ok(_) => erro.clone(),
        Err(f) => {
            tracing::error!(
                erro = %f,
                "não foi possível persistir falha do Integra Contador"
            );
            ErroIntegraContador::ErroInterno("falha ao registrar operação".to_owned())
        }
    }
}

/// Estado canônico de persistência a partir da falha (usado no bilhete).
fn estado_persistencia_de_erro(erro: &ErroIntegraContador) -> &'static str {
    match erro {
        ErroIntegraContador::Catalogo(_) => "CATALOGO",
        ErroIntegraContador::PayloadInvalido { .. } => "PAYLOAD_INVALIDO",
        ErroIntegraContador::ProcuracaoNecessaria { .. } | ErroIntegraContador::Procuracao(_) => {
            "PROCURACAO"
        }
        ErroIntegraContador::Assinatura(_) => "ASSINATURA",
        ErroIntegraContador::TokenExpirado => "NAO_AUTORIZADO",
        ErroIntegraContador::AutenticacaoSapiFalhou { .. } => "SAPI",
        ErroIntegraContador::RequisicaoFalhou { status, .. } if *status == 400 => {
            "PAYLOAD_INVALIDO"
        }
        ErroIntegraContador::RequisicaoFalhou { status, .. } if *status == 404 => "NAO_ENCONTRADO",
        ErroIntegraContador::RequisicaoFalhou { .. } => "RESPOSTA_INVALIDA",
        ErroIntegraContador::RespostaInvalida { .. } => "RESPOSTA_INVALIDA",
        ErroIntegraContador::EstadoIndeterminado => "INDETERMINADO",
        ErroIntegraContador::ProcessamentoAssincrono { .. } => "PROCESSAMENTO_ASSINCRONO",
        ErroIntegraContador::ProcessamentoNaoConcluido => "EM_PROCESSAMENTO_COM_ETAG",
        ErroIntegraContador::TokenProcuradorValido => "TOKEN_PROCURADOR_VALIDO",
        ErroIntegraContador::AcessoNegadoSerpro { .. } => "ACESSO_NEGADO",
        ErroIntegraContador::RateLimitExcedido { .. } => "LIMITE_EXCEDIDO",
        ErroIntegraContador::ServicoIndisponivel { .. } => "INDISPONIVEL",
        ErroIntegraContador::ConfirmacaoExplicitaNecessaria => "PAYLOAD_INVALIDO",
        ErroIntegraContador::ChaveIdempotenciaNecessaria => "PAYLOAD_INVALIDO",
        ErroIntegraContador::IdempotenciaConflito => "CONFLITO_IDEMPOTENCIA",
        ErroIntegraContador::ErroInterno(_) => "ERRO_INTERNO",
    }
}

fn registro_base(
    comando: &ComandoIntegraContador,
    ambiente: AmbienteSerpro,
    request_id: &str,
) -> RegistroConsultaFiscal {
    RegistroConsultaFiscal {
        escritorio_id: comando.escritorio_id,
        cliente_id: None,
        produto: PRODUTO_INTEGRA_CONTADOR.to_owned(),
        servico: format!("{}/{}", comando.id_sistema, comando.id_servico),
        verbo: comando.verbo.to_string(),
        ambiente: ambiente.as_str().to_owned(),
        chave_consulta_hash: None,
        chave_consulta_mascarada: Some(mascarar_documento(&comando.contribuinte.numero)),
        fonte_consulta: None,
        status_code: None,
        estado: "EM_ANDAMENTO".to_owned(),
        protocolo: None,
        request_redigido: serde_json::json!({
            "verbo": comando.verbo.to_string(),
            "idSistema": comando.id_sistema,
            "idServico": comando.id_servico,
            "contribuinteMascarado": mascarar_documento(&comando.contribuinte.numero),
            "autorMascarado": mascarar_documento(&comando.autor_pedido_dados.numero),
            "requestId": request_id,
        })
        .into(),
        response_redigido: None,
    }
}

/// Hash canônico do payload idempotente — independe de token/headers.
fn hash_payload(
    verbo: &VerboIntegraContador,
    id_sistema: &str,
    id_servico: &str,
    versao_sistema: &str,
    contratante: &str,
    autor: &str,
    contribuinte: &str,
    dados: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verbo.to_string().as_bytes());
    for campo in [
        id_sistema,
        id_servico,
        versao_sistema,
        contratante,
        autor,
        contribuinte,
        dados,
    ] {
        hasher.update([0xFF]);
        hasher.update(campo.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Monta o envelope com `contratante` do tenant e versão confirmada do
/// catálogo (nunca a informada com valor global).
fn montar_request(
    contratante: Identificacao,
    comando: &ComandoIntegraContador,
    servico: &ServicoReferencia,
) -> IntegraContadorRequest {
    IntegraContadorRequest {
        contratante,
        autor_pedido_dados: comando.autor_pedido_dados.clone(),
        contribuinte: comando.contribuinte.clone(),
        pedido_dados: PedidoDados {
            id_sistema: servico.id_sistema.clone(),
            id_servico: servico.id_servico.clone(),
            versao_sistema: servico.versao_sistema.clone().unwrap_or_default(),
            dados: comando.dados.clone(),
        },
    }
}

fn mensagens_do_corpo(entrega: &EntregaIntegraContador) -> String {
    entrega
        .corpo
        .as_ref()
        .map(IntegraContadorResponse::descricoes)
        .unwrap_or_default()
}

fn protocolo_de(resposta: &IntegraContadorResponse) -> Option<String> {
    resposta
        .dados
        .as_deref()
        .and_then(|dados| serde_json::from_str::<serde_json::Value>(dados).ok())
        .and_then(|dados| {
            dados
                .get("protocolo")
                .and_then(|p| p.as_str().map(str::to_owned))
        })
}

/// Interpreta a entrega (HTTP + corpo) em estado de domínio. O status do
/// corpo vence quando traz 4xx/5xx — alguns produtos da SERPRO reportam
/// erros de negócio no corpo com HTTP 200.
pub fn interpretar_entrega(
    entrega: &EntregaIntegraContador,
) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
    let status_corpo = entrega.corpo.as_ref().map(|corpo| corpo.status);
    let status_efetivo = status_corpo
        // corpo estritamente divergente de 4xx/5xx vence:
        .filter(|&status| (400..=599).contains(&status) && status != entrega.http_status as i32)
        .unwrap_or(entrega.http_status as i32);

    match status_efetivo {
        // Estados específicos de transporte ANTES da faixa de 2xx:
        202 => Ok(ResultadoIntegraContador {
            estado: "PROCESSAMENTO_ASSINCRONO".to_owned(),
            resposta: entrega.corpo.clone(),
            protocolo: None,
        }),
        204 => {
            if entrega.etag.is_some() {
                Ok(ResultadoIntegraContador {
                    estado: "EM_PROCESSAMENTO_COM_ETAG".to_owned(),
                    resposta: None,
                    protocolo: None,
                })
            } else {
                Err(ErroIntegraContador::RespostaInvalida {
                    motivo: "HTTP 204 sem eTag de continuidade".to_owned(),
                })
            }
        }
        304 => Ok(ResultadoIntegraContador {
            estado: "TOKEN_PROCURADOR_VALIDO".to_owned(),
            resposta: None,
            protocolo: None,
        }),
        // Faixa de sucesso (200/201/…), depois dos particulares:
        200..=299 => {
            let resposta =
                entrega
                    .corpo
                    .clone()
                    .ok_or_else(|| ErroIntegraContador::RespostaInvalida {
                        motivo: "resposta sem corpo".to_owned(),
                    })?;
            let protocolo = protocolo_de(&resposta);
            Ok(ResultadoIntegraContador {
                estado: "SUCESSO".to_owned(),
                resposta: Some(resposta),
                protocolo,
            })
        }
        400 => Err(ErroIntegraContador::RequisicaoFalhou {
            status: 400,
            mensagem: mensagens_do_corpo(entrega),
        }),
        401 => Err(ErroIntegraContador::TokenExpirado),
        403 => Err(ErroIntegraContador::AcessoNegadoSerpro {
            motivo: mensagens_do_corpo(entrega),
        }),
        404 => Err(ErroIntegraContador::RequisicaoFalhou {
            status: 404,
            mensagem: mensagens_do_corpo(entrega),
        }),
        429 => Err(ErroIntegraContador::RateLimitExcedido {
            retry_after: entrega.retry_after.unwrap_or(60),
        }),
        500 => Err(ErroIntegraContador::RespostaInvalida {
            motivo: format!("erro interno SERPRO: {}", mensagens_do_corpo(entrega)),
        }),
        503 => Err(ErroIntegraContador::ServicoIndisponivel {
            motivo: mensagens_do_corpo(entrega),
        }),
        504 => Err(ErroIntegraContador::EstadoIndeterminado),
        status => Err(ErroIntegraContador::RespostaInvalida {
            motivo: format!("status {status} fora do contrato da SPEC"),
        }),
    }
}

/// Estados que produzem resultado pré-computável em `idempotencia_operacoes`.
fn estado_para_idempotencia(resultado: &ResultadoIntegraContador) -> Option<(&str, Option<u16>)> {
    match resultado.estado.as_str() {
        "SUCESSO" => Some(("SUCESSO", Some(200))),
        "PROCESSAMENTO_ASSINCRONO" => Some(("PROCESSAMENTO_ASSINCRONO", Some(202))),
        "EM_PROCESSAMENTO_COM_ETAG" => Some(("EM_PROCESSAMENTO_COM_ETAG", Some(204))),
        _ => None,
    }
}

fn resultado_replay(
    reserva: ReservaIdempotencia,
) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
    let estado = match reserva.estado.as_str() {
        "SUCESSO" | "PROCESSAMENTO_ASSINCRONO" | "EM_PROCESSAMENTO_COM_ETAG" => {
            reserva.estado.clone()
        }
        "INDETERMINADO" => return Err(ErroIntegraContador::EstadoIndeterminado),
        _ => return Err(ErroIntegraContador::IdempotenciaConflito),
    };

    Ok(ResultadoIntegraContador {
        estado,
        // A resposta persistida pela chave de idempotência é redigida; não
        // contém dados fiscais nem protocolo que possam ser reproduzidos.
        resposta: resposta_redigida(reserva.resposta.as_ref()),
        protocolo: None,
    })
}

fn resposta_redigida(
    resposta: Option<&IntegraContadorResponse>,
) -> Option<IntegraContadorResponse> {
    resposta.map(|resposta| IntegraContadorResponse {
        status: resposta.status,
        mensagens: resposta
            .mensagens
            .iter()
            .map(
                |mensagem| crate::domain::integra_contador::envelope::MensagemSerpro {
                    codigo: mensagem.codigo.clone(),
                    descricao: String::new(),
                    tipo: None,
                },
            )
            .collect(),
        dados: None,
    })
}

fn status_codigo_canonico(estado: &str) -> i32 {
    match estado {
        "SUCESSO" => 200,
        "PROCESSAMENTO_ASSINCRONO" => 202,
        "EM_PROCESSAMENTO_COM_ETAG" => 204,
        "TOKEN_PROCURADOR_VALIDO" => 304,
        _ => 500,
    }
}

/// Remove pontuação de CPF/CNPJ do cadastro do escritório.
fn limpar_documento(texto: &str) -> String {
    texto.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[derive(Clone)]
pub struct ExecutarIntegraContadorUseCase {
    cliente: Arc<dyn ClienteIntegraContador>,
    validador_procuracao: Arc<dyn ValidadorProcuracao>,
    autenticador_procurador: Arc<dyn AutenticadorProcurador>,
    idempotencia: Arc<dyn RepositorioIdempotencia>,
    catalogo_escritorios: Arc<dyn CatalogoEscritoriosIntegra>,
    limitador: Arc<dyn LimitadorConsultas>,
    repositorio: Arc<dyn RepositorioConsultasFiscais>,
    catalogo_clientes: Option<Arc<dyn CatalogoClientesPorDocumento>>,
    catalogo_operadores: Option<Arc<dyn CatalogoOperadores>>,
    operador_header_habilitado: bool,
}

impl ExecutarIntegraContadorUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        cliente: Arc<dyn ClienteIntegraContador>,
        validador_procuracao: Arc<dyn ValidadorProcuracao>,
        autenticador_procurador: Arc<dyn AutenticadorProcurador>,
        idempotencia: Arc<dyn RepositorioIdempotencia>,
        catalogo_escritorios: Arc<dyn CatalogoEscritoriosIntegra>,
        limitador: Arc<dyn LimitadorConsultas>,
        repositorio: Arc<dyn RepositorioConsultasFiscais>,
    ) -> Self {
        Self {
            cliente,
            validador_procuracao,
            autenticador_procurador,
            idempotencia,
            catalogo_escritorios,
            limitador,
            repositorio,
            catalogo_clientes: None,
            catalogo_operadores: None,
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

    pub async fn executar(
        &self,
        comando: ComandoIntegraContador,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        // `Apoiar`/`Monitorar`: nunca bilhetados, qualquer que seja o resultado.
        let registro_original = if comando.verbo.e_bilhetado() {
            Some(registro_base(&comando, ambiente, request_id))
        } else {
            None
        };

        // ===== 1. Catálogo (D4/P4) =====
        let servico = match validar_servico(
            &comando.id_sistema,
            &comando.id_servico,
            comando.versao_sistema.as_deref(),
            comando.verbo,
            comando.contribuinte.tipo,
        ) {
            Ok(servico) => servico,
            Err(erro) => {
                return self
                    .cair_com_ou_sem_bilhete(registro_original, ErroIntegraContador::from(erro));
            }
        };

        // ===== 2. Gates de idempotência (mutáveis) =====
        let mutavel = comando.verbo.e_mutavel();
        if mutavel {
            let erro_gate = if !comando.confirmacao_explicita {
                Some(ErroIntegraContador::ConfirmacaoExplicitaNecessaria)
            } else if comando.chave_idempotencia.is_none() {
                Some(ErroIntegraContador::ChaveIdempotenciaNecessaria)
            } else {
                None
            };
            if let Some(erro) = erro_gate {
                return self.cair_com_ou_sem_bilhete(registro_original, erro);
            }
        }

        // ===== 3. Contratante resolvido do tenant =====
        let contratante = match self.contratante_do_tenant(comando.escritorio_id).await {
            Ok(contratante) => contratante,
            Err(erro) => return self.cair_com_ou_sem_bilhete(registro_original, erro),
        };

        // ===== 4. Limite local por escritório =====
        if let Err(_erro) = self.limitador.permitir(comando.escritorio_id) {
            return self.cair_com_ou_sem_bilhete(
                registro_original,
                ErroIntegraContador::RateLimitExcedido { retry_after: 60 },
            );
        }

        // ===== 5. Procuração (SPEC §6) =====
        let token_procurador = match self
            .passo_procuracao(&comando, &contratante, ambiente, request_id)
            .await
        {
            Ok(token) => token,
            Err(erro) => return self.cair_com_ou_sem_bilhete(registro_original, erro),
        };

        // ===== 6. Replay/conflito de idempotência (leitura antecipada) =====
        let hash = hash_payload(
            &comando.verbo,
            &servico.id_sistema,
            &servico.id_servico,
            servico.versao_sistema.as_deref().unwrap_or_default(),
            &contratante.numero,
            &comando.autor_pedido_dados.numero,
            &comando.contribuinte.numero,
            &comando.dados,
        );
        if mutavel {
            let chave = comando.chave_idempotencia.as_ref().expect("gate garante");
            match self
                .idempotencia
                .verificar(comando.escritorio_id, chave)
                .await
            {
                Ok(Some(reserva)) => {
                    if reserva.payload_hash != hash {
                        return self.cair_com_ou_sem_bilhete(
                            registro_original,
                            ErroIntegraContador::IdempotenciaConflito,
                        );
                    }
                    // MESMA chave + MESMO payload ⇒ reproduz o estado
                    // persistido, sem transmitir novamente. Estados em
                    // execução/indeterminados falham fechados.
                    return match resultado_replay(reserva) {
                        Ok(resultado) => Ok(resultado),
                        Err(erro) => self.cair_com_ou_sem_bilhete(registro_original, erro),
                    };
                }
                Ok(None) => {}
                Err(f) => {
                    return self.cair_com_ou_sem_bilhete(
                        registro_original,
                        ErroIntegraContador::ErroInterno(f.to_string()),
                    );
                }
            }
        }

        // ===== 7. Reserva idempotente (sempre antes da chamada externa) =====
        if mutavel {
            let chave = comando.chave_idempotencia.as_ref().expect("gate garante");
            match self
                .idempotencia
                .reservar(
                    comando.escritorio_id,
                    chave,
                    comando.verbo,
                    &servico.id_sistema,
                    &servico.id_servico,
                    servico.versao_sistema.as_deref().unwrap_or_default(),
                    ambiente,
                    &hash,
                )
                .await
            {
                Ok(reserva) if reserva.criada_agora => {}
                Ok(reserva) => {
                    if reserva.payload_hash != hash {
                        return self.cair_com_ou_sem_bilhete(
                            registro_original,
                            ErroIntegraContador::IdempotenciaConflito,
                        );
                    }
                    return match resultado_replay(reserva) {
                        Ok(resultado) => Ok(resultado),
                        Err(erro) => self.cair_com_ou_sem_bilhete(registro_original, erro),
                    };
                }
                Err(f) => {
                    return self.cair_com_ou_sem_bilhete(
                        registro_original,
                        ErroIntegraContador::ErroInterno(f.to_string()),
                    );
                }
            }
        }

        // ===== 8. Chamada externa =====
        let request = montar_request(contratante, &comando, &servico);
        let entrega = match self
            .cliente
            .executar(
                comando.escritorio_id,
                comando.verbo,
                &request,
                token_procurador.as_deref(),
                ambiente,
                request_id,
            )
            .await
        {
            Ok(entrega) => entrega,
            Err(erro_transportado) => {
                return self
                    .encaixar_falha_externa(&comando, registro_original, erro_transportado)
                    .await;
            }
        };

        let resultado = match interpretar_entrega(&entrega) {
            Ok(resultado) => resultado,
            Err(erro_interpretado) => {
                return self
                    .encaixar_falha_externa(&comando, registro_original, erro_interpretado)
                    .await;
            }
        };

        // ===== 9. Idempotência: finalizar com resultado pré-computado =====
        if mutavel {
            let chave = comando.chave_idempotencia.as_ref().expect("gate garante");
            if let Some((estado, http_status)) = estado_para_idempotencia(&resultado) {
                let resposta = resposta_redigida(resultado.resposta.as_ref());
                if let Err(f) = self
                    .idempotencia
                    .finalizar(
                        comando.escritorio_id,
                        chave,
                        estado,
                        resposta.as_ref(),
                        http_status,
                    )
                    .await
                {
                    tracing::error!(erro = %f, "idempotência não finalizada");
                }
            }
        }

        // ===== 10. Bilhetagem =====
        if let Some(mut registro) = registro_original {
            if !ESTADOS_NAO_BILHETADOS.contains(&resultado.estado.as_str()) {
                if let Some(catalogo) = &self.catalogo_clientes {
                    match catalogo
                        .buscar_por_documento(comando.escritorio_id, &comando.contribuinte.numero)
                    {
                        Ok(cliente_id) => registro.cliente_id = cliente_id,
                        Err(erro) => {
                            tracing::warn!(erro = %erro, "cliente não associado ao bilhete")
                        }
                    }
                }
                registro.status_code = Some(status_codigo_canonico(&resultado.estado));
                registro.estado = resultado.estado.clone();
                registro.protocolo = resultado.protocolo.clone();
                registro.response_redigido = resultado.resposta.as_ref().map(|resposta| {
                    serde_json::json!({
                        "status": resposta.status,
                        "mensagens": resposta.mensagens,
                        "temDados": resposta.dados.is_some(),
                    })
                });
                if let Err(f) = self.repositorio.registrar(&registro) {
                    tracing::error!(erro = %f, "bilhete do Integra Contador não persistido");
                }
            }
        }

        Ok(resultado)
    }

    /// Falha em fase anterior à chamada externa (ou de interpretação do
    /// catálogo/gates): bilhetes são gravados conforme a regra.
    fn cair_com_ou_sem_bilhete(
        &self,
        registro_original: Option<RegistroConsultaFiscal>,
        erro: ErroIntegraContador,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        if let Some(base) = registro_original {
            let estado = estado_persistencia_de_erro(&erro);
            if !ESTADOS_NAO_BILHETADOS.contains(&estado) {
                return Err(registrar_falha(&*self.repositorio, base, &erro));
            }
        }
        Err(erro)
    }

    /// Falhas DEPOIS da chamada externa: para 504, a reserva idempotente
    /// fica com estado `INDETERMINADO` (resposta `None`) — replay nunca
    /// reexecuta uma operação que pode ter sido aplicada.
    async fn encaixar_falha_externa(
        &self,
        comando: &ComandoIntegraContador,
        registro_original: Option<RegistroConsultaFiscal>,
        erro: ErroIntegraContador,
    ) -> Result<ResultadoIntegraContador, ErroIntegraContador> {
        if comando.verbo.e_mutavel() {
            if ErroIntegraContador::EstadoIndeterminado == erro {
                if let Some(chave) = comando.chave_idempotencia.as_deref() {
                    let _ = self
                        .idempotencia
                        .finalizar(comando.escritorio_id, chave, "INDETERMINADO", None, None)
                        .await;
                }
            }
        }
        // Skip-list de transporte: não bilheta 204/304/400/401/404/429/500/503/504.
        if let Some(base) = registro_original {
            let estado = estado_persistencia_de_erro(&erro);
            if !ESTADOS_NAO_BILHETADOS.contains(&estado) {
                return Err(registrar_falha(&*self.repositorio, base, &erro));
            }
        }
        Err(erro)
    }

    async fn contratante_do_tenant(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Identificacao, ErroIntegraContador> {
        let cnpj_escritorio = self
            .catalogo_escritorios
            .buscar_cnpj(escritorio_id)
            .await
            .map_err(|f| ErroIntegraContador::ErroInterno(f.to_string()))?
            .ok_or_else(|| ErroIntegraContador::PayloadInvalido {
                motivo: "escritório sem CNPJ contratante configurado".to_owned(),
            })?;
        Identificacao::cnpj(limpar_documento(&cnpj_escritorio)).map_err(|motivo| {
            ErroIntegraContador::PayloadInvalido {
                motivo: format!("CNPJ contratante do escritório inválido: {motivo}"),
            }
        })
    }

    /// Passo de procuração + token de procurador. Devolve o token quando
    /// o fluxo exigir (autor ≠ contribuinte e regra aplicável). Fail-closed:
    /// sem regra ou com verificação falha, a operação é negada.
    async fn passo_procuracao(
        &self,
        comando: &ComandoIntegraContador,
        contratante: &Identificacao,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<Option<String>, ErroIntegraContador> {
        let ator_diferente = comando.autor_pedido_dados.numero != comando.contribuinte.numero;
        if !ator_diferente {
            return Ok(None);
        }

        // Fail-closed: serviço sem regra no catálogo de procurações nunca
        // é executado com procurador.
        let Some(regra) = regra_procuracao_para_servico(&comando.id_servico) else {
            return Err(ErroIntegraContador::Procuracao(
                ErroProcuracao::CodigoNaoPermitido(comando.id_servico.clone()),
            ));
        };
        if matches!(regra, RegraProcuracao::NaoSeAplica) {
            return Ok(None);
        }

        let validacao = self
            .validador_procuracao
            .validar(
                comando.escritorio_id,
                &comando.contribuinte,
                &comando.autor_pedido_dados,
                ambiente,
                request_id,
            )
            .await
            .map_err(|erro| {
                tracing::warn!(erro = %erro, "procuração não verificável — fail-closed");
                ErroIntegraContador::Procuracao(erro)
            })?;

        if !validacao.cobre_regra(&regra) {
            tracing::warn!(
                codigo = %validacao.codigo_servico,
                "procuração não cobre o código exigido pelo serviço"
            );
            return Err(ErroIntegraContador::ProcuracaoNecessaria {
                autor: comando.autor_pedido_dados.numero.clone(),
                contribuinte: comando.contribuinte.numero.clone(),
            });
        }
        if !validacao.e_valida_agora(chrono::Utc::now()) {
            let limite = validacao
                .valido_ate
                .map(|fim| fim.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "<sem vigência registrada>".to_owned());
            return Err(ErroIntegraContador::Procuracao(ErroProcuracao::Vencida(
                limite,
            )));
        }

        // Token de procurador para a chamada real (SPEC §7): termo
        // assinado + ENVIOXMLASSINADO81, com cache particionado por
        // (tenant, ambiente, outorgante, outorgado). `Apoiar` não é bilhetado.
        let token = self
            .autenticador_procurador
            .obter_token(
                comando.escritorio_id,
                contratante,
                &comando.autor_pedido_dados,
                &comando.contribuinte,
                ambiente,
                request_id,
            )
            .await?;
        Ok(Some(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_preserva_estado_sem_reexpor_dados_da_resposta() {
        let reserva = ReservaIdempotencia {
            payload_hash: "hash".to_owned(),
            resposta: Some(IntegraContadorResponse {
                status: 202,
                mensagens: vec![crate::domain::integra_contador::envelope::MensagemSerpro {
                    codigo: "PROCESSANDO".to_owned(),
                    descricao: "contribuinte 12345678909".to_owned(),
                    tipo: Some("INFO".to_owned()),
                }],
                dados: Some(r#"{"protocolo":"123","cpf":"12345678909"}"#.to_owned()),
            }),
            http_status: Some(202),
            estado: "PROCESSAMENTO_ASSINCRONO".to_owned(),
            criada_agora: false,
        };

        let replay = resultado_replay(reserva).expect("replay");
        assert_eq!(replay.estado, "PROCESSAMENTO_ASSINCRONO");
        let resposta = replay.resposta.expect("resposta redigida");
        assert_eq!(resposta.status, 202);
        assert_eq!(resposta.dados, None);
        assert_eq!(resposta.mensagens[0].codigo, "PROCESSANDO");
        assert!(resposta.mensagens[0].descricao.is_empty());
        assert_eq!(resposta.mensagens[0].tipo, None);
        assert_eq!(replay.protocolo, None);
    }

    #[test]
    fn replay_indeterminado_nao_se_apresenta_como_sucesso() {
        let reserva = ReservaIdempotencia {
            payload_hash: "hash".to_owned(),
            resposta: None,
            http_status: None,
            estado: "INDETERMINADO".to_owned(),
            criada_agora: false,
        };

        assert!(matches!(
            resultado_replay(reserva),
            Err(ErroIntegraContador::EstadoIndeterminado)
        ));
    }

    #[test]
    fn hash_payload_deterministico_e_divergente() {
        let a = hash_payload(
            &VerboIntegraContador::Declarar,
            "PGDASD",
            "TRANSDECLARACAO11",
            "1",
            "12345678000190",
            "12345678909",
            "98765432000190",
            "{}",
        );
        let b = hash_payload(
            &VerboIntegraContador::Declarar,
            "PGDASD",
            "TRANSDECLARACAO11",
            "1",
            "12345678000190",
            "12345678909",
            "98765432000190",
            "{}",
        );
        let c = hash_payload(
            &VerboIntegraContador::Declarar,
            "PGDASD",
            "TRANSDECLARACAO11",
            "1",
            "12345678000190",
            "12345678909",
            "98765432000191",
            "{}",
        );
        let d = hash_payload(
            &VerboIntegraContador::Declarar,
            "PGDASD",
            "TRANSDECLARACAO11",
            "1",
            "12345678000190",
            "12345678909",
            "98765432000190",
            "{\"pa\":202602}",
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn interpretar_202_processamento_assincrono() {
        let entrega = EntregaIntegraContador {
            http_status: 202,
            corpo: Some(IntegraContadorResponse {
                status: 202,
                mensagens: vec![],
                dados: None,
            }),
            tempo_espera: Some(120),
            retry_after: None,
            etag: None,
        };
        let resultado = interpretar_entrega(&entrega).expect("interpreta");
        assert_eq!(resultado.estado, "PROCESSAMENTO_ASSINCRONO");
    }

    #[test]
    fn interpretar_204_sem_etag_informa_erro() {
        let entrega = EntregaIntegraContador {
            http_status: 204,
            corpo: None,
            tempo_espera: None,
            retry_after: None,
            etag: None,
        };
        assert!(matches!(
            interpretar_entrega(&entrega),
            Err(ErroIntegraContador::RespostaInvalida { .. })
        ));
    }

    #[test]
    fn interpretar_429_com_retry_after() {
        let entrega = EntregaIntegraContador {
            http_status: 429,
            corpo: None,
            tempo_espera: None,
            retry_after: Some(42),
            etag: None,
        };
        assert!(matches!(
            interpretar_entrega(&entrega),
            Err(ErroIntegraContador::RateLimitExcedido { retry_after: 42 })
        ));
    }

    #[test]
    fn corpo_400_com_http_200_vence() {
        let entrega = EntregaIntegraContador {
            http_status: 200,
            corpo: Some(IntegraContadorResponse {
                status: 400,
                mensagens: vec![],
                dados: None,
            }),
            tempo_espera: None,
            retry_after: None,
            etag: None,
        };
        assert!(matches!(
            interpretar_entrega(&entrega),
            Err(ErroIntegraContador::RequisicaoFalhou { status: 400, .. })
        ));
    }

    #[test]
    fn estados_nao_bilhetados_completo() {
        for estado in [
            "EM_PROCESSAMENTO_COM_ETAG",
            "TOKEN_PROCURADOR_VALIDO",
            "PAYLOAD_INVALIDO",
            "NAO_AUTORIZADO",
            "NAO_ENCONTRADO",
            "LIMITE_EXCEDIDO",
            "ERRO_SERPRO",
            "INDISPONIVEL",
            "INDETERMINADO",
        ] {
            assert!(
                ESTADOS_NAO_BILHETADOS.contains(&estado),
                "faltou {estado} na skip-list"
            );
        }
    }

    #[test]
    fn limpar_documento_remove_pontuacao() {
        assert_eq!(limpar_documento("12.345.678/0001-90"), "12345678000190");
    }

    #[test]
    fn protocolo_extraido_de_sucesso() {
        let resposta = IntegraContadorResponse {
            status: 200,
            mensagens: vec![],
            dados: Some(r#"{"protocolo":"123"}"#.to_owned()),
        };
        assert_eq!(protocolo_de(&resposta), Some("123".to_owned()));
    }
}
