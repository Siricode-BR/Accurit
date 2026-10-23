use std::sync::Arc;

use crate::application::certificados::{CertificadoCriptografia, CertificadoError};
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use ring::digest;
use zeroize::Zeroize;

#[derive(Zeroize, Clone)]
#[zeroize(drop)]
struct ChaveMestra([u8; 32]);

impl ChaveMestra {
    fn nova(bytes: &[u8]) -> Result<Self, CertificadoError> {
        if bytes.len() != 32 {
            return Err(CertificadoError::Criptografia);
        }
        let mut chave = [0u8; 32];
        chave.copy_from_slice(bytes);
        Ok(Self(chave))
    }

    fn como_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub struct CriptografiaAesGcm {
    chave: ChaveMestra,
}

impl CriptografiaAesGcm {
    pub fn nova(chave_mestra: &[u8]) -> Result<Self, CertificadoError> {
        Ok(Self {
            chave: ChaveMestra::nova(chave_mestra)?,
        })
    }
}

impl CertificadoCriptografia for CriptografiaAesGcm {
    fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CertificadoError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.chave.como_bytes()));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut buffer = dados.to_vec();
        let ciphertext = cipher
            .encrypt(&nonce, buffer.as_slice())
            .map_err(|_| CertificadoError::Criptografia)?;
        buffer.zeroize();

        let mut resultado = nonce.to_vec();
        resultado.extend_from_slice(&ciphertext);
        Ok(resultado)
    }

    fn decifrar(&self, dados_cifrados: &[u8]) -> Result<Vec<u8>, CertificadoError> {
        if dados_cifrados.len() < 12 {
            return Err(CertificadoError::Criptografia);
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.chave.como_bytes()));
        let nonce = Nonce::from_slice(&dados_cifrados[..12]);
        let ciphertext = &dados_cifrados[12..];

        let mut plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CertificadoError::Criptografia)?;

        let resultado = plaintext.clone();
        plaintext.zeroize();
        Ok(resultado)
    }

    fn gerar_fingerprint(&self, conteudo: &[u8]) -> String {
        let digest = digest::digest(&digest::SHA256, conteudo);
        hex::encode(digest.as_ref())
    }
}

pub fn criar_criptografia(
    chave_mestra: &[u8],
) -> Result<Arc<dyn CertificadoCriptografia>, CertificadoError> {
    Ok(Arc::new(CriptografiaAesGcm::nova(chave_mestra)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::certificados::CertificadoCriptografia;

    fn chave_teste() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn cifra_e_decifra_dados() {
        let crypto = CriptografiaAesGcm::nova(&chave_teste()).expect("criptografia válida");
        let dados = b"conteudo do certificado A1";
        let cifrado = crypto.cifrar(dados).expect("cifrar");
        let decifrado = crypto.decifrar(&cifrado).expect("decifrar");
        assert_eq!(decifrado, dados);
    }

    #[test]
    fn cifra_dados_diferentes_produz_cifras_diferentes() {
        let crypto = CriptografiaAesGcm::nova(&chave_teste()).expect("criptografia válida");
        let dados = b"mesmo conteudo";
        let c1 = crypto.cifrar(dados).expect("cifrar 1");
        let c2 = crypto.cifrar(dados).expect("cifrar 2");
        assert_ne!(c1, c2);
    }

    #[test]
    fn fingerprint_e_deterministico() {
        let crypto = CriptografiaAesGcm::nova(&chave_teste()).expect("criptografia válida");
        let dados = b"conteudo para fingerprint";
        let fp1 = crypto.gerar_fingerprint(dados);
        let fp2 = crypto.gerar_fingerprint(dados);
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64);
    }

    #[test]
    fn rejeita_chave_tamanho_errado() {
        let chave_curta = [1u8; 16];
        assert!(CriptografiaAesGcm::nova(&chave_curta).is_err());
    }

    #[test]
    fn rejeita_chave_base64_invalido() {
        assert!(CriptografiaAesGcm::nova(b"!!!").is_err());
    }
}
