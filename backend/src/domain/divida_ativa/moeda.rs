//! Dinheiro da Dívida Ativa: os valores chegam como **string** em pt-BR
//! (`"805,06"`, `"4.457,56"`, `"1234.56"`) e são convertidos para
//! `rust_decimal::Decimal` — decimal exato, nunca `f32`/`f64`
//! (SPEC §4.3/§4.5, ADR-0004: sanitizar antes da serialização/persistência).
//!
//! Regras de interpretação (estritas, sem inferência):
//!
//! 1. Só dígitos → número inteiro (`805` → 805).
//! 2. `vírgula` presente (uma única) → vírgula separa os decimais; pontos
//!    existentes (se houver) precisam ser milhar válida em grupos de 3
//!    (`4.457,56` → 4457.56). Parte decimal: 1 ou 2 dígitos.
//! 3. somente `ponto` → o último grupo separa **decimais** quando tem 1 ou
//!    2 dígitos e o resto forma milhar válida (`1234.56` → 1234.56;
//!    `1.234.56` → 1234.56; `0.5` → 0.5); quando o último grupo tem 3
//!    dígitos e o resto forma milhar válida, o ponto separa **milhar**
//!    (`1.234` → 1234, `0.000` → 0, `1.234.560` → 1234560). Estruturas
//!    ambíguas (`1.23456`, `12.3456`) são rejeitadas.
//! 4. espaços em volta são ignorados; qualquer outro caractere (`R$`,
//!    sinal, letras, vírgulas repetidas) é rejeitado.
//!
//! A parte inteira é limitada a 12 dígitos e a escala a 2 casas — os
//! saldos da Dívida Ativa não extrapolam isso; estouro vira erro de
//! resposta inválida no chamador (nunca `f32`/`f64`).

use std::str::FromStr;

use rust_decimal::Decimal;
use thiserror::Error;

/// Número máximo de dígitos inteiros aceitos.
const DIGITOS_INTEIROS_MAXIMOS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErroMoeda {
    #[error("valor monetário ausente ou em branco")]
    Vazio,
    #[error("valor monetário com caracteres não numéricos")]
    CaracteresNaoNumericos,
    #[error("valor monetário fora do padrão pt-BR esperado")]
    FormatoInvalido,
}

pub fn parse_moeda_ptbr_estrito(bruto: &str) -> Result<Decimal, ErroMoeda> {
    let texto = bruto.trim();
    if texto.is_empty() {
        return Err(ErroMoeda::Vazio);
    }

    if e_digitos(texto) {
        return decimais(texto.to_owned(), "");
    }

    if texto.contains(',') {
        return com_virgula(texto);
    }

    if texto.contains('.') {
        return com_apenas_pontos(texto);
    }

    Err(ErroMoeda::CaracteresNaoNumericos)
}

fn com_virgula(texto: &str) -> Result<Decimal, ErroMoeda> {
    if texto.matches(',').count() != 1 {
        return Err(ErroMoeda::FormatoInvalido);
    }

    let (parte_inteira, parte_decimal) = texto
        .split_once(',')
        .expect("split_once já garantiu que existe vírgula");

    if !e_digitos(parte_decimal) {
        return Err(ErroMoeda::CaracteresNaoNumericos);
    }
    match parte_decimal.len() {
        1 | 2 => {}
        _ => return Err(ErroMoeda::FormatoInvalido),
    }

    let inteira = if parte_inteira.is_empty() {
        "0".to_owned()
    } else {
        if !e_digitos(parte_inteira.replace('.', "").as_str()) {
            return Err(ErroMoeda::CaracteresNaoNumericos);
        }
        if !milhar_valida_com_pontos(parte_inteira) {
            return Err(ErroMoeda::FormatoInvalido);
        }
        parte_inteira.replace('.', "")
    };

    decimais(inteira, parte_decimal)
}

