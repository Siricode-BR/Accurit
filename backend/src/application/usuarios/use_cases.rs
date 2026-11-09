use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::application::admin::{
    ProvisionarError, validar_cpf, validar_email, validar_nome, validar_senha,
};
use crate::application::usuarios::ports::{
    AlteracaoUsuario, Ator, EventoAuditoriaUsuario, FiltroUsuarios, ListagemUsuarios,
    OrigemAtor, RepositorioError, UsuarioDetalhe, UsuarioError, UsuarioGestaoNovo,
    UsuarioGestaoRepositorio,
};
use crate::domain::auth::Papel;
use crate::infrastructure::auth::HashArgon2;

#[derive(Debug, Clone)]
pub struct CriarUsuarioCommand {
    /// Só usado pelo backoffice; no portal o escritório vem do token.
    pub escritorio_id: Option<Uuid>,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub papel: Papel,
    pub senha: String,
}

#[derive(Debug, Clone, Default)]
pub struct AtualizarUsuarioCommand {
    pub nome: Option<String>,
    pub papel: Option<Papel>,
    pub ativo: Option<bool>,
}

/// Gestão de usuários (`CLIENTE`/`OPERADOR`) — portal do titular e backoffice SaaS.
#[derive(Clone)]
pub struct GestaoUsuariosUseCase {
    repo: Arc<dyn UsuarioGestaoRepositorio>,
    argon2: Arc<HashArgon2>,
}

fn validacao(resultado: Result<(), ProvisionarError>) -> Result<(), UsuarioError> {
    resultado.map_err(|erro| match erro {
        ProvisionarError::Validacao(msg) => UsuarioError::Validacao(msg),
        ProvisionarError::Repositorio(e) => UsuarioError::Repositorio(match e {
            crate::application::admin::RepositorioError::Indisponivel => {
                RepositorioError::Indisponivel
            }
            _ => RepositorioError::Dados,
        }),
        ProvisionarError::Senha => UsuarioError::Senha,
    })
}

fn papel_gerenciavel(papel: Papel) -> Result<(), UsuarioError> {
    if matches!(papel, Papel::Cliente | Papel::Operador) {
        Ok(())
    } else {
        Err(UsuarioError::Validacao(
            "papel deve ser CLIENTE ou OPERADOR".into(),
        ))
    }
}

fn snapshot(usuario: &UsuarioDetalhe) -> serde_json::Value {
    json!({
        "nome": usuario.nome,
        "email": usuario.email,
        "papel": usuario.papel.como_str(),
        "ativo": usuario.ativo,
    })
}

impl GestaoUsuariosUseCase {
    pub fn novo(repo: Arc<dyn UsuarioGestaoRepositorio>, argon2: Arc<HashArgon2>) -> Self {
        Self { repo, argon2 }
    }

    pub fn listar(
        &self,
        ator: &Ator,
        mut filtro: FiltroUsuarios,
    ) -> Result<Vec<UsuarioDetalhe>, UsuarioError> {
        if let Some(escopo) = ator.escopo() {
            filtro.escritorio_id = Some(escopo);
        }
        filtro.busca = filtro
            .busca
            .map(|b| b.trim().to_owned())
            .filter(|b| !b.is_empty());
        Ok(self.repo.listar(&filtro)?)
    }

    /// Fase E — listagem paginada no servidor (página/tamanho normalizados).
    pub fn listar_paginado(
        &self,
        ator: &Ator,
        mut filtro: FiltroUsuarios,
    ) -> Result<ListagemUsuarios, UsuarioError> {
        if let Some(escopo) = ator.escopo() {
            filtro.escritorio_id = Some(escopo);
        }
        filtro.busca = filtro
            .busca
            .map(|b| b.trim().to_owned())
            .filter(|b| !b.is_empty());

        const TAMANHO_PADRAO: i64 = 20;
        const TAMANHO_MAXIMO: i64 = 100;
        let tamanho = filtro
            .tamanho
            .unwrap_or(TAMANHO_PADRAO)
            .clamp(1, TAMANHO_MAXIMO);
        let pagina = filtro.pagina.unwrap_or(1).max(1);
        filtro.pagina = Some(pagina);
        filtro.tamanho = Some(tamanho);

        Ok(self.repo.listar_paginado(&filtro)?)
    }

    pub fn obter(&self, ator: &Ator, id: Uuid) -> Result<UsuarioDetalhe, UsuarioError> {
        self.repo
            .obter(id, ator.escopo())?
            .ok_or(UsuarioError::NaoEncontrado)
    }

