use std::sync::Arc;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::infrastructure::persistence::pool::DbPool;
use crate::infrastructure::persistence::schema::{
    auditoria_eventos, clientes, consentimentos_lgpd, consultas_fiscais, guias_emissao,
    politicas_retencao_escritorio, relatorios, socio_contatos,
};

#[derive(Debug, Clone)]
pub struct PoliticaRetencao {
    pub consultas_dias: i32,
    pub relatorios_dias: i32,
    pub auditoria_dias: i32,
}

type LinhaClienteExport = (
    Uuid,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    DateTime<Utc>,
    DateTime<Utc>,
);

type LinhaConsentimento = (Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>);

#[derive(Debug, Clone)]
pub struct ConsentimentoLgpd {
    pub id: Uuid,
    pub tipo: String,
    pub versao: String,
    pub aceito_em: DateTime<Utc>,
    pub revogado_em: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum ErroLgpd {
    NaoEncontrado,
    Validacao(String),
    Persistencia,
}

#[derive(Clone)]
pub struct LgpdDiesel {
    pool: DbPool,
}

impl LgpdDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl LgpdDiesel {
    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroLgpd,
    > {
        self.pool.get().map_err(|_| ErroLgpd::Persistencia)
    }
}

impl LgpdDiesel {
    pub fn exportar_cliente(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Value, ErroLgpd> {
        use crate::infrastructure::persistence::schema::clientes::dsl as c;
        let mut conexao = self.conexao()?;

        let cliente: Option<LinhaClienteExport> = c::clientes
            .filter(c::id.eq(cliente_id))
            .filter(c::escritorio_id.eq(escritorio_id))
            .select((
                c::id,
                c::tipo,
                c::cpf_cnpj,
                c::nome_razao_social,
                c::nome_fantasia,
                c::email,
                c::telefone,
                c::criado_em,
                c::atualizado_em,
            ))
            .first(&mut conexao)
            .optional()
            .map_err(|_| ErroLgpd::Persistencia)?;
        let Some(cliente) = cliente else {
            return Err(ErroLgpd::NaoEncontrado);
        };

        let consultas: Vec<(Uuid, String, String, String, DateTime<Utc>)> =
            consultas_fiscais::table
                .filter(consultas_fiscais::escritorio_id.eq(escritorio_id))
                .filter(consultas_fiscais::cliente_id.eq(cliente_id))
                .select((
                    consultas_fiscais::id,
                    consultas_fiscais::produto,
                    consultas_fiscais::servico,
                    consultas_fiscais::estado,
                    consultas_fiscais::criado_em,
                ))
                .load(&mut conexao)
                .map_err(|_| ErroLgpd::Persistencia)?;

        let guias: Vec<(Uuid, String, String, DateTime<Utc>)> = guias_emissao::table
            .filter(guias_emissao::escritorio_id.eq(escritorio_id))
            .filter(guias_emissao::cliente_id.eq(cliente_id))
            .select((
                guias_emissao::id,
                guias_emissao::tipo_guia,
                guias_emissao::status,
                guias_emissao::criado_em,
            ))
            .load(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;

        let relatorios_cliente: Vec<(Uuid, String, String, DateTime<Utc>)> = relatorios::table
            .filter(relatorios::escritorio_id.eq(escritorio_id))
            .filter(relatorios::cliente_id.eq(cliente_id))
            .select((
                relatorios::id,
                relatorios::tipo_relatorio,
                relatorios::formato,
                relatorios::criado_em,
            ))
            .load(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;

        let consentimentos: Vec<LinhaConsentimento> = consentimentos_lgpd::table
            .filter(consentimentos_lgpd::escritorio_id.eq(escritorio_id))
            .filter(consentimentos_lgpd::cliente_id.eq(cliente_id))
            .select((
                consentimentos_lgpd::id,
                consentimentos_lgpd::tipo,
                consentimentos_lgpd::versao,
                consentimentos_lgpd::aceito_em,
                consentimentos_lgpd::revogado_em,
            ))
            .order(consentimentos_lgpd::aceito_em.desc())
            .load(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;

        Ok(json!({
            "escritorioId": escritorio_id,
            "geradoEm": Utc::now(),
            "cliente": {
                "id": cliente.0,
                "tipo": cliente.1,
                "cpfCnpj": cliente.2,
                "nomeRazaoSocial": cliente.3,
                "nomeFantasia": cliente.4,
                "email": cliente.5,
                "telefone": cliente.6,
                "criadoEm": cliente.7,
                "atualizadoEm": cliente.8,
            },
            "consultas": consultas.into_iter().map(|(id, produto, servico, status, criado_em)| json!({
                "id": id, "produto": produto, "servico": servico, "status": status, "criadoEm": criado_em
            })).collect::<Vec<_>>(),
            "guias": guias.into_iter().map(|(id, tipo, status, criado_em)| json!({
                "id": id, "tipoGuia": tipo, "status": status, "criadoEm": criado_em
            })).collect::<Vec<_>>(),
            "relatorios": relatorios_cliente.into_iter().map(|(id, tipo, formato, criado_em)| json!({
                "id": id, "tipoRelatorio": tipo, "formato": formato, "criadoEm": criado_em
            })).collect::<Vec<_>>(),
            "consentimentos": consentimentos.into_iter().map(|(id, tipo, versao, aceito_em, revogado_em)| json!({
                "id": id, "tipo": tipo, "versao": versao, "aceitoEm": aceito_em, "revogadoEm": revogado_em
            })).collect::<Vec<_>>(),
        }))
    }

    pub fn anonimizar_cliente(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        ator_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<(), ErroLgpd> {
        use crate::infrastructure::persistence::schema::clientes::dsl as c;
        let mut conexao = self.conexao()?;

        let cliente: Option<(String, Uuid)> = c::clientes
            .filter(c::id.eq(cliente_id))
            .filter(c::escritorio_id.eq(escritorio_id))
            .select((c::tipo, c::id))
            .first(&mut conexao)
            .optional()
            .map_err(|_| ErroLgpd::Persistencia)?;
        let Some((tipo, id)) = cliente else {
            return Err(ErroLgpd::NaoEncontrado);
        };

        let digitos = id.as_u128().to_string();
        let documento = if tipo == "PF" {
            format!("9{}", &digitos[..10])
        } else {
            format!("9{}", &digitos[..13])
        };
        diesel::update(
            c::clientes
                .filter(c::id.eq(cliente_id))
                .filter(c::escritorio_id.eq(escritorio_id)),
        )
        .set((
            c::cpf_cnpj.eq(documento.clone()),
            c::nome_razao_social.eq(format!("Cliente anonimizado {}", &digitos[..8])),
            c::nome_fantasia.eq::<Option<String>>(None),
            c::representante_nome.eq::<Option<String>>(None),
            c::email.eq::<Option<String>>(None),
            c::telefone.eq::<Option<String>>(None),
            c::atualizado_em.eq(Utc::now()),
        ))
        .execute(&mut conexao)
        .map_err(|_| ErroLgpd::Persistencia)?;

        diesel::update(
            socio_contatos::table
                .filter(socio_contatos::escritorio_id.eq(escritorio_id))
                .filter(socio_contatos::cliente_id.eq(cliente_id)),
        )
        .set((
            socio_contatos::nome.eq("Contato anonimizado"),
            socio_contatos::cpf_cnpj.eq::<Option<String>>(None),
            socio_contatos::email.eq::<Option<String>>(None),
            socio_contatos::telefone.eq::<Option<String>>(None),
            socio_contatos::atualizado_em.eq(Utc::now()),
        ))
        .execute(&mut conexao)
        .map_err(|_| ErroLgpd::Persistencia)?;

        diesel::insert_into(auditoria_eventos::table)
            .values((
                auditoria_eventos::id.eq(Uuid::now_v7()),
                auditoria_eventos::escritorio_id.eq(escritorio_id),
                auditoria_eventos::usuario_id.eq(Some(ator_id)),
                auditoria_eventos::administrador_id.eq(None::<Uuid>),
                auditoria_eventos::acao.eq("LGPD_CLIENTE_ANONIMIZADO"),
                auditoria_eventos::entidade.eq(Some("clientes")),
                auditoria_eventos::entidade_id.eq(Some(cliente_id)),
                auditoria_eventos::valor_novo.eq(Some(json!({
                    "cpfCnpj": documento,
                    "anonimizado": true
                }))),
                auditoria_eventos::ip.eq(ip.map(ipnet::IpNet::from)),
            ))
            .execute(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;
        Ok(())
    }
    pub fn obter_retencao(&self, escritorio_id: Uuid) -> Result<PoliticaRetencao, ErroLgpd> {
        let mut conexao = self.conexao()?;
        let linha: Option<(i32, i32, i32)> = politicas_retencao_escritorio::table
            .filter(politicas_retencao_escritorio::escritorio_id.eq(escritorio_id))
            .select((
                politicas_retencao_escritorio::consultas_dias,
                politicas_retencao_escritorio::relatorios_dias,
                politicas_retencao_escritorio::auditoria_dias,
            ))
            .first(&mut conexao)
            .optional()
            .map_err(|_| ErroLgpd::Persistencia)?;
        Ok(linha
            .map(
                |(consultas_dias, relatorios_dias, auditoria_dias)| PoliticaRetencao {
                    consultas_dias,
                    relatorios_dias,
                    auditoria_dias,
                },
            )
            .unwrap_or(PoliticaRetencao {
                consultas_dias: 90,
                relatorios_dias: 30,
                auditoria_dias: 365,
            }))
    }

    pub fn salvar_retencao(
        &self,
        escritorio_id: Uuid,
        politica: &PoliticaRetencao,
    ) -> Result<(), ErroLgpd> {
        if politica.consultas_dias < 1
            || politica.relatorios_dias < 1
            || politica.auditoria_dias < 1
            || politica.consultas_dias > 3650
            || politica.relatorios_dias > 3650
            || politica.auditoria_dias > 3650
        {
            return Err(ErroLgpd::Validacao(
                "prazos de retenção devem ficar entre 1 e 3650 dias".into(),
            ));
        }
        let mut conexao = self.conexao()?;
        diesel::sql_query(
            "INSERT INTO politicas_retencao_escritorio \
             (escritorio_id, consultas_dias, relatorios_dias, auditoria_dias) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (escritorio_id) DO UPDATE SET \
               consultas_dias = EXCLUDED.consultas_dias, \
               relatorios_dias = EXCLUDED.relatorios_dias, \
               auditoria_dias = EXCLUDED.auditoria_dias, \
               atualizado_em = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(escritorio_id)
        .bind::<diesel::sql_types::Integer, _>(politica.consultas_dias)
        .bind::<diesel::sql_types::Integer, _>(politica.relatorios_dias)
        .bind::<diesel::sql_types::Integer, _>(politica.auditoria_dias)
        .execute(&mut conexao)
        .map_err(|_| ErroLgpd::Persistencia)?;
        Ok(())
    }

    pub fn registrar_consentimento(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        tipo: &str,
        versao: &str,
        ip: Option<std::net::IpAddr>,
    ) -> Result<ConsentimentoLgpd, ErroLgpd> {
        if !matches!(
            tipo,
            "tratamento_dados" | "compartilhamento_serpro" | "comunicacoes"
        ) {
            return Err(ErroLgpd::Validacao("tipo de consentimento inválido".into()));
        }
        let mut conexao = self.conexao()?;
        let existe: bool = diesel::select(diesel::dsl::exists(
            clientes::table
                .filter(clientes::id.eq(cliente_id))
                .filter(clientes::escritorio_id.eq(escritorio_id)),
        ))
        .get_result(&mut conexao)
        .map_err(|_| ErroLgpd::Persistencia)?;
        if !existe {
            return Err(ErroLgpd::NaoEncontrado);
        }
        let id = Uuid::now_v7();
        let criado_em = Utc::now();
        diesel::insert_into(consentimentos_lgpd::table)
            .values((
                consentimentos_lgpd::id.eq(id),
                consentimentos_lgpd::escritorio_id.eq(escritorio_id),
                consentimentos_lgpd::cliente_id.eq(cliente_id),
                consentimentos_lgpd::tipo.eq(tipo),
                consentimentos_lgpd::versao.eq(versao),
                consentimentos_lgpd::aceito_em.eq(criado_em),
                consentimentos_lgpd::ip.eq(ip.map(ipnet::IpNet::from)),
            ))
            .execute(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;
        Ok(ConsentimentoLgpd {
            id,
            tipo: tipo.to_owned(),
            versao: versao.to_owned(),
            aceito_em: criado_em,
            revogado_em: None,
        })
    }

    pub fn listar_consentimentos(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<ConsentimentoLgpd>, ErroLgpd> {
        let mut conexao = self.conexao()?;
        let linhas: Vec<LinhaConsentimento> = consentimentos_lgpd::table
            .filter(consentimentos_lgpd::escritorio_id.eq(escritorio_id))
            .filter(consentimentos_lgpd::cliente_id.eq(cliente_id))
            .select((
                consentimentos_lgpd::id,
                consentimentos_lgpd::tipo,
                consentimentos_lgpd::versao,
                consentimentos_lgpd::aceito_em,
                consentimentos_lgpd::revogado_em,
            ))
            .order(consentimentos_lgpd::aceito_em.desc())
            .load(&mut conexao)
            .map_err(|_| ErroLgpd::Persistencia)?;
        Ok(linhas
            .into_iter()
            .map(
                |(id, tipo, versao, aceito_em, revogado_em)| ConsentimentoLgpd {
                    id,
                    tipo,
                    versao,
                    aceito_em,
                    revogado_em,
                },
            )
            .collect())
    }
}

pub type LgpdArc = Arc<LgpdDiesel>;