fn com_apenas_pontos(texto: &str) -> Result<Decimal, ErroMoeda> {
    if !texto
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return Err(ErroMoeda::CaracteresNaoNumericos);
    }

    let grupos: Vec<&str> = texto.split('.').collect();
    let ultima = *grupos
        .last()
        .expect("split sempre devolve ao menos um grupo");
    let anteriores = &grupos[..grupos.len() - 1];

    match ultima.len() {
        1 | 2 => {
            // Decimal: um único ponto aceita prefixo qualquer de dígitos
            // ("1234.56"); com vários pontos, o resto precisa ser milhar
            // válida ("1.234.56").
            let inteira = if grupos.len() == 2 {
                let pura = anteriores.concat();
                if !e_digitos(pura.as_str()) {
                    return Err(ErroMoeda::CaracteresNaoNumericos);
                }
                pura
            } else {
                partes_como_digito_sem_pontos(anteriores)?
            };
            decimais(inteira, ultima)
        }
        3 => {
            // Milhar: tríade final + milhar válida à esquerda (com um
            // único ponto, o prefixo é apenas dígitos: "1.234" → 1234).
            let inteira = if grupos.len() == 2 {
                let pura = anteriores.concat();
                if !e_digitos(pura.as_str()) {
                    return Err(ErroMoeda::CaracteresNaoNumericos);
                }
                pura
            } else {
                partes_como_digito_sem_pontos(anteriores)?
            };
            let decimais_inteira = format!("{inteira}{ultima}");
            decimais(decimais_inteira, "")
        }
        _ => Err(ErroMoeda::FormatoInvalido),
    }
}

/// Valida grupos de milhar (`1.234.560` / `12.345`) e devolve os dígitos
/// sem pontos. Grupo vazio devolve `0`.
#[allow(clippy::unnecessary_wraps)]
fn partes_como_digito_sem_pontos(anteriores: &[&str]) -> Result<String, ErroMoeda> {
    let pura = anteriores.concat().replace('.', ""); // concat já remove; seguro p/ clareza
    if pura.is_empty() {
        return Ok("0".to_owned());
    }
    if !e_digitos(pura.as_str()) {
        return Err(ErroMoeda::CaracteresNaoNumericos);
    }
    if !milhar_valida_com_grupos(anteriores) {
        return Err(ErroMoeda::FormatoInvalido);
    }
    validar_comprimento(&pura)?;
    Ok(pura)
}

/// Regra de milhar nos grupos já divididos por ponto: o primeiro grupo tem
/// 1..=3 dígitos, os seguintes obrigatoriamente têm 3.
fn milhar_valida_com_grupos(grupos: &[&str]) -> bool {
    if grupos.is_empty() {
        return true;
    }
    let primeira = grupos[0];
    if primeira.is_empty() || primeira.len() > 3 || !e_digitos(primeira) {
        return false;
    }
    grupos[1..]
        .iter()
        .all(|grupo| grupo.len() == 3 && grupo.bytes().all(|byte| byte.is_ascii_digit()))
}

fn milhar_valida_com_pontos(parte_inteira: &str) -> bool {
    if !parte_inteira.contains('.') {
        return true;
    }
    let grupos: Vec<&str> = parte_inteira.split('.').collect();
    milhar_valida_com_grupos(&grupos)
}

fn e_digitos(texto: &str) -> bool {
    !texto.is_empty() && texto.bytes().all(|byte| byte.is_ascii_digit())
}

fn validar_comprimento(inteira: &str) -> Result<(), ErroMoeda> {
    let significativo = inteira.trim_start_matches('0');
    let significativo = if significativo.is_empty() {
        "0"
    } else {
        significativo
    };
    if significativo.len() > DIGITOS_INTEIROS_MAXIMOS {
        return Err(ErroMoeda::FormatoInvalido);
    }
    Ok(())
}

