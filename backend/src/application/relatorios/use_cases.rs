//! Fase F — Casos de uso de relatórios e link seguro de compartilhamento.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::clientes::ClienteRepositorio;
use crate::application::relatorios::gerador::gerar_conteudo;
use crate::domain::clientes::{FiltroClientes, Paginacao};
use crate::domain::relatorios::entidades::{
    LinkCompartilhamento, NovoLinkCompartilhamento, NovoRelatorio, Relatorio,
};
use crate::domain::relatorios::enums::{FormatoRelatorio, StatusRelatorio, TipoRelatorio};
use crate::domain::relatorios::erros::ErroRelatorio;
use crate::domain::relatorios::ports::{RepositorioLinksCompartilhamento, RepositorioRelatorios};

/// Gerar um relatório (situação fiscal, histórico de consultas ou
/// pendências por carteira) para o escritório do usuário autenticado.
pub struct GerarRelatorioUseCase {
    relatorios: Arc<dyn RepositorioRelatorios>,
    clientes: Arc<dyn ClienteRepositorio>,
}

impl GerarRelatorioUseCase {
    pub fn novo(
        relatorios: Arc<dyn RepositorioRelatorios>,
        clientes: Arc<dyn ClienteRepositorio>,
    ) -> Self {
        Self {
            relatorios,
            clientes,
        }
    }

    pub async fn executar(
        &self,
        comando: ComandoGerarRelatorio,
    ) -> Result<Relatorio, ErroRelatorio> {
        let agora = Utc::now();
        let expira_em = agora + chrono::Duration::days(30);

        // Cria o relatório com status "gerando".
        let relatorio = self
            .relatorios
            .criar(&NovoRelatorio {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                cliente_id: comando.cliente_id,
                tipo_relatorio: comando.tipo_relatorio,
                formato: comando.formato,
                competencia: comando.competencia,
            })
            .await?;

        // Busca os clientes do escritório em páginas de até 100 (limite do
        // repositório), respeitando a carteira do usuário quando informado.
        let mut itens = Vec::new();
        let mut pagina = 1;
        loop {
            let listagem = self
                .clientes
                .listar(
                    comando.escritorio_id,
                    FiltroClientes::Todos,
                    Paginacao {
                        pagina,
                        tamanho: 100,
                    },
                    comando.apenas_ids.as_deref(),
                )
                .map_err(|e| ErroRelatorio::Persistencia(e.to_string()))?;
            let recebidos = listagem.itens.len();
            itens.extend(listagem.itens);
            if recebidos == 0 || itens.len() as i64 >= listagem.total {
                break;
            }
            pagina += 1;
        }

        // Gera o conteúdo (PDF ou XLSX).
        let (conteudo, nome_arquivo) = gerar_conteudo(
            &comando.tipo_relatorio,
            &comando.formato,
            &itens,
            comando.escritorio_id,
        )
        .map_err(ErroRelatorio::Interno)?;

        // Atualiza o relatório com o conteúdo gerado.
        self.relatorios
            .atualizar_status_e_conteudo(
                relatorio.id,
                StatusRelatorio::Concluido,
                Some(conteudo.clone()),
                Some(nome_arquivo.clone()),
            )
            .await?;

        Ok(Relatorio {
            id: relatorio.id,
            escritorio_id: relatorio.escritorio_id,
            usuario_id: relatorio.usuario_id,
            cliente_id: relatorio.cliente_id,
            tipo_relatorio: relatorio.tipo_relatorio,
            formato: relatorio.formato,
            status: StatusRelatorio::Concluido,
            conteudo: Some(conteudo),
            nome_arquivo: Some(nome_arquivo),
            competencia: relatorio.competencia,
            expira_em,
            criado_em: relatorio.criado_em,
            atualizado_em: agora,
        })
    }
}

/// Listar relatórios de um escritório.
pub struct ListarRelatoriosUseCase {
    relatorios: Arc<dyn RepositorioRelatorios>,
}

