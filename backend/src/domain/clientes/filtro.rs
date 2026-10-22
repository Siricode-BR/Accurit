use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("filtro de clientes inválido")]
pub struct FiltroInvalido;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiltroClientes {
    Todos,
    Cpf,
    CnpjMei,
    CnpjSimplesNacional,
    CnpjLucroPresumido,
    CnpjLucroReal,
}

impl FiltroClientes {
    pub const fn como_query(self) -> Option<&'static str> {
        match self {
            Self::Todos => None,
            Self::Cpf => Some("cpf"),
            Self::CnpjMei => Some("cnpj-mei"),
            Self::CnpjSimplesNacional => Some("cnpj-simples-nacional"),
            Self::CnpjLucroPresumido => Some("cnpj-lucro-presumido"),
            Self::CnpjLucroReal => Some("cnpj-lucro-real"),
        }
    }

    pub fn de_query(valor: &str) -> Option<Self> {
        let valor = valor.trim().to_ascii_lowercase();
        match valor.as_str() {
            "todos" | "" => Some(Self::Todos),
            "cpf" => Some(Self::Cpf),
            "cpf-mei" => Some(Self::CnpjMei),
            "cpf-simples-nacional" => Some(Self::CnpjSimplesNacional),
            "cpf-lucro-presumido" => Some(Self::CnpjLucroPresumido),
            "cpf-lucro-real" => Some(Self::CnpjLucroReal),
            _ => None,
        }
    }
}

impl FromStr for FiltroClientes {
    type Err = FiltroInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::de_query(valor).ok_or(FiltroInvalido)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("paginação inválida")]
pub struct PaginacaoInvalida;

const TAMANHO_PADRAO: i64 = 20;
const TAMANHO_MAXIMO: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paginacao {
    pub pagina: i64,
    pub tamanho: i64,
}

impl Paginacao {
    pub fn nova(pagina: Option<i64>, tamanho: Option<i64>) -> Result<Self, PaginacaoInvalida> {
        Self::validada(pagina.unwrap_or(1), tamanho.unwrap_or(TAMANHO_PADRAO))
    }

    pub fn validada(pagina: i64, tamanho: i64) -> Result<Self, PaginacaoInvalida> {
        if pagina < 1 || tamanho < 1 || tamanho > TAMANHO_MAXIMO {
            return Err(PaginacaoInvalida);
        }

        Ok(Self {
            pagina,
            tamanho,
        })
    }

    pub const fn deslocamento(&self) -> i64 {
        (self.pagina - 1) * self.tamanho
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtro_ida_e_volta() {
        for (valor, esperado) in [
            ("todos", None),
            ("cpf", Some("cpf")),
            ("cpf-mei", Some("cnpj-mei")),
            ("cpf-simples-nacional", Some("cnpj-simples-nacional")),
            ("cpf-lucro-presumido", Some("cnpj-lucro-presumido")),
            ("cpf-lucro-real", Some("cnpj-lucro-real")),
        ] {
            let filtro = FiltroClientes::de_query(valor).expect("filtro válido");
            assert_eq!(filtro.como_query(), esperado);
        }
    }

    #[test]
    fn rejeita_filtro_desconhecido() {
        assert!(FiltroClientes::de_query("rural").is_none());
        assert!("rural".parse::<FiltroClientes>().is_err());
    }

    #[test]
    fn paginacao_com_valores_padrao_e_limites() {
        let padrao = Paginacao::nova(None, None).expect("padrão");
        assert_eq!(padrao.pagina, 1);
        assert_eq!(padrao.tamanho, 20);
        assert_eq!(padrao.deslocamento(), 0);

        assert!(Paginacao::nova(Some(0), None).is_err());
        assert!(Paginacao::nova(None, Some(0)).is_err());
        assert!(Paginacao::nova(None, Some(101)).is_err());

        let limite = Paginacao::nova(None, Some(100)).expect("limite");
        assert_eq!(limite.deslocamento(), 0);

        let terceira = Paginacao::nova(Some(3), Some(20)).expect("válida");
        assert_eq!(terceira.deslocamento(), 40);
    }
}
