use std::sync::Arc;

use secrecy::SecretString;

use crate::application::serpro::ports::{
    AlterarModoTrialError, AlterarModoTrialSerproCommand, AlterarModoTrialSerproResultado,
    AuditoriaSerproRepositorio, CertificadosSerproConsulta, CriptografiaSerpro,
    EscritorioSerproRepositorio, EventoAuditoriaSerpro, InvalicadorCachesSerpro,
    ObterConfiguracaoError, ObterConfiguracaoSerproCommand, ObterConfiguracaoSerproResultado,
    RepositorioError,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::credencial::CredencialSerpro;
use crate::domain::serpro::erros::{ErroAmbiente, ErroCredencial};
use crate::domain::serpro::ports::{SerproAmbienteResolver, SerproCredentialProvider};
use crate::domain::serpro::produto::ProdutoSerpro;

#[derive(Clone)]
pub struct ObterConfiguracaoSerproUseCase {
    escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
}

impl ObterConfiguracaoSerproUseCase {
    pub fn novo(escritorio_repo: Arc<dyn EscritorioSerproRepositorio>) -> Self {
        Self { escritorio_repo }
    }

    pub async fn executar(
        &self,
        comando: ObterConfiguracaoSerproCommand,
    ) -> Result<ObterConfiguracaoSerproResultado, ObterConfiguracaoError> {
        let (modo_trial, versao, atualizado_em) = self
            .escritorio_repo
            .buscar_modo_trial(comando.escritorio_id)?
            .ok_or(ObterConfiguracaoError::EscritorioNaoEncontrado)?;

        Ok(ObterConfiguracaoSerproResultado {
            escritorio_id: comando.escritorio_id,
            modo_trial_serpro: modo_trial,
            ambiente_efetivo: AmbienteSerpro::from_bool(modo_trial),
            versao,
            atualizado_em,
        })
    }
}

#[derive(Clone)]
pub struct AlterarModoTrialSerproUseCase {
    escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
    certificados: Arc<dyn CertificadosSerproConsulta>,
    auditoria: Arc<dyn AuditoriaSerproRepositorio>,
    caches: Arc<dyn InvalicadorCachesSerpro>,
}

impl AlterarModoTrialSerproUseCase {
    pub fn novo(
        escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
        certificados: Arc<dyn CertificadosSerproConsulta>,
        auditoria: Arc<dyn AuditoriaSerproRepositorio>,
        caches: Arc<dyn InvalicadorCachesSerpro>,
    ) -> Self {
        Self {
            escritorio_repo,
            certificados,
            auditoria,
            caches,
        }
    }

    pub async fn executar(
        &self,
        comando: AlterarModoTrialSerproCommand,
    ) -> Result<AlterarModoTrialSerproResultado, AlterarModoTrialError> {
        // Existência vem antes de qualquer validação: escritório alvo
        // inexistente é 404 (SPEC §10.1), não "sem credencial" (400).
        let estado = self
            .escritorio_repo
            .buscar_modo_trial(comando.escritorio_id)
            .map_err(AlterarModoTrialError::Repositorio)?;
        if estado.is_none() {
            return Err(AlterarModoTrialError::Ambiente(
                ErroAmbiente::EscritorioNaoEncontrado(comando.escritorio_id),
            ));
        }

        if !comando.modo_trial_serpro {
            self.validar_configuracao_producao(comando.escritorio_id)
                .await?;
        }

        let (versao_nova, atualizado_em) = self
            .escritorio_repo
            .atualizar_modo_trial(
                comando.escritorio_id,
                comando.modo_trial_serpro,
                comando.versao_esperada,
            )
            .map_err(|erro| match erro {
                RepositorioError::Concorrencia => AlterarModoTrialError::Ambiente(
                    self.conflito_de_versao(comando.escritorio_id, comando.versao_esperada),
                ),
                RepositorioError::NaoEncontrado => AlterarModoTrialError::Ambiente(
                    ErroAmbiente::EscritorioNaoEncontrado(comando.escritorio_id),
                ),
                _ => AlterarModoTrialError::Repositorio(erro),
            })?;

        self.auditoria
            .registrar(&EventoAuditoriaSerpro {
                escritorio_id: comando.escritorio_id,
                administrador_id: comando.administrador_id,
                acao: "SERPRO_MODO_TRIAL_ALTERADO",
                entidade: "escritorios",
                entidade_id: comando.escritorio_id,
                valor_anterior: Some(serde_json::json!({
                    "modo_trial_serpro": !comando.modo_trial_serpro,
                })),
                valor_novo: Some(serde_json::json!({
                    "modo_trial_serpro": comando.modo_trial_serpro,
                })),
                ip: comando.ip,
            })
            .map_err(|erro| AlterarModoTrialError::Auditoria(erro.to_string()))?;

        // ADR-0005 §6: invalida caches voláteis do escritório após trocar o
        // ambiente. Falha aqui é logada e não desfaz a troca: tokens são
        // persistidos particionados por ambiente, então um token de ambiente
        // antigo não cruza a fronteira.
        if let Err(erro) = self.caches.invalidar_todos_do_tenant(comando.escritorio_id) {
            tracing::warn!(
                escritorio = %comando.escritorio_id,
                erro = %erro,
                "falha ao invalidar caches SERPRO após troca de ambiente"
            );
        }

        Ok(AlterarModoTrialSerproResultado {
            modo_trial_serpro: comando.modo_trial_serpro,
            ambiente_efetivo: AmbienteSerpro::from_bool(comando.modo_trial_serpro),
            versao_atual: versao_nova,
            atualizado_em,
        })
    }

    async fn validar_configuracao_producao(
        &self,
        escritorio_id: uuid::Uuid,
    ) -> Result<(), AlterarModoTrialError> {
        // ADR-0005 §5: produção exige credencial do Integra Contador. A
        // Dívida Ativa de produção é bloqueada por decisão própria (D5,
        // URL de produção indefinida) e NÃO pode exigir credencial aqui,
        // senão o toggle se torna impossível de ligar.
        let credencial_integra = self
            .escritorio_repo
            .buscar_credencial(
                escritorio_id,
                ProdutoSerpro::IntegraContador,
                AmbienteSerpro::Producao,
            )
            .map_err(AlterarModoTrialError::Repositorio)?;

        if credencial_integra.is_none() {
            return Err(AlterarModoTrialError::Ambiente(
                ErroAmbiente::ProducaoBloqueada(
                    "credencial INTEGRA_CONTADOR de produção não configurada; \
                     cadastre-a antes de desativar o modo trial"
                        .to_owned(),
                ),
            ));
        }

        // O toggle muda o ambiente para todos os produtos (globais por
        // produto/ambiente); consultas de produção incluem chamadas com
        // mTLS, então um certificado A1 dentro do prazo também é exigido.
        let agora = chrono::Utc::now();
        let certificado = self
            .certificados
            .buscar_certificado_em_validade(escritorio_id, agora)
            .map_err(AlterarModoTrialError::Repositorio)?;

        if certificado.is_none() {
            return Err(AlterarModoTrialError::Ambiente(
                ErroAmbiente::ProducaoBloqueada(
                    "certificado A1 ativo não encontrado ou fora do prazo de validade".to_owned(),
                ),
            ));
        }

        Ok(())
    }

    fn conflito_de_versao(&self, escritorio_id: uuid::Uuid, esperada: i64) -> ErroAmbiente {
        match self.escritorio_repo.buscar_modo_trial(escritorio_id) {
            Ok(Some((_, atual, _))) => ErroAmbiente::ConflitoVersao {
                esperado: esperada,
                atual,
            },
            _ => ErroAmbiente::RepositorioIndisponivel,
        }
    }
}

/// Credencial global de demonstração (decisão D6, "sem fallback silencioso").
/// O ponto de composição (`routes::construir`) só fornece este fallback
/// quando `APP_ENV=development` e as globais `SERPRO_CONSUMER_KEY/SECRET`
/// estão presentes; caso contrário ele não existe e escritórios sem
/// credencial TRIAL própria recebem erro explícito.
#[derive(Clone)]
pub struct CredencialFallbackGlobal {
    pub consumer_key: SecretString,
    pub consumer_secret: SecretString,
}

#[derive(Clone)]
pub struct SerproCredentialProviderImpl {
    escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
    criptografia: Arc<dyn CriptografiaSerpro>,
    fallback_trial_dev: Option<CredencialFallbackGlobal>,
}

impl SerproCredentialProviderImpl {
    pub fn novo(
        escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
        criptografia: Arc<dyn CriptografiaSerpro>,
        fallback_trial_dev: Option<CredencialFallbackGlobal>,
    ) -> Self {
        Self {
            escritorio_repo,
            criptografia,
            fallback_trial_dev,
        }
    }
}

#[async_trait::async_trait]
impl SerproCredentialProvider for SerproCredentialProviderImpl {
    async fn resolver(
        &self,
        escritorio_id: uuid::Uuid,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<CredencialSerpro, ErroCredencial> {
        let armazenada = self
            .escritorio_repo
            .buscar_credencial(escritorio_id, produto, ambiente)
            .map_err(|_| ErroCredencial::RepositorioIndisponivel)?;

        let Some(credencial) = armazenada else {
            // Fallback global vale apenas para TRIAL — e só quando o ponto
            // de composição o fornece (dev); produção sempre falha claro.
            if let (AmbienteSerpro::Trial, Some(fallback)) =
                (ambiente, self.fallback_trial_dev.as_ref())
            {
                return Ok(CredencialSerpro {
                    escritorio_id,
                    produto,
                    ambiente,
                    consumer_key: fallback.consumer_key.clone(),
                    consumer_secret: fallback.consumer_secret.clone(),
                    certificado: None,
                });
            }
            return Err(ErroCredencial::NaoEncontrada {
                escritorio_id,
                produto: produto.as_str().to_owned(),
                ambiente: ambiente.as_str().to_owned(),
            });
        };

        let consumer_key = self
            .criptografia
            .decifrar(&credencial.consumer_key_cifrado)
            .map_err(|erro| ErroCredencial::Criptografia(erro.to_string()))?;

        let consumer_secret = self
            .criptografia
            .decifrar(&credencial.consumer_secret_cifrado)
            .map_err(|erro| ErroCredencial::Criptografia(erro.to_string()))?;

        Ok(CredencialSerpro {
            escritorio_id,
            produto,
            ambiente,
            consumer_key: string_de_bytes(&consumer_key),
            consumer_secret: string_de_bytes(&consumer_secret),
            certificado: None,
        })
    }
}

fn string_de_bytes(bytes: &[u8]) -> SecretString {
    let texto = std::str::from_utf8(bytes)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned());
    SecretString::from(texto)
}

#[derive(Clone)]
pub struct SerproAmbienteResolverImpl {
    escritorio_repo: Arc<dyn EscritorioSerproRepositorio>,
}

impl SerproAmbienteResolverImpl {
    pub fn novo(escritorio_repo: Arc<dyn EscritorioSerproRepositorio>) -> Self {
        Self { escritorio_repo }
    }
}

#[async_trait::async_trait]
impl SerproAmbienteResolver for SerproAmbienteResolverImpl {
    async fn resolver_ambiente(
        &self,
        escritorio_id: uuid::Uuid,
    ) -> Result<AmbienteSerpro, ErroAmbiente> {
        let modo_trial = self.obter_modo_trial(escritorio_id).await?;
        Ok(AmbienteSerpro::from_bool(modo_trial))
    }

    async fn obter_modo_trial(&self, escritorio_id: uuid::Uuid) -> Result<bool, ErroAmbiente> {
        let mensagem = self
            .escritorio_repo
            .buscar_modo_trial(escritorio_id)
            .map_err(|_| ErroAmbiente::RepositorioIndisponivel)?
            .ok_or(ErroAmbiente::EscritorioNaoEncontrado(escritorio_id))?;
        Ok(mensagem.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::serpro::ports::{
        CertificadoAtivoResumo, CredencialArmazenada, CriptografiaError, ObterConfiguracaoError,
    };
    use secrecy::ExposeSecret;
    use std::sync::Mutex;
    use uuid::Uuid;

    const ESC: Uuid = Uuid::nil();

    struct BancoFalso {
        estado: Mutex<Option<(bool, i64)>>,
        credenciais: Mutex<Vec<(ProdutoSerpro, AmbienteSerpro)>>,
        certificado_valido: bool,
        auditoria_falha: bool,
        cache_falha: bool,
        auditorias: Mutex<Vec<EventoAuditoriaSerpro>>,
        invalidacoes: Mutex<Vec<Uuid>>,
    }

    impl Default for BancoFalso {
        fn default() -> Self {
            Self {
                estado: Mutex::new(Some((true, 1))),
                credenciais: Mutex::new(Vec::new()),
                certificado_valido: false,
                auditoria_falha: false,
                cache_falha: false,
                auditorias: Mutex::new(Vec::new()),
                invalidacoes: Mutex::new(Vec::new()),
            }
        }
    }

    impl BancoFalso {
        fn nenhum_escritorio(self) -> Self {
            *self.estado.lock().unwrap() = None;
            self
        }

        fn em_producao_versao(self, versao: i64) -> Self {
            *self.estado.lock().unwrap() = Some((false, versao));
            self
        }

        fn versao_divergente(self) -> Self {
            *self.estado.lock().unwrap() = Some((true, 42));
            self
        }

        fn com_credencial_integra_prod(self) -> Self {
            self.credenciais
                .lock()
                .unwrap()
                .push((ProdutoSerpro::IntegraContador, AmbienteSerpro::Producao));
            self
        }

        fn com_certificado_valido(mut self) -> Self {
            self.certificado_valido = true;
            self
        }

        fn com_auditoria_falha(mut self) -> Self {
            self.auditoria_falha = true;
            self
        }

        fn com_cache_falha(mut self) -> Self {
            self.cache_falha = true;
            self
        }
    }

    impl EscritorioSerproRepositorio for BancoFalso {
        fn buscar_modo_trial(
            &self,
            _escritorio_id: Uuid,
        ) -> Result<Option<(bool, i64, chrono::DateTime<chrono::Utc>)>, RepositorioError> {
            Ok((*self.estado.lock().unwrap()).map(|(m, v)| (m, v, chrono::Utc::now())))
        }

        fn atualizar_modo_trial(
            &self,
            _escritorio_id: Uuid,
            modo_trial: bool,
            versao_esperada: i64,
        ) -> Result<(i64, chrono::DateTime<chrono::Utc>), RepositorioError> {
            let mut escritorio = self.estado.lock().unwrap();
            match *escritorio {
                None => Err(RepositorioError::NaoEncontrado),
                Some((_, v)) => {
                    if v != versao_esperada {
                        return Err(RepositorioError::Concorrencia);
                    }
                    *escritorio = Some((modo_trial, v + 1));
                    Ok((v + 1, chrono::Utc::now()))
                }
            }
        }

        fn buscar_credencial(
            &self,
            _escritorio_id: Uuid,
            produto: ProdutoSerpro,
            ambiente: AmbienteSerpro,
        ) -> Result<Option<CredencialArmazenada>, RepositorioError> {
            let existe = self
                .credenciais
                .lock()
                .unwrap()
                .contains(&(produto, ambiente));
            if !existe {
                return Ok(None);
            }
            Ok(Some(CredencialArmazenada {
                id: Uuid::now_v7(),
                escritorio_id: ESC,
                produto,
                ambiente,
                consumer_key_cifrado: b"chave-cifrada".to_vec(),
                consumer_secret_cifrado: b"secreto-cifrado".to_vec(),
                criado_em: chrono::Utc::now(),
                atualizado_em: chrono::Utc::now(),
            }))
        }
    }

    impl CertificadosSerproConsulta for BancoFalso {
        fn buscar_certificado_em_validade(
            &self,
            _escritorio_id: Uuid,
            _agora: chrono::DateTime<chrono::Utc>,
        ) -> Result<Option<CertificadoAtivoResumo>, RepositorioError> {
            if self.certificado_valido {
                Ok(Some(CertificadoAtivoResumo {
                    id: Uuid::now_v7(),
                    fingerprint_sha256: "ab".repeat(32),
                    valido_ate: chrono::Utc::now() + chrono::Duration::days(30),
                }))
            } else {
                Ok(None)
            }
        }
    }

    impl AuditoriaSerproRepositorio for BancoFalso {
        fn registrar(&self, evento: &EventoAuditoriaSerpro) -> Result<(), RepositorioError> {
            if self.auditoria_falha {
                return Err(RepositorioError::Dados);
            }
            self.auditorias.lock().unwrap().push(evento.clone());
            Ok(())
        }
    }

    impl InvalicadorCachesSerpro for BancoFalso {
        fn invalidar_todos_do_tenant(&self, escritorio_id: Uuid) -> Result<(), RepositorioError> {
            if self.cache_falha {
                return Err(RepositorioError::Indisponivel);
            }
            self.invalidacoes.lock().unwrap().push(escritorio_id);
            Ok(())
        }
    }

    #[allow(clippy::type_complexity)]
    fn use_case(banco: &Arc<BancoFalso>) -> AlterarModoTrialSerproUseCase {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = banco.clone();
        let certificados: Arc<dyn CertificadosSerproConsulta> = banco.clone();
        let auditorias: Arc<dyn AuditoriaSerproRepositorio> = banco.clone();
        let caches: Arc<dyn InvalicadorCachesSerpro> = banco.clone();
        AlterarModoTrialSerproUseCase::novo(escritorios, certificados, auditorias, caches)
    }

    fn comando_producao(versao_esperada: i64) -> AlterarModoTrialSerproCommand {
        AlterarModoTrialSerproCommand {
            escritorio_id: ESC,
            modo_trial_serpro: false,
            administrador_id: ESC,
            ip: None,
            versao_esperada,
        }
    }

    fn comando_trial(versao_esperada: i64) -> AlterarModoTrialSerproCommand {
        AlterarModoTrialSerproCommand {
            escritorio_id: ESC,
            modo_trial_serpro: true,
            administrador_id: ESC,
            ip: None,
            versao_esperada,
        }
    }

    #[tokio::test]
    async fn obter_configuracao_retorna_modo_trial() {
        let banco = BancoFalso::default();
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(banco);
        let use_case = ObterConfiguracaoSerproUseCase::novo(escritorios);

        let resultado = use_case
            .executar(ObterConfiguracaoSerproCommand { escritorio_id: ESC })
            .await
            .unwrap();

        assert!(resultado.modo_trial_serpro);
        assert_eq!(resultado.ambiente_efetivo, AmbienteSerpro::Trial);
        assert_eq!(resultado.versao, 1);
    }

    #[tokio::test]
    async fn obter_configuracao_com_escritorio_ausente_falha() {
        let banco = BancoFalso::default().nenhum_escritorio();
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(banco);
        let use_case = ObterConfiguracaoSerproUseCase::novo(escritorios);

        assert!(matches!(
            use_case
                .executar(ObterConfiguracaoSerproCommand { escritorio_id: ESC })
                .await,
            Err(ObterConfiguracaoError::EscritorioNaoEncontrado)
        ));
    }

    #[tokio::test]
    async fn alterar_para_producao_sem_credencial_bloqueia_e_nao_persiste_nada() {
        let banco = Arc::new(BancoFalso::default());
        let use_case = use_case(&banco);

        let erro = use_case.executar(comando_producao(1)).await.unwrap_err();

        if let AlterarModoTrialError::Ambiente(ErroAmbiente::ProducaoBloqueada(msg)) = erro {
            assert!(msg.contains("INTEGRA_CONTADOR"));
        } else {
            panic!("esperava ProducaoBloqueada por credencial ausente, veio {erro:?}");
        }
        assert_eq!(banco.estado.lock().unwrap().expect("estado"), (true, 1));
        assert!(banco.auditorias.lock().unwrap().is_empty());
        assert!(banco.invalidacoes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn alterar_em_escritorio_inexistente_falha_com_404_antes_de_validar() {
        let banco = Arc::new(BancoFalso::default().nenhum_escritorio());
        let use_case = use_case(&banco);

        let erro = use_case.executar(comando_producao(1)).await.unwrap_err();

        assert!(matches!(
            erro,
            AlterarModoTrialError::Ambiente(ErroAmbiente::EscritorioNaoEncontrado(_))
        ));
    }

    #[tokio::test]
    async fn alterar_para_producao_com_credencial_sem_certificado_bloqueia() {
        let banco = Arc::new(BancoFalso::default().com_credencial_integra_prod());
        let use_case = use_case(&banco);

        let erro = use_case.executar(comando_producao(1)).await.unwrap_err();

        if let AlterarModoTrialError::Ambiente(ErroAmbiente::ProducaoBloqueada(msg)) = erro {
            assert!(msg.contains("certificado A1"));
        } else {
            panic!("esperava ProducaoBloqueada por certificado ausente, veio {erro:?}");
        }
        assert_eq!(banco.estado.lock().unwrap().expect("estado"), (true, 1));
    }

    #[tokio::test]
    async fn alterar_para_producao_com_requisitos_sucede_com_auditoria_e_cache() {
        let banco = Arc::new(
            BancoFalso::default()
                .com_credencial_integra_prod()
                .com_certificado_valido(),
        );
        let use_case = use_case(&banco);

        let resultado = use_case.executar(comando_producao(1)).await.unwrap();

        assert!(!resultado.modo_trial_serpro);
        assert_eq!(resultado.ambiente_efetivo, AmbienteSerpro::Producao);
        assert_eq!(resultado.versao_atual, 2);

        let eventos = banco.auditorias.lock().unwrap();
        assert_eq!(eventos.len(), 1);
        assert_eq!(eventos[0].acao, "SERPRO_MODO_TRIAL_ALTERADO");
        assert_eq!(eventos[0].entidade, "escritorios");
        assert_eq!(
            eventos[0].valor_anterior,
            Some(serde_json::json!({ "modo_trial_serpro": true }))
        );
        assert_eq!(
            eventos[0].valor_novo,
            Some(serde_json::json!({ "modo_trial_serpro": false }))
        );
        drop(eventos);
        assert_eq!(*banco.invalidacoes.lock().unwrap(), vec![ESC]);
    }

    #[tokio::test]
    async fn alterar_para_trial_sucede_sem_credencial_de_producao() {
        let banco = Arc::new(BancoFalso::default().em_producao_versao(3));
        let use_case = use_case(&banco);

        let resultado = use_case.executar(comando_trial(3)).await.unwrap();

        assert!(resultado.modo_trial_serpro);
        assert_eq!(resultado.ambiente_efetivo, AmbienteSerpro::Trial);
        assert_eq!(resultado.versao_atual, 4);
        assert_eq!(*banco.invalidacoes.lock().unwrap(), vec![ESC]);
    }

    #[tokio::test]
    async fn alterar_com_versao_divergente_retorna_conflito_descritivo() {
        let banco = Arc::new(BancoFalso::default().versao_divergente());
        let use_case = use_case(&banco);

        let erro = use_case.executar(comando_trial(1)).await.unwrap_err();

        assert!(matches!(
            erro,
            AlterarModoTrialError::Ambiente(ErroAmbiente::ConflitoVersao {
                esperado: 1,
                atual: 42
            })
        ));
        // Troca não aconteceu.
        assert_eq!(banco.estado.lock().unwrap().expect("estado"), (true, 42));
    }

    #[tokio::test]
    async fn alterar_em_escritorio_ausente_retorna_nao_encontrado() {
        let banco = Arc::new(BancoFalso::default().nenhum_escritorio());
        let use_case = use_case(&banco);

        let erro = use_case.executar(comando_trial(1)).await.unwrap_err();

        assert!(matches!(
            erro,
            AlterarModoTrialError::Ambiente(ErroAmbiente::EscritorioNaoEncontrado(_))
        ));
    }

    #[tokio::test]
    async fn falha_de_auditoria_propaga_erro_e_impede_cache() {
        let banco = Arc::new(
            BancoFalso::default()
                .com_credencial_integra_prod()
                .com_certificado_valido()
                .com_auditoria_falha(),
        );
        let use_case = use_case(&banco);

        let resultado = use_case.executar(comando_producao(1)).await;

        assert!(matches!(
            resultado,
            Err(AlterarModoTrialError::Auditoria(_))
        ));
        assert!(banco.invalidacoes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn falha_de_invalidacao_nao_desfaz_a_troca() {
        let banco = Arc::new(
            BancoFalso::default()
                .com_credencial_integra_prod()
                .com_certificado_valido()
                .com_cache_falha(),
        );
        let use_case = use_case(&banco);

        let resultado = use_case.executar(comando_producao(1)).await;

        assert!(resultado.is_ok());
        assert_eq!(banco.estado.lock().unwrap().expect("estado"), (false, 2));
        assert!(!banco.auditorias.lock().unwrap().is_empty());
    }

    struct CriptoIdentidade;

    impl CriptografiaSerpro for CriptoIdentidade {
        fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CriptografiaError> {
            Ok(dados.to_vec())
        }

        fn decifrar(
            &self,
            dados_cifrados: &[u8],
        ) -> Result<zeroize::Zeroizing<Vec<u8>>, CriptografiaError> {
            Ok(zeroize::Zeroizing::new(dados_cifrados.to_vec()))
        }
    }

    #[tokio::test]
    async fn provider_decifra_credencial_do_escritorio() {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(
            BancoFalso::default()
                .com_credencial_integra_prod()
                .em_producao_versao(1),
        );
        let provider =
            SerproCredentialProviderImpl::novo(escritorios, Arc::new(CriptoIdentidade), None);

        let resultado = provider
            .resolver(
                ESC,
                ProdutoSerpro::IntegraContador,
                AmbienteSerpro::Producao,
            )
            .await
            .unwrap();

        assert_eq!(resultado.produto, ProdutoSerpro::IntegraContador);
        assert_eq!(resultado.ambiente, AmbienteSerpro::Producao);
        assert_eq!(resultado.consumer_key.expose_secret(), "chave-cifrada");
        assert_eq!(resultado.consumer_secret.expose_secret(), "secreto-cifrado");
    }

    #[tokio::test]
    async fn provider_sem_credencial_falha_claro() {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(BancoFalso::default());
        let provider =
            SerproCredentialProviderImpl::novo(escritorios, Arc::new(CriptoIdentidade), None);

        let erro = provider
            .resolver(
                ESC,
                ProdutoSerpro::IntegraContador,
                AmbienteSerpro::Producao,
            )
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroCredencial::NaoEncontrada { .. }));
    }

    #[tokio::test]
    async fn provider_aplica_fallback_trial_somente_quando_fornecido() {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(BancoFalso::default());
        let fallback = Some(CredencialFallbackGlobal {
            consumer_key: SecretString::from("global-key"),
            consumer_secret: SecretString::from("global-secret"),
        });
        let provider =
            SerproCredentialProviderImpl::novo(escritorios, Arc::new(CriptoIdentidade), fallback);

        let resultado = provider
            .resolver(ESC, ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial)
            .await
            .unwrap();

        assert_eq!(resultado.consumer_key.expose_secret(), "global-key");
        assert_eq!(resultado.consumer_secret.expose_secret(), "global-secret");
        assert!(resultado.certificado.is_none());
    }

    #[tokio::test]
    async fn provider_sem_fallback_entrada_trial_falha_claro() {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(BancoFalso::default());
        let provider =
            SerproCredentialProviderImpl::novo(escritorios, Arc::new(CriptoIdentidade), None);

        let erro = provider
            .resolver(ESC, ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial)
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroCredencial::NaoEncontrada { .. }));
    }

    #[tokio::test]
    async fn provider_nunca_aplica_fallback_em_producao() {
        let escritorios: Arc<dyn EscritorioSerproRepositorio> = Arc::new(BancoFalso::default());
        let fallback = Some(CredencialFallbackGlobal {
            consumer_key: SecretString::from("global-key"),
            consumer_secret: SecretString::from("global-secret"),
        });
        let provider =
            SerproCredentialProviderImpl::novo(escritorios, Arc::new(CriptoIdentidade), fallback);

        let erro = provider
            .resolver(
                ESC,
                ProdutoSerpro::IntegraContador,
                AmbienteSerpro::Producao,
            )
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroCredencial::NaoEncontrada { .. }));
    }
}
