use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::usuarios::UsuarioDetalhe;
use crate::domain::auth::Papel;

fn papel_de<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Papel, D::Error> {
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn papel_opcional<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Papel>, D::Error> {
    Option::<String>::deserialize(d)?
        .map(|s| s.parse().map_err(serde::de::Error::custom))
        .transpose()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioDto {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub escritorio_nome: String,
    pub escritorio_slug: String,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub papel: String,
    pub ativo: bool,
    pub ultimo_login_em: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
}

impl From<UsuarioDetalhe> for UsuarioDto {
    fn from(u: UsuarioDetalhe) -> Self {
        Self {
            id: u.id,
            escritorio_id: u.escritorio_id,
            escritorio_nome: u.escritorio_nome,
            escritorio_slug: u.escritorio_slug,
            nome: u.nome,
            email: u.email,
            cpf: u.cpf,
            papel: u.papel.como_str().to_owned(),
            ativo: u.ativo,
            ultimo_login_em: u.ultimo_login_em,
            criado_em: u.criado_em,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaUsuariosResponse {
    pub itens: Vec<UsuarioDto>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListarUsuariosQuery {
    pub escritorio_id: Option<Uuid>,
    pub busca: Option<String>,
    pub ativo: Option<bool>,
    pub pagina: Option<i64>,
    pub tamanho: Option<i64>,
}

// ===== Fase E — função e carteira do usuário =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcessoUsuarioDto {
    pub funcao: String,
    pub pode_administrar_usuarios: bool,
    pub clientes_permitidos: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinirAcessoRequest {
    pub funcao: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinirCarteiraRequest {
    pub clientes: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriarUsuarioRequest {
    pub escritorio_id: Option<Uuid>,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    #[serde(deserialize_with = "papel_de")]
    pub papel: Papel,
    pub senha: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtualizarUsuarioRequest {
    pub nome: Option<String>,
    #[serde(default, deserialize_with = "papel_opcional")]
    pub papel: Option<Papel>,
    pub ativo: Option<bool>,
}

#[derive(Deserialize)]
pub struct RedefinirSenhaRequest {
    pub senha: String,
}