impl ListarRelatoriosUseCase {
    pub fn novo(relatorios: Arc<dyn RepositorioRelatorios>) -> Self {
        Self { relatorios }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<RelatorioResumo>, ErroRelatorio> {
        let relatorios = self
            .relatorios
            .listar_por_escritorio(escritorio_id, limite)
            .await?;
        Ok(relatorios
            .into_iter()
            .map(|r| RelatorioResumo {
                id: r.id,
                tipo_relatorio: r.tipo_relatorio.as_str().to_owned(),
                formato: r.formato.as_str().to_owned(),
                status: r.status.as_str().to_owned(),
                nome_arquivo: r.nome_arquivo,
                expira_em: r.expira_em,
                criado_em: r.criado_em,
            })
            .collect())
    }
}

/// Buscar um relatório por ID (somente do escritório do usuário).
pub struct BuscarRelatorioUseCase {
    relatorios: Arc<dyn RepositorioRelatorios>,
}

impl BuscarRelatorioUseCase {
    pub fn novo(relatorios: Arc<dyn RepositorioRelatorios>) -> Self {
        Self { relatorios }
    }

    pub async fn executar(
        &self,
        id: Uuid,
        escritorio_id: Uuid,
    ) -> Result<Relatorio, ErroRelatorio> {
        let relatorio = self
            .relatorios
            .buscar_por_id(id)
            .await?
            .ok_or(ErroRelatorio::RelatorioNaoEncontrado)?;

        if relatorio.escritorio_id != escritorio_id {
            return Err(ErroRelatorio::RelatorioNaoEncontrado);
        }

        Ok(relatorio)
    }
}

/// Criar um link de compartilhamento para um relatório.
pub struct CriarLinkCompartilhamentoUseCase {
    links: Arc<dyn RepositorioLinksCompartilhamento>,
    relatorios: Arc<dyn RepositorioRelatorios>,
}

impl CriarLinkCompartilhamentoUseCase {
    pub fn novo(
        links: Arc<dyn RepositorioLinksCompartilhamento>,
        relatorios: Arc<dyn RepositorioRelatorios>,
    ) -> Self {
        Self { links, relatorios }
    }

    pub async fn executar(
        &self,
        comando: ComandoCriarLink,
    ) -> Result<(LinkCompartilhamento, String), ErroRelatorio> {
        // Verifica se o relatório existe e pertence ao escritório.
        let _relatorio = self
            .relatorios
            .buscar_por_id(comando.relatorio_id)
            .await?
            .ok_or(ErroRelatorio::RelatorioNaoEncontrado)?;

        if _relatorio.escritorio_id != comando.escritorio_id {
            return Err(ErroRelatorio::RelatorioNaoEncontrado);
        }

        // Token opaco com alta entropia; o banco guarda apenas o hash.
        let token = format!("{}{}", Uuid::now_v7().simple(), Uuid::now_v7().simple());
        let max_acessos = comando.max_acessos.clamp(1, 1000);
        let link = self
            .links
            .criar(&NovoLinkCompartilhamento {
                relatorio_id: comando.relatorio_id,
                token_hash: crate::domain::relatorios::entidades::hash_token(&token),
                expira_em: comando.expira_em,
                max_acessos,
            })
            .await?;

        Ok((link, token))
    }
}

/// Validar um token de link de compartilhamento (existe e não expirou).
pub struct ValidarLinkCompartilhamentoUseCase {
    links: Arc<dyn RepositorioLinksCompartilhamento>,
}

impl ValidarLinkCompartilhamentoUseCase {
    pub fn novo(links: Arc<dyn RepositorioLinksCompartilhamento>) -> Self {
        Self { links }
    }

    pub async fn executar(
        &self,
        token: &str,
        agora: DateTime<Utc>,
    ) -> Result<LinkCompartilhamento, ErroRelatorio> {
        let link = self
            .links
            .buscar_por_token(token)
            .await?
            .ok_or(ErroRelatorio::LinkNaoEncontrado)?;

        if agora > link.expira_em {
            return Err(ErroRelatorio::LinkExpirado);
        }
        if link.revogado_em.is_some() {
            return Err(ErroRelatorio::LinkRevogado);
        }
        if link.acessos >= link.max_acessos {
            return Err(ErroRelatorio::LinkLimiteAtingido);
        }

        Ok(link)
    }
}

/// Acessar o conteúdo de um relatório por link público (somente leitura).
pub struct AcessarRelatorioPorLinkUseCase {
    links: Arc<dyn RepositorioLinksCompartilhamento>,
    relatorios: Arc<dyn RepositorioRelatorios>,
}

impl AcessarRelatorioPorLinkUseCase {
    pub fn novo(
        links: Arc<dyn RepositorioLinksCompartilhamento>,
        relatorios: Arc<dyn RepositorioRelatorios>,
    ) -> Self {
        Self { links, relatorios }
    }

