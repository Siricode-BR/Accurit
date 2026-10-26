//! VOs das chaves de consulta da Dívida Ativa.
//!
//! - [`NumeroInscricao`]: número de inscrição na Dívida Ativa, **somente
//!   dígitos** (ex.: `9061700096419`); aceita máscara na entrada (`-`, `/`,
//!   espaços), nunca é reformatado e respeita um comprimento plausível.
//! - [`hash_consulta`]: SHA-256 hex (64 chars) da chave normalizada —
//!   pseudonimização para `consultas_fiscais.chave_consulta_hash`
//!   (SPEC §9.11: não registrar a chave completa em claro).
//! - [`mascarar_chave`]: exibição redigida, para `chave_consulta_mascarada`
//!   e `request_redigido` (ADR-0004: minimização).
//!
//! A validação de CPF/CNPJ segue sendo de `domain::clientes::CpfCnpj`
//! (mesmas regras de dígitos verificadores).

use crate::domain::clientes::CpfCnpj;

/// Comprimento plausível de inscrição: as amostras usam 13 dígitos; o
/// máximo carinhoso evita lixo de path como chave.
const INSCRICAO_DIGITOS_MINIMOS: usize = 6;
const INSCRICAO_DIGITOS_MAXIMOS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ErroNumeroInscricao {
    #[error("inscrição vazia")]
    Vazio,
    #[error("inscrição com comprimento implausível para a Dívida Ativa")]
    Comprimento,
}

/// Número de inscrição apenas com dígitos (sem regra de DV local: isso é
/// responsabilidade do SERPRO — não inventamos regra).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumeroInscricao {
    digitos: String,
}

impl NumeroInscricao {
    pub fn novo(bruto: impl AsRef<str>) -> Result<Self, ErroNumeroInscricao> {
        let digitos: String = bruto
            .as_ref()
            .chars()
            .filter(|caractere| caractere.is_ascii_digit())
            .collect();

        if digitos.is_empty() {
            return Err(ErroNumeroInscricao::Vazio);
        }
        if !(INSCRICAO_DIGITOS_MINIMOS..=INSCRICAO_DIGITOS_MAXIMOS).contains(&digitos.len()) {
            return Err(ErroNumeroInscricao::Comprimento);
        }

        Ok(Self { digitos })
    }

    pub fn como_str(&self) -> &str {
        &self.digitos
    }
}

impl std::fmt::Display for NumeroInscricao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.digitos)
    }
}

/// Horário da consulta (UTC), formato ISO 8601 para `request_redigido`.
pub fn horario_consulta(agora: chrono::DateTime<chrono::Utc>) -> String {
    agora.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// SHA-256 hex (64 chars) de uma chave de consulta normalizada (CPF/CNPJ
/// em dígitos ou inscrição). Nunca registrar a chave completa em claro.
pub fn hash_consulta(chave_normalizada: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(chave_normalizada.as_bytes());
    let resumo = hasher.finalize();
    hex::encode(resumo)
}

/// Tipo de chave consultada (rotula a máscara e a `fonte_consulta`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TipoChave {
    /// CPF/CNPJ já validado (verificador incluído).
    CpfCnpj(CpfCnpj),
    /// Inscrição na Dívida Ativa.
    Inscricao(NumeroInscricao),
    /// Código de Contribuinte alternativo (fallback D2, opaco).
    CodigoContribuinte(String),
}

/// Máscara de exibição redigida, por tipo de chave:
/// - CPF (11): `***.***.987-65`
/// - CNPJ (14): `12.***.***-0001-7X` (mantém base+DV, sempre ≤ 20)
/// - Inscrição: mantém inicial/final de 3 dígitos
/// - Código de Contribuinte: margens curtas preservadas.
pub fn mascarar_chave(tipo: &TipoChave, chave_normalizada: &str) -> String {
    match tipo {
        TipoChave::CpfCnpj(_) => mascarar_documento(chave_normalizada),
        TipoChave::Inscricao(_) => mascarar_extremos(chave_normalizada, 3, 3),
        TipoChave::CodigoContribuinte(_) => mascarar_extremos(chave_normalizada, 2, 2),
    }
}

fn mascarar_documento(digitos: &str) -> String {
    match digitos.len() {
        11 => format!("***.***.{}-{}", &digitos[6..9], &digitos[9..11]),
        14 => format!(
            "{}.***.***-{}-{}",
            &digitos[0..2],
            &digitos[8..12],
            &digitos[12..14]
        ),
        _ => mascarar_extremos(digitos, 2, 2),
    }
}

/// Preserva `inicio` dígitos no começo e `fim` no fim; o meio vira `*`.
fn mascarar_extremos(digitos: &str, inicio: usize, fim: usize) -> String {
    let tamanho = digitos.len();
    if tamanho == 0 {
        return String::new();
    }
    if tamanho <= inicio + fim {
        return "*".repeat(tamanho);
    }

    format!(
        "{}{}{}",
        &digitos[..inicio],
        "*".repeat(tamanho - inicio - fim),
        &digitos[tamanho - fim..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inscricao_normaliza_so_digitos() {
        let inscricao = NumeroInscricao::novo("90 6 17 000964-19").expect("inscrição válida");
        assert_eq!(inscricao.como_str(), "9061700096419");
        assert_eq!(inscricao.to_string(), "9061700096419");
    }

    #[test]
    fn inscricao_preserva_zeros_a_esquerda() {
        let inscricao = NumeroInscricao::novo("00 0 00 0000641-9").expect("inscrição válida");
        assert_eq!(inscricao.como_str(), "0000000006419");
    }

    #[test]
    fn inscricao_rejeita_vazia_e_longa() {
        assert_eq!(
            NumeroInscricao::novo("###").unwrap_err(),
            ErroNumeroInscricao::Vazio
        );
        let longa = "9".repeat(21);
        assert_eq!(
            NumeroInscricao::novo(longa).unwrap_err(),
            ErroNumeroInscricao::Comprimento
        );
    }

    #[test]
    fn hash_consulta_tem_64_chars_e_determinismo() {
        let hash1 = hash_consulta("09781911768");
        let hash2 = hash_consulta("09781911768");
        let hash3 = hash_consulta("09781911769");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
        assert!(
            hash1
                .bytes()
                .all(|byte| byte == b'-' || byte.is_ascii_digit() || byte.is_ascii_hexdigit())
        );
    }

    #[test]
    fn mascara_de_cpf_e_cnpj() {
        assert_eq!(
            mascarar_chave(
                &TipoChave::CpfCnpj(CpfCnpj::novo("529.982.247-25").expect("CPF válido")),
                "52998224725"
            ),
            "***.***.247-25"
        );
        assert_eq!(
            mascarar_chave(
                &TipoChave::CpfCnpj(CpfCnpj::novo("11.222.333/0001-81").expect("CNPJ válido")),
                "11222333000181"
            ),
            "11.***.***-0001-81"
        );
    }

    #[test]
    fn mascara_de_inscricao_e_codigo() {
        assert_eq!(
            mascarar_chave(
                &TipoChave::Inscricao(
                    NumeroInscricao::novo("90 6 17 000964-19").expect("inscrição")
                ),
                "9061700096419"
            ),
            "906*******419"
        );
        assert_eq!(
            mascarar_chave(
                &TipoChave::CodigoContribuinte(String::from("AB3456")),
                "AB3456"
            ),
            "AB**56"
        );
    }
}
