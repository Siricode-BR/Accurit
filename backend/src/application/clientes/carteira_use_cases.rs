//! Fase E — Gestão de carteira: casos de uso de sócios/contatos, filiais,
//! tags, carteira por cliente (responsável/observações) e acesso por função.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::clientes::ports::{
    ClienteError, ClienteRepositorio, NovoClienteFilialCommand, NovaClienteTagCommand,
    NovoSocioContatoCommand, RepositorioError,
};
use crate::application::clientes::ports::{
    AuditoriaClienteRepositorio, EventoAuditoriaCliente, RepositorioAcessoUsuarios,
    RepositorioCarteiraClientes,
};
use crate::domain::auth::Papel;
use crate::domain::clientes::{
    AcessoCarteira, AtualizarCarteiraCliente, CpfCnpj, FuncaoUsuario, NovoClienteFilial,
    NovoSocioContato, PapelSocioContato,
};
use crate::domain::clientes::{ClienteTag, NovaClienteTag};

/// Calcula o acesso de um usuário à carteira: função efetiva + vínculos.
#[derive(Clone)]
pub struct VerificarAcessoUseCase {
    acesso: Arc<dyn RepositorioAcessoUsuarios>,
}

impl VerificarAcessoUseCase {
    pub fn novo(acesso: Arc<dyn RepositorioAcessoUsuarios>) -> Self {
        Self { acesso }
    }

    pub fn executar(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        papel: Papel,
    ) -> Result<AcessoCarteira, ClienteError> {
        let funcao = self
            .acesso
            .funcao_efetiva(escritorio_id, usuario_id, papel)?;
        let vinculos = self.acesso.carteira_do_usuario(escritorio_id, usuario_id)?;
        let clientes_permitidos = if vinculos.is_empty() {
            None
        } else {
            Some(vinculos)
        };
        Ok(AcessoCarteira {
            funcao,
            clientes_permitidos,
        })
    }
}

