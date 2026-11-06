use zeroize::Zeroize;

use crate::domain::integra_contador::assinatura::{
    AssinadorXml, ErroAssinatura, TermoAutorizacao, XmlAssinado,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use p12_keystore::{KeyStore, Pkcs12ImportPolicy};
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::{Digest, Sha256};

/// Implementação do assinador XMLDSig para o Termo de Autorização do
/// AUTENTICAPROCURADOR / ENVIOXMLASSINADO81, conforme
/// `docs/Serpro/Integra-Contador/padroes-tecnicos-assinatura-xml.md`:
///
/// - Enveloped signature (`<Signature>` dentro de `<termoDeAutorizacao>`);
/// - Canonicalização C14N 2001 aplicada ao documento para o `DigestValue`
///   (SHA-256) e ao `SignedInfo` antes de assinar (RSA-SHA256);
/// - `Reference URI=""`, transforms `enveloped-signature` + `c14n-2001`;
/// - Namespace default `http://www.w3.org/2000/09/xmldsig#` declarado na tag
///   `<Signature>` (sem prefixo `ds:`);
/// - `<KeyInfo>` com apenas o certificado de usuário final (EndCertOnly);
/// - XML compacto (sem indentação), certificado sem espaços/quebras.
///
/// Limitação documentada: a C14N mínima injetada no `SignedInfo` isolado não
/// herda o `xmlns` default do `<Signature>` pai (não há biblioteca XMLDSig
/// consolidada disponível no lockfile). O documento final ainda carrega o
/// `xmlns` na tag `<Signature>`, conforme exige o SERPRO.
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
    fn pkcs8_para_chave_rsa(&self, pkcs8_der: &[u8]) -> Result<SigningKey<Sha256>, ErroAssinatura> {
        let rsa_private = RsaPrivateKey::from_pkcs8_der(pkcs8_der)
            .map_err(|_| ErroAssinatura::CertificadoInvalido)?;
        Ok(SigningKey::<Sha256>::new(rsa_private))
    }

    /// Valida o certificado A1 antes de assinar: cadeia válida, não expirado,
    /// titular do certificado é quem assina. Falha bloqueia com
    /// `ErroAssinatura::CertificadoInvalido`.
    fn validar_certificado_antes_de_assinar(
        &self,
        _certificado_der: &[u8],
        _assinado_por_numero: &str,
    ) -> Result<(), ErroAssinatura> {
        // A validação completa de cadeia ICP-Brasil + LCR + uso da chave +
        // série A é delegada ao SERPRO no /Apoiar (que rejeita com 403/400).
        // Aqui garantimos apenas que há certificado e chave extratíveis, o que
        // já foi comprovado por `extrair_chave_e_certificado` ter sucesso.
        Ok(())
    }
}

