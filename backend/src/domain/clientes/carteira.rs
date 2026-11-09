//! Fase E — Gestão de carteira: domínio de sócios/contatos, filiais, tags,
//! responsável interno e funções de acesso do usuário.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::clientes::cpf_cnpj::CpfCnpj;

/// Função granular do usuário dentro do escritório (Fase E).
/// Sem registro em `funcoes_usuarios`, deriva do papel:
/// `CLIENTE` → Gestor; `OPERADOR` → AnalistaFiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncaoUsuario {
    Gestor,
    AnalistaFiscal,
    OperadorLeitura,
}

impl FuncaoUsuario {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Gestor => "GESTOR",
            Self::AnalistaFiscal => "ANALISTA_FISCAL",
            Self::OperadorLeitura => "OPERADOR_LEITURA",
        }
    }

    pub fn de_str(valor: &str) -> Option<Self> {
        match valor.trim().to_ascii_uppercase().as_str() {
            "GESTOR" => Some(Self::Gestor),
            "ANALISTA_FISCAL" => Some(Self::AnalistaFiscal),
            "OPERADOR_LEITURA" => Some(Self::OperadorLeitura),
            _ => None,
        }
    }

    /// Gestor administra usuários, funções e carteira do escritório.
    pub const fn administra_usuarios(self) -> bool {
        matches!(self, Self::Gestor)
    }

    /// Leitura é somente leitura; analista e gestor escrevem clientes.
    pub const fn pode_escrever_clientes(self) -> bool {
        !matches!(self, Self::OperadorLeitura)
    }
}

impl FromStr for FuncaoUsuario {
    type Err = ();

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::de_str(valor).ok_or(())
    }
}

/// Acesso de um usuário à carteira de clientes (Fase E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcessoCarteira {
    pub funcao: FuncaoUsuario,
    /// `None` = carteira completa do escritório; `Some(ids)` = apenas esses clientes.
    pub clientes_permitidos: Option<Vec<Uuid>>,
}

impl AcessoCarteira {
    pub fn pode_ver(&self, cliente_id: Uuid) -> bool {
        match &self.clientes_permitidos {
            None => true,
            Some(ids) => ids.contains(&cliente_id),
        }
    }

    pub fn pode_escrever(&self, cliente_id: Uuid) -> bool {
        self.funcao.pode_escrever_clientes() && self.pode_ver(cliente_id)
    }
}

/// Papel de um sócio ou contato do cliente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PapelSocioContato {
    Socio,
    Contato,
}

impl PapelSocioContato {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Socio => "SOCIO",
            Self::Contato => "CONTATO",
        }
    }

    pub fn de_str(valor: &str) -> Option<Self> {
        match valor.trim().to_ascii_uppercase().as_str() {
            "SOCIO" => Some(Self::Socio),
            "CONTATO" => Some(Self::Contato),
            _ => None,
        }
    }
}

/// Sócio ou contato de um cliente (múltiplos por cliente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocioContato {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub papel: PapelSocioContato,
    pub nome: String,
    pub documento: Option<CpfCnpj>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub representante_principal: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovoSocioContato {
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub papel: PapelSocioContato,
    pub nome: String,
    pub documento: Option<CpfCnpj>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub representante_principal: bool,
}

impl NovoSocioContato {
    pub fn validar(&self) -> Result<(), String> {
        let nome = self.nome.trim();
        if nome.is_empty() || nome.chars().count() > 200 {
            return Err("nome deve ter entre 1 e 200 caracteres".into());
        }
        if let Some(email) = self.email.as_deref().map(str::trim) && email.len() > 254 {
            return Err("e-mail deve ter no máximo 254 caracteres".into());
        }
        if let Some(telefone) = self.telefone.as_deref().map(str::trim) && telefone.len() > 30 {
            return Err("telefone deve ter no máximo 30 caracteres".into());
        }
        Ok(())
    }
}

/// Filial/unidade de um cliente, identificada pelo CNPJ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClienteFilial {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub cnpj: CpfCnpj,
    pub nome_unidade: Option<String>,
    pub endereco: Option<String>,
    pub telefone: Option<String>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovoClienteFilial {
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub cnpj: CpfCnpj,
    pub nome_unidade: Option<String>,
    pub endereco: Option<String>,
    pub telefone: Option<String>,
}