/// Gestão de sócios/contatos de um cliente.
#[derive(Clone)]
pub struct GestaoSociosContatosUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    carteira: Arc<dyn RepositorioCarteiraClientes>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl GestaoSociosContatosUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        carteira: Arc<dyn RepositorioCarteiraClientes>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            clientes,
            carteira,
            auditoria,
        }
    }

    fn cliente_do_escritorio(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<(), RepositorioError> {
        self.clientes
            .buscar_por_id(escritorio_id, cliente_id)?
            .ok_or(RepositorioError::NaoEncontrado)?;
        Ok(())
    }

    pub fn criar(
        &self,
        comando: &NovoSocioContatoCommand,
    ) -> Result<crate::domain::clientes::SocioContato, ClienteError> {
        self.cliente_do_escritorio(comando.escritorio_id, comando.cliente_id)?;
        let documento = match comando.documento.as_deref().map(str::trim) {
            Some("") | None => None,
            Some(digitos) => Some(CpfCnpj::novo(digitos)?),
        };
        let novo = NovoSocioContato {
            escritorio_id: comando.escritorio_id,
            cliente_id: comando.cliente_id,
            papel: comando.papel,
            nome: comando.nome.clone(),
            documento,
            email: comando.email.clone(),
            telefone: comando.telefone.clone(),
            representante_principal: comando.representante_principal,
        };
        novo.validar()?;
        let criado = self.carteira.criar_socio_contato(&novo)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "criar",
            entidade: "socio_contato",
            entidade_id: criado.id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({
                "papel": criado.papel.como_str(),
                "nome": criado.nome,
                "representante_principal": criado.representante_principal,
            })),
            ip: comando.ip,
        })?;
        Ok(criado)
    }

    pub fn listar(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<crate::domain::clientes::SocioContato>, ClienteError> {
        Ok(self.carteira.listar_socios_contatos(escritorio_id, cliente_id)?)
    }

    pub fn remover(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        id: Uuid,
        usuario_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<bool, ClienteError> {
        self.cliente_do_escritorio(escritorio_id, cliente_id)?;
        let removido = self
            .carteira
            .remover_socio_contato(escritorio_id, cliente_id, id)?;
        if removido {
            self.auditoria.registrar(&EventoAuditoriaCliente {
                escritorio_id,
                usuario_id,
                acao: "remover",
                entidade: "socio_contato",
                entidade_id: id,
                valor_anterior: None,
                valor_novo: None,
                ip,
            })?;
        }
        Ok(removido)
    }
}

/// Gestão de filiais/unidades de um cliente.
#[derive(Clone)]
pub struct GestaoFiliaisUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    carteira: Arc<dyn RepositorioCarteiraClientes>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl GestaoFiliaisUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        carteira: Arc<dyn RepositorioCarteiraClientes>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            clientes,
            carteira,
            auditoria,
        }
    }

    fn cliente_do_escritorio(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<(), RepositorioError> {
        self.clientes
            .buscar_por_id(escritorio_id, cliente_id)?
            .ok_or(RepositorioError::NaoEncontrado)?;
        Ok(())
    }

    pub fn criar(
        &self,
        comando: &NovoClienteFilialCommand,
    ) -> Result<crate::domain::clientes::ClienteFilial, ClienteError> {
        self.cliente_do_escritorio(comando.escritorio_id, comando.cliente_id)?;
        let cnpj = CpfCnpj::novo(comando.cnpj.trim())?;
        let novo = NovoClienteFilial {
            escritorio_id: comando.escritorio_id,
            cliente_id: comando.cliente_id,
            cnpj,
            nome_unidade: comando.nome_unidade.clone(),
            endereco: comando.endereco.clone(),
            telefone: comando.telefone.clone(),
        };
        novo.validar()?;
        let criado = self.carteira.criar_filial(&novo)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "criar",
            entidade: "cliente_filial",
            entidade_id: criado.id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({ "cnpj": criado.cnpj.como_str() })),
            ip: comando.ip,
        })?;
        Ok(criado)
    }

    pub fn listar(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<crate::domain::clientes::ClienteFilial>, ClienteError> {
        Ok(self.carteira.listar_filiais(escritorio_id, cliente_id)?)
    }

    pub fn remover(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        id: Uuid,
        usuario_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<bool, ClienteError> {
        self.cliente_do_escritorio(escritorio_id, cliente_id)?;
        let removido = self.carteira.remover_filial(escritorio_id, cliente_id, id)?;
        if removido {
            self.auditoria.registrar(&EventoAuditoriaCliente {
                escritorio_id,
                usuario_id,
                acao: "remover",
                entidade: "cliente_filial",
                entidade_id: id,
                valor_anterior: None,
                valor_novo: None,
                ip,
            })?;
        }
        Ok(removido)
    }
}

/// Gestão de tags do escritório e seus vínculos com clientes.
#[derive(Clone)]
pub struct GestaoClienteTagsUseCase {
    carteira: Arc<dyn RepositorioCarteiraClientes>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl GestaoClienteTagsUseCase {
    pub fn novo(
        carteira: Arc<dyn RepositorioCarteiraClientes>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            carteira,
            auditoria,
        }
    }

    pub fn criar_tag(
        &self,
        comando: &NovaClienteTagCommand,
    ) -> Result<ClienteTag, ClienteError> {
        let nova = NovaClienteTag {
            escritorio_id: comando.escritorio_id,
            nome: comando.nome.clone(),
            cor: comando.cor.clone(),
        };
        nova.validar()?;
        let criada = self.carteira.criar_tag(&nova)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id: comando.escritorio_id,
            usuario_id: comando.usuario_id,
            acao: "criar",
            entidade: "cliente_tag",
            entidade_id: criada.id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({ "nome": criada.nome })),
            ip: comando.ip,
        })?;
        Ok(criada)
    }

    pub fn listar_tags(&self, escritorio_id: Uuid) -> Result<Vec<ClienteTag>, ClienteError> {
        Ok(self.carteira.listar_tags(escritorio_id)?)
    }

    pub fn tags_do_cliente(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<ClienteTag>, ClienteError> {
        Ok(self.carteira.tags_do_cliente(escritorio_id, cliente_id)?)
    }

    pub fn vincular(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        tag_id: Uuid,
        usuario_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<(), ClienteError> {
        let tags = self.carteira.tags_do_cliente(escritorio_id, cliente_id)?;
        if tags.iter().any(|t| t.id == tag_id) {
            return Ok(()); // idempotente
        }
        self.carteira.vincular_tag(escritorio_id, cliente_id, tag_id)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id,
            usuario_id,
            acao: "vincular",
            entidade: "cliente_tag_vinculo",
            entidade_id: tag_id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({ "cliente_id": cliente_id })),
            ip,
        })?;
        Ok(())
    }

    pub fn desvincular(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        tag_id: Uuid,
        usuario_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<bool, ClienteError> {
        let removido = self
            .carteira
            .desvincular_tag(escritorio_id, cliente_id, tag_id)?;
        if removido {
            self.auditoria.registrar(&EventoAuditoriaCliente {
                escritorio_id,
                usuario_id,
                acao: "desvincular",
                entidade: "cliente_tag_vinculo",
                entidade_id: tag_id,
                valor_anterior: Some(serde_json::json!({ "cliente_id": cliente_id })),
                valor_novo: None,
                ip,
            })?;
        }
        Ok(removido)
    }

    pub fn clientes_com_tag(
        &self,
        escritorio_id: Uuid,
        tag_id: Uuid,
    ) -> Result<Vec<Uuid>, ClienteError> {
        Ok(self.carteira.clientes_com_tag(escritorio_id, tag_id)?)
    }
}

