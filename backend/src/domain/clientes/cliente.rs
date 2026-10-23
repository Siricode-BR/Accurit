use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::cpf_cnpj::{CpfCnpj, TipoDocumento};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoCliente {
    Pf,
    Pj,
}

impl TipoCliente {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Pf => "PF",
            Self::Pj => "PJ",
        }
    }
}

impl fmt::Display for TipoCliente {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

impl FromStr for TipoCliente {
    type Err = ();

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "PF" => Ok(Self::Pf),
            "PJ" => Ok(Self::Pj),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PorteRegime {
    Mei,
    SimplesNacional,
    LucroPresumido,
    LucroReal,
}

impl PorteRegime {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Mei => "MEI",
            Self::SimplesNacional => "SIMPLES_NACIONAL",
            Self::LucroPresumido => "LUCRO_PRESUMIDO",
            Self::LucroReal => "LUCRO_REAL",
        }
    }
}

impl fmt::Display for PorteRegime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

impl FromStr for PorteRegime {
    type Err = ();

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "MEI" => Ok(Self::Mei),
            "SIMPLES_NACIONAL" => Ok(Self::SimplesNacional),
            "LUCRO_PRESUMIDO" => Ok(Self::LucroPresumido),
            "LUCRO_REAL" => Ok(Self::LucroReal),
            _ => Err(()),
        }
    }
}

const LIMITE_NOME: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ClienteInvalido {
    #[error("cliente pessoa física não deve ter porte ou regime tributário")]
    PfComPorte,
    #[error("documento deve ser CPF para pessoa física e CNPJ para pessoa jurídica")]
    TipoIncompativelComDocumento,
    #[error("nome deve ter entre 1 e 200 caracteres")]
    Nome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovoCliente {
    pub tipo: TipoCliente,
    pub porte_regime: Option<PorteRegime>,
    pub documento: CpfCnpj,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

impl NovoCliente {
    pub fn validar(&self) -> Result<(), ClienteInvalido> {
        match self.tipo {
            TipoCliente::Pf => {
                if self.porte_regime.is_some() {
                    return Err(ClienteInvalido::PfComPorte);
                }
            }
            TipoCliente::Pj => {
                if matches!(self.documento.tipo(), TipoDocumento::Cpf) {
                    return Err(ClienteInvalido::TipoIncompativelComDocumento);
                }
            }
        }

        if self.tipo == TipoCliente::Pf && !matches!(self.documento.tipo(), TipoDocumento::Cpf) {
            return Err(ClienteInvalido::TipoIncompativelComDocumento);
        }

        let nome = self.nome_razao_social.trim();
        if nome.is_empty() || nome.chars().count() > LIMITE_NOME {
            return Err(ClienteInvalido::Nome);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cliente {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub tipo: TipoCliente,
    pub porte_regime: Option<PorteRegime>,
    pub documento: CpfCnpj,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn documento_pf() -> CpfCnpj {
        CpfCnpj::novo("52998224725").expect("CPF válido")
    }

    fn documento_pj() -> CpfCnpj {
        CpfCnpj::novo("11444777000161").expect("CNPJ válido")
    }

    fn novo_cliente(
        tipo: TipoCliente,
        documento: CpfCnpj,
        porte: Option<PorteRegime>,
    ) -> NovoCliente {
        NovoCliente {
            tipo,
            porte_regime: porte,
            documento,
            nome_razao_social: "Alfa Serviços".to_string(),
            nome_fantasia: None,
            email: None,
            telefone: None,
        }
    }

    #[test]
    fn aceita_pf_sem_porte() {
        assert!(
            novo_cliente(TipoCliente::Pf, documento_pf(), None)
                .validar()
                .is_ok()
        );
    }

    #[test]
    fn rejeita_pf_com_porte() {
        assert_eq!(
            novo_cliente(TipoCliente::Pf, documento_pf(), Some(PorteRegime::Mei))
                .validar()
                .unwrap_err(),
            ClienteInvalido::PfComPorte
        );
    }

    #[test]
    fn aceita_pj_com_e_sem_porte() {
        assert!(
            novo_cliente(TipoCliente::Pj, documento_pj(), Some(PorteRegime::Mei))
                .validar()
                .is_ok()
        );
        assert!(
            novo_cliente(TipoCliente::Pj, documento_pj(), None)
                .validar()
                .is_ok()
        );
    }

    #[test]
    fn rejeita_cross_tipos_de_documento() {
        assert_eq!(
            novo_cliente(TipoCliente::Pf, documento_pj(), None)
                .validar()
                .unwrap_err(),
            ClienteInvalido::TipoIncompativelComDocumento
        );
        assert_eq!(
            novo_cliente(TipoCliente::Pj, documento_pf(), None)
                .validar()
                .unwrap_err(),
            ClienteInvalido::TipoIncompativelComDocumento
        );
    }

    #[test]
    fn rejeita_nome_vazio_ou_excessivo() {
        let mut cliente = novo_cliente(TipoCliente::Pf, documento_pf(), None);
        cliente.nome_razao_social = "   ".to_string();
        assert_eq!(cliente.validar().unwrap_err(), ClienteInvalido::Nome);

        cliente.nome_razao_social = "a".repeat(201);
        assert_eq!(cliente.validar().unwrap_err(), ClienteInvalido::Nome);
    }
}
