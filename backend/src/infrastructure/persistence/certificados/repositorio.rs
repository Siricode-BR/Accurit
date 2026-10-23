use std::sync::Arc;

use crate::application::certificados::{
    AuditoriaCertificadoRepositorio, CertificadoRepositorio, EventoAuditoriaCertificado,
    ListagemCertificados, RepositorioError,
};
use crate::domain::certificados::{Certificado, FingerprintSha256};
use crate::domain::tenancy::EscritorioId;
use crate::infrastructure::persistence::schema::certificados_a1;
#[allow(unused_imports)]
use crate::infrastructure::persistence::schema::certificados_a1::dsl::*;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

#[derive(Clone)]
pub struct CertificadoRepositorioDiesel {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl CertificadoRepositorioDiesel {
    pub fn novo(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self { pool }
    }

    fn obter_conexao(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositorioError> {
        self.pool.get().map_err(|_| RepositorioError::Indisponivel)
    }
}

impl CertificadoRepositorio for CertificadoRepositorioDiesel {
    fn criar(
        &self,
        _escritorio_id_param: EscritorioId,
        certificado: &Certificado,
    ) -> Result<(), RepositorioError> {
        let mut conn = self.obter_conexao()?;

        let novo = CertificadoNovo {
            id: certificado.id,
            escritorio_id: certificado.escritorio_id.into(),
            tipo: certificado.tipo.como_str().to_string(),
            titular_cpf_cnpj: certificado.titular_cpf_cnpj.clone(),
            conteudo_cifrado: certificado.conteudo_cifrado.clone(),
            senha_cifrada: certificado.senha_cifrada.clone(),
            algoritmo: certificado.algoritmo.como_str().to_string(),
            fingerprint_sha256: certificado.fingerprint.como_str().to_string(),
            valido_de: certificado.valido_de,
            valido_ate: certificado.valido_ate,
            ativo: certificado.ativo,
            criado_em: certificado.criado_em,
        };

        diesel::insert_into(certificados_a1::table)
            .values(&novo)
            .execute(&mut conn)
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    RepositorioError::FingerprintDuplicado
                } else {
                    RepositorioError::Dados
                }
            })?;

        Ok(())
    }

    fn buscar_por_id(
        &self,
        escritorio_id_param: EscritorioId,
        certificado_id: Uuid,
    ) -> Result<Option<Certificado>, RepositorioError> {
        let mut conn = self.obter_conexao()?;

        let resultado = certificados_a1::table
            .filter(certificados_a1::id.eq(certificado_id))
            .filter(certificados_a1::escritorio_id.eq(escritorio_id_param.as_uuid()))
            .first::<CertificadoDb>(&mut conn)
            .optional()
            .map_err(|_| RepositorioError::Dados)?;

        Ok(resultado.map(|c| c.into()))
    }

    fn buscar_por_fingerprint(
        &self,
        escritorio_id_param: EscritorioId,
        fingerprint: &str,
    ) -> Result<Option<Certificado>, RepositorioError> {
        let mut conn = self.obter_conexao()?;

        let resultado = certificados_a1::table
            .filter(certificados_a1::fingerprint_sha256.eq(fingerprint))
            .filter(certificados_a1::escritorio_id.eq(escritorio_id_param.as_uuid()))
            .first::<CertificadoDb>(&mut conn)
            .optional()
            .map_err(|_| RepositorioError::Dados)?;

        Ok(resultado.map(|c| c.into()))
    }

    fn listar(
        &self,
        escritorio_id_param: EscritorioId,
        apenas_validos: bool,
        pagina: i64,
        tamanho: i64,
    ) -> Result<ListagemCertificados, RepositorioError> {
        let mut conn = self.obter_conexao()?;

        let build_query = |apenas_validos: bool| {
            let q = certificados_a1::table
                .filter(certificados_a1::escritorio_id.eq(escritorio_id_param.as_uuid()));
            if apenas_validos {
                let agora = chrono::Utc::now();
                q.filter(certificados_a1::ativo.eq(true))
                    .filter(certificados_a1::valido_de.le(agora))
                    .filter(certificados_a1::valido_ate.ge(agora))
                    .into_boxed()
            } else {
                q.into_boxed()
            }
        };

        let total: i64 = build_query(apenas_validos)
            .count()
            .get_result(&mut conn)
            .map_err(|_| RepositorioError::Dados)?;

        let deslocamento = (pagina - 1) * tamanho;

        let itens = build_query(apenas_validos)
            .order(certificados_a1::criado_em.desc())
            .limit(tamanho)
            .offset(deslocamento)
            .load::<CertificadoDb>(&mut conn)
            .map_err(|_| RepositorioError::Dados)?;

        Ok(ListagemCertificados {
            itens: itens
                .into_iter()
                .map(|c| {
                    let cert: Certificado = c.into();
                    (&cert).into()
                })
                .collect(),
            total,
            pagina,
            tamanho,
        })
    }

    fn revogar(
        &self,
        _escritorio_id_param: EscritorioId,
        certificado_id: Uuid,
    ) -> Result<bool, RepositorioError> {
        let mut conn = self.obter_conexao()?;

        let linhas = diesel::update(
            certificados_a1::table
                .filter(certificados_a1::id.eq(certificado_id))
                .filter(certificados_a1::escritorio_id.eq(_escritorio_id_param.as_uuid())),
        )
        .set(certificados_a1::ativo.eq(false))
        .execute(&mut conn)
        .map_err(|_| RepositorioError::Dados)?;

        Ok(linhas > 0)
    }

    fn atualizar(
        &self,
        escritorio_id_param: EscritorioId,
        certificado: &Certificado,
    ) -> Result<(), RepositorioError> {
        let mut conn = self.obter_conexao()?;

        diesel::update(
            certificados_a1::table
                .filter(certificados_a1::id.eq(certificado.id))
                .filter(certificados_a1::escritorio_id.eq(escritorio_id_param.as_uuid())),
        )
        .set((
            certificados_a1::ativo.eq(certificado.ativo),
            certificados_a1::valido_ate.eq(certificado.valido_ate),
        ))
        .execute(&mut conn)
        .map_err(|_| RepositorioError::Dados)?;

        Ok(())
    }
}