    pub async fn executar(
        &self,
        token: &str,
        agora: DateTime<Utc>,
    ) -> Result<RelatorioParaCompartilhamento, ErroRelatorio> {
        let link = self.validar(token, agora).await?;

        // Registra o acesso ao link.
        self.links.registrar_acesso(link.id).await?;

        // Busca o relatório.
        let relatorio = self
            .relatorios
            .buscar_por_id(link.relatorio_id)
            .await?
            .ok_or(ErroRelatorio::RelatorioNaoEncontrado)?;

        if relatorio.status != StatusRelatorio::Concluido {
            return Err(ErroRelatorio::RelatorioNaoEncontrado);
        }

        Ok(RelatorioParaCompartilhamento {
            id: relatorio.id,
            tipo_relatorio: relatorio.tipo_relatorio.as_str().to_owned(),
            formato: relatorio.formato.as_str().to_owned(),
            conteudo: relatorio
                .conteudo
                .ok_or(ErroRelatorio::RelatorioNaoEncontrado)?,
            nome_arquivo: relatorio
                .nome_arquivo
                .unwrap_or_else(|| "relatorio".to_owned()),
        })
    }

    async fn validar(
        &self,
        token: &str,
        agora: DateTime<Utc>,
    ) -> Result<LinkCompartilhamento, ErroRelatorio> {
        let link = self
            .links
            .buscar_por_token(token)
            .await?
            .ok_or(ErroRelatorio::LinkNaoEncontrado)?;

        if agora > link.expira_em {
            return Err(ErroRelatorio::LinkExpirado);
        }
        if link.revogado_em.is_some() {
            return Err(ErroRelatorio::LinkRevogado);
        }
        if link.acessos >= link.max_acessos {
            return Err(ErroRelatorio::LinkLimiteAtingido);
        }

        Ok(link)
    }
}

/// Revoga todos os links de compartilhamento de um relatório do escritório.
pub struct RevogarLinksRelatorioUseCase {
    links: Arc<dyn RepositorioLinksCompartilhamento>,
    relatorios: Arc<dyn RepositorioRelatorios>,
}

impl RevogarLinksRelatorioUseCase {
    pub fn novo(
        links: Arc<dyn RepositorioLinksCompartilhamento>,
        relatorios: Arc<dyn RepositorioRelatorios>,
    ) -> Self {
        Self { links, relatorios }
    }

    pub async fn executar(
        &self,
        relatorio_id: Uuid,
        escritorio_id: Uuid,
    ) -> Result<u64, ErroRelatorio> {
        let relatorio = self
            .relatorios
            .buscar_por_id(relatorio_id)
            .await?
            .ok_or(ErroRelatorio::RelatorioNaoEncontrado)?;
        if relatorio.escritorio_id != escritorio_id {
            return Err(ErroRelatorio::RelatorioNaoEncontrado);
        }
        self.links.revogar_por_relatorio(relatorio_id).await
    }
}

/// Comando para gerar um relatório.
#[derive(Debug, Clone)]
pub struct ComandoGerarRelatorio {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub tipo_relatorio: TipoRelatorio,
    pub formato: FormatoRelatorio,
    pub competencia: Option<String>,
    pub apenas_ids: Option<Vec<Uuid>>,
}

/// Resumo de um relatório para listagem.
#[derive(Debug, Clone)]
pub struct RelatorioResumo {
    pub id: Uuid,
    pub tipo_relatorio: String,
    pub formato: String,
    pub status: String,
    pub nome_arquivo: Option<String>,
    pub expira_em: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
}

/// Relatório pronto para ser compartilhado via link público.
#[derive(Debug, Clone)]
pub struct RelatorioParaCompartilhamento {
    pub id: Uuid,
    pub tipo_relatorio: String,
    pub formato: String,
    pub conteudo: Vec<u8>,
    pub nome_arquivo: String,
}

/// Comando para criar um link de compartilhamento.
#[derive(Debug, Clone)]
pub struct ComandoCriarLink {
    pub relatorio_id: Uuid,
    pub escritorio_id: Uuid,
    pub expira_em: DateTime<Utc>,
    /// Limite de acessos do link (1..=1000; padrão 50).
    pub max_acessos: i32,
}

/// Comando para listar relatórios (compatível com filtros futuros).
#[derive(Debug, Clone, Default)]
pub struct ComandoListarRelatorios {
    pub escritorio_id: Uuid,
    pub limite: i64,
}