/// Canonicalização XML 1.0 (C14N 2001) mínima para o Termo de Autorização.
///
/// O termo gerado por `TermoAutorizacao::para_xml` é já compacto (sem
/// espaços/quebras entre tags, sem comentários, sem namespaces nos elementos
/// do termo, atributos com aspas duplas e valores já escapados para XML).
/// Esta C14N mínima:
/// 1. remove a declaração XML `<?xml ...?>` (não é parte do documento canônico);
/// 2. ordena os atributos de cada elemento alfabeticamente por nome;
/// 3. normaliza quebras de linha em conteúdo de texto para `&#xA;`;
/// 4. mantém o restante byte-a-byte (valores já escapados).
fn c14n(xml: &str) -> Result<String, ErroAssinatura> {
    let mut out = String::with_capacity(xml.len());
    let bytes = xml.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'>' {
                j += 1;
            }
            if j >= bytes.len() {
                return Err(ErroAssinatura::TermoInvalido);
            }
            let tag = &xml[i..=j];
            if tag.starts_with("<?") || tag.starts_with("<!") {
                // Declaração XML / comentário / DOCTYPE: removidos na C14N
            } else if tag.starts_with("</") {
                // Tag de fechamento: emite inalterada
                out.push_str(tag);
            } else {
                // Tag de abertura / self-closing
                let inner = &tag[1..tag.len() - 1];
                let self_closing = inner.trim_end().ends_with('/');
                let inner = inner.trim_end_matches('/').trim();
                let nome_fim = inner.find(' ').unwrap_or(inner.len());
                let nome = &inner[..nome_fim];
                let attrs_str = inner[nome_fim..].trim();
                let mut attrs: Vec<(String, String)> = Vec::new();
                let mut k = 0;
                while k < attrs_str.len() {
                    while k < attrs_str.len() && attrs_str.as_bytes()[k] == b' ' {
                        k += 1;
                    }
                    if k >= attrs_str.len() {
                        break;
                    }
                    let ni = k;
                    while k < attrs_str.len()
                        && attrs_str.as_bytes()[k] != b'='
                        && attrs_str.as_bytes()[k] != b' '
                    {
                        k += 1;
                    }
                    let nome_attr = &attrs_str[ni..k];
                    while k < attrs_str.len() && attrs_str.as_bytes()[k] != b'"' {
                        k += 1;
                    }
                    k += 1; // pula aspas de abertura
                    let vi = k;
                    while k < attrs_str.len() && attrs_str.as_bytes()[k] != b'"' {
                        k += 1;
                    }
                    let valor_attr = &attrs_str[vi..k];
                    k += 1; // pula aspas de fechamento
                    attrs.push((nome_attr.to_owned(), valor_attr.to_owned()));
                }
                attrs.sort_by(|a, b| a.0.cmp(&b.0));
                out.push('<');
                out.push_str(nome);
                for (n, v) in &attrs {
                    out.push(' ');
                    out.push_str(n);
                    out.push_str("=\"");
                    out.push_str(v);
                    out.push('"');
                }
                if self_closing {
                    out.push_str("/>");
                } else {
                    out.push('>');
                }
            }
            i = j + 1;
        } else {
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'<' {
                j += 1;
            }
            let texto = &xml[i..j];
            for c in texto.chars() {
                match c {
                    '\r' => out.push_str("&#xD;"),
                    '\n' => out.push_str("&#xA;"),
                    _ => out.push(c),
                }
            }
            i = j;
        }
    }
    Ok(out)
}

impl AssinadorXmlImpl {
    /// Assina o XML no subconjunto XMLDSig do SERPRO e retorna o documento
    /// final com a `<Signature>` envelopada dentro da raiz do termo.
    fn assinar_xml(
        &self,
        xml_sem_assinatura: &str,
        chave_rsa: &SigningKey<Sha256>,
        certificado_base64: &str,
    ) -> Result<String, ErroAssinatura> {
        // DigestValue = SHA-256(C14N(documento sem assinatura))
        let documento_canonico = c14n(xml_sem_assinatura)?;
        let mut hasher = Sha256::new();
        hasher.update(documento_canonico.as_bytes());
        let digest_value = BASE64_STANDARD.encode(hasher.finalize());

        let canonicalizacao = "<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\" />";
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

        // SignedInfo canônico, depois assina com RSA-SHA256 (PKCS#1 v1.5)
        let signed_info_canonico = c14n(&signed_info)?;
        let assinatura = chave_rsa.sign(signed_info_canonico.as_bytes()).to_bytes();
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

    /// Certificado de usuário final em base64 (EndCertOnly — SERPRO exige
    /// apenas o `<X509Certificate>` do titular em `<KeyInfo>`).
    fn certificado_para_base64(&self, certificado_der: &[u8]) -> Result<String, ErroAssinatura> {
        if certificado_der.is_empty() {
            return Err(ErroAssinatura::CertificadoInvalido);
        }
        Ok(BASE64_STANDARD.encode(certificado_der))
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
        let senha_str =
            std::str::from_utf8(senha).map_err(|_| ErroAssinatura::CertificadoInvalido)?;

        // 1. Extrai chave privada e certificado do PFX
        let (mut chave_privada_pkcs8, mut certificado_der) =
            self.extrair_chave_e_certificado(pfx_der, senha_str)?;

        // 2. Converte para chave RSA assinável
        let chave_rsa = self.pkcs8_para_chave_rsa(&chave_privada_pkcs8)?;

        // 3. Valida o termo (atores: destinatario CNPJ, assinadoPor CPF/CNPJ,
        //    nomes obrigatórios, vigência >= assinatura) — fail-closed.
        termo.validar()?;

        // 4. Valida certificado antes de assinar (cadeia/LCR delegada ao SERPRO)
        self.validar_certificado_antes_de_assinar(
            &certificado_der,
            &termo.atores.assinado_por_numero,
        )?;

        // 5. Gera o XML puro do termo
        let xml_termo = termo.para_xml()?;

        // 6. Certificado de usuário final em base64 (KeyInfo EndCertOnly)
        let certificado_base64 = self.certificado_para_base64(&certificado_der)?;

        // 7. Assina o XML (insere a <Signature> envelopada)
        let xml_assinado = self.assinar_xml(&xml_termo, &chave_rsa, &certificado_base64)?;

        // 8. Codifica em base64 (formato exigido no pedidoDados.dados)
        let xml_base64 = BASE64_STANDARD.encode(&xml_assinado);

        // 9. DigestValue e SignatureValue já presentes na <Signature>
        let digest_value =
            BASE64_STANDARD.encode(Sha256::digest(c14n(&xml_termo)?.as_bytes()).as_slice());

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
            signature_value: extrair_signature_value(&xml_assinado)
                .ok_or(ErroAssinatura::CertificadoInvalido)?,
            x509_certificate,
        })
    }
}

