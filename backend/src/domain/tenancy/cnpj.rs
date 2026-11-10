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

    let primeiro = calcular_digito_cnpj(&numeros[..12], &[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
    if numeros[12] != primeiro {
        return false;
    }

    let segundo = calcular_digito_cnpj(&numeros[..13], &[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
    numeros[13] == segundo
}

fn calcular_digito_cnpj(digitos: &[u32], pesos: &[u32]) -> u32 {
    let soma: u32 = digitos
        .iter()
        .zip(pesos)
        .map(|(digito, peso)| digito * peso)
        .sum();
    let resto = soma % 11;
    if resto < 2 { 0 } else { 11 - resto }
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
            Cnpj::novo("11222333000181").unwrap().como_str(),
            "11222333000181"
        );
        assert_eq!(
            Cnpj::novo("11.222.333/0001-81").unwrap().como_str(),
            "11222333000181"
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
