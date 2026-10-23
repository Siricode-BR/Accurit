use secrecy::SecretString;
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

#[derive(Debug, Clone)]
pub struct CredencialSerpro {
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub ambiente: AmbienteSerpro,
    pub consumer_key: SecretString,
    pub consumer_secret: SecretString,
    pub certificado: Option<CertificadoA1>,
}

/// Metadados do certificado A1 associado à credencial (o conteúdo decifrado
/// usado em mTLS é obtido pela infraestrutura na fase de integração SAPI).
#[derive(Debug, Clone)]
pub struct CertificadoA1 {
    pub id: Uuid,
    pub titular_cpf_cnpj: String,
    pub fingerprint_sha256: String,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn credencial_serpro_cria_com_dados() {
        let certificado = CertificadoA1 {
            id: Uuid::now_v7(),
            titular_cpf_cnpj: "12345678000190".to_owned(),
            fingerprint_sha256: "ab".repeat(32),
            valido_de: chrono::Utc::now(),
            valido_ate: chrono::Utc::now() + Duration::days(365),
        };
        let credencial = CredencialSerpro {
            escritorio_id: Uuid::now_v7(),
            produto: ProdutoSerpro::IntegraContador,
            ambiente: AmbienteSerpro::Trial,
            consumer_key: SecretString::from("key"),
            consumer_secret: SecretString::from("secret"),
            certificado: Some(certificado),
        };
        assert_eq!(credencial.produto, ProdutoSerpro::IntegraContador);
        assert_eq!(credencial.ambiente, AmbienteSerpro::Trial);
        assert!(credencial.certificado.is_some());
    }

    #[test]
    fn credencial_sem_certificado_e_trial_e_valida() {
        let credencial = CredencialSerpro {
            escritorio_id: Uuid::now_v7(),
            produto: ProdutoSerpro::DividaAtiva,
            ambiente: AmbienteSerpro::Trial,
            consumer_key: SecretString::from("key"),
            consumer_secret: SecretString::from("secret"),
            certificado: None,
        };
        assert!(credencial.certificado.is_none());
    }
}
