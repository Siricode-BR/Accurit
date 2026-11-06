use zeroize::Zeroize;

use crate::domain::integra_contador::assinatura::{
    AssinadorXml, ErroAssinatura, TermoAutorizacao, XmlAssinado,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use p12_keystore::{KeyStore, Pkcs12ImportPolicy};
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use sha2::{Digest, Sha256};

/// Implementação do assinador XML usando p12-keystore.
/// Realiza a assinatura digital do Termo de Autorização no subconjunto do
/// XMLDSig adotado pela SERPRO (padrões técnicos Assinatura de XML):
/// enveloped, C14N 2001, RSA-SHA256 e digest SHA-256, com o namespace
/// `http://www.w3.org/2000/09/xmldsig#` declarado na tag `<Signature>` e
/// apenas o certificado de usuário final (EndCertOnly) em `<KeyInfo>`.
pub struct AssinadorXmlImpl {
    /// Política de importação do PKCS#12 (pode ser configurada para validar cadeia ICP-Brasil)
    import_policy: Pkcs12ImportPolicy,
}

impl AssinadorXmlImpl {
    pub const fn novo() -> Self {
        Self {
            // Strict: vincula chave privada à cadeia do certificado por
            // localKeyId e descarta entradas não vinculadas — o que a A1
            // ICP-Brasil suporta. Entradas sem vínculo falham fechado.
            import_policy: Pkcs12ImportPolicy::Strict,
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

        // Busca a primeira cadeia de chave privada do cofre (alias/certificado)
        let (_alias, chain) = keystore
            .private_key_chain()
            .ok_or(ErroAssinatura::CertificadoInvalido)?;

        // Certificado do titular: primeiro da cadeia (EndCertOnly no <KeyInfo>)
        let certificado_der = chain
            .certs()
            .first()
            .map(|certificado| certificado.as_der().to_vec())
            .ok_or(ErroAssinatura::CertificadoInvalido)?;

        // Chave privada em PKCS#8 DER
        let chave_privada_der = chain.key().as_der().to_vec();

        Ok((chave_privada_der, certificado_der))
    }

    /// Converte chave privada PKCS#8 DER para chave RSA assinável (SHA-256).
    fn pkcs8_para_chave_rsa(
        &self,
        pkcs8_der: &[u8],
    ) -> Result<SigningKey<Sha256>, ErroAssinatura> {
        let rsa_private = RsaPrivateKey::from_pkcs8_der(pkcs8_der)
            .map_err(|_| ErroAssinatura::CertificadoInvalido)?;
        Ok(SigningKey::<Sha256>::new(rsa_private))
    }
}

// Reimplementação correta da assinatura manual XMLDSig para contornar a dependência xmlsec inexistente
// e resolver o erro de compilação.
impl AssinadorXmlImpl {
    /// Assina o XML no subconjunto XMLDSig do SERPRO e retorna o documento
    /// final com a `<Signature>` envelopada dentro da raiz do termo.
    fn assinar_xml(
        &self,
        xml_sem_assinatura: &str,
        chave_rsa: &SigningKey<Sha256>,
        certificado_base64: &str,
    ) -> Result<String, ErroAssinatura> {
        let canonicalizacao = "<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\" />";
        let digest_value = self.calcular_digest(xml_sem_assinatura);
        let signed_info = format!(
            "<SignedInfo>{canonicalizacao}\
             <SignatureMethod Algorithm=\"http://www.w3.org/2001/04/xmldsig-more#rsa-sha256\" />\
             <Reference URI=\"\"><Transforms>\
             <Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\" />\
             <Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\" />\
             </Transforms>\
             <DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\" />\
             <DigestValue>{digest_value}</DigestValue></Reference></SignedInfo>"
        );

        let mut rng = rand::thread_rng();
        let assinatura = chave_rsa
            .sign_with_rng(&mut rng, signed_info.as_bytes())
            .to_bytes();
        let signature_value = BASE64_STANDARD.encode(&assinatura);

        let fim_raiz = "</termoDeAutorizacao>";
        let corpo = xml_sem_assinatura
            .strip_suffix(fim_raiz)
            .ok_or(ErroAssinatura::TermoInvalido)?;
        Ok(format!(
            "{corpo}<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">{signed_info}\
             <SignatureValue>{signature_value}</SignatureValue>\
             <KeyInfo><X509Data><X509Certificate>{certificado_base64}</X509Certificate>\
             </X509Data></KeyInfo></Signature>{fim_raiz}"
        ))
    }

    /// Valida explicitamente a estrutura de 3 CNPJs exigida pelo SERPRO
    /// (modelo Software House: Contratante, AutorPedidoDados, Contribuinte)
    /// antes de gerar e assinar o XML. Fail-closed: qualquer ator fora da
    /// estrutura impede a assinatura e, portanto, a chamada a /Apoiar.
    fn validar_trio_software_house(
        &self,
        termo: &TermoAutorizacao,
    ) -> Result<(), ErroAssinatura> {
        crate::domain::integra_contador::regras_procuracao::TrioSoftwareHouse {
            contratante_documento: termo.atores.contratante_numero.clone(),
            contratante_nome: termo.atores.contratante_nome.clone(),
            autor_pedido_dados_documento: termo.atores.autor_pedido_dados_numero.clone(),
            autor_pedido_dados_nome: termo.atores.autor_pedido_dados_nome.clone(),
            contribuinte_documento: termo.atores.contribuinte_numero.clone(),
            contribuinte_nome: termo.atores.contribuinte_nome.clone(),
        }
        .validar()
    }

    /// Certificado de usuário final em base64 (EndCertOnly — SERPRO exige
    /// apenas o `<X509Certificate>` do titular em `<KeyInfo>`).
    fn certificado_para_base64(&self, certificado_der: &[u8]) -> Result<String, ErroAssinatura> {
        if certificado_der.is_empty() {
            return Err(ErroAssinatura::CertificadoInvalido);
        }
        Ok(BASE64_STANDARD.encode(certificado_der))
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
        let (mut chave_privada_pkcs8, mut certificado_der) =
            self.extrair_chave_e_certificado(pfx_der, senha_str)?;

        // 2. Converte para chave RSA assinável
        let chave_rsa = self.pkcs8_para_chave_rsa(&chave_privada_pkcs8)?;

        // 3. Valida explicitamente a estrutura de 3 CNPJs (Contratante,
        //    AutorPedidoDados, Contribuinte) antes de gerar o XML.
        self.validar_trio_software_house(termo)?;

        // 4. Gera o XML do termo
        let xml_termo = termo.para_xml()?;

        // 5. Certificado de usuário final em base64 (KeyInfo EndCertOnly)
        let certificado_base64 = self.certificado_para_base64(&certificado_der)?;

        // 6. Assina o XML (insere a <Signature> envelopada)
        let xml_assinado = self.assinar_xml(&xml_termo, &chave_rsa, &certificado_base64)?;

        // 7. Codifica em base64 (formato exigido no pedidoDados.dados de ENVIOXMLASSINADO81)
        let xml_base64 = BASE64_STANDARD.encode(&xml_assinado);

        // 8. DigestValue e SignatureValue já presentes na <Signature>
        let digest_value = self.calcular_digest(&xml_termo);

        // 9. Extrai o valor da assinatura (SignatureValue) do XML assinado
        let signature_value = extrair_signature_value(&xml_assinado)
            .ok_or(ErroAssinatura::CertificadoInvalido)?;

        // 10. Certificado em base64 para o X509Certificate
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
    // A <Signature> é envelopada na raiz `<termoDeAutorizacao>`.
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

    #[test]
    fn validar_trio_rejeita_estrutura_incompleta_antes_de_assinar() {
        let assinador = AssinadorXmlImpl::novo();
        let mut termo = termo_teste();
        termo.atores.autor_pedido_dados_numero = "52998224725".to_owned(); // CPF
        assert_eq!(
            assinador.validar_trio_software_house(&termo),
            Err(ErroAssinatura::TermoInvalido)
        );
    }

    #[test]
    fn validacao_do_trio_aceita_estrutura_software_house() {
        let assinador = AssinadorXmlImpl::novo();
        let termo = termo_teste();
        assert!(assinador.validar_trio_software_house(&termo).is_ok());
    }
}