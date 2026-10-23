use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::fingerprint::FingerprintSha256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoCertificado {
    A1,
    A3,
}

impl TipoCertificado {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::A3 => "A3",
        }
    }
}

impl fmt::Display for TipoCertificado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

impl FromStr for TipoCertificado {
    type Err = ();

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor.to_ascii_uppercase().as_str() {
            "A1" => Ok(Self::A1),
            "A3" => Ok(Self::A3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlgoritmoCifragem {
    #[default]
    Aes256Gcm,
}

impl AlgoritmoCifragem {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Aes256Gcm => "AES-256-GCM",
        }
    }
}

impl fmt::Display for AlgoritmoCifragem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

impl std::str::FromStr for AlgoritmoCifragem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "AES-256-GCM" => Ok(Self::Aes256Gcm),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CertificadoInvalido {
    #[error("titular deve ser CPF (11 dígitos) ou CNPJ (14 dígitos)")]
    TitularFormato,
    #[error("conteúdo do certificado não pode ser vazio")]
    ConteudoVazio,
    #[error("fingerprint deve ter 64 caracteres hexadecimais")]
    FingerprintFormato,
    #[error("validade final deve ser posterior à validade inicial")]
    ValidadeInvalida,
    #[error("certificado já expirado")]
    Expirado,
}

impl From<crate::domain::certificados::FingerprintInvalido> for CertificadoInvalido {
    fn from(_: crate::domain::certificados::FingerprintInvalido) -> Self {
        CertificadoInvalido::FingerprintFormato
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovoCertificado {
    pub tipo: TipoCertificado,
    pub titular_cpf_cnpj: String,
    pub conteudo: Vec<u8>,
    pub senha: Option<String>,
    pub algoritmo: AlgoritmoCifragem,
    pub fingerprint: FingerprintSha256,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
}

impl NovoCertificado {
    pub fn validar(&self) -> Result<(), CertificadoInvalido> {
        let digitos: String = self
            .titular_cpf_cnpj
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        if !(digitos.len() == 11 || digitos.len() == 14) {
            return Err(CertificadoInvalido::TitularFormato);
        }

        if self.conteudo.is_empty() {
            return Err(CertificadoInvalido::ConteudoVazio);
        }

        self.fingerprint.validar()?;

        if self.valido_ate <= self.valido_de {
            return Err(CertificadoInvalido::ValidadeInvalida);
        }

        if self.valido_ate <= chrono::Utc::now() {
            return Err(CertificadoInvalido::Expirado);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Certificado {
    pub id: Uuid,
    pub escritorio_id: crate::domain::tenancy::EscritorioId,
    pub tipo: TipoCertificado,
    pub titular_cpf_cnpj: String,
    pub conteudo_cifrado: Vec<u8>,
    pub senha_cifrada: Option<Vec<u8>>,
    pub algoritmo: AlgoritmoCifragem,
    pub fingerprint: FingerprintSha256,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
    pub ativo: bool,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

impl Certificado {
    pub fn esta_valido(&self) -> bool {
        let agora = chrono::Utc::now();
        self.ativo && agora >= self.valido_de && agora <= self.valido_ate
    }

    pub fn dias_para_expirar(&self) -> i64 {
        let agora = chrono::Utc::now();
        (self.valido_ate - agora).num_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn novo_certificado_base() -> NovoCertificado {
        NovoCertificado {
            tipo: TipoCertificado::A1,
            titular_cpf_cnpj: "11.222.333/0001-81".to_string(),
            conteudo: vec![1, 2, 3, 4],
            senha: Some("senha123".to_string()),
            algoritmo: AlgoritmoCifragem::Aes256Gcm,
            fingerprint: FingerprintSha256::novo("a".repeat(64)).expect("fingerprint válido"),
            valido_de: chrono::Utc::now() - chrono::Duration::days(365),
            valido_ate: chrono::Utc::now() + chrono::Duration::days(365),
        }
    }

    #[test]
    fn aceita_certificado_a1_valido() {
        assert!(novo_certificado_base().validar().is_ok());
    }

    #[test]
    fn aceita_certificado_a3_valido() {
        let mut c = novo_certificado_base();
        c.tipo = TipoCertificado::A3;
        assert!(c.validar().is_ok());
    }

    #[test]
    fn rejeita_titular_formato_invalido() {
        let mut c = novo_certificado_base();
        c.titular_cpf_cnpj = "123".to_string();
        assert_eq!(
            c.validar().unwrap_err(),
            CertificadoInvalido::TitularFormato
        );
    }

    #[test]
    fn rejeita_conteudo_vazio() {
        let mut c = novo_certificado_base();
        c.conteudo = vec![];
        assert_eq!(c.validar().unwrap_err(), CertificadoInvalido::ConteudoVazio);
    }

    #[test]
    fn rejeita_fingerprint_invalido() {
        let mut c = novo_certificado_base();
        // Create an invalid fingerprint directly by using an invalid string
        c.fingerprint = FingerprintSha256::novo_para_teste("abc".to_string()); // Invalid: only 3 chars, not hex
        assert_eq!(
            c.validar().unwrap_err(),
            CertificadoInvalido::FingerprintFormato
        );
    }

    #[test]
    fn rejeita_validade_invalida() {
        let mut c = novo_certificado_base();
        c.valido_ate = c.valido_de - chrono::Duration::days(1);
        assert_eq!(
            c.validar().unwrap_err(),
            CertificadoInvalido::ValidadeInvalida
        );
    }

    #[test]
    fn rejeita_certificado_expirado() {
        let mut c = novo_certificado_base();
        c.valido_ate = chrono::Utc::now() - chrono::Duration::days(1);
        assert_eq!(c.validar().unwrap_err(), CertificadoInvalido::Expirado);
    }
}
