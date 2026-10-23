use std::fmt;
use std::str::FromStr;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TipoDocumento {
    Cpf,
    Cnpj,
}

impl TipoDocumento {
    pub const fn quantidade_digitos(self) -> usize {
        match self {
            Self::Cpf => 11,
            Self::Cnpj => 14,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CpfCnpjInvalido {
    #[error("documento deve conter 11 dígitos (CPF) ou 14 dígitos (CNPJ)")]
    Formato,
    #[error("dígito verificador do documento inválido")]
    DigitoVerificador,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct CpfCnpj {
    digitos: String,
    tipo: TipoDocumento,
}

impl CpfCnpj {
    pub fn novo(valor: impl AsRef<str>) -> Result<Self, CpfCnpjInvalido> {
        let digitos: String = valor
            .as_ref()
            .chars()
            .filter(|caractere| caractere.is_ascii_digit())
            .collect();

        let tipo = match digitos.len() {
            11 => TipoDocumento::Cpf,
            14 => TipoDocumento::Cnpj,
            _ => return Err(CpfCnpjInvalido::Formato),
        };

        if todos_digitos_iguais(&digitos) || !digitos_verificadores_validos(&digitos, tipo) {
            return Err(CpfCnpjInvalido::DigitoVerificador);
        }

        Ok(Self { digitos, tipo })
    }

    pub const fn tipo(&self) -> TipoDocumento {
        self.tipo
    }

    pub fn como_str(&self) -> &str {
        &self.digitos
    }
}

fn todos_digitos_iguais(digitos: &str) -> bool {
    digitos.bytes().all(|byte| byte == digitos.as_bytes()[0])
}

fn digitos_verificadores_validos(digitos: &str, tipo: TipoDocumento) -> bool {
    let numeros: Vec<u32> = digitos.bytes().map(|byte| u32::from(byte - b'0')).collect();

    match tipo {
        TipoDocumento::Cpf => {
            let primeiro = calcular_digito_cpf(&numeros[..9], 10);
            let segundo = calcular_digito_cpf(&numeros[..10], 11);
            numeros[9] == primeiro && numeros[10] == segundo
        }
        TipoDocumento::Cnpj => {
            let primeiro = calcular_digito_cnpj(&numeros[..12], 5);
            let segundo = calcular_digito_cnpj(&numeros[..13], 6);
            numeros[12] == primeiro && numeros[13] == segundo
        }
    }
}

fn calcular_digito_cpf(digitos: &[u32], peso_inicial: u32) -> u32 {
    let soma: u32 = digitos
        .iter()
        .enumerate()
        .map(|(indice, digito)| digito * (peso_inicial - indice as u32))
        .sum();
    let resto = (soma * 10) % 11;
    if resto == 10 { 0 } else { resto }
}

fn calcular_digito_cnpj(digitos: &[u32], peso_inicial: u32) -> u32 {
    let soma: u32 = digitos
        .iter()
        .enumerate()
        .map(|(indice, digito)| digito * peso_cnpj(indice as u32, peso_inicial))
        .sum();
    let resto = soma % 11;
    if resto < 2 { 0 } else { 11 - resto }
}

fn peso_cnpj(indice: u32, peso_inicial: u32) -> u32 {
    if indice <= peso_inicial - 2 {
        peso_inicial - indice
    } else {
        9 - ((indice - (peso_inicial - 1)) % 8)
    }
}

impl fmt::Display for CpfCnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.digitos)
    }
}

impl FromStr for CpfCnpj {
    type Err = CpfCnpjInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::novo(valor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_cpf_valido_com_e_sem_mascara() {
        let documento = CpfCnpj::novo("529.982.247-25").expect("CPF válido");
        assert_eq!(documento.como_str(), "52998224725");
        assert_eq!(documento.tipo(), TipoDocumento::Cpf);
    }

    #[test]
    fn aceita_cnpj_valido_com_e_sem_mascara() {
        let documento = CpfCnpj::novo("11.222.333/0001-81").expect("CNPJ válido");
        assert_eq!(documento.como_str(), "11222333000181");
        assert_eq!(documento.tipo(), TipoDocumento::Cnpj);

        let segundo = CpfCnpj::novo("12.345.678/0001-95").expect("CNPJ válido");
        assert_eq!(segundo.tipo(), TipoDocumento::Cnpj);
    }

    #[test]
    fn rejeita_formato_invalido() {
        assert_eq!(CpfCnpj::novo("123").unwrap_err(), CpfCnpjInvalido::Formato);
        assert_eq!(
            CpfCnpj::novo("123456789012345").unwrap_err(),
            CpfCnpjInvalido::Formato
        );
        assert_eq!(
            CpfCnpj::novo("1234567890123").unwrap_err(),
            CpfCnpjInvalido::Formato
        );
    }

    #[test]
    fn rejeita_digitos_verificadores_invalidos() {
        assert_eq!(
            CpfCnpj::novo("52998224724").unwrap_err(),
            CpfCnpjInvalido::DigitoVerificador
        );
        assert_eq!(
            CpfCnpj::novo("11444777000162").unwrap_err(),
            CpfCnpjInvalido::DigitoVerificador
        );
    }

    #[test]
    fn rejeita_digitos_iguais() {
        assert_eq!(
            CpfCnpj::novo("11111111111").unwrap_err(),
            CpfCnpjInvalido::DigitoVerificador
        );
        assert_eq!(
            CpfCnpj::novo("00000000000000").unwrap_err(),
            CpfCnpjInvalido::DigitoVerificador
        );
    }
}
