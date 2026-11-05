use std::sync::Arc;
use zeroize::Zeroize;

use crate::domain::integra_contador::assinatura::{
    AssinadorXml, ErroAssinatura, TermoAutorizacao, XmlAssinado,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use p12_keystore::{KeyStore, Pkcs12ImportPolicy};
use pkcs8::PrivateKeyInfo;
use rsa::{pkcs1v15::SigningKey, Pkcs1v15Sign, RsaPrivateKey};
use sha2::{Digest, Sha256};
use x509_parser::prelude::X509Certificate;
use xmlsec::{sign, SignatureAlgorithm, Transform};

/// Implementação do assinador XML usando p12-keystore e xmlsec.
/// Realiza a assinatura digital do Termo de Autorização conforme padrão XMLDSig/W3C
/// adotado pela SERPRO (mesmo padrão NFe/eSocial).
pub struct AssinadorXmlImpl {
    /// Política de importação do PKCS#12 (pode ser configurada para validar cadeia ICP-Brasil)
    import_policy: Pkcs12ImportPolicy,
}

impl AssinadorXmlImpl {
    pub const fn novo() -> Self {
        Self {
            // Política padrão: não valida cadeia ICP-Brasil aqui (validada separadamente)
            // Em produção, considerar Pkcs12ImportPolicy::Default para validar
            import_policy: Pkcs12ImportPolicy::AcceptAll,
        }
    }

    /// Cria uma instância com política de importação customizada.
    pub const fn com_politica(import_policy: Pkcs12ImportPolicy) -> Self {
        Self { import_policy }
    }

    /// Extrai a chave privada e certificado do arquivo PKCS#12 (A1).
    /// O `senha` é a senha do arquivo .pfx/.p12.
    fn extrair_chave_e_certificado(
        &self,
        pfx_der: &[u8],
        senha: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), ErroAssinatura> {
        let keystore = KeyStore::from_pkcs12(pfx_der, senha, self.import_policy)
            .map_err(|_| ErroAssinatura::CertificadoInvalido)?;

        // Busca a primeira entrada de chave privada (PrivateKeyChain)
        let mut chave_privada_der: Option<Vec<u8>> = None;
        let mut certificado_der: Option<Vec<u8>> = None;

        for alias in keystore.aliases() {
            if let Ok(entry) = keystore.get_key_entry(&alias, senha) {
                // entry é um PrivateKeyChain
                if let Some(cert_chain) = entry.cert_chain() {
                    // Pega o primeiro certificado da cadeia (o certificado do titular)
                    if let Some(cert) = cert_chain.first() {
                        certificado_der = Some(cert.to_der().map_err(|_| ErroAssinatura::CertificadoInvalido)?);
                    }
                }
                // Extrai a chave privada em formato PKCS#8 DER
                let pkcs8_der = entry
                    .private_key()
                    .to_pkcs8_der()
                    .map_err(|_| ErroAssinatura::CertificadoInvalido)?;
                chave_privada_der = Some(pkcs8_der.as_bytes().to_vec());
                break; // Usa a primeira chave encontrada
            }
        }

        let chave_privada = chave_privada_der.ok_or(ErroAssinatura::CertificadoInvalido)?;
        let certificado = certificado_der.ok_or(ErroAssinatura::CertificadoInvalido)?;

        Ok((chave_privada, certificado))
    }

    /// Converte chave privada PKCS#8 DER para chave RSA assinável.
    fn pkcs8_para_chave_rsa(&self, pkcs8_der: &[u8]) -> Result<SigningKey<RsaPrivateKey>, ErroAssinatura> {
        let pkcs8 = PrivateKeyInfo::try_from(pkcs8_der).map_err(|_| ErroAssinatura::CertificadoInvalido)?;
        let rsa_private = RsaPrivateKey::try_from(pkcs8).map_err(|_| ErroAssinatura::CertificadoInvalido)?;
        Ok(SigningKey::<RsaPrivateKey>::new(rsa_private))
    }

    /// Assina o XML usando xmlsec com algoritmo RSA-SHA256 (conforme padrão NFe/eSocial).
    fn assinar_xml(&self, xml: &str, chave_rsa: &SigningKey<RsaPrivateKey>) -> Result<String, ErroAssinatura> {
        // Canonicaliza o XML (C14N) e assina com RSA-SHA256
        let signed = sign(
            xml.as_bytes(),
            chave_rsa,
            SignatureAlgorithm::RSA_SHA256,
            &[Transform::Enveloped],
            Some("ReferenceURI"), // ID da referência
        )
        .map_err(|_| ErroAssinatura::CertificadoInvalido)?;

        Ok(String::from_utf8(signed).map_err(|_| ErroAssinatura::CertificadoInvalido)?)
    }

    /// Extrai o certificado X.509 em base64 (para inclusão no XML assinado).
    fn certificado_para_base64(&self, certificado_der: &[u8]) -> Result<String, ErroAssinatura> {
        let (_, cert) = X509Certificate::from_der(certificado_der)
            .map_err(|_| ErroAssinatura::CertificadoInvalido)?;
        Ok(BASE64_STANDARD.encode(cert.tbs_certificate.raw))
    }

    /// Calcula o digest SHA256 do XML canônico (para o campo DigestValue).
    fn calcular_digest(&self, xml: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(xml.as_bytes());
        BASE64_STANDARD.encode(hasher.finalize())
    }
}

#[async_trait::async_trait]
impl AssinadorXml for AssinadorXmlImpl {
    async fn assinar(
        &self,
        termo: &TermoAutorizacao,
        pfx_der: &mut [u8],
        senha: &mut [u8],
    ) -> Result<XmlAssinado, ErroAssinatura> {
        // Converte senha de bytes para string
        let senha_str = std::str::from_utf8(senha).map_err(|_| ErroAssinatura::CertificadoInvalido)?;

        // 1. Extrai chave privada e certificado do PFX
        let (chave_privada_pkcs8, certificado_der) =
            self.extrair_chave_e_certificado(pfx_der, senha_str)?;

        // 2. Converte para chave RSA assinável
        let chave_rsa = self.pkcs8_para_chave_rsa(&chave_privada_pkcs8)?;

        // 3. Gera o XML do termo
        let xml_termo = termo.para_xml()?;

        // 4. Assina o XML
        let xml_assinado = self.assinar_xml(&xml_termo, &chave_rsa)?;

        // 5. Codifica o XML assinado em base64 (conforme exigido pela SERPRO)
        let xml_base64 = BASE64_STANDARD.encode(&xml_assinado);

        // 6. Calcula o digest do XML original (antes da assinatura)
        let digest_value = self.calcular_digest(&xml_termo);

        // 7. Extrai o valor da assinatura (SignatureValue) do XML assinado
        // O xmlsec já inclui a assinatura no XML; extraímos o SignatureValue
        let signature_value = extrair_signature_value(&xml_assinado)
            .ok_or(ErroAssinatura::CertificadoInvalido)?;

        // 8. Certificado em base64 para o X509Certificate
        let x509_certificate = self.certificado_para_base64(&certificado_der)?;

        // Limpa dados sensíveis da memória
        chave_privada_pkcs8.zeroize();
        certificado_der.zeroize();
        pfx_der.zeroize();
        senha.zeroize();

        Ok(XmlAssinado {
            xml_base64,
            digest_value,
            signature_value,
            x509_certificate,
        })
    }
}

/// Extrai o valor do SignatureValue do XML assinado.
fn extrair_signature_value(xml_assinado: &str) -> Option<String> {
    // Busca pelo padrão <SignatureValue>...</SignatureValue>
    // O xmlsec gera o XML com namespace ds: http://www.w3.org/2000/09/xmldsig#
    let inicio = xml_assinado.find("<SignatureValue>")?;
    let fim = xml_assinado[inicio..].find("</SignatureValue>")?;
    let valor = &xml_assinado[inicio + "<SignatureValue>".len()..inicio + fim];
    Some(valor.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::integra_contador::assinatura::AtoresTermoAutorizacao;
    use chrono::NaiveDate;

    fn termo_teste() -> TermoAutorizacao {
        TermoAutorizacao {
            data_assinatura: NaiveDate::from_ymd_opt(2026, 10, 20).expect("data válida"),
            vigencia: NaiveDate::from_ymd_opt(2027, 10, 20).expect("data válida"),
            atores: AtoresTermoAutorizacao {
                contratante_numero: "11222333000181".to_owned(),
                contratante_nome: "Accurit Tecnologia Ltda".to_owned(),
                autor_pedido_dados_numero: "11222333000181".to_owned(),
                autor_pedido_dados_nome: "Escritório Modelo".to_owned(),
                contribuinte_numero: "52998224725".to_owned(),
                contribuinte_nome: "João da Silva".to_owned(),
            },
        }
    }

    #[test]
    fn termo_para_xml_contem_tres_atores() {
        let xml = termo_teste().para_xml().expect("XML válido");
        assert!(xml.contains("<contratante"));
        assert!(xml.contains("<autorPedidoDados"));
        assert!(xml.contains("<contribuinte"));
    }
}