impl AuditoriaCertificadoRepositorio for CertificadoRepositorioDiesel {
    fn registrar(&self, evento: &EventoAuditoriaCertificado) -> Result<(), RepositorioError> {
        let mut conn = self.obter_conexao()?;

        use crate::infrastructure::persistence::schema::auditoria_eventos::dsl::auditoria_eventos;

        let ip_net = evento.ip.map(|ip| match ip {
            std::net::IpAddr::V4(v4) => {
                ipnet::IpNet::V4(ipnet::Ipv4Net::new(v4, 32).expect("valid IPv4"))
            }
            std::net::IpAddr::V6(v6) => {
                ipnet::IpNet::V6(ipnet::Ipv6Net::new(v6, 128).expect("valid IPv6"))
            }
        });

        let novo = AuditoriaEventoNovo {
            id: Uuid::now_v7(),
            escritorio_id: Some(evento.escritorio_id.as_uuid()),
            usuario_id: Some(evento.usuario_id),
            acao: evento.acao.clone(),
            entidade: Some(evento.entidade.clone()),
            entidade_id: Some(evento.entidade_id),
            valor_anterior: evento.valor_anterior.clone(),
            valor_novo: evento.valor_novo.clone(),
            ip: ip_net,
            criado_em: chrono::Utc::now(),
        };

        diesel::insert_into(auditoria_eventos)
            .values(&novo)
            .execute(&mut conn)
            .map_err(|_| RepositorioError::Dados)?;

        Ok(())
    }
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = certificados_a1)]
struct CertificadoDb {
    id: Uuid,
    escritorio_id: Uuid,
    tipo: String,
    titular_cpf_cnpj: String,
    conteudo_cifrado: Vec<u8>,
    senha_cifrada: Option<Vec<u8>>,
    algoritmo: String,
    fingerprint_sha256: String,
    valido_de: chrono::DateTime<chrono::Utc>,
    valido_ate: chrono::DateTime<chrono::Utc>,
    ativo: bool,
    criado_em: chrono::DateTime<chrono::Utc>,
}

impl From<CertificadoDb> for Certificado {
    fn from(db: CertificadoDb) -> Self {
        Self {
            id: db.id,
            escritorio_id: crate::domain::tenancy::EscritorioId::de_uuid(db.escritorio_id),
            tipo: db
                .tipo
                .parse()
                .unwrap_or(crate::domain::certificados::TipoCertificado::A1),
            titular_cpf_cnpj: db.titular_cpf_cnpj,
            conteudo_cifrado: db.conteudo_cifrado,
            senha_cifrada: db.senha_cifrada,
            algoritmo: db
                .algoritmo
                .parse()
                .unwrap_or(crate::domain::certificados::AlgoritmoCifragem::Aes256Gcm),
            fingerprint: FingerprintSha256::novo(&db.fingerprint_sha256)
                .expect("fingerprint válido do banco"),
            valido_de: db.valido_de,
            valido_ate: db.valido_ate,
            ativo: db.ativo,
            criado_em: db.criado_em,
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = certificados_a1)]
struct CertificadoNovo {
    id: Uuid,
    escritorio_id: Uuid,
    tipo: String,
    titular_cpf_cnpj: String,
    conteudo_cifrado: Vec<u8>,
    senha_cifrada: Option<Vec<u8>>,
    algoritmo: String,
    fingerprint_sha256: String,
    valido_de: chrono::DateTime<chrono::Utc>,
    valido_ate: chrono::DateTime<chrono::Utc>,
    ativo: bool,
    criado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::infrastructure::persistence::schema::auditoria_eventos)]
struct AuditoriaEventoNovo {
    id: Uuid,
    escritorio_id: Option<Uuid>,
    usuario_id: Option<Uuid>,
    acao: String,
    entidade: Option<String>,
    entidade_id: Option<Uuid>,
    valor_anterior: Option<serde_json::Value>,
    valor_novo: Option<serde_json::Value>,
    ip: Option<ipnet::IpNet>,
    criado_em: chrono::DateTime<chrono::Utc>,
}

use diesel::PgConnection;
use diesel::r2d2::PooledConnection;