/// Carteira por cliente: responsável interno e observações.
#[derive(Clone)]
pub struct CarteiraClienteUseCase {
    clientes: Arc<dyn ClienteRepositorio>,
    carteira: Arc<dyn RepositorioCarteiraClientes>,
    auditoria: Arc<dyn AuditoriaClienteRepositorio>,
}

impl CarteiraClienteUseCase {
    pub fn novo(
        clientes: Arc<dyn ClienteRepositorio>,
        carteira: Arc<dyn RepositorioCarteiraClientes>,
        auditoria: Arc<dyn AuditoriaClienteRepositorio>,
    ) -> Self {
        Self {
            clientes,
            carteira,
            auditoria,
        }
    }

    pub fn obter(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Option<crate::domain::clientes::CarteiraCliente>, ClienteError> {
        Ok(self
            .carteira
            .carteira_do_cliente(escritorio_id, cliente_id)?)
    }

    /// Define responsável e/ou observações (upsert da linha de carteira).
    #[allow(clippy::too_many_arguments)]
    pub fn definir(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        responsavel_usuario_id: Option<Uuid>,
        limpar_responsavel: bool,
        observacoes: Option<String>,
        limpar_observacoes: bool,
        usuario_id: Uuid,
        ip: Option<std::net::IpAddr>,
    ) -> Result<crate::domain::clientes::CarteiraCliente, ClienteError> {
        self.clientes
            .buscar_por_id(escritorio_id, cliente_id)?
            .ok_or(RepositorioError::NaoEncontrado)?;
        if let Some(observacoes) = observacoes.as_ref()
            && observacoes.chars().count() > 2000
        {
            return Err("observações devem ter no máximo 2000 caracteres".into());
        }
        let observacoes_alteradas = limpar_observacoes || observacoes.is_some();
        let atualizacao = AtualizarCarteiraCliente {
            responsavel_usuario_id,
            limpar_responsavel,
            observacoes,
            limpar_observacoes,
        };
        let carteira = self
            .carteira
            .definir_carteira(cliente_id, escritorio_id, &atualizacao)?;
        self.auditoria.registrar(&EventoAuditoriaCliente {
            escritorio_id,
            usuario_id,
            acao: "definir",
            entidade: "carteira_cliente",
            entidade_id: cliente_id,
            valor_anterior: None,
            valor_novo: Some(serde_json::json!({
                "responsavel_usuario_id": carteira.responsavel_usuario_id,
                "observacoes_alteradas": observacoes_alteradas,
            })),
            ip,
        })?;
        Ok(carteira)
    }
}

/// Gestão de função e carteira de usuários (exclusivo para gestores).
#[derive(Clone)]
pub struct GestaoAcessoUsuariosUseCase {
    acesso: Arc<dyn RepositorioAcessoUsuarios>,
}

impl GestaoAcessoUsuariosUseCase {
    pub fn novo(acesso: Arc<dyn RepositorioAcessoUsuarios>) -> Self {
        Self { acesso }
    }

    /// Detalhe de acesso de um usuário: função declarada (ou derivada) e vínculos.
    pub fn detalhe(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        papel: Papel,
    ) -> Result<AcessoCarteira, ClienteError> {
        let funcao = self
            .acesso
            .funcao_efetiva(escritorio_id, usuario_id, papel)?;
        let vinculos = self.acesso.carteira_do_usuario(escritorio_id, usuario_id)?;
        Ok(AcessoCarteira {
            funcao,
            clientes_permitidos: if vinculos.is_empty() {
                None
            } else {
                Some(vinculos)
            },
        })
    }

    pub fn definir_funcao(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        funcao: FuncaoUsuario,
    ) -> Result<(), ClienteError> {
        self.acesso.definir_funcao(escritorio_id, usuario_id, funcao)?;
        Ok(())
    }

    pub fn definir_carteira(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        ids: &[Uuid],
    ) -> Result<(), ClienteError> {
        self.acesso
            .definir_carteira_usuario(escritorio_id, usuario_id, ids)?;
        Ok(())
    }
}

/// Papel de sócio/contato reexportado para a camada de apresentação.
pub const PAPEL_SOCIO: PapelSocioContato = PapelSocioContato::Socio;