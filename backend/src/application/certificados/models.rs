use crate::domain::certificados::{
    AlgoritmoCifragem, Certificado, CertificadoInvalido, NovoCertificado,
};
use crate::domain::tenancy::EscritorioId;
use chrono::Utc;
use uuid::Uuid;

const LIMITE_CONTEUDO_BYTES: usize = 50_000;

#[derive(Clone, Default)]
pub struct PoliticaCertificados {
    pub max_titular_len: usize,
    pub max_conteudo_bytes: usize,
}

impl PoliticaCertificados {
    pub const fn nova() -> Self {
        Self {
            max_titular_len: 14,
            max_conteudo_bytes: 50_000,
        }
    }
}

pub fn validar_titular(cpf_cnpj: &str) -> Result<(), CertificadoInvalido> {
    let digitos: String = cpf_cnpj.chars().filter(|c| c.is_ascii_digit()).collect();

    if !(digitos.len() == 11 || digitos.len() == 14) {
        return Err(CertificadoInvalido::TitularFormato);
    }
    Ok(())
}

pub fn validar_conteudo(conteudo: &[u8]) -> Result<(), CertificadoInvalido> {
    if conteudo.is_empty() {
        return Err(CertificadoInvalido::ConteudoVazio);
    }
    if conteudo.len() > LIMITE_CONTEUDO_BYTES {
        return Err(CertificadoInvalido::ConteudoVazio);
    }
    Ok(())
}

pub fn montar_novo_certificado(
    comando: &crate::application::certificados::ports::CriarCertificadoCommand,
    conteudo_cifrado: Vec<u8>,
    _senha_cifrada: Option<Vec<u8>>,
    fingerprint: crate::domain::certificados::FingerprintSha256,
    algoritmo: AlgoritmoCifragem,
) -> Result<NovoCertificado, CertificadoInvalido> {
    let novo = NovoCertificado {
        tipo: comando.tipo,
        titular_cpf_cnpj: comando.titular_cpf_cnpj.clone(),
        conteudo: conteudo_cifrado,
        senha: comando.senha.clone(),
        algoritmo,
        fingerprint,
        valido_de: comando.valido_de,
        valido_ate: comando.valido_ate,
    };
    novo.validar()?;
    Ok(novo)
}

pub fn montar_certificado(
    id: Uuid,
    escritorio_id: EscritorioId,
    novo: &NovoCertificado,
    conteudo_cifrado: Vec<u8>,
    senha_cifrada: Option<Vec<u8>>,
) -> Certificado {
    Certificado {
        id,
        escritorio_id,
        tipo: novo.tipo,
        titular_cpf_cnpj: novo.titular_cpf_cnpj.clone(),
        conteudo_cifrado,
        senha_cifrada,
        algoritmo: novo.algoritmo,
        fingerprint: novo.fingerprint.clone(),
        valido_de: novo.valido_de,
        valido_ate: novo.valido_ate,
        ativo: true,
        criado_em: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validar_titular_cpf_valido() {
        assert!(validar_titular("529.982.247-25").is_ok());
        assert!(validar_titular("52998224725").is_ok());
    }

    #[test]
    fn validar_titular_cnpj_valido() {
        assert!(validar_titular("11.222.333/0001-81").is_ok());
        assert!(validar_titular("11222333000181").is_ok());
    }

    #[test]
    fn rejeita_titular_formato_invalido() {
        assert!(validar_titular("123").is_err());
        assert!(validar_titular("1234567890123").is_err());
        assert!(validar_titular("123456789012345").is_err());
    }

    #[test]
    fn validar_conteudo_nao_vazio() {
        assert!(validar_conteudo(&[1, 2, 3]).is_ok());
    }

    #[test]
    fn rejeita_conteudo_vazio() {
        assert!(validar_conteudo(&[]).is_err());
    }
}
