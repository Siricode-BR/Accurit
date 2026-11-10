//! Fase E — Importação em massa de clientes (CSV/XLSX).
//!
//! Duas etapas: `analisar` (prévia sem persistência, com relatório de erros
//! por linha) e `confirmar` (persistência dos clientes válidos, contando
//! duplicados ignorados). O conteúdo é reanalisado no `confirmar`, de modo
//! que a prévia é apenas informativa.

use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use crate::application::clientes::ports::{
    AuditoriaClienteRepositorio, ClienteError, ClienteRepositorio, EventoAuditoriaCliente,
    RepositorioError,
};
use crate::domain::clientes::{Cliente, CpfCnpj, NovoCliente, PorteRegime, TipoCliente};

/// Exige tipo, documento e nome mapeados no cabeçalho.
fn campos_essenciais_mapeados(ordens: &[Option<(&'static str, usize)>]) -> bool {
    let mapeados: Vec<&str> = ordens.iter().flatten().map(|(c, _)| *c).collect();
    mapeados.contains(&"tipo") && mapeados.contains(&"cpf_cnpj") && mapeados.contains(&"nome")
}

/// Quantidade máxima de linhas aceitas por importação.
pub const MAX_LINHAS: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FormatoImportacao {
    Csv,
    Xlsx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErroLinhaImportacao {
    pub linha: usize,
    pub campo: Option<String>,
    pub mensagem: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinhaClienteImportacao {
    pub linha: usize,
    pub tipo: String,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub representante_nome: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub porte_regime: Option<String>,
}

/// Resultado da análise prévia (nada é gravado aqui).
#[derive(Debug, Clone, Serialize)]
pub struct AnaliseImportacao {
    pub formato: FormatoImportacao,
    pub total_linhas: usize,
    pub validas: Vec<LinhaClienteImportacao>,
    pub erros: Vec<ErroLinhaImportacao>,
}

/// Resultado da confirmação.
#[derive(Debug, Clone, Serialize)]
pub struct RelatorioImportacao {
    pub criados: usize,
    pub ignorados_duplicados: usize,
    pub erros: Vec<ErroLinhaImportacao>,
}

/// Nomes aceitos de coluna, normalizados (sem acentos, minúsculo).
const COL_TIPO: &[&str] = &["tipo"];
const COL_DOCUMENTO: &[&str] = &[
    "cpf_cnpj",
    "cpf-cnpj",
    "documento",
    "cpf",
    "cnpj",
    "cpf/cnpj",
];
const COL_NOME: &[&str] = &["nome_razao_social", "nome", "razao_social", "razao"];
const COL_FANTASIA: &[&str] = &["nome_fantasia", "fantasia"];
const COL_REPRESENTANTE: &[&str] = &["representante_nome", "representante"];
const COL_EMAIL: &[&str] = &["email", "e-mail", "correo"];
const COL_TELEFONE: &[&str] = &["telefone", "fone", "tel", "celular"];
const COL_PORTE: &[&str] = &["porte_regime", "porte", "regime", "regime_tributario"];

fn normalizar(texto: &str) -> String {
    texto.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// Grupos de colunas; em cada lista, o primeiro nome é o canônico.
/// (`indice`, `nomes`).
const COLS_FASE_E: &[(&[&[&str]], usize)] = &[
    (&[COL_TIPO], 0),
    (&[COL_DOCUMENTO], 1),
    (&[COL_NOME], 2),
    (&[COL_FANTASIA], 3),
    (&[COL_REPRESENTANTE], 4),
    (&[COL_EMAIL], 5),
    (&[COL_TELEFONE], 6),
    (&[COL_PORTE], 7),
];

fn mapear_campo(nome: &str) -> Option<(&'static str, usize)> {
    let norm = normalizar(nome);
    for (grupos, ordem) in COLS_FASE_E {
        for lista in *grupos {
            if lista.contains(&norm.as_str()) {
                return Some((lista[0], *ordem));
            }
        }
    }
    None
}

/// Somente dígitos; descarta pontuação e aparas não numéricas.
fn so_digitos(texto: &str) -> String {
    texto
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
}

fn limpar_opcional(texto: &str) -> Option<String> {
    let t = texto.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

fn normalizar_tipo(texto: &str) -> Option<TipoCliente> {
    match normalizar(texto).as_str() {
        "pf" => Some(TipoCliente::Pf),
        "pj" => Some(TipoCliente::Pj),
        _ => None,
    }
}

fn normalizar_porte(texto: &str) -> Option<PorteRegime> {
    let norm = normalizar(texto).replace("__", "_");
    match norm.as_str() {
        "mei" => Some(PorteRegime::Mei),
        "simples" | "simples_nacional" => Some(PorteRegime::SimplesNacional),
        "presumido" | "lucro_presumido" => Some(PorteRegime::LucroPresumido),
        "real" | "lucro_real" => Some(PorteRegime::LucroReal),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroAnalise {
    SemDados,
    CabecalhoInvalido,
    FormatoNaoReconhecido,
}

impl std::fmt::Display for ErroAnalise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemDados => write!(f, "arquivo sem linhas"),
            Self::CabecalhoInvalido => {
                write!(
                    f,
                    "cabeçalho obrigatório ausente: tipo, cpf_cnpj, nome_razao_social"
                )
            }
            Self::FormatoNaoReconhecido => write!(f, "formato de arquivo não reconhecido"),
        }
    }
}

/// Extrai as linhas brutas do arquivo (após o cabeçalho).
fn linhas_do_arquivo(
    conteudo: &[u8],
    formato: FormatoImportacao,
) -> Result<Vec<Vec<String>>, ErroAnalise> {
    match formato {
        FormatoImportacao::Csv => {
            let texto = String::from_utf8_lossy(conteudo);
            let mut leitor = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(Cursor::new(texto.as_bytes()));
            let cabecalhos = leitor
                .headers()
                .map_err(|_| ErroAnalise::FormatoNaoReconhecido)?
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>();
            let ordens: Vec<Option<(&'static str, usize)>> =
                cabecalhos.iter().map(|h| mapear_campo(h)).collect();
            if !campos_essenciais_mapeados(&ordens) {
                return Err(ErroAnalise::CabecalhoInvalido);
            }
            let mut linhas = Vec::new();
            for resultado in leitor.records() {
                let registro = resultado.map_err(|_| ErroAnalise::FormatoNaoReconhecido)?;
                let mut campos = vec![String::new(); 8];
                for (i, ordem) in ordens.iter().enumerate() {
                    if let Some((_, o)) = ordem {
                        campos[*o] = registro.get(i).map(str::to_owned).unwrap_or_default();
                    }
                }
                linhas.push(campos);
            }
            Ok(linhas)
        }
        FormatoImportacao::Xlsx => {
            use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
            let mut workbook: Xlsx<_> = open_workbook_from_rs(Cursor::new(conteudo.to_vec()))
                .map_err(|_| ErroAnalise::FormatoNaoReconhecido)?;
            let range = workbook
                .worksheet_range_at(0)
                .ok_or(ErroAnalise::SemDados)?
                .map_err(|_| ErroAnalise::FormatoNaoReconhecido)?;
            let celula = |d: &Data| -> String {
                match d {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Bool(b) => b.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => format!("{f:.0}"),
                    outro => outro.to_string(),
                }
            };
            let mut linhas = range
                .rows()
                .map(|row| row.iter().map(celula).collect::<Vec<_>>());
            let cabecalho = linhas.next().ok_or(ErroAnalise::SemDados)?;
            let ordens: Vec<Option<(&'static str, usize)>> =
                cabecalho.iter().map(|h| mapear_campo(h)).collect();
            if !campos_essenciais_mapeados(&ordens) {
                return Err(ErroAnalise::CabecalhoInvalido);
            }
            Ok(linhas
                .map(|linha| {
                    let mut campos = vec![String::new(); 8];
                    for (i, ordem) in ordens.iter().enumerate() {
                        if let Some((_, o)) = ordem {
                            campos[*o] = linha.get(i).cloned().unwrap_or_default();
                        }
                    }
                    campos
                })
                .collect())
        }
    }
}

/// Analisador de arquivo: teste unitário foca na validação por linha.
fn validar_linha(
    numero: usize,
    campos: &[String],
) -> Result<LinhaClienteImportacao, ErroLinhaImportacao> {
    let erro = |campo: Option<&str>, msg: String| ErroLinhaImportacao {
        linha: numero,
        campo: campo.map(str::to_owned),
        mensagem: msg,
    };
    let tipo = normalizar_tipo(&campos[0])
        .ok_or_else(|| erro(Some("tipo"), "tipo deve ser PF ou PJ".into()))?;
    let digitos = so_digitos(&campos[1]);
    if digitos.len() < 11 {
        return Err(erro(
            Some("cpf_cnpj"),
            "CPF/CNPJ precisa de 11 ou 14 dígitos".into(),
        ));
    }
    let documento = CpfCnpj::novo(&digitos)
        .map_err(|e| erro(Some("cpf_cnpj"), format!("CPF/CNPJ inválido: {e}")))?;
    let nome_razao_social = campos[2].trim().to_owned();
    let porte_regime = limpar_opcional(&campos[7]).map(|v| {
        normalizar_porte(&v).ok_or_else(|| {
            erro(
                Some("porte_regime"),
                "porte/regime deve ser MEI, SIMPLES_NACIONAL, LUCRO_PRESUMIDO ou LUCRO_REAL".into(),
            )
        })
    });
    let porte_regime = match porte_regime {
        Some(Ok(v)) => Some(v),
        Some(Err(e)) => return Err(e),
        None => None,
    };
    let novo = NovoCliente {
        tipo,
        porte_regime,
        documento,
        nome_razao_social: nome_razao_social.clone(),
        nome_fantasia: limpar_opcional(&campos[3]),
        representante_nome: limpar_opcional(&campos[4]),
        email: limpar_opcional(&campos[5]),
        telefone: limpar_opcional(&campos[6]),
    };
    let documento_texto = novo.documento.como_str().to_owned();
    novo.validar()
        .map_err(|e| erro(None, format!("validação: {e}")))?;
    Ok(LinhaClienteImportacao {
        linha: numero,
        tipo: tipo.como_str().to_owned(),
        cpf_cnpj: documento_texto,
        nome_razao_social,
        nome_fantasia: novo.nome_fantasia,
        representante_nome: novo.representante_nome,
        email: novo.email,
        telefone: novo.telefone,
        porte_regime: porte_regime.map(|p| p.como_str().to_owned()),
    })
}

/// Contrato de análise: dado o conteúdo cru, devolve válidas + erros.
fn analisar_conteudo(
    conteudo: &[u8],
    formato: FormatoImportacao,
) -> Result<AnaliseImportacao, ErroAnalise> {
    let linhas = linhas_do_arquivo(conteudo, formato)?;
    if linhas.is_empty() {
        return Err(ErroAnalise::SemDados);
    }
    if linhas.len() > MAX_LINHAS {
        return Err(ErroAnalise::FormatoNaoReconhecido);
    }
    let mut validas = Vec::new();
    let mut erros = Vec::new();
    for (indice, campos) in linhas.iter().enumerate() {
        if campos.iter().all(|c| c.trim().is_empty()) {
            continue; // linha vazia no meio do arquivo
        }
        match validar_linha(indice + 1, campos) {
            Ok(linha) => validas.push(linha),
            Err(e) => erros.push(e),
        }
    }
    Ok(AnaliseImportacao {
        formato,
        total_linhas: validas.len() + erros.len(),
        validas,
        erros,
    })
}

/// Caso de uso da importação em massa.
#[derive(Clone)]
pub struct ImportarClientesUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl ImportarClientesUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            clientes,
            auditoria,
        }
    }

    /// Etapa 1: prévia. Nada persiste.
    pub fn analisar(
        &self,
        conteudo: &[u8],
        formato: FormatoImportacao,
    ) -> Result<AnaliseImportacao, ClienteError> {
        analisar_conteudo(conteudo, formato).map_err(|e| ClienteError::Validacao(e.to_string()))
    }

    /// Etapa 2: confirmar, criando apenas as linhas ainda válidas e não
    /// duplicadas no escritório.
    #[allow(clippy::too_many_arguments)]
    pub fn confirmar(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        linhas: &[LinhaClienteImportacao],
        ip: Option<std::net::IpAddr>,
    ) -> Result<RelatorioImportacao, ClienteError> {
        if linhas.len() > MAX_LINHAS {
            return Err("importação excede o limite de linhas".into());
        }
        let mut relatorio = RelatorioImportacao {
            criados: 0,
            ignorados_duplicados: 0,
            erros: Vec::new(),
        };
        for linha in linhas {
            let resultado = self.criar_um(escritorio_id, usuario_id, linha, ip);
            match resultado {
                Ok(_) => relatorio.criados += 1,
                Err(ClienteError::Repositorio(RepositorioError::Duplicado)) => {
                    relatorio.ignorados_duplicados += 1;
                }
                Err(e) => relatorio.erros.push(ErroLinhaImportacao {
                    linha: linha.linha,
                    campo: None,
                    mensagem: e.to_string(),
                }),
            }
        }
        Ok(relatorio)
    }

    fn criar_um(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        linha: &LinhaClienteImportacao,
        ip: Option<std::net::IpAddr>,
    ) -> Result<(), ClienteError> {
        let tipo = TipoCliente::from_str(&linha.tipo).map_err(|_| "tipo inválido")?;
        let porte_regime = linha
            .porte_regime
            .as_deref()
            .map(PorteRegime::from_str)
            .transpose()
            .map_err(|_| "porte/regime inválido")?;
        let documento = CpfCnpj::novo(&linha.cpf_cnpj)?;
        if self
            .clientes
            .buscar_por_cpf_cnpj(escritorio_id, documento.como_str())?
            .is_some()
        {
            return Err(ClienteError::Repositorio(RepositorioError::Duplicado));
        }
        let cliente = Cliente {
            id: Uuid::now_v7(),
            escritorio_id,
            tipo,
            porte_regime,
            documento,
            nome_razao_social: linha.nome_razao_social.trim().to_owned(),
            nome_fantasia: linha.nome_fantasia.clone(),
            representante_nome: linha.representante_nome.clone(),
            email: linha.email.clone(),
            telefone: linha.telefone.clone(),
            criado_em: chrono::Utc::now(),
            atualizado_em: chrono::Utc::now(),
        };
        self.clientes.criar(escritorio_id, &cliente)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id,
            usuario_id,
            acao: "CLIENTE_IMPORTADO",
            entidade: "clientes",
            entidade_id: cliente.id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({
                "linha": linha.linha,
                "cpf_cnpj": cliente.documento.como_str(),
            })),
            ip,
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV_OK: &str = "tipo,cpf_cnpj,nome_razao_social,nome_fantasia,representante_nome,email,telefone,porte_regime\nPF,52998224725,Maria Silva,,,'',,\nPJ,11444777000161,Alfa Serviços,Alfa,João Alfa,contato@alfa.com,11999999999,MEI\n";

    const CSV_COM_ERRO: &str = "tipo,cpf_cnpj,nome_razao_social,nome_fantasia,representante_nome,email,telefone,porte_regime\nPF,11111111111,Maria,,,\nJS,123,Maria,,,\n";

    #[test]
    fn csv_validado() {
        let analise =
            analisar_conteudo(CSV_OK.as_bytes(), FormatoImportacao::Csv).expect("análise");
        assert_eq!(analise.total_linhas, 2);
        assert_eq!(analise.validas.len(), 2);
        assert_eq!(analise.erros.len(), 0);
        assert_eq!(analise.validas[0].cpf_cnpj, "52998224725");
        assert_eq!(analise.validas[1].porte_regime.as_deref(), Some("MEI"));
    }

    #[test]
    fn csv_com_erro_por_linha() {
        let analise =
            analisar_conteudo(CSV_COM_ERRO.as_bytes(), FormatoImportacao::Csv).expect("análise");
        assert_eq!(analise.validas.len(), 0);
        assert_eq!(analise.erros.len(), 2);
        assert_eq!(analise.erros[0].linha, 1);
        assert!(analise.erros[0].mensagem.contains("inválido"));
        assert_eq!(analise.erros[1].campo.as_deref(), Some("tipo"));
    }

    #[test]
    fn csv_rejeita_sem_cabecalho() {
        let resultado = analisar_conteudo(b"PF,52998224725,Maria\n", FormatoImportacao::Csv);
        assert!(resultado.is_err());
    }

    #[test]
    fn csv_limpa_mascara_do_documento() {
        let conteudo = "tipo,cpf_cnpj,nome_razao_social\nPF,529.982.247-25,Maria\n";
        let analise = analisar_conteudo(conteudo.as_bytes(), FormatoImportacao::Csv)
            .expect("deve aceitar documento pontuado");
        assert_eq!(analise.validas[0].cpf_cnpj, "52998224725");
    }
}
