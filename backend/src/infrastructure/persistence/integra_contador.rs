use chrono::{DateTime, TimeDelta, Utc};
use diesel::prelude::*;
use diesel::sql_types::{
    BigInt, Binary, Bool, Integer, Jsonb, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use serde_json::Value;
use uuid::Uuid;

use crate::application::integra_contador::ports::{
    AuditoriaIntegraContador, CatalogoEscritoriosIntegra, CertificadoA1Provider,
    ContextoIntegraContador, EventoAuditoriaIntegra, EventoChamadaIntegra, LinhaCatalogoServico,
    NovoRegistroProcuracao, ProvedorCatalogoIntegra, RegistroProcuracao, RepositorioIdempotencia,
    RepositorioProcuracoes, ReservaIdempotencia, ResultadoReservaIdempotencia, ServicoResolvido,
    TokensSerproRepositorio, ValidadorProcuracao,
};
use crate::domain::integra_contador::envelope::{Identificacao, VerboIntegraContador};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct IntegraContadorRepositorioDiesel {
    pool: DbPool,
}

impl IntegraContadorRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroIntegraContador,
    > {
        self.pool
            .get()
            .map_err(|_| ErroIntegraContador::Persistencia)
    }
}

#[derive(QueryableByName)]
struct ContextoRow {
    #[diesel(sql_type = Nullable<Text>)]
    cnpj: Option<String>,
    #[diesel(sql_type = Text)]
    nome_escritorio: String,
    #[diesel(sql_type = Text)]
    cpf_operador: String,
    #[diesel(sql_type = Text)]
    documento_contribuinte: String,
}

