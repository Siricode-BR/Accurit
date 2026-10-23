use std::sync::Arc;

use uuid::Uuid;

use crate::application::clientes::{
    AtualizarClienteCommand, AuditoriaClienteRepositorio, ClienteError, ClientePublico,
    ClienteRepositorio, CriarClienteCommand, EventoAuditoriaCliente, ExcluirClienteCommand,
    ListagemClientes, ListarClientesCommand, ObterClienteCommand, PoliticaClientes,
    RepositorioError,
};
use crate::domain::clientes::{Cliente, CpfCnpj, NovoCliente, TipoCliente};

#[derive(Clone)]
pub struct CriarClienteUseCase {
    repositorio: Arc<dyn ClienteRepositorio>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    politica: PoliticaClientes,
}

impl CriarClienteUseCase {
    pub fn novo(
        repositorio: Arc<dyn ClienteRepositorio>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
        politica: PoliticaClientes,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
            politica,
        }
    }

    pub async fn executar(
        &self,
        comando: CriarClienteCommand,
    ) -> Result<ClientePublico, ClienteError> {
        let cpf_cnpj = CpfCnpj::novo(&comando.cpf_cnpj)?;
        let documento_normalizado = cpf_cnpj.como_str().to_owned();

        if self
            .repositorio
            .buscar_por_cpf_cnpj(comando.escritorio_id, &documento_normalizado)?
            .is_some()
        {
            return Err(ClienteError::Repositorio(RepositorioError::Duplicado));
        }

        validar_nome(&comando.nome_razao_social, self.politica.max_nome_len)?;
        if let Some(ref fantasia) = comando.nome_fantasia {
            validar_opcional_max_len(fantasia, self.politica.max_fantasia_len, "nome_fantasia")?;
        }
        if let Some(ref email) = comando.email {
            validar_opcional_max_len(email, self.politica.max_email_len, "email")?;
        }
        if let Some(ref telefone) = comando.telefone {
            validar_opcional_max_len(telefone, self.politica.max_telefone_len, "telefone")?;
        }

        let novo = NovoCliente {
            tipo: comando.tipo,
            porte_regime: comando.porte_regime,
            documento: cpf_cnpj,
            nome_razao_social: comando.nome_razao_social.trim().to_owned(),
            nome_fantasia: comando.nome_fantasia.as_ref().map(|s| s.trim().to_owned()),
            email: comando.email.as_ref().map(|s| s.trim().to_owned()),
            telefone: comando.telefone.as_ref().map(|s| s.trim().to_owned()),
        };

        novo.validar()?;

        let cliente = Cliente {
            id: Uuid::now_v7(),
            escritorio_id: comando.escritorio_id,
            tipo: novo.tipo,
            porte_regime: novo.porte_regime,
            documento: novo.documento,
            nome_razao_social: novo.nome_razao_social,
            nome_fantasia: novo.nome_fantasia,
            email: novo.email,
            telefone: novo.telefone,
            criado_em: chrono::Utc::now(),
            atualizado_em: chrono::Utc::now(),
        };

        self.repositorio.criar(comando.escritorio_id, &cliente)?;

        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "CLIENTE_CRIADO",
            entidade: "clientes",
            entidade_id: cliente.id,
            valor_anterior: None,
            valor_novo: Some(serde_json::to_value(&cliente).unwrap_or_default()),
            ip: comando.ip,
        })?;

        Ok((&cliente).into())
    }
}

#[derive(Clone)]
pub struct ObterClienteUseCase {
    repositorio: Arc<dyn ClienteRepositorio>,
}

impl ObterClienteUseCase {
    pub fn novo(repositorio: Arc<dyn ClienteRepositorio>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: ObterClienteCommand,
    ) -> Result<ClientePublico, ClienteError> {
        let cliente = self
            .repositorio
            .buscar_por_id(comando.escritorio_id, comando.id)?
            .ok_or(ClienteError::Repositorio(RepositorioError::NaoEncontrado))?;

        Ok((&cliente).into())
    }
}

#[derive(Clone)]
pub struct ListarClientesUseCase {
    repositorio: Arc<dyn ClienteRepositorio>,
}

impl ListarClientesUseCase {
    pub fn novo(repositorio: Arc<dyn ClienteRepositorio>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: ListarClientesCommand,
    ) -> Result<ListagemClientes, ClienteError> {
        Ok(self
            .repositorio
            .listar(comando.escritorio_id, comando.filtro, comando.paginacao)?)
    }
}

#[derive(Clone)]
pub struct AtualizarClienteUseCase {
    repositorio: Arc<dyn ClienteRepositorio>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    politica: PoliticaClientes,
}