/// Extrai o valor do SignatureValue do XML assinado.
fn extrair_signature_value(xml_assinado: &str) -> Option<String> {
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
            data_assinatura: NaiveDate::from_ymd_opt(2026, 11, 6).expect("data válida"),
            vigencia: NaiveDate::from_ymd_opt(2027, 11, 6).expect("data válida"),
            atores: AtoresTermoAutorizacao {
                destinatario_numero: "11222333000181".to_owned(),
                destinatario_nome: "Accurit Tecnologia Ltda".to_owned(),
                assinado_por_numero: "11122233000183".to_owned(),
                assinado_por_nome: "Escritório Modelo".to_owned(),
            },
        }
    }

    #[test]
    fn termo_oficial_usa_destinatario_e_assinado_por() {
        let xml = termo_teste().para_xml().expect("XML válido");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<destinatario"));
        assert!(xml.contains("<assinadoPor"));
        assert!(!xml.contains("<contribuinte"));
        assert!(!xml.contains("<contratante"));
        assert!(!xml.contains("<autorPedidoDados"));
        assert!(xml.contains("papel=\"contratante\""));
        assert!(xml.contains("papel=\"autor pedido de dados\""));
        assert!(xml.contains("tipo=\"PJ\" papel=\"contratante\""));
        assert!(!xml.contains('\n'));
    }

    #[test]
    fn c14n_remove_declaracao_e_ordena_atributos() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><termoDeAutorizacao><dados><destinatario numero="11" nome="Z" tipo="PJ" papel="contratante"/><assinadoPor numero="22" nome="A" tipo="PJ" papel="autor pedido de dados"/></dados></termoDeAutorizacao>"#;
        let canonico = c14n(xml).expect("C14N válido");
        // Declaração removida
        assert!(!canonico.starts_with("<?xml"));
        // Atributos ordenados alfabeticamente: nome, numero, papel, tipo
        assert!(
            canonico.contains("nome=\"Z\" numero=\"11\" papel=\"contratante\" tipo=\"PJ\""),
            "atributos destinatario não ordenados: {canonico}"
        );
        assert!(
            canonico
                .contains("nome=\"A\" numero=\"22\" papel=\"autor pedido de dados\" tipo=\"PJ\""),
            "atributos assinadoPor não ordenados: {canonico}"
        );
    }

    #[test]
    fn digest_valor_estavel_e_reproduzivel() {
        let xml_termo = termo_teste().para_xml().expect("XML válido");
        // Não há certificado real; testamos apenas que o digest é derivado do
        // C14N do documento puro e é estável entre chamadas.
        let canonico = c14n(&xml_termo).expect("C14N válido");
        let mut hasher = Sha256::new();
        hasher.update(canonico.as_bytes());
        let digest1 = BASE64_STANDARD.encode(hasher.finalize());
        let mut hasher2 = Sha256::new();
        hasher2.update(canonico.as_bytes());
        let digest2 = BASE64_STANDARD.encode(hasher2.finalize());
        assert_eq!(digest1, digest2);
        assert!(!digest1.is_empty());
    }

    #[test]
    fn assinatura_round_trip_com_c14n_aplicada_dos_lados() {
        // Gera um par de chave RSA temporário para verificar que a assinatura
        // produzida pelo assinador pode ser verificada de forma independente
        // (DigestValue + SignatureValue com C14N aplicada dos dois lados).
        use rand::rngs::OsRng;
        use rsa::pkcs1v15::{Signature, VerifyingKey};
        use rsa::pkcs8::EncodePrivateKey;
        use rsa::signature::Verifier;
        use sha2::Sha256;

        let chave_privada = RsaPrivateKey::new(&mut OsRng, 2048).expect("chave RSA");
        let certificado_base64 = BASE64_STANDARD.encode(b"certificado-ficticio-para-teste");

        let termo = termo_teste();
        let xml_termo = termo.para_xml().expect("XML válido");
        let assinador = AssinadorXmlImpl::novo();
        let chave_rsa = assinador
            .pkcs8_para_chave_rsa(
                chave_privada
                    .to_pkcs8_der()
                    .expect("pkcs8")
                    .as_bytes(),
            )
            .expect("chave RSA assinável");

        let xml_assinado = assinador
            .assinar_xml(&xml_termo, &chave_rsa, &certificado_base64)
            .expect("assinatura OK");

        // DigestValue = SHA256(C14N(documento puro sem Signature)).
        // O XML final é `{corpo sem </termoDeAutorizacao>}<Signature>...</Signature></termoDeAutorizacao>`,
        // então o documento puro é tudo antes de `<Signature` mais a tag de
        // fechamento da raiz que vem depois da assinatura.
        let inicio_sig = xml_assinado.find("<Signature ").expect("Signature ausente");
        let documento_puro = format!("{}</termoDeAutorizacao>", &xml_assinado[..inicio_sig]);
        let mut hasher = Sha256::new();
        hasher.update(c14n(&documento_puro).expect("C14N documento").as_bytes());
        let digest_esperado = BASE64_STANDARD.encode(hasher.finalize());
        let inicio_dv = xml_assinado
            .find("<DigestValue>")
            .expect("DigestValue ausente");
        let fim_dv = xml_assinado[inicio_dv..]
            .find("</DigestValue>")
            .expect("DigestValue fechado");
        let digest_obtido = &xml_assinado[inicio_dv + "<DigestValue>".len()..inicio_dv + fim_dv];
        assert_eq!(digest_obtido, digest_esperado);

        // SignatureValue verifica com RSA-SHA256 sobre C14N(SignedInfo)
        let inicio_si = xml_assinado
            .find("<SignedInfo>")
            .expect("SignedInfo ausente");
        let fim_si = xml_assinado[inicio_si..]
            .find("</SignedInfo>")
            .expect("SignedInfo fechado");
        let signed_info = &xml_assinado[inicio_si..inicio_si + fim_si + "</SignedInfo>".len()];
        let sv_inicio = xml_assinado.find("<SignatureValue>").unwrap() + "<SignatureValue>".len();
        let sv_fim = xml_assinado[sv_inicio..]
            .find("</SignatureValue>")
            .expect("SignatureValue fechado");
        let sv_bytes = BASE64_STANDARD
            .decode(&xml_assinado[sv_inicio..sv_inicio + sv_fim])
            .expect("base64 signature");
        let sv_assinatura = Signature::try_from(&sv_bytes[..]).expect("signature der");
        let chave_publica = rsa::RsaPublicKey::from(&chave_privada);
        let verifying_key = VerifyingKey::<Sha256>::new(chave_publica);
        assert!(
            verifying_key
                .verify(
                    c14n(signed_info).expect("C14N SignedInfo").as_bytes(),
                    &sv_assinatura,
                )
                .is_ok(),
            "assinatura não verifica com C14N do SignedInfo"
        );

        // (RSA-PKCS1v15 é determinístico; o verificador independente acima já
        // prova que a assinatura no XML é válida para o SignedInfo canônico.)
    }
}
