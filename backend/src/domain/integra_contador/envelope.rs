use serde::{Deserialize, Serialize};

use crate::domain::clientes::CpfCnpj;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerboIntegraContador {
    Apoiar,
    Consultar,
    Declarar,
    Emitir,
    Monitorar,
}

impl VerboIntegraContador {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Apoiar => "Apoiar",
            Self::Consultar => "Consultar",
            Self::Declarar => "Declarar",
            Self::Emitir => "Emitir",
            Self::Monitorar => "Monitorar",
        }
    }

    pub const fn exige_confirmacao(self) -> bool {
        matches!(self, Self::Declarar | Self::Emitir)
    }

    pub const fn pode_repetir_leitura(self) -> bool {
        matches!(self, Self::Apoiar | Self::Consultar | Self::Monitorar)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identificacao {
    pub numero: String,
    #[serde(rename = "tipo")]
    pub tipo: u8,
}

impl Identificacao {
    pub fn nova(numero: &str) -> Result<Self, String> {
        let documento = CpfCnpj::novo(numero).map_err(|_| "CPF/CNPJ inválido".to_owned())?;
        let digitos = documento.como_str().to_owned();
        let tipo = if digitos.len() == 11 { 1 } else { 2 };
        Ok(Self {
            numero: digitos,
            tipo,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub dados: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    pub contratante: Identificacao,
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    pub pedido_dados: PedidoDados,
}

impl IntegraContadorRequest {
    pub fn validar_dados(&self) -> Result<serde_json::Value, String> {
        if self.pedido_dados.dados.len() > 262_144 {
            return Err("payload acima do limite permitido".to_owned());
        }

        serde_json::from_str(&self.pedido_dados.dados)
            .map_err(|_| "pedidoDados.dados deve conter uma string JSON válida".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorResponse {
    pub status: i32,
    #[serde(default)]
    pub mensagens: Vec<MensagemSerpro>,
    #[serde(default)]
    pub dados: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MensagemSerpro {
    #[serde(default)]
    pub codigo: String,
    #[serde(default)]
    pub texto: Option<String>,
    #[serde(default)]
    pub descricao: Option<String>,
    #[serde(default, rename = "tipo")]
    pub tipo: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_preserva_dados_como_string_json() {
        let requisicao = IntegraContadorRequest {
            contratante: Identificacao::nova("11222333000181").expect("CNPJ válido"),
            autor_pedido_dados: Identificacao::nova("11222333000181").expect("CNPJ válido"),
            contribuinte: Identificacao::nova("52998224725").expect("CPF válido"),
            pedido_dados: PedidoDados {
                id_sistema: "PROCURACOES".to_owned(),
                id_servico: "OBTERPROCURACAO41".to_owned(),
                versao_sistema: "1".to_owned(),
                dados: r#"{"outorgante":"52998224725"}"#.to_owned(),
            },
        };

        let serializado = serde_json::to_value(requisicao).expect("serializa envelope");
        assert!(serializado["pedidoDados"]["dados"].is_string());
        assert_eq!(
            serializado["pedidoDados"]["dados"],
            r#"{"outorgante":"52998224725"}"#
        );
    }

    #[test]
    fn rejeita_identificacao_invalida() {
        assert!(Identificacao::nova("00000000000").is_err());
    }
}