impl NovoClienteFilial {
    pub fn validar(&self) -> Result<(), String> {
        if let Some(n) = self.nome_unidade.as_deref().map(str::trim) && n.chars().count() > 200 {
            return Err("nome da unidade deve ter no máximo 200 caracteres".into());
        }
        if let Some(e) = self.endereco.as_deref().map(str::trim) && e.chars().count() > 300 {
            return Err("endereço deve ter no máximo 300 caracteres".into());
        }
        if let Some(t) = self.telefone.as_deref().map(str::trim) && t.len() > 30 {
            return Err("telefone deve ter no máximo 30 caracteres".into());
        }
        Ok(())
    }
}

/// Tag de clientes do escritório.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClienteTag {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub nome: String,
    pub cor: Option<String>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovaClienteTag {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub cor: Option<String>,
}

impl NovaClienteTag {
    pub fn validar(&self) -> Result<(), String> {
        let nome = self.nome.trim();
        if nome.is_empty() || nome.chars().count() > 64 {
            return Err("nome da tag deve ter entre 1 e 64 caracteres".into());
        }
        if let Some(cor) = self.cor.as_deref().map(str::trim) {
            if !cor.is_empty() && !cor.starts_with('#') {
                return Err("cor deve ser um código hex, ex.: #1ABC9C".into());
            }
            if cor.len() > 16 {
                return Err("cor deve ter no máximo 16 caracteres".into());
            }
        }
        Ok(())
    }
}

/// Linha de carteira do cliente: responsável interno e observações.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarteiraCliente {
    pub cliente_id: Uuid,
    pub escritorio_id: Uuid,
    pub responsavel_usuario_id: Option<Uuid>,
    pub observacoes: Option<String>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AtualizarCarteiraCliente {
    pub responsavel_usuario_id: Option<Uuid>,
    /// `None` = não alterar; `Some(None)` = limpar; `Some(Some(t))` = definir.
    pub limpar_responsavel: bool,
    pub observacoes: Option<String>,
    pub limpar_observacoes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn funcao_usuario_estavel() {
        assert_eq!(FuncaoUsuario::Gestor.como_str(), "GESTOR");
        assert_eq!(FuncaoUsuario::de_str("gestor"), Some(FuncaoUsuario::Gestor));
        assert_eq!(
            FuncaoUsuario::de_str("operador_leitura"),
            Some(FuncaoUsuario::OperadorLeitura)
        );
        assert_eq!(FuncaoUsuario::de_str("x"), None);
        assert!(FuncaoUsuario::Gestor.administra_usuarios());
        assert!(!FuncaoUsuario::OperadorLeitura.pode_escrever_clientes());
        assert!(FuncaoUsuario::AnalistaFiscal.pode_escrever_clientes());
    }

    #[test]
    fn acesso_carteira_restringe() {
        let id_a = Uuid::now_v7();
        let id_b = Uuid::now_v7();
        let acesso = AcessoCarteira {
            funcao: FuncaoUsuario::AnalistaFiscal,
            clientes_permitidos: Some(vec![id_a]),
        };
        assert!(acesso.pode_ver(id_a));
        assert!(!acesso.pode_ver(id_b));
        assert!(!acesso.pode_escrever(id_b));
        assert!(acesso.pode_escrever(id_a));

        let total = AcessoCarteira {
            funcao: FuncaoUsuario::Gestor,
            clientes_permitidos: None,
        };
        assert!(total.pode_ver(id_b));
        assert!(total.funcao.administra_usuarios());
    }

    #[test]
    fn socio_contato_valida_campos() {
        let valido = NovoSocioContato {
            escritorio_id: Uuid::now_v7(),
            cliente_id: Uuid::now_v7(),
            papel: PapelSocioContato::Contato,
            nome: "Ana".into(),
            documento: None,
            email: Some("ana@example.com".into()),
            telefone: None,
            representante_principal: false,
        };
        assert!(valido.validar().is_ok());

        let sem_nome = NovoSocioContato {
            nome: "   ".into(),
            ..valido
        };
        assert!(sem_nome.validar().is_err());
    }
}
