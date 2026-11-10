use std::fmt;
use std::str::FromStr;

use thiserror::Error;

const TAMANHO_CNPJ: usize = 14;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cnpj(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CnpjInvalido {
    #[error("CNPJ deve conter exatamente 14 dígitos")]
    Formato,
    #[error("CNPJ inválido")]
    DigitoVerificador,
}

impl Cnpj {
    pub fn novo(valor: impl AsRef<str>) -> Result<Self, CnpjInvalido> {
        let digitos: String = valor
            .as_ref()
            .chars()
            .filter(|caractere| caractere.is_ascii_digit())
            .collect();

        if digitos.len() != TAMANHO_CNPJ {
            return Err(CnpjInvalido::Formato);
        }

        if todos_digitos_iguais(&digitos) || !digitos_verificadores_validos(&digitos) {
            return Err(CnpjInvalido::DigitoVerificador);
        }

        Ok(Self(digitos))
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

fn todos_digitos_iguais(digitos: &str) -> bool {
    digitos.bytes().all(|byte| byte == digitos.as_bytes()[0])
}

fn digitos_verificadores_validos(cnpj: &str) -> bool {
    let numeros: Vec<u32> = cnpj.bytes().map(|byte| u32::from(byte - b'0')).collect();

    // Primeiro dígito verificador
    let primeiro_peso: Vec<u32> = vec![2u32; 5]
        .into_iter()
        .chain(vec![3u32; 4])
        .chain(vec![4u32; 3])
        .chain(vec![5u32; 2])
        .chain(std::iter::once(6u32))
        .take(12)
        .collect();

    let soma1: u32 = numeros[..12]
        .iter()
        .enumerate()
        .map(|(indice, digito)| digito * primeiro_peso[indice])
        .sum();

    let resto1 = (soma1 * 10) % 11;
    let digito1 = if resto1 == 10 { 0 } else { resto1 };

    if numeros[12] != digito1 {
        return false;
    }

    // Segundo dígito verificador
    let segundo_peso: Vec<u32> = vec![2u32; 6]
        .into_iter()
        .chain(vec![3u32; 5])
        .chain(vec![4u32; 4])
        .chain(vec![5u32; 3])
        .chain(vec![6u32; 2])
        .take(12)
        .collect();

    let soma2: u32 = numeros[..12]
        .iter()
        .enumerate()
        .map(|(indice, digito)| digito * segundo_peso[indice])
        .sum();

    let resto2 = (soma2 * 10) % 11;
    let digito2 = if resto2 == 10 { 0 } else { resto2 };

    numeros[13] == digito2
}

impl fmt::Display for Cnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Cnpj {
    type Err = CnpjInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::novo(valor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_cnpj_valido_com_e_sem_mascara() {
        assert_eq!(
            Cnpj::novo("52998224725896").unwrap().como_str(),
            "52998224725896"
        );
        assert_eq!(
            Cnpj::novo("52.998.224/7258-96").unwrap().como_str(),
            "52998224725896"
        );
    }

    #[test]
    fn rejeita_cnpj_com_formato_invalido() {
        assert_eq!(Cnpj::novo("123").unwrap_err(), CnpjInvalido::Formato);
        assert_eq!(
            Cnpj::novo("52998224725897").unwrap_err(),
            CnpjInvalido::DigitoVerificador
        );
        assert_eq!(
            Cnpj::novo("11111111111111").unwrap_err(),
            CnpjInvalido::DigitoVerificador
        );
    }

    #[test]
    fn rejeita_cnpj_com_todos_digitos_iguais() {
        assert_eq!(
            Cnpj::novo("00000000000000").unwrap_err(),
            CnpjInvalido::DigitoVerificador
        );
    }
}