#[async_trait::async_trait]
impl CatalogoEscritoriosIntegra for IntegraContadorRepositorioDiesel {
    async fn resolver_contexto(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: &str,
    ) -> Result<ContextoIntegraContador, ErroIntegraContador> {
        let documento = crate::domain::clientes::CpfCnpj::novo(documento_contribuinte)
            .map_err(|_| ErroIntegraContador::PayloadInvalido)?;
        let documento = documento.como_str().to_owned();
        let repositorio = self.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let linha = diesel::sql_query(
                "SELECT e.cnpj, e.nome AS nome_escritorio, u.cpf AS cpf_operador, c.cpf_cnpj AS documento_contribuinte \
                 FROM escritorios e \
                 INNER JOIN usuarios u \
                   ON u.escritorio_id = e.id AND u.id = $2 AND u.ativo = TRUE \
                 INNER JOIN clientes c \
                   ON c.escritorio_id = e.id AND c.cpf_cnpj = $3 \
                 WHERE e.id = $1",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<SqlUuid, _>(usuario_id)
            .bind::<Text, _>(documento)
            .get_result::<ContextoRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;

            let cnpj = linha
                .cnpj
                .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;
            let contratante = Identificacao::nova(&cnpj)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
            let contribuinte = Identificacao::nova(&linha.documento_contribuinte)
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;

            Ok(ContextoIntegraContador {
                escritorio_id,
                contratante: contratante.clone(),
                autor_pedido_dados: contratante,
                contribuinte,
                cpf_operador: Some(linha.cpf_operador),
                escritorio_nome: linha.nome_escritorio,
            })
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

#[derive(QueryableByName)]
struct IdempotenciaRow {
    #[diesel(sql_type = Text)]
    verbo: String,
    #[diesel(sql_type = Text)]
    id_sistema: String,
    #[diesel(sql_type = Text)]
    id_servico: String,
    #[diesel(sql_type = Text)]
    versao_sistema: String,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Text)]
    payload_hash: String,
    #[diesel(sql_type = Text)]
    estado: String,
    #[diesel(sql_type = Nullable<Integer>)]
    resposta_status: Option<i32>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    resposta_redigida: Option<Value>,
}

#[async_trait::async_trait]
impl RepositorioIdempotencia for IntegraContadorRepositorioDiesel {
    async fn reservar(
        &self,
        reserva: &ReservaIdempotencia,
    ) -> Result<ResultadoReservaIdempotencia, ErroIntegraContador> {
        let repositorio = self.clone();
        let reserva = reserva.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let inseridos = diesel::sql_query(
                "INSERT INTO idempotencia_operacoes \
                 (id, escritorio_id, chave_idempotencia, verbo, id_sistema, id_servico, \
                  versao_sistema, ambiente, payload_hash, estado) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'EM_ANDAMENTO') \
                 ON CONFLICT (escritorio_id, chave_idempotencia) DO NOTHING",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(&reserva.chave)
            .bind::<Text, _>(reserva.verbo.como_str())
            .bind::<Text, _>(&reserva.id_sistema)
            .bind::<Text, _>(&reserva.id_servico)
            .bind::<Text, _>(&reserva.versao_sistema)
            .bind::<Text, _>(reserva.ambiente.as_str())
            .bind::<Text, _>(&reserva.payload_hash)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            if inseridos == 1 {
                return Ok(ResultadoReservaIdempotencia::Nova);
            }

            let existente = diesel::sql_query(
                "SELECT verbo, id_sistema, id_servico, versao_sistema, ambiente, payload_hash, \
                        estado, resposta_status, resposta_redigida \
                 FROM idempotencia_operacoes \
                 WHERE escritorio_id = $1 AND chave_idempotencia = $2",
            )
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(&reserva.chave)
            .get_result::<IdempotenciaRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;

            let mesmo_pedido = existente.verbo == reserva.verbo.como_str()
                && existente.id_sistema == reserva.id_sistema
                && existente.id_servico == reserva.id_servico
                && existente.versao_sistema == reserva.versao_sistema
                && existente.ambiente == reserva.ambiente.as_str()
                && existente.payload_hash == reserva.payload_hash;
            if !mesmo_pedido {
                return Err(ErroIntegraContador::ConflitoIdempotencia);
            }
            if existente.estado == "CONCLUIDO" || existente.estado == "REJEITADO" {
                return Ok(ResultadoReservaIdempotencia::Concluida {
                    status: existente.resposta_status,
                    resposta_redigida: existente.resposta_redigida,
                });
            }
            Err(ErroIntegraContador::ConflitoIdempotencia)
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn atualizar(
        &self,
        reserva: &ReservaIdempotencia,
        estado: &str,
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let reserva = reserva.clone();
        let estado = estado.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let atualizados = diesel::sql_query(
                "UPDATE idempotencia_operacoes \
                 SET estado = $1, resposta_status = $2, resposta_redigida = $3, \
                     processado_em = CASE \
                       WHEN $1 IN ('CONCLUIDO', 'REJEITADO', 'INDETERMINADO') THEN NOW() \
                       ELSE processado_em END \
                 WHERE escritorio_id = $4 AND chave_idempotencia = $5 AND payload_hash = $6",
            )
            .bind::<Text, _>(estado)
            .bind::<Nullable<Integer>, _>(status)
            .bind::<Nullable<Jsonb>, _>(resposta_redigida)
            .bind::<SqlUuid, _>(reserva.escritorio_id)
            .bind::<Text, _>(reserva.chave)
            .bind::<Text, _>(reserva.payload_hash)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            if atualizados != 1 {
                return Err(ErroIntegraContador::Persistencia);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

#[async_trait::async_trait]
impl AuditoriaIntegraContador for IntegraContadorRepositorioDiesel {
    async fn registrar_chamada(
        &self,
        evento: &EventoChamadaIntegra,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let evento = evento.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            diesel::sql_query(
                "INSERT INTO chamadas_operacionais \
                 (id, escritorio_id, usuario_id, verbo, id_sistema, id_servico, ambiente, \
                  estado, status_code, referencia_resposta, request_id, bilhetavel) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(evento.escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(evento.usuario_id))
            .bind::<Text, _>(evento.verbo.como_str())
            .bind::<Text, _>(evento.id_sistema)
            .bind::<Text, _>(evento.id_servico)
            .bind::<Text, _>(evento.ambiente.as_str())
            .bind::<Text, _>(evento.estado)
            .bind::<Nullable<Integer>, _>(evento.status_code)
            .bind::<Nullable<Text>, _>(evento.referencia_resposta)
            .bind::<Nullable<Text>, _>(Some(evento.request_id))
            .bind::<Bool, _>(evento.bilhetavel)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn registrar_mutacao(
        &self,
        evento: &EventoAuditoriaIntegra,
    ) -> Result<(), ErroIntegraContador> {
        let repositorio = self.clone();
        let evento = evento.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let valores = serde_json::json!({
                "verbo": evento.verbo.como_str(),
                "idSistema": evento.id_sistema,
                "idServico": evento.id_servico,
                "ambiente": evento.ambiente.as_str(),
                "estado": evento.estado,
                "payloadHash": evento.payload_hash,
            });
            diesel::sql_query(
                "INSERT INTO auditoria_eventos \
                 (id, escritorio_id, usuario_id, acao, entidade, entidade_id, valor_novo) \
                 VALUES ($1, $2, $3, $4, 'INTEGRA_CONTADOR', $5, $6)",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(evento.escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(evento.usuario_id))
            .bind::<Text, _>(evento.acao)
            .bind::<Nullable<SqlUuid>, _>(Some(Uuid::now_v7()))
            .bind::<Nullable<Jsonb>, _>(Some(valores))
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

// ===== FASE A â€” procuraÃ§Ãµes persistentes (OBTERPROCURACAO41 agregado, P7) =====

/// Linha bruta da tabela `procuracoes`.
#[derive(QueryableByName)]
struct ProcuracaoRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    cliente_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Text)]
    contribuinte_cpf_cnpj: String,
    #[diesel(sql_type = Text)]
    procurador_cpf_cnpj: String,
    #[diesel(sql_type = Nullable<Text>)]
    codigo_servico: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    valido_de: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    valido_ate: Option<DateTime<Utc>>,
    #[diesel(sql_type = Timestamptz)]
    verificado_em: DateTime<Utc>,
    #[diesel(sql_type = Text)]
    origem: String,
    #[diesel(sql_type = Nullable<Jsonb>)]
    payload_redigido: Option<Value>,
}

impl ProcuracaoRow {
    fn registro(self, escritorio_id: Uuid) -> Result<RegistroProcuracao, ErroIntegraContador> {
        let ambiente = match self.ambiente.as_str() {
            "TRIAL" => AmbienteSerpro::Trial,
            "PRODUCAO" => AmbienteSerpro::Producao,
            _ => return Err(ErroIntegraContador::Persistencia),
        };
        Ok(RegistroProcuracao {
            id: self.id,
            escritorio_id,
            cliente_id: self.cliente_id,
            ambiente,
            contribuinte_cpf_cnpj: self.contribuinte_cpf_cnpj,
            procurador_cpf_cnpj: self.procurador_cpf_cnpj,
            codigo_servico: self.codigo_servico,
            status: self.status,
            valido_de: self.valido_de,
            valido_ate: self.valido_ate,
            verificado_em: self.verificado_em,
            origem: self.origem,
            payload_redigido: self.payload_redigido,
        })
    }
}

const COLUNAS_PROCURACAO: &str = "id, cliente_id, ambiente, contribuinte_cpf_cnpj, \
     procurador_cpf_cnpj, codigo_servico, status, valido_de, valido_ate, verificado_em, \
     origem, payload_redigido";

/// RepositÃ³rio de procuraÃ§Ãµes persistentes (FASE A). Sempre com
/// `escritorio_id` como escopo (isolamento multi-tenant).
#[derive(Clone)]
pub struct ProcuracoesRepositorioDiesel {
    pool: DbPool,
}

impl ProcuracoesRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroIntegraContador,
    > {
        self.pool
            .get()
            .map_err(|_| ErroIntegraContador::Persistencia)
    }
}

#[async_trait::async_trait]
impl RepositorioProcuracoes for ProcuracoesRepositorioDiesel {
    async fn salvar(
        &self,
        novo: &NovoRegistroProcuracao,
    ) -> Result<RegistroProcuracao, ErroIntegraContador> {
        let repositorio = self.clone();
        let novo = novo.clone();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;

            // Upsert manual (SELECT-then-UPDATE/INSERT): a constraint Ãºnica
            // usa NULLS NOT DISTINCT, o que exige comparaÃ§Ã£o com
            // IS NOT DISTINCT FROM; UPDATE-then-INSERT evita depender de
            // versÃ£o/equerÃªncia do ON CONFLICT para o caso agregado.
            let registrado = diesel::sql_query(format!(
                "UPDATE procuracoes SET \
                     status = $6, valido_de = $7, valido_ate = $8, \
                     verificado_em = NOW(), origem = $9, payload_redigido = $10, \
                     cliente_id = COALESCE($11, cliente_id) \
                 WHERE escritorio_id = $1 AND ambiente = $2 \
                   AND contribuinte_cpf_cnpj = $3 AND procurador_cpf_cnpj = $4 \
                   AND codigo_servico IS NOT DISTINCT FROM $5 \
                 RETURNING {COLUNAS_PROCURACAO}"
            ))
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<Text, _>(novo.ambiente.as_str())
            .bind::<Text, _>(&novo.contribuinte_cpf_cnpj)
            .bind::<Text, _>(&novo.procurador_cpf_cnpj)
            .bind::<Nullable<Text>, _>(novo.codigo_servico.clone())
            .bind::<Text, _>(&novo.status)
            .bind::<Nullable<Timestamptz>, _>(novo.valido_de)
            .bind::<Nullable<Timestamptz>, _>(novo.valido_ate)
            .bind::<Text, _>(&novo.origem)
            .bind::<Nullable<Jsonb>, _>(novo.payload_redigido.clone())
            .bind::<Nullable<SqlUuid>, _>(novo.cliente_id)
            .get_result::<ProcuracaoRow>(&mut conexao)
            .map(|row| row.registro(novo.escritorio_id));

            let registro = match registrado {
                Ok(registro) => registro,
                Err(diesel::result::Error::NotFound) => diesel::sql_query(format!(
                    "INSERT INTO procuracoes \
                         (id, escritorio_id, cliente_id, ambiente, contribuinte_cpf_cnpj, \
                          procurador_cpf_cnpj, codigo_servico, status, valido_de, valido_ate, \
                          verificado_em, origem, payload_redigido) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), $11, $12) \
                     RETURNING {COLUNAS_PROCURACAO}"
                ))
                .bind::<SqlUuid, _>(Uuid::now_v7())
                .bind::<SqlUuid, _>(novo.escritorio_id)
                .bind::<Nullable<SqlUuid>, _>(novo.cliente_id)
                .bind::<Text, _>(novo.ambiente.as_str())
                .bind::<Text, _>(&novo.contribuinte_cpf_cnpj)
                .bind::<Text, _>(&novo.procurador_cpf_cnpj)
                .bind::<Nullable<Text>, _>(novo.codigo_servico.clone())
                .bind::<Text, _>(&novo.status)
                .bind::<Nullable<Timestamptz>, _>(novo.valido_de)
                .bind::<Nullable<Timestamptz>, _>(novo.valido_ate)
                .bind::<Text, _>(&novo.origem)
                .bind::<Nullable<Jsonb>, _>(novo.payload_redigido)
                .get_result::<ProcuracaoRow>(&mut conexao)
                .map_err(|_| ErroIntegraContador::Persistencia)
                .and_then(|row| row.registro(novo.escritorio_id)),
                Err(_) => Err(ErroIntegraContador::Persistencia),
            }?;
            Ok(registro)
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn listar_por_cliente(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<RegistroProcuracao>, ErroIntegraContador> {
        let repositorio = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let linhas = diesel::sql_query(format!(
                "SELECT {COLUNAS_PROCURACAO} FROM procuracoes \
                 WHERE escritorio_id = $1 AND cliente_id = $2 \
                 ORDER BY verificado_em DESC, id DESC"
            ))
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<SqlUuid, _>(cliente_id)
            .load::<ProcuracaoRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            linhas
                .into_iter()
                .map(|row| row.registro(escritorio_id))
                .collect()
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn buscar_agregado_recente(
        &self,
        escritorio_id: Uuid,
        ambiente: AmbienteSerpro,
        contribuinte_cpf_cnpj: &str,
        procurador_cpf_cnpj: &str,
    ) -> Result<Option<RegistroProcuracao>, ErroIntegraContador> {
        let repositorio = self.clone();
        let contribuinte = contribuinte_cpf_cnpj.to_owned();
        let procurador = procurador_cpf_cnpj.to_owned();
        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let encontrado = diesel::sql_query(format!(
                "SELECT {COLUNAS_PROCURACAO} FROM procuracoes \
                 WHERE escritorio_id = $1 AND ambiente = $2 \
                   AND contribuinte_cpf_cnpj = $3 AND procurador_cpf_cnpj = $4 \
                   AND codigo_servico IS NULL \
                 ORDER BY verificado_em DESC, id DESC LIMIT 1"
            ))
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Text, _>(ambiente.as_str())
            .bind::<Text, _>(&contribuinte)
            .bind::<Text, _>(&procurador)
            .get_result::<ProcuracaoRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            encontrado
                .map(|row| row.registro(escritorio_id))
                .transpose()
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn existe_valida(
        &self,
        escritorio_id: Uuid,
        ambiente: AmbienteSerpro,
        contribuinte_cpf_cnpj: &str,
        procurador_cpf_cnpj: &str,
        codigo_servico: &str,
        idade_maxima: Option<TimeDelta>,
    ) -> Result<bool, ErroIntegraContador> {
        let repositorio = self.clone();
        let contribuinte = contribuinte_cpf_cnpj.to_owned();
        let procurador = procurador_cpf_cnpj.to_owned();
        let codigo = codigo_servico.to_owned();
        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let corte: Option<DateTime<Utc>> = idade_maxima.map(|idade| Utc::now() - idade);
            let total = diesel::sql_query(
                "SELECT COUNT(*) AS total FROM procuracoes \
                 WHERE escritorio_id = $1 AND ambiente = $2 \
                   AND contribuinte_cpf_cnpj = $3 AND procurador_cpf_cnpj = $4 \
                   AND codigo_servico = $5 \
                   AND status IN ('VALIDA', 'ATIVA') \
                   AND (valido_de IS NULL OR valido_de <= NOW()) \
                   AND (valido_ate IS NULL OR valido_ate >= NOW()) \
                   AND ($6::timestamptz IS NULL OR verificado_em >= $6::timestamptz)",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Text, _>(ambiente.as_str())
            .bind::<Text, _>(&contribuinte)
            .bind::<Text, _>(&procurador)
            .bind::<Text, _>(&codigo)
            .bind::<Nullable<Timestamptz>, _>(corte)
            .get_result::<TotalRow>(&mut conexao)
            .map(|linha| linha.total)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(total > 0)
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

#[derive(QueryableByName)]
struct TotalRow {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

/// Adaptador da porta `ValidadorProcuracao`: fail-closed, responde com base
/// no registro vigente do repositÃ³rio com a idade mÃ¡xima (TTL) da
/// configuraÃ§Ã£o. Nenhuma chamada ao SERPRO acontece aqui (a atualizaÃ§Ã£o Ã©
/// responsabilidade do `ValidarProcuracoesUseCase`).
pub struct ValidadorProcuracaoDiesel {
    repositorio: std::sync::Arc<ProcuracoesRepositorioDiesel>,
    idade_maxima: Option<TimeDelta>,
}

impl ValidadorProcuracaoDiesel {
    pub fn novo(
        repositorio: std::sync::Arc<ProcuracoesRepositorioDiesel>,
        idade_maxima_secs: Option<u64>,
    ) -> Self {
        Self {
            repositorio,
            idade_maxima: idade_maxima_secs
                .filter(|segundos| *segundos > 0)
                .and_then(|segundos| TimeDelta::try_seconds(segundos as i64)),
        }
    }
}

#[async_trait::async_trait]
impl ValidadorProcuracao for ValidadorProcuracaoDiesel {
    async fn validar(
        &self,
        contexto: &ContextoIntegraContador,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<bool, ErroIntegraContador> {
        self.repositorio
            .existe_valida(
                contexto.escritorio_id,
                ambiente,
                &contexto.contribuinte.numero,
                &contexto.autor_pedido_dados.numero,
                codigo_servico,
                self.idade_maxima,
            )
            .await
    }
}

// ===== FASE A â€” catÃ¡logo configurÃ¡vel de serviÃ§os do Integra Contador =====

/// Adaptador da porta `ProvedorCatalogoIntegra` sobre a tabela
/// `catalogo_servicos_integra` (ADR-0013).
#[derive(Clone)]
pub struct CatalogoServicosIntegraDiesel {
    pool: DbPool,
}

impl CatalogoServicosIntegraDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroIntegraContador,
    > {
        self.pool
            .get()
            .map_err(|_| ErroIntegraContador::Persistencia)
    }
}

fn verbo_de(valor: &str) -> Result<VerboIntegraContador, ErroIntegraContador> {
    match valor {
        "Apoiar" => Ok(VerboIntegraContador::Apoiar),
        "Consultar" => Ok(VerboIntegraContador::Consultar),
        "Declarar" => Ok(VerboIntegraContador::Declarar),
        "Emitir" => Ok(VerboIntegraContador::Emitir),
        "Monitorar" => Ok(VerboIntegraContador::Monitorar),
        _ => Err(ErroIntegraContador::Persistencia),
    }
}

#[derive(QueryableByName)]
struct LinhaCatalogoRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    id_sistema: String,
    #[diesel(sql_type = Text)]
    id_servico: String,
    #[diesel(sql_type = Text)]
    verbo: String,
    #[diesel(sql_type = Nullable<Text>)]
    versao_sistema: Option<String>,
    #[diesel(sql_type = Bool)]
    versao_confirmada: bool,
    #[diesel(sql_type = Bool)]
    habilitado: bool,
    #[diesel(sql_type = Bool)]
    apenas_consulta: bool,
    #[diesel(sql_type = Bool)]
    requer_procuracao: bool,
    #[diesel(sql_type = diesel::sql_types::Array<Nullable<Text>>)]
    codigos_procuracao: Vec<Option<String>>,
    #[diesel(sql_type = Text)]
    descricao: String,
    #[diesel(sql_type = Timestamptz)]
    atualizado_em: DateTime<Utc>,
}

impl LinhaCatalogoRow {
    fn linha(self) -> Result<LinhaCatalogoServico, ErroIntegraContador> {
        Ok(LinhaCatalogoServico {
            id: self.id,
            id_sistema: self.id_sistema,
            id_servico: self.id_servico,
            verbo: verbo_de(&self.verbo)?,
            versao_sistema: self.versao_sistema,
            versao_confirmada: self.versao_confirmada,
            habilitado: self.habilitado,
            apenas_consulta: self.apenas_consulta,
            requer_procuracao: self.requer_procuracao,
            codigos_procuracao: self.codigos_procuracao.into_iter().flatten().collect(),
            descricao: self.descricao,
            atualizado_em: self.atualizado_em,
        })
    }
}

#[async_trait::async_trait]
impl ProvedorCatalogoIntegra for CatalogoServicosIntegraDiesel {
    async fn resolver(
        &self,
        id_sistema: &str,
        id_servico: &str,
        versao_sistema: &str,
        verbo: VerboIntegraContador,
    ) -> Result<ServicoResolvido, ErroIntegraContador> {
        let repositorio = self.clone();
        let id_sistema = id_sistema.to_owned();
        let id_servico = id_servico.to_owned();
        let versao = versao_sistema.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let linha = diesel::sql_query(
                "SELECT id, id_sistema, id_servico, verbo, versao_sistema, \
                        versao_confirmada, habilitado, apenas_consulta, \
                        requer_procuracao, codigos_procuracao, descricao, atualizado_em \
                 FROM catalogo_servicos_integra \
                 WHERE id_sistema = $1 AND id_servico = $2",
            )
            .bind::<Text, _>(id_sistema)
            .bind::<Text, _>(id_servico)
            .get_result::<LinhaCatalogoRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroIntegraContador::Persistencia)?
            .ok_or(ErroIntegraContador::ServicoNaoHabilitado)?;

            if linha.verbo != verbo.como_str() {
                return Err(ErroIntegraContador::ServicoNaoHabilitado);
            }
            if !linha.habilitado || !linha.versao_confirmada {
                return Err(ErroIntegraContador::ServicoNaoHabilitado);
            }
            let versao_confirmada = linha
                .versao_sistema
                .clone()
                .ok_or(ErroIntegraContador::VersaoNaoConfirmada)?;
            if versao_confirmada != versao {
                return Err(ErroIntegraContador::VersaoNaoConfirmada);
            }

            Ok(ServicoResolvido {
                id_sistema: linha.id_sistema.clone(),
                id_servico: linha.id_servico.clone(),
                versao_sistema: versao_confirmada,
                apenas_consulta: linha.apenas_consulta,
                requer_procuracao: linha.requer_procuracao,
                codigos_procuracao: linha.codigos_procuracao.into_iter().flatten().collect(),
            })
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn listar(&self) -> Result<Vec<LinhaCatalogoServico>, ErroIntegraContador> {
        let repositorio = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            let linhas = diesel::sql_query(
                "SELECT id, id_sistema, id_servico, verbo, versao_sistema, \
                        versao_confirmada, habilitado, apenas_consulta, \
                        requer_procuracao, codigos_procuracao, descricao, atualizado_em \
                 FROM catalogo_servicos_integra \
                 ORDER BY id_sistema, id_servico",
            )
            .load::<LinhaCatalogoRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            linhas.into_iter().map(LinhaCatalogoRow::linha).collect()
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }

    async fn definir_habilitado(
        &self,
        id_sistema: &str,
        id_servico: &str,
        habilitado: bool,
    ) -> Result<LinhaCatalogoServico, ErroIntegraContador> {
        let repositorio = self.clone();
        let id_sistema = id_sistema.to_owned();
        let id_servico = id_servico.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut conexao = repositorio.conexao()?;
            // Habilitar exige versÃ£o confirmada (fail-closed: uma linha sem
            // `versao_confirmada` Ã© tratada como se nÃ£o existisse). VersÃ£o
            // confirmada Ã© Ãºnica origem: a atualizaÃ§Ã£o manual de versÃ£o Ã©
            // um ato de engenharia (migraÃ§Ã£o de semente), nÃ£o de runtime.
            let atualizados = diesel::sql_query(
                "UPDATE catalogo_servicos_integra \
                 SET habilitado = $3, atualizado_em = NOW() \
                 WHERE id_sistema = $1 AND id_servico = $2 \
                   AND (NOT $3 OR versao_confirmada) \
                 RETURNING id, id_sistema, id_servico, verbo, versao_sistema, \
                           versao_confirmada, habilitado, apenas_consulta, \
                           requer_procuracao, codigos_procuracao, descricao, atualizado_em",
            )
            .bind::<Text, _>(&id_sistema)
            .bind::<Text, _>(&id_servico)
            .bind::<Bool, _>(habilitado)
            .get_result::<LinhaCatalogoRow>(&mut conexao)
            .map_err(|_| ErroIntegraContador::ServicoNaoHabilitado)?;
            atualizados.linha()
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

// ===== Modelo Software House: certÃ­ficado A1 do Procurador e token JWT por cliente =====

/// Adaptador da porta `CertificadoA1Provider`: busca o certificado A1 ativo
/// e vÃ¡lido do escritÃ³rio (Procurador) â€” usado para assinar o Termo de
/// AutorizaÃ§Ã£o do AUTENTICAPROCURADOR. O conteÃºdo continua cifrado na
/// saÃ­da (descifragem ocorre apenas em memÃ³ria, no caso de uso).
pub struct CertificadoA1ProviderDiesel {
    repo: std::sync::Arc<
        crate::infrastructure::persistence::certificados::CertificadoRepositorioDiesel,
    >,
}

impl CertificadoA1ProviderDiesel {
    pub fn novo(
        repo: std::sync::Arc<
            crate::infrastructure::persistence::certificados::CertificadoRepositorioDiesel,
        >,
    ) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl CertificadoA1Provider for CertificadoA1ProviderDiesel {
    async fn obter_ativo(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<crate::domain::certificados::Certificado>, ErroIntegraContador> {
        let repo = self.repo.clone();
        tokio::task::spawn_blocking(move || {
            repo.buscar_ativo(escritorio_id)
                .map_err(|_| ErroIntegraContador::Persistencia)
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

/// Adaptador da porta `TokensSerproRepositorio`: persiste o token JWT do
/// AUTENTICAPROCURADOR **cifrado** (AES-256-GCM, mesmo formato de
/// `credenciais_serpro`) em `tokens_serpro`, vinculando `escritorio_id` e
/// `cliente_id` (modelo por cliente/contribuinte).
#[derive(Clone)]
pub struct TokensSerproRepositorioDiesel {
    pool: DbPool,
    criptografia: std::sync::Arc<dyn crate::application::serpro::ports::CriptografiaSerpro>,
}

impl TokensSerproRepositorioDiesel {
    pub fn novo(
        pool: DbPool,
        criptografia: std::sync::Arc<dyn crate::application::serpro::ports::CriptografiaSerpro>,
    ) -> Self {
        Self { pool, criptografia }
    }
}

#[async_trait::async_trait]
impl TokensSerproRepositorio for TokensSerproRepositorioDiesel {
    async fn salvar_jwt_procurador(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        ambiente: AmbienteSerpro,
        jwt_token: &str,
        expira_em: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ErroIntegraContador> {
        let repo = self.clone();
        let jwt = jwt_token.to_owned();
        tokio::task::spawn_blocking(move || {
            let token_cifrado = repo
                .criptografia
                .cifrar(jwt.as_bytes())
                .map_err(|_| ErroIntegraContador::Persistencia)?;
            let mut conexao = repo
                .pool
                .get()
                .map_err(|_| ErroIntegraContador::Persistencia)?;

            // Upsert pela constraint Ãºnica nomeada uq_tokens_serpro
            // (UNIQUE NULLS NOT DISTINCT): um escritÃ³rio tem um token vigente
            // por (cliente, produto, ambiente, tipo_token, autor_pedido_dados);
            // para AUTENTICAR_PROCURADOR o cliente_id Ã© obrigatÃ³rio (CHECK).
            diesel::sql_query(
                "INSERT INTO tokens_serpro \
                 (id, escritorio_id, cliente_id, produto, ambiente, tipo_token, \
                  autor_pedido_dados, token_cifrado, escopo, expira_em, criado_em) \
                 VALUES ($1, $2, $3, 'INTEGRA_CONTADOR', $4, 'AUTENTICAR_PROCURADOR', \
                  '', $5, 'AUTENTICAPROCURADOR', $6, NOW()) \
                 ON CONFLICT ON CONSTRAINT uq_tokens_serpro \
                 DO UPDATE SET token_cifrado = EXCLUDED.token_cifrado, \
                               expira_em = EXCLUDED.expira_em, \
                               criado_em = NOW()",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(cliente_id))
            .bind::<Text, _>(ambiente.as_str())
            .bind::<Binary, _>(token_cifrado)
            .bind::<Timestamptz, _>(expira_em)
            .execute(&mut conexao)
            .map_err(|_| ErroIntegraContador::Persistencia)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroIntegraContador::Persistencia)?
    }
}

/// Auxiliar: linha de token procurador vigente por cliente (uso futuro do
/// cliente HTTP para o header `autenticar_procurador_token`).
#[derive(QueryableByName)]
#[allow(dead_code)]
struct TokenSecret {
    #[diesel(sql_type = Binary)]
    token_cifrado: Vec<u8>,
    #[diesel(sql_type = Timestamptz)]
    expira_em: chrono::DateTime<chrono::Utc>,
}
