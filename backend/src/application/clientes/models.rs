#[derive(Debug, Clone)]
pub struct PoliticaClientes {
    pub max_nome_len: usize,
    pub max_fantasia_len: usize,
    pub max_email_len: usize,
    pub max_telefone_len: usize,
}

impl Default for PoliticaClientes {
    fn default() -> Self {
        Self {
            max_nome_len: 200,
            max_fantasia_len: 200,
            max_email_len: 254,
            max_telefone_len: 30,
        }
    }
}