fn decimais(digitos_inteiros: String, digitos_decimais: &str) -> Result<Decimal, ErroMoeda> {
    validar_comprimento(&digitos_inteiros)?;

    let canonica = if digitos_decimais.is_empty() {
        digitos_inteiros
    } else {
        format!("{digitos_inteiros}.{digitos_decimais}")
    };

    Decimal::from_str(&canonica).map_err(|_| ErroMoeda::FormatoInvalido)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{ErroMoeda, parse_moeda_ptbr_estrito};

    fn valor(inteiros: i64, centavos: u32) -> Decimal {
        Decimal::new(inteiros * 100 + i64::from(centavos), 2)
    }

    #[test]
    fn centavos_com_virgula() {
        assert_eq!(parse_moeda_ptbr_estrito("805,06").unwrap(), valor(805, 6));
        assert_eq!(parse_moeda_ptbr_estrito("0,5").unwrap(), valor(0, 50));
        assert_eq!(parse_moeda_ptbr_estrito("0,05").unwrap(), valor(0, 5));
    }

    #[test]
    fn milhar_com_ponto_e_centavos_com_virgula() {
        assert_eq!(
            parse_moeda_ptbr_estrito("4.457,56").unwrap(),
            valor(4_457, 56)
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("2.403,59").unwrap(),
            valor(2_403, 59)
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("1.234.560,78").unwrap(),
            valor(1_234_560, 78)
        );
    }

    #[test]
    fn ponto_unico_e_decimal() {
        assert_eq!(
            parse_moeda_ptbr_estrito("1234.56").unwrap(),
            valor(1234, 56)
        );
        assert_eq!(parse_moeda_ptbr_estrito("12.3").unwrap(), valor(12, 30));
        assert_eq!(parse_moeda_ptbr_estrito("0.5").unwrap(), valor(0, 50));
        assert_eq!(
            parse_moeda_ptbr_estrito("1.234.56").unwrap(),
            valor(1234, 56)
        );
    }

    #[test]
    fn pontoes_de_milhar_sem_centavos() {
        assert_eq!(
            parse_moeda_ptbr_estrito("1.234.560").unwrap(),
            valor(1_234_560, 0)
        );
        assert_eq!(parse_moeda_ptbr_estrito("1.234").unwrap(), valor(1234, 0));
        assert_eq!(parse_moeda_ptbr_estrito("0.000").unwrap(), valor(0, 0));
    }

    #[test]
    fn ambiguidades_sao_rejeitadas() {
        assert_eq!(
            parse_moeda_ptbr_estrito("1.23456").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("12.3456").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("45.7,56").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("8.0.5,06").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("12.34,56").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("805,,06").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("805,0,6").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
    }

    #[test]
    fn espacos_sao_ignorados() {
        assert_eq!(
            parse_moeda_ptbr_estrito("  805,06  ").unwrap(),
            valor(805, 6)
        );
    }

    #[test]
    fn so_digitos_sao_inteiros() {
        assert_eq!(parse_moeda_ptbr_estrito("805").unwrap(), valor(805, 0));
        assert_eq!(
            parse_moeda_ptbr_estrito("1234560").unwrap(),
            valor(1_234_560, 0)
        );
    }

    #[test]
    fn rejeita_vazios_e_nao_numericos() {
        assert_eq!(parse_moeda_ptbr_estrito("").unwrap_err(), ErroMoeda::Vazio);
        assert_eq!(
            parse_moeda_ptbr_estrito("   ").unwrap_err(),
            ErroMoeda::Vazio
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("abc").unwrap_err(),
            ErroMoeda::CaracteresNaoNumericos
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("R$ 805,06").unwrap_err(),
            ErroMoeda::CaracteresNaoNumericos
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("-805,06").unwrap_err(),
            ErroMoeda::CaracteresNaoNumericos
        );
    }

    #[test]
    fn parte_inteira_tem_limite() {
        assert_eq!(
            parse_moeda_ptbr_estrito("999999999999,99").unwrap(),
            Decimal::new(99_999_999_999_999, 2)
        );
        assert_eq!(
            parse_moeda_ptbr_estrito("9999999999999,99").unwrap_err(),
            ErroMoeda::FormatoInvalido
        );
    }
}
