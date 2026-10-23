use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use zeroize::Zeroizing;

use crate::application::serpro::ports::{CriptografiaError, CriptografiaSerpro};

pub struct SerproCriptografia {
    cipher: Aes256Gcm,
}

impl SerproCriptografia {
    pub fn novo(chave_mestra: &[u8]) -> Result<Self, CriptografiaError> {
        if chave_mestra.len() != 32 {
            return Err(CriptografiaError::Cifrar(
                "chave mestra deve ter 32 bytes".into(),
            ));
        }
        let cipher = Aes256Gcm::new_from_slice(chave_mestra)
            .map_err(|e| CriptografiaError::Cifrar(e.to_string()))?;
        Ok(Self { cipher })
    }
}

impl CriptografiaSerpro for SerproCriptografia {
    fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CriptografiaError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, dados)
            .map_err(|e| CriptografiaError::Cifrar(e.to_string()))?;

        let mut resultado = Vec::with_capacity(nonce.len() + ciphertext.len());
        resultado.extend_from_slice(&nonce);
        resultado.extend_from_slice(&ciphertext);
        Ok(resultado)
    }

    fn decifrar(&self, dados_cifrados: &[u8]) -> Result<Zeroizing<Vec<u8>>, CriptografiaError> {
        if dados_cifrados.len() < 12 {
            return Err(CriptografiaError::Decifrar(
                "dados cifrados muito curtos".into(),
            ));
        }

        let (nonce_bytes, ciphertext) = dados_cifrados.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CriptografiaError::Decifrar(e.to_string()))?;

        // Zeroizing: o plaintext é limpo da memória ao sair do escopo.
        Ok(Zeroizing::new(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chave_teste() -> Vec<u8> {
        vec![7u8; 32]
    }

    #[test]
    fn cifra_e_decifra() {
        let cripto = SerproCriptografia::novo(&chave_teste()).unwrap();
        let original = b"consumer_secret_teste_123";

        let cifrado = cripto.cifrar(original).unwrap();
        assert_ne!(cifrado, original);

        let decifrado = cripto.decifrar(&cifrado).unwrap();
        assert_eq!(decifrado.as_slice(), original);
    }

    #[test]
    fn cifra_produz_resultado_diferente_cada_vez() {
        let cripto = SerproCriptografia::novo(&chave_teste()).unwrap();
        let original = b"mesmo_segredo";

        let cifrado1 = cripto.cifrar(original).unwrap();
        let cifrado2 = cripto.cifrar(original).unwrap();

        assert_ne!(cifrado1, cifrado2);

        assert_eq!(cripto.decifrar(&cifrado1).unwrap().as_slice(), original);
        assert_eq!(cripto.decifrar(&cifrado2).unwrap().as_slice(), original);
    }

    #[test]
    fn decifrar_falha_com_dados_invalidos() {
        let cripto = SerproCriptografia::novo(&chave_teste()).unwrap();
        let invalido = b"dados_invalidos_curto";

        assert!(matches!(
            cripto.decifrar(invalido),
            Err(CriptografiaError::Decifrar(_))
        ));
    }

    #[test]
    fn decifrar_com_tamper_falha() {
        let cripto = SerproCriptografia::novo(&chave_teste()).unwrap();
        let original = b"payload_autentico";

        let mut cifrado = cripto.cifrar(original).unwrap();
        // corrompe o último byte do ciphertext (tag inválida)
        let ultima = cifrado.len() - 1;
        cifrado[ultima] ^= 0xFF;

        assert!(matches!(
            cripto.decifrar(&cifrado),
            Err(CriptografiaError::Decifrar(_))
        ));
    }

    #[test]
    fn chave_tamanho_errado_falha() {
        let chave_curta = vec![7u8; 16];
        assert!(matches!(
            SerproCriptografia::novo(&chave_curta),
            Err(CriptografiaError::Cifrar(_))
        ));
    }
}
