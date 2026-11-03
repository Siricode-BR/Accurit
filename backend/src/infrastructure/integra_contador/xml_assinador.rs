use zeroize::Zeroize;

use crate::domain::integra_contador::assinatura::{
    AssinadorXml, ErroAssinatura, TermoAutorizacao, XmlAssinado,
};

pub struct AssinadorXmlImpl;

impl AssinadorXmlImpl {
    pub const fn novo() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AssinadorXml for AssinadorXmlImpl {
    async fn assinar(
        &self,
        termo: &TermoAutorizacao,
        chave_privada_pkcs8_der: &mut [u8],
        certificado_der: &mut [u8],
    ) -> Result<XmlAssinado, ErroAssinatura> {
        let resultado = termo
            .para_xml()
            .and_then(|_| Err(ErroAssinatura::ValidacaoCadeiaIndisponivel));
        chave_privada_pkcs8_der.zeroize();
        certificado_der.zeroize();
        resultado
    }
}
