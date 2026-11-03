//! Casos de uso do Integra Contador.

use std::sync::Arc;
use std::time::Duration;

use secrecy::SecretString;
use uuid::Uuid;

use crate::application::integra_contador::ports::{
    AssinadorTermoAutorizacao, CatalogoClientesPorDocumentoIntegra, CatalogoOperadoresIntegra,
    ClienteIntegraContador, CredencialIntegraContadorProvider, GerenciadorTokensSerpro,
    RepositorioAuditoriaIntegraContador, RepositorioIdempotencia, ValidadorProcuracao,
};
use crate::domain::integra_contador::assinatura::{TermoAutorizacao, XmlAssinado};
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, PedidoDados, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::{ErroIntegraContador, ErroProcuracao};
use crate::domain::integra_contador::procuração::{regra_procuracao_para_servico, ValidacaoProcuracao};
use crate::domain::integra_contador::servicos::{
    endpoint_para_servico, servico_e_bilhetado, servico_exige_procuracao, validar_servico,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::credencial::CredencialSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

/// Comando para executar uma operação do Integra Contador.
#[derive(Debug, Clone)]
pub struct ExecutarIntegraContadorCommand {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub contratante: Identificacao,
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    pub pedido_dados: PedidoDados,
    pub confirmacao_explicita: bool, // Obrigatório para verbos mutáveis
    pub chave_idempotencia: Option<String>, // Obrigatório para verbos mutáveis
}

/// Resultado da execução.
#[derive(Debug, Clone)]
pub struct ExecutarIntegraContadorResultado {
    pub response: IntegraContadorResponse,
    pub status_code: u16,
    pub estado: String,
    pub protocolo: Option<String>,
}

/// Use case principal para executar operações do Integra Contador.
#[derive(Clone)]
pub struct ExecutarIntegraContadorUseCase {
    cliente: Arc<dyn ClienteIntegraContador>,
    validador_procuracao: Arc<dyn ValidadorProcuracao>,
    gerenciador_tokens: Arc<dyn GerenciadorTokensSerpro>,
    assinador_termo: Arc<dyn AssinadorTermoAutorizacao>,
    credencial_provider: Arc<dyn CredencialIntegraContadorProvider>,
    idempotencia: Arc<dyn RepositorioIdempotencia>,
    auditoria: Arc<dyn RepositorioAuditoriaIntegraContador>,
    catalogo_clientes: Option<Arc<dyn CatalogoClientesPorDocumentoIntegra>>,
    catalogo_operadores: Option<Arc<dyn CatalogoOperadoresIntegra>>,
    operador_header_habilitado: bool,
}

impl ExecutarIntegraContadorUseCase {
    pub fn novo(
        cliente: Arc<dyn ClienteIntegraContador>,
        validador_procuracao: Arc<dyn ValidadorProcuracao>,
        gerenciador_tokens: Arc<dyn GerenciadorTokensSerpro>,
        assinador_termo: Arc<dyn AssinadorTermoAutorizacao>,
        credencial_provider: Arc<dyn CredencialIntegraContadorProvider>,
        idempotencia: Arc<dyn RepositorioIdempotencia>,
        auditoria: Arc<dyn RepositorioAuditoriaIntegraContador>,
    ) -> Self {
        Self {
            cliente,
            validador_procuracao,
            gerenciador_tokens,
            assinador_termo,
            credencial_provider,
            idempotencia,
            auditoria,
            catalogo_clientes: None,
            catalogo_operadores: None,
            operador_header_habilitado: false,
        }
    }

    pub fn com_catalogo_clientes(mut self, catalogo: Arc<dyn CatalogoClientesPorDocumentoIntegra>) -> Self {
        self.catalogo_clientes = Some(catalogo);
        self
    }

    pub fn com_operadores(mut self, catalogo: Arc<dyn CatalogoOperadoresIntegra>, habilitado: bool) -> Self {
        self.catalogo_operadores = Some(catalogo);
        self.operador_header_habilitado = habilitado;
        self
    }

    pub async fn executar(
        &self,
        comando: ExecutarIntegraContadorCommand,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<ExecutarIntegraContadorResultado, ErroIntegraContador> {
        // 1. Validar serviço no catálogo
        let servico_ref = validar_servico(&comando.id_sistema, &comando.id_servico, &comando.versao_sistema)
            .map_err(|e| ErroIntegraContador::ServicoNaoSuportado {
                id_sistema: comando.id_sistema.clone(),
                id_servico: comando.id_servico.clone(),
            })?;

        // 2. Verificar se verbo é compatível
        if !comando.verbo.as_str().eq_ignore_ascii_case(&servico_ref.verbo_padrao) {
            // Permitir verbos diferentes se explicitamente solicitado, mas logar
            tracing::warn!(
                verbo_solicitado = %comando.verbo,
                verbo_padrao = %servico_ref.verbo_padrao,
                "verbo diferente do padrão do serviço"
            );
        }

        // 3. Verificar se serviço está bloqueado pela allowlist (D4)
        if let Some(codigo) = crate::domain::integra_contador::procuração::CodigoServicoProcuracao::from_str(&comando.id_servico) {
            if codigo.esta_bloqueado_allowlist() {
                return Err(ErroIntegraContador::ServicoNaoSuportado {
                    id_sistema: comando.id_sistema.clone(),
                    id_servico: comando.id_servico.clone(),
                });
            }
        }

        // 4. Para verbos mutáveis, exigir confirmação explícita e idempotência
        if comando.verbo.e_mutavel() {
            if !comando.confirmacao_explicita {
                return Err(ErroIntegraContador::RequerConfirmacaoEIdempotencia);
            }
            if comando.chave_idempotencia.is_none() {
                return Err(ErroIntegraContador::RequerConfirmacaoEIdempotencia);
            }
        }

        // 5. Verificar idempotência para verbos mutáveis
        if comando.verbo.e_mutavel() {
            let chave = comando.chave_idempotencia.as_ref().unwrap();
            if let Some(resposta_cache) = self.idempotencia.verificar(comando.escritorio_id, chave).await? {
                return Err(ErroIntegraContador::IdempotenciaJaProcessada { chave: chave.clone() });
            }
        }

        // 6. Verificar necessidade de procuração
        let precisa_procuracao = self.precisa_procuracao(&comando, &servico_ref)?;
        let mut autenticar_procurador_token = None;

        if precisa_procuracao {
            // Validar procuração via OBTERPROCURACAO41
            let validacao = self.validador_procuracao.validar(
                &comando.contribuinte,
                &comando.autor_pedido_dados,
                &comando.id_servico,
                ambiente,
                request_id,
            ).await?;

            if !validacao.e_valida_agora(chrono::Utc::now()) {
                return Err(ErroIntegraContador::ProcuracaoInvalida {
                    motivo: "procuração vencida ou não iniciada".to_owned(),
                });
            }

            // Obter token de autenticar_procurador se autor ≠ contratante
            if comando.autor_pedido_dados.numero != comando.contratante.numero {
                // Verificar se já temos token válido em cache
                // Se não, precisamos assinar o Termo de Autorização (ENVIOXMLASSINADO81)
                let termo = self.construir_termo_autorizacao(&comando)?;
                let xml_assinado = self.assinador_termo.assinar(&termo, comando.escritorio_id).await?;
                
                autenticar_procurador_token = Some(
                    self.gerenciador_tokens.obter_autenticar_procurador_token(
                        comando.escritorio_id,
                        &xml_assinado,
                        ambiente,
                        request_id,
                    ).await?
                );
            }
        }

        // 7. Obter tokens de acesso (OAuth2/SAPI)
        let access_token = self.gerenciador_tokens.obter_access_token(
            comando.escritorio_id,
            "INTEGRA_CONTADOR",
            ambiente,
        ).await?;

        let jwt_token = self.gerenciador_tokens.obter_jwt_token(
            comando.escritorio_id,
            "INTEGRA_CONTADOR",
            ambiente,
        ).await?;

        // 8. Obter CPF do operador para header x-cpf-usuario
        let cpf_operador = if self.operador_header_habilitado {
            self.obter_cpf_operador(comando.usuario_id).await?
        } else {
            None
        };

        // 9. Construir request final
        let mut request = IntegraContadorRequest {
            contratante: comando.contratante.clone(),
            autor_pedido_dados: comando.autor_pedido_dados.clone(),
            contribuinte: comando.contribuinte.clone(),
            id_sistema: comando.id_sistema.clone(),
            id_servico: comando.id_servico.clone(),
            versao_sistema: comando.versao_sistema.clone(),
            pedido_dados: comando.pedido_dados.clone(),
            autenticar_procurador_token,
        };

        // 10. Registrar idempotência antes da execução (para verbos mutáveis)
        if comando.verbo.e_mutavel() {
            let payload_hash = self.calcular_hash_payload(&request)?;
            self.idempotencia.registrar(
                comando.escritorio_id,
                comando.chave_idempotencia.as_ref().unwrap(),
                comando.verbo,
                &comando.id_sistema,
                &comando.id_servico,
                ambiente,
                &payload_hash,
            ).await?;
        }

        // 11. Executar a chamada
        let response = self.cliente.executar(&request, comando.verbo, ambiente, request_id).await?;

        // 12. Determinar status code e estado baseado na resposta
        let (status_code, estado, protocolo) = self.interpretar_resposta(&response)?;

        // 13. Finalizar idempotência
        if comando.verbo.e_mutavel() {
            self.idempotencia.finalizar(
                comando.escritorio_id,
                comando.chave_idempotencia.as_ref().unwrap(),
                &response,
                Some(status_code),
            ).await?;
        }

        // 14. Registrar auditoria
        self.registrar_auditoria(
            &comando,
            ambiente,
            &response,
            status_code,
            &estado,
            protocolo.as_deref(),
            request_id,
        ).await?;

        Ok(ExecutarIntegraContadorResultado {
            response,
            status_code,
            estado,
            protocolo,
        })
    }

    fn precisa_procuracao(
        &self,
        comando: &ExecutarIntegraContadorCommand,
        servico_ref: &crate::domain::integra_contador::servicos::ServicoReferencia,
    ) -> Result<bool, ErroIntegraContador> {
        // Se autor == contribuinte, não precisa de procuração
        if comando.autor_pedido_dados.numero == comando.contribuinte.numero {
            return Ok(false);
        }

        // Se serviço não exige procuração, não precisa
        if !servico_ref.exige_procuracao {
            return Ok(false);
        }

        // Verificar regra de procuração do catálogo
        if let Some(regra) = regra_procuracao_para_servico(&comando.id_servico) {
            if !regra.se_aplica() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn construir_termo_autorizacao(
        &self,
        comando: &ExecutarIntegraContadorCommand,
    ) -> Result<TermoAutorizacao, ErroIntegraContador> {
        let agora = chrono::Utc::now();
        let data_assinatura = agora.format("%Y%m%d").to_string();
        let vigencia = (agora + chrono::Duration::days(365)).format("%Y%m%d").to_string();

        Ok(TermoAutorizacao::novo_oficial(
            comando.contratante.numero.clone(),
            comando.contratante.nome.clone(),
            comando.autor_pedido_dados.numero.clone(),
            comando.autor_pedido_dados.nome.clone(),
            comando.autor_pedido_dados.tipo.clone(),
            data_assinatura,
            vigencia,
        ))
    }

    async fn obter_cpf_operador(&self, usuario_id: Uuid) -> Result<Option<String>, ErroIntegraContador> {
        let catalogo = self.catalogo_operadores.as_ref().ok_or_else(|| {
            ErroIntegraContador::ErroInterno("catálogo de operadores não configurado".to_owned())
        })?;
        catalogo.buscar_cpf(usuario_id).await
            .map_err(|e| ErroIntegraContador::ErroInterno(e.to_string()))?
            .map(Some)
            .ok_or_else(|| ErroIntegraContador::ErroInterno("CPF do operador não encontrado".to_owned()))
    }

    fn calcular_hash_payload(&self, request: &IntegraContadorRequest) -> Result<String, ErroIntegraContador> {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_vec(request)
            .map_err(|e| ErroIntegraContador::ErroInterno(e.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(&json);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn interpretar_resposta(
        &self,
        response: &IntegraContadorResponse,
    ) -> Result<(u16, String, Option<String>), ErroIntegraContador> {
        match response.status.as_str() {
            "200" | "SUCESSO" | "OK" => Ok((200, "SUCESSO".to_owned(), None)),
            "202" | "PROCESSAMENTO_ASSINCRONO" => {
                let tempo = response.tempo_espera.unwrap_or(30);
                Err(ErroIntegraContador::ProcessamentoAssincrono { tempo_espera: tempo })
            }
            "204" | "PROCESSAMENTO_NAO_CONCLUIDO" => {
                Err(ErroIntegraContador::ProcessamentoNaoConcluido)
            }
            "304" | "TOKEN_VALIDO" => {
                Err(ErroIntegraContador::TokenProcuradorValido)
            }
            "400" | "PAYLOAD_INVALIDO" => Ok((400, "PAYLOAD_INVALIDO".to_owned(), None)),
            "401" | "NAO_AUTORIZADO" => Ok((401, "NAO_AUTORIZADO".to_owned(), None)),
            "403" | "ACESSO_NEGADO" => {
                let motivo = response.mensagens.join("; ");
                Err(ErroIntegraContador::AcessoNegadoSerpro { motivo })
            }
            "404" | "NAO_ENCONTRADO" => Ok((404, "NAO_ENCONTRADO".to_owned(), None)),
            "429" | "RATE_LIMIT" => {
                let retry_after = response.tempo_espera.unwrap_or(60) as u64;
                Err(ErroIntegraContador::RateLimitExcedido { retry_after })
            }
            "500" | "ERRO_INTERNO" => Ok((500, "ERRO_INTERNO".to_owned(), None)),
            "503" | "INDISPONIVEL" => {
                let motivo = response.mensagens.join("; ");
                Err(ErroIntegraContador::ServicoIndisponivel { motivo })
            }
            "504" | "INDETERMINADO" => {
                Err(ErroIntegraContador::EstadoIndeterminado)
            }
            _ => {
                // Tentar extrair protocolo se houver
                let protocolo = response.dados.as_ref().and_then(|d| {
                    serde_json::from_str::<serde_json::Value>(d).ok()
                        .and_then(|v| v.get("protocolo").and_then(|p| p.as_str().map(|s| s.to_owned())))
                });
                Ok((200, "SUCESSO".to_owned(), protocolo))
            }
        }
    }

    async fn registrar_auditoria(
        &self,
        comando: &ExecutarIntegraContadorCommand,
        ambiente: AmbienteSerpro,
        response: &IntegraContadorResponse,
        status_code: u16,
        estado: &str,
        protocolo: Option<&str>,
        request_id: &str,
    ) -> Result<(), ErroIntegraContador> {
        let cliente_id = if let Some(catalogo) = &self.catalogo_clientes {
            catalogo.buscar_por_documento(comando.escritorio_id, &comando.contribuinte.numero).await
                .map_err(|e| ErroIntegraContador::ErroInterno(e.to_string()))?
        } else {
            comando.cliente_id
        };

        let request_redigido = serde_json::json!({
            "idSistema": comando.id_sistema,
            "idServico": comando.id_servico,
            "verbo": comando.verbo.as_str(),
            "contribuinteMascarado": mascarar_documento(&comando.contribuinte.numero),
            "autorPedidoDados": comando.autor_pedido_dados.numero.chars().take(3).collect::<String>() + "***",
            "requestId": request_id,
        });

        let response_redigido = serde_json::json!({
            "status": response.status,
            "mensagens": response.mensagens,
            "tempoEspera": response.tempo_espera,
            "temDados": response.dados.is_some(),
        });

        let (erro_codigo, erro_mensagem) = if status_code >= 400 {
            (Some(response.status.clone()), Some(response.mensagens.join("; ")))
        } else {
            (None, None)
        };

        self.auditoria.registrar(
            comando.escritorio_id,
            cliente_id,
            comando.verbo,
            &comando.id_sistema,
            &comando.id_servico,
            &comando.versao_sistema,
            ambiente,
            &comando.autor_pedido_dados.numero,
            &comando.contribuinte.numero,
            Some(status_code),
            estado,
            protocolo.map(|s| s.to_owned()),
            request_redigido,
            Some(response_redigido),
            erro_codigo,
            erro_mensagem,
        ).await.map_err(|e| ErroIntegraContador::AuditoriaFalhou(e.to_string()))
    }
}

/// Mascara CPF/CNPJ para logs/auditoria.
fn mascarar_documento(doc: &str) -> String {
    if doc.len() <= 4 {
        return "*".repeat(doc.len());
    }
    let visiveis = 3;
    let mascarados = doc.len() - visiveis;
    format!("{}{}", &doc[..visiveis], "*".repeat(mascarados))
}

/// Use case para consultar procuração (OBTERPROCURACAO41).
#[derive(Clone)]
pub struct ConsultarProcuracaoUseCase {
    cliente: Arc<dyn ClienteIntegraContador>,
    gerenciador_tokens: Arc<dyn GerenciadorTokensSerpro>,
    credencial_provider: Arc<dyn CredencialIntegraContadorProvider>,
}

impl ConsultarProcuracaoUseCase {
    pub fn novo(
        cliente: Arc<dyn ClienteIntegraContador>,
        gerenciador_tokens: Arc<dyn GerenciadorTokensSerpro>,
        credencial_provider: Arc<dyn CredencialIntegraContadorProvider>,
    ) -> Self {
        Self { cliente, gerenciador_tokens, credencial_provider }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
        contribuinte: &Identificacao,
        procurador: &Identificacao,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<ValidacaoProcuracao, ErroProcuracao> {
        // Construir request para OBTERPROCURACAO41
        let request = IntegraContadorRequest {
            contratante: procurador.clone(),
            autor_pedido_dados: procurador.clone(),
            contribuinte: contribuinte.clone(),
            id_sistema: "PROCURACOES".to_owned(),
            id_servico: "OBTERPROCURACAO41".to_owned(),
            versao_sistema: "1".to_owned(),
            pedido_dados: PedidoDados {
                dados: serde_json::json!({
                    "outorgante": contribuinte.numero,
                    "tipoOutorgante": if contribuinte.tipo == "PF" { "1" } else { "2" },
                    "outorgado": procurador.numero,
                    "tipoOutorgado": if procurador.tipo == "PF" { "1" } else { "2" },
                }).to_string(),
            },
            autenticar_procurador_token: None,
        };

        let response = self.cliente.executar(&request, VerboIntegraContador::Consultar, ambiente, request_id).await
            .map_err(|e| ErroProcuracao::NaoVerificavel { motivo: e.to_string() })?;

        // Parse da resposta para ValidacaoProcuracao
        // A estrutura exata depende do contrato oficial
        self.parsear_resposta_procuracao(&response, contribuinte, procurador, codigo_servico, ambiente)
    }

    fn parsear_resposta_procuracao(
        &self,
        response: &IntegraContadorResponse,
        contribuinte: &Identificacao,
        procurador: &Identificacao,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<ValidacaoProcuracao, ErroProcuracao> {
        // Implementação simplificada - a estrutura real depende do contrato
        if let Some(dados_str) = &response.dados {
            if let Ok(dados) = serde_json::from_str::<serde_json::Value>(dados_str) {
                let status = dados.get("situacao").and_then(|s| s.as_str()).unwrap_or("DESCONHECIDO");
                let valido_de = dados.get("validoDe").and_then(|s| s.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc));
                let valido_ate = dados.get("validoAte").and_then(|s| s.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc));

                return Ok(ValidacaoProcuracao {
                    contribuinte_cpf_cnpj: contribuinte.numero.clone(),
                    procurador_cpf_cnpj: procurador.numero.clone(),
                    codigo_servico: codigo_servico.to_owned(),
                    status: status.to_owned(),
                    valido_de,
                    valido_ate,
                    verificado_em: chrono::Utc::now(),
                    ambiente: ambiente.as_str().to_owned(),
                });
            }
        }

        Err(ErroProcuracao::NaoVerificavel {
            motivo: "resposta da SERPRO não contém dados de procuração válidos".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::integra_contador::envelope::VerboIntegraContador;

    #[test]
    fn verbo_mutavel_exige_confirmacao() {
        assert!(VerboIntegraContador::Declarar.e_mutavel());
        assert!(VerboIntegraContador::Emitir.e_mutavel());
        assert!(!VerboIntegraContador::Consultar.e_mutavel());
    }

    #[test]
    fn mascarar_documento_funciona() {
        assert_eq!(mascarar_documento("12345678901"), "123********");
        assert_eq!(mascarar_documento("12345678000190"), "123************");
        assert_eq!(mascarar_documento("123"), "***");
    }
}