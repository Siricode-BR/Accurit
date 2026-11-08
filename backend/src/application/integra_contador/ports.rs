use async_trait::async_trait;
use secrecy::SecretString;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::certificados::Certificado;
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;

#[derive(Debug, Clone)]
pub struct ContextoIntegraContador {
    pub escritorio_id: Uuid,
    pub contratante: Identificacao,
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    pub cpf_operador: Option<String>,
    /// Razão social do escritório (Procurador / autor do pedido de dados),
    /// exigida como `autorPedidoDados.nome` no Termo de Autorização.
    pub escritorio_nome: String,
}

/// Identidade da Software House contratante — a plataforma SaaS. Usada como
/// `contratante` no Termo de Autorização e no envelope do modelo C (Software
/// House), nunca como contratante de escritório. Resolvida da configuração
/// (`SERPRO_CONTRATANTE_CNPJ` / `SERPRO_CONTRATANTE_NOME`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContratantePlataforma {
    pub numero: String,
    pub nome: String,
}

#[async_trait]
pub trait CatalogoEscritoriosIntegra: Send + Sync {
    async fn resolver_contexto(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: &str,
    ) -> Result<ContextoIntegraContador, ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntregaIntegraContador {
    pub http_status: u16,
    pub estado: String,
    pub resposta: Option<IntegraContadorResponse>,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
}

#[async_trait]
pub trait ClienteIntegraContador: Send + Sync {
    async fn executar(
        &self,
        verbo: VerboIntegraContador,
        request: &IntegraContadorRequest,
        ambiente: AmbienteSerpro,
        cpf_operador: Option<&str>,
        request_id: &str,
    ) -> Result<EntregaIntegraContador, ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoChamadaIntegra {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub ambiente: AmbienteSerpro,
    pub estado: String,
    pub status_code: Option<i32>,
    pub referencia_resposta: Option<String>,
    pub request_id: String,
    pub bilhetavel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservaIdempotencia {
    pub escritorio_id: Uuid,
    pub chave: String,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub ambiente: AmbienteSerpro,
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultadoReservaIdempotencia {
    Nova,
    Concluida {
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    },
}

#[async_trait]
pub trait RepositorioIdempotencia: Send + Sync {
    async fn reservar(
        &self,
        reserva: &ReservaIdempotencia,
    ) -> Result<ResultadoReservaIdempotencia, ErroIntegraContador>;

    async fn atualizar(
        &self,
        reserva: &ReservaIdempotencia,
        estado: &str,
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    ) -> Result<(), ErroIntegraContador>;
}

#[async_trait]
pub trait AuditoriaIntegraContador: Send + Sync {
    async fn registrar_chamada(
        &self,
        evento: &EventoChamadaIntegra,
    ) -> Result<(), ErroIntegraContador>;

    async fn registrar_mutacao(
        &self,
        evento: &EventoAuditoriaIntegra,
    ) -> Result<(), ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoAuditoriaIntegra {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub acao: &'static str,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub ambiente: AmbienteSerpro,
    pub estado: String,
    pub payload_hash: String,
}

#[async_trait]
pub trait ValidadorProcuracao: Send + Sync {
    async fn validar(
        &self,
        contexto: &ContextoIntegraContador,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<bool, ErroIntegraContador>;
}

// ===== FASE A — catálogo configurável e procurações persistentes =====

/// Serviço resolvido do catálogo configurável (tabela
/// `catalogo_servicos_integra`). Carrega apenas o que a execução precisa:
/// o contrato (`idSistema`/`idServico`), a versão confirmada e a regra de
/// procuração em forma normal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicoResolvido {
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    /// TRUE = o verbo Consultar é o único permitido (Integra-Parcelamentos,
    /// ADR-0013). A checagem por verbo permanece no domínio.
    pub apenas_consulta: bool,
    /// Procuração exigida quando o autor do pedido difere do contribuinte.
    pub requer_procuracao: bool,
    /// Códigos de procuração que satisfazem a exigência (vazio =
    /// fail-closed: se exigida, nenhum código cobre).
    pub codigos_procuracao: Vec<String>,
}

impl ServicoResolvido {
    /// Converte a linha do catálogo na regra de procuração normalizada,
    /// reutilizando as infraestruturas de validação do domínio.
    pub fn exige_procuracao(&self) -> bool {
        self.requer_procuracao
    }

    pub fn codigos_possiveis(&self) -> Vec<String> {
        self.codigos_procuracao.clone()
    }
}

/// Port do catálogo configurável de serviços do Integra Contador
/// (tabela `catalogo_servicos_integra`, ADR-0013). Um serviço só fica
/// habilitado quando a linha existe com `habilitado`/`versao_confirmada`
/// verdadeiros — caso contrário o erro é explícito
/// (`ServicoNaoHabilitado`/`VersaoNaoConfirmada`).
/// Linha do catálogo configurável (tabela `catalogo_servicos_integra`)
/// para listagem no painel e para a validação de habilitação no admin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinhaCatalogoServico {
    pub id: Uuid,
    pub id_sistema: String,
    pub id_servico: String,
    pub verbo: VerboIntegraContador,
    /// `None` = versão pendente de confirmação no catálogo oficial (P4).
    pub versao_sistema: Option<String>,
    pub versao_confirmada: bool,
    pub habilitado: bool,
    /// TRUE = apenas o verbo Consultar é viável (Integra-Parcelamentos,
    /// ADR-0013).
    pub apenas_consulta: bool,
    pub requer_procuracao: bool,
    pub codigos_procuracao: Vec<String>,
    pub descricao: String,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait ProvedorCatalogoIntegra: Send + Sync {
    /// Resolve um serviço habilitado do catálogo configurável. Só devolve
    /// `Ok` quando existe linha com `habilitado = TRUE` e
    /// `versao_confirmada = TRUE`; caso contrário, erro explícito.
    async fn resolver(
        &self,
        id_sistema: &str,
        id_servico: &str,
        versao_sistema: &str,
        verbo: VerboIntegraContador,
    ) -> Result<ServicoResolvido, ErroIntegraContador>;

    /// Lista todas as linhas do catálogo (para o painel admin).
    async fn listar(&self) -> Result<Vec<LinhaCatalogoServico>, ErroIntegraContador>;

    /// Liga/desliga uma linha do catálogo. A validação contra o catálogo de
    /// referência do domínio fica no caso de uso; aqui só persiste.
    async fn definir_habilitado(
        &self,
        id_sistema: &str,
        id_servico: &str,
        habilitado: bool,
    ) -> Result<LinhaCatalogoServico, ErroIntegraContador>;
}

/// Registro persistido de procuração em `procuracoes` (FASE A). O
/// `codigo_servico` é `None` no registro agregado da OBTERPROCURACAO41
/// enquanto o formato de resposta não for confirmado por teste live (P7).
#[derive(Debug, Clone, PartialEq)]
pub struct RegistroProcuracao {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub ambiente: AmbienteSerpro,
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    pub codigo_servico: Option<String>,
    pub status: String,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub verificado_em: chrono::DateTime<chrono::Utc>,
    pub origem: String,
    pub payload_redigido: Option<Value>,
}

/// Dados para persistir/atualizar o registro de procuração de um cliente.
#[derive(Debug, Clone)]
pub struct NovoRegistroProcuracao {
    pub escritorio_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub ambiente: AmbienteSerpro,
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    pub codigo_servico: Option<String>,
    pub status: String,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub payload_redigido: Option<Value>,
    pub origem: String,
}

#[async_trait]
pub trait RepositorioProcuracoes: Send + Sync {
    /// Persiste/atualiza o registro (upsert pelo par contribuinte/procurador
    /// + `codigo_servico`, quando existir) e devolve o registro vigente.
    async fn salvar(
        &self,
        novo: &NovoRegistroProcuracao,
    ) -> Result<RegistroProcuracao, ErroIntegraContador>;

    /// Lista os registros do cliente (mais recentes primeiro).
    async fn listar_por_cliente(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<RegistroProcuracao>, ErroIntegraContador>;

    /// Último registro agregado (sem `codigo_servico`) do par
    /// contribuinte/procurador no ambiente — usado pelo TTL de validação.
    async fn buscar_agregado_recente(
        &self,
        escritorio_id: Uuid,
        ambiente: AmbienteSerpro,
        contribuinte_cpf_cnpj: &str,
        procurador_cpf_cnpj: &str,
    ) -> Result<Option<RegistroProcuracao>, ErroIntegraContador>;

    /// Fail-closed: existe procuração válida cobrindo o `codigo_servico`,
    /// vigente e verificada dentro da idade máxima (TTL)?
    async fn existe_valida(
        &self,
        escritorio_id: Uuid,
        ambiente: AmbienteSerpro,
        contribuinte_cpf_cnpj: &str,
        procurador_cpf_cnpj: &str,
        codigo_servico: &str,
        idade_maxima: Option<chrono::TimeDelta>,
    ) -> Result<bool, ErroIntegraContador>;
}

/// Port para obter o certificado A1 ativo do escritório.
#[async_trait]
pub trait CertificadoA1Provider: Send + Sync {
    async fn obter_ativo(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<Certificado>, ErroIntegraContador>;
}

/// Resultado da autenticação do procurador (token JWT retornado pelo SERPRO).
///
/// `jwt_token` é um `SecretString`: nunca é serializado em logs/respostas e é
/// zeroizado da memória após ser persistido cifrado no banco.
#[derive(Debug, Clone)]
pub struct AutenticarProcuradorResultado {
    pub jwt_token: SecretString,
    pub expira_em: chrono::DateTime<chrono::Utc>,
}

/// Port para salvar o token JWT obtido via AUTENTICAPROCURADOR.
#[async_trait]
pub trait TokensSerproRepositorio: Send + Sync {
    async fn salvar_jwt_procurador(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        ambiente: AmbienteSerpro,
        jwt_token: &str,
        expira_em: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ErroIntegraContador>;
}

/// Port para o cliente HTTP do endpoint AUTENTICAPROCURADOR (ENVIOXMLASSINADO81).
#[async_trait]
pub trait AutenticaProcuradorCliente: Send + Sync {
    async fn enviar_xml_assinado(
        &self,
        request: &IntegraContadorRequest,
        xml_base64: &str,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<AutenticarProcuradorResultado, ErroIntegraContador>;
}

pub type RespostaRedigida = Value;