    pub fn criar(
        &self,
        ator: &Ator,
        comando: CriarUsuarioCommand,
    ) -> Result<UsuarioDetalhe, UsuarioError> {
        let escritorio_id = match ator.origem {
            OrigemAtor::Escritorio(id) => id,
            OrigemAtor::Backoffice => comando
                .escritorio_id
                .ok_or_else(|| UsuarioError::Validacao("escritorioId é obrigatório".into()))?,
        };
        papel_gerenciavel(comando.papel)?;
        validacao(validar_nome(&comando.nome))?;
        validacao(validar_email(&comando.email))?;
        validacao(validar_cpf(&comando.cpf))?;
        validacao(validar_senha(&comando.senha))?;

        let senha_hash = self
            .argon2
            .gerar_hash(&comando.senha)
            .map_err(|_| UsuarioError::Senha)?;

        let id = self
            .repo
            .criar(&UsuarioGestaoNovo {
                escritorio_id,
                nome: comando.nome.trim().to_owned(),
                email: comando.email.trim().to_ascii_lowercase(),
                cpf: comando.cpf.chars().filter(char::is_ascii_digit).collect(),
                senha_hash,
                papel: comando.papel,
            })
            .map_err(|erro| match erro {
                RepositorioError::EscritorioInexistente => UsuarioError::EscritorioNaoEncontrado,
                outro => UsuarioError::Repositorio(outro),
            })?;

        let criado = self
            .repo
            .obter(id, Some(escritorio_id))?
            .ok_or(UsuarioError::NaoEncontrado)?;
        self.repo.registrar_auditoria(&EventoAuditoriaUsuario {
            escritorio_id,
            ator: ator.clone(),
            acao: "USUARIO_CRIADO",
            usuario_alvo_id: id,
            valor_anterior: None,
            valor_novo: Some(snapshot(&criado)),
        })?;
        Ok(criado)
    }

    pub fn atualizar(
        &self,
        ator: &Ator,
        id: Uuid,
        comando: AtualizarUsuarioCommand,
    ) -> Result<UsuarioDetalhe, UsuarioError> {
        if comando.nome.is_none() && comando.papel.is_none() && comando.ativo.is_none() {
            return Err(UsuarioError::Validacao(
                "nenhuma alteração informada".into(),
            ));
        }
        if let Some(nome) = &comando.nome {
            validacao(validar_nome(nome))?;
        }
        if let Some(papel) = comando.papel {
            papel_gerenciavel(papel)?;
        }

        let atual = self
            .repo
            .obter(id, ator.escopo())?
            .ok_or(UsuarioError::NaoEncontrado)?;

        let desativando = comando.ativo == Some(false) && atual.ativo;
        let reativando = comando.ativo == Some(true) && !atual.ativo;
        let deixando_de_ser_titular =
            comando.papel == Some(Papel::Operador) && atual.papel == Papel::Cliente;

        if desativando && ator.id == atual.id {
            return Err(UsuarioError::Conflito(
                "Você não pode desativar a sua própria conta".into(),
            ));
        }
        if atual.papel == Papel::Cliente
            && atual.ativo
            && (desativando || deixando_de_ser_titular)
            && self
                .repo
                .contar_titulares_ativos(atual.escritorio_id, atual.id)?
                == 0
        {
            return Err(UsuarioError::Conflito(
                "O escritório precisa manter ao menos um titular ativo".into(),
            ));
        }

        let alteracao = AlteracaoUsuario {
            nome: comando.nome.map(|n| n.trim().to_owned()),
            papel: comando.papel.filter(|p| *p != atual.papel),
            ativo: comando.ativo.filter(|a| *a != atual.ativo),
        };
        let papel_mudou = alteracao.papel.is_some();

        self.repo.atualizar(id, &alteracao)?;
        // Tokens carregam o papel e a sessão não deve sobreviver à desativação.
        if desativando || papel_mudou {
            self.repo.revogar_sessoes(id)?;
        }

        let novo = self
            .repo
            .obter(id, Some(atual.escritorio_id))?
            .ok_or(UsuarioError::NaoEncontrado)?;
        let acao = if desativando {
            "USUARIO_DESATIVADO"
        } else if reativando {
            "USUARIO_REATIVADO"
        } else {
            "USUARIO_ATUALIZADO"
        };
        self.repo.registrar_auditoria(&EventoAuditoriaUsuario {
            escritorio_id: atual.escritorio_id,
            ator: ator.clone(),
            acao,
            usuario_alvo_id: id,
            valor_anterior: Some(snapshot(&atual)),
            valor_novo: Some(snapshot(&novo)),
        })?;
        Ok(novo)
    }

    pub fn redefinir_senha(&self, ator: &Ator, id: Uuid, senha: &str) -> Result<(), UsuarioError> {
        validacao(validar_senha(senha))?;
        let alvo = self
            .repo
            .obter(id, ator.escopo())?
            .ok_or(UsuarioError::NaoEncontrado)?;
        let senha_hash = self
            .argon2
            .gerar_hash(senha)
            .map_err(|_| UsuarioError::Senha)?;

        self.repo.definir_senha(id, &senha_hash)?;
        self.repo.revogar_sessoes(id)?;
        // A senha nunca entra na auditoria.
        self.repo.registrar_auditoria(&EventoAuditoriaUsuario {
            escritorio_id: alvo.escritorio_id,
            ator: ator.clone(),
            acao: "USUARIO_SENHA_REDEFINIDA",
            usuario_alvo_id: id,
            valor_anterior: None,
            valor_novo: None,
        })?;
        Ok(())
    }
}
