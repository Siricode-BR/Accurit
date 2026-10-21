use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cpf(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CpfInvalido {
    #[error("CPF deve conter exatamente 11 dígitos")]
    Formato,
    #[error("CPF inválido")]
    DigitoVerificador,
}

impl Cpf {
    pub fn novo(valor: impl AsRef<str>) -> Result<Self, CpfInvalido> {
        let digitos: String = valor
            .as_ref()
            .chars()
            .filter(|caractere| caractere.is_ascii_digit())
            .collect();

        if digitos.len() != 11 {
            return Err(CpfInvalido::Formato);
        }

        if todos_digitos_iguais(&digitos) || !digitos_verificadores_validos(&digitos) {
            return Err(CpfInvalido::DigitoVerificador);
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

fn digitos_verificadores_validos(digitos: &str) -> bool {
    let numeros: Vec<u32> = digitos.bytes().map(|byte| u32::from(byte - b'0')).collect();

    let primeiro = calcular_digito(&numeros[..9], 10);
    let segundo = calcular_digito(&numeros[..10], 11);

    numeros[9] == primeiro && numeros[10] == segundo
}

fn calcular_digito(digitos: &[u32], peso_inicial: u32) -> u32 {
    let soma: u32 = digitos
        .iter()
        .enumerate()
        .map(|(indice, digito)| digito * (peso_inicial - indice as u32))
        .sum();
    let resto = (soma * 10) % 11;
    if resto == 10 { 0 } else { resto }
}

impl fmt::Display for Cpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Cpf {
    type Err = CpfInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::novo(valor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_cpf_valido_com_e_sem_mascara() {
        assert_eq!(Cpf::novo("52998224725").unwrap().como_str(), "52998224725");
        assert_eq!(
            Cpf::novo("529.982.247-25").unwrap().como_str(),
            "52998224725"
        );
    }

    #[test]
    fn rejeita_cpf_com_formato_ou_digito_invalido() {
        assert_eq!(Cpf::novo("123").unwrap_err(), CpfInvalido::Formato);
        assert_eq!(
            Cpf::novo("52998224724").unwrap_err(),
            CpfInvalido::DigitoVerificador
        );
        assert_eq!(
            Cpf::novo("11111111111").unwrap_err(),
            CpfInvalido::DigitoVerificador
        );
    }
}