impl AtualizarClienteUseCase {
    pub fn novo(
        repositorio: Arc<dyn ClienteRepositorio>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
        politica: PoliticaClientes,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
            politica,
        }
    }

    pub async fn executar(
        &self,
        comando: AtualizarClienteCommand,
    ) -> Result<ClientePublico, ClienteError> {
        let mut cliente = self
            .repositorio
            .buscar_por_id(comando.escritorio_id, comando.id)?
            .ok_or(ClienteError::Repositorio(RepositorioError::NaoEncontrado))?;

        let valor_anterior = serde_json::to_value(&cliente).unwrap_or_default();

        if let Some(tipo) = comando.tipo
            && tipo != cliente.tipo
        {
            if tipo == TipoCliente::Pf && cliente.porte_regime.is_some() {
                return Err(ClienteError::Dominio(
                    crate::domain::clientes::ClienteInvalido::PfComPorte,
                ));
            }
            if tipo == TipoCliente::Pj
                && matches!(
                    cliente.documento.tipo(),
                    crate::domain::clientes::TipoDocumento::Cpf
                )
            {
                return Err(ClienteError::Dominio(
                    crate::domain::clientes::ClienteInvalido::TipoIncompativelComDocumento,
                ));
            }
            cliente.tipo = tipo;
        }

        if let Some(porte_regime) = comando.porte_regime {
            if cliente.tipo == TipoCliente::Pf {
                return Err(ClienteError::Dominio(
                    crate::domain::clientes::ClienteInvalido::PfComPorte,
                ));
            }
            cliente.porte_regime = Some(porte_regime);
        }

        if let Some(cpf_cnpj) = comando.cpf_cnpj {
            let documento = CpfCnpj::novo(&cpf_cnpj)?;
            if cliente.tipo == TipoCliente::Pf
                && !matches!(
                    documento.tipo(),
                    crate::domain::clientes::TipoDocumento::Cpf
                )
            {
                return Err(ClienteError::Dominio(
                    crate::domain::clientes::ClienteInvalido::TipoIncompativelComDocumento,
                ));
            }
            if cliente.tipo == TipoCliente::Pj
                && !matches!(
                    documento.tipo(),
                    crate::domain::clientes::TipoDocumento::Cnpj
                )
            {
                return Err(ClienteError::Dominio(
                    crate::domain::clientes::ClienteInvalido::TipoIncompativelComDocumento,
                ));
            }

            let doc_norm = documento.como_str().to_owned();
            if doc_norm != cliente.documento.como_str()
                && self
                    .repositorio
                    .buscar_por_cpf_cnpj(comando.escritorio_id, &doc_norm)?
                    .is_some()
            {
                return Err(ClienteError::Repositorio(RepositorioError::Duplicado));
            }
            cliente.documento = documento;
        }

        if let Some(nome) = comando.nome_razao_social {
            validar_nome(&nome, self.politica.max_nome_len)?;
            cliente.nome_razao_social = nome.trim().to_owned();
        }

        aplicar_opcional(
            &mut cliente.nome_fantasia,
            comando.nome_fantasia,
            self.politica.max_fantasia_len,
            "nome_fantasia",
        )?;
        aplicar_opcional(
            &mut cliente.email,
            comando.email,
            self.politica.max_email_len,
            "email",
        )?;
        aplicar_opcional(
            &mut cliente.telefone,
            comando.telefone,
            self.politica.max_telefone_len,
            "telefone",
        )?;

        cliente.atualizado_em = chrono::Utc::now();

        self.repositorio
            .atualizar(comando.escritorio_id, &cliente)?;

        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "CLIENTE_ATUALIZADO",
            entidade: "clientes",
            entidade_id: cliente.id,
            valor_anterior: Some(valor_anterior),
            valor_novo: Some(serde_json::to_value(&cliente).unwrap_or_default()),
            ip: comando.ip,
        })?;

        Ok((&cliente).into())
    }
}

#[derive(Clone)]
pub struct ExcluirClienteUseCase {
    repositorio: Arc<dyn ClienteRepositorio>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl ExcluirClienteUseCase {
    pub fn novo(
        repositorio: Arc<dyn ClienteRepositorio>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
        }
    }

    pub async fn executar(&self, comando: ExcluirClienteCommand) -> Result<(), ClienteError> {
        let cliente = self
            .repositorio
            .buscar_por_id(comando.escritorio_id, comando.id)?
            .ok_or(ClienteError::Repositorio(RepositorioError::NaoEncontrado))?;

        let excluido = self
            .repositorio
            .excluir(comando.escritorio_id, comando.id)?;

        if !excluido {
            return Err(ClienteError::Repositorio(RepositorioError::NaoEncontrado));
        }

        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "CLIENTE_EXCLUIDO",
            entidade: "clientes",
            entidade_id: cliente.id,
            valor_anterior: Some(serde_json::to_value(&cliente).unwrap_or_default()),
            valor_novo: None,
            ip: comando.ip,
        })?;

        Ok(())
    }
}

fn validar_nome(nome: &str, max_len: usize) -> Result<(), ClienteError> {
    let limpo = nome.trim();
    if limpo.is_empty() || limpo.chars().count() > max_len {
        return Err(ClienteError::Validacao(
            "nome deve ter entre 1 e 200 caracteres".into(),
        ));
    }
    Ok(())
}

fn validar_opcional_max_len(valor: &str, max_len: usize, campo: &str) -> Result<(), ClienteError> {
    let limpo = valor.trim();
    if !limpo.is_empty() && limpo.chars().count() > max_len {
        return Err(ClienteError::Validacao(format!(
            "{} deve ter no m\u{00e1}ximo {} caracteres",
            campo, max_len
        )));
    }
    Ok(())
}

fn aplicar_opcional(
    campo_atual: &mut Option<String>,
    comando: Option<Option<String>>,
    max_len: usize,
    campo: &str,
) -> Result<(), ClienteError> {
    if let Some(valor) = comando {
        match valor {
            Some(valor) if !valor.trim().is_empty() => {
                validar_opcional_max_len(&valor, max_len, campo)?;
                *campo_atual = Some(valor.trim().to_owned());
            }
            _ => *campo_atual = None,
        }
    }
    Ok(())
}
