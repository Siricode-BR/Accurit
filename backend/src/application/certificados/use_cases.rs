use std::str::FromStr;
use std::sync::Arc;

use crate::application::certificados::models::{
    PoliticaCertificados, montar_certificado, montar_novo_certificado, validar_conteudo,
    validar_titular,
};
use crate::application::certificados::ports::{
    AuditoriaCertificadoRepositorio, CertificadoCriptografia, CertificadoError, CertificadoPublico,
    CertificadoRepositorio, CriarCertificadoCommand, ListagemCertificados,
    ListarCertificadosCommand, ObterCertificadoCommand, RevogarCertificadoCommand,
};
#[allow(unused_imports)]
use crate::domain::certificados::{
    AlgoritmoCifragem, CertificadoInvalido, FingerprintSha256, TipoCertificado,
};
#[allow(unused_imports)]
use crate::domain::tenancy::EscritorioId;
#[allow(unused_imports)]
use async_trait::async_trait;
#[allow(unused_imports)]
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CriarCertificadoUseCase {
    repositorio: Arc<dyn CertificadoRepositorio>,
    auditoria: Arc<dyn AuditoriaCertificadoRepositorio>,
    criptografia: Arc<dyn CertificadoCriptografia>,
    politica: PoliticaCertificados,
}

impl CriarCertificadoUseCase {
    pub fn novo(
        repositorio: Arc<dyn CertificadoRepositorio>,
        auditoria: Arc<dyn AuditoriaCertificadoRepositorio>,
        criptografia: Arc<dyn CertificadoCriptografia>,
        politica: PoliticaCertificados,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
            criptografia,
            politica,
        }
    }

    pub async fn executar(
        &self,
        comando: CriarCertificadoCommand,
    ) -> Result<CertificadoPublico, CertificadoError> {
        validar_titular(&comando.titular_cpf_cnpj)?;
        validar_conteudo(&comando.conteudo)?;

        if comando.conteudo.len() > self.politica.max_conteudo_bytes {
            return Err(CertificadoError::Dominio(
                CertificadoInvalido::ConteudoVazio,
            ));
        }

        let fingerprint = FingerprintSha256::novo(&comando.fingerprint)?;

        if self
            .repositorio
            .buscar_por_fingerprint(comando.escritorio_id, fingerprint.como_str())?
            .is_some()
        {
            return Err(CertificadoError::Repositorio(
                crate::application::certificados::ports::RepositorioError::FingerprintDuplicado,
            ));
        }

        let algoritmo =
            AlgoritmoCifragem::from_str(&comando.algoritmo).unwrap_or(AlgoritmoCifragem::Aes256Gcm);

        let conteudo_cifrado = self
            .criptografia
            .cifrar(&comando.conteudo)
            .map_err(|_| CertificadoError::Criptografia)?;

        let senha_cifrada = if let Some(senha) = &comando.senha {
            Some(
                self.criptografia
                    .cifrar(senha.as_bytes())
                    .map_err(|_| CertificadoError::Criptografia)?,
            )
        } else {
            None
        };

        let novo = montar_novo_certificado(
            &comando,
            conteudo_cifrado.clone(),
            senha_cifrada.clone(),
            fingerprint,
            algoritmo,
        )?;

        let certificado = montar_certificado(
            Uuid::now_v7(),
            comando.escritorio_id,
            &novo,
            conteudo_cifrado,
            senha_cifrada,
        );

        self.repositorio
            .criar(comando.escritorio_id, &certificado)?;

        self.auditoria.registrar(
            &crate::application::certificados::ports::EventoAuditoriaCertificado {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                acao: "CERTIFICADO_CRIADO".to_string(),
                entidade: "certificados_a1".to_string(),
                entidade_id: certificado.id,
                valor_anterior: None,
                valor_novo: Some(serde_json::json!({
                    "tipo": certificado.tipo.como_str(),
                    "titular_cpf_cnpj": certificado.titular_cpf_cnpj,
                    "fingerprint": certificado.fingerprint.como_str(),
                    "valido_de": certificado.valido_de,
                    "valido_ate": certificado.valido_ate,
                })),
                ip: comando.ip,
            },
        )?;

        Ok((&certificado).into())
    }
}

#[derive(Clone)]
pub struct ListarCertificadosUseCase {
    repositorio: Arc<dyn CertificadoRepositorio>,
}

impl ListarCertificadosUseCase {
    pub fn novo(repositorio: Arc<dyn CertificadoRepositorio>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: ListarCertificadosCommand,
    ) -> Result<ListagemCertificados, CertificadoError> {
        let pagina = comando.pagina.max(1);
        let tamanho = comando.tamanho.clamp(1, 100);

        Ok(self.repositorio.listar(
            comando.escritorio_id,
            comando.apenas_validos,
            pagina,
            tamanho,
        )?)
    }
}

#[derive(Clone)]
pub struct ObterCertificadoUseCase {
    repositorio: Arc<dyn CertificadoRepositorio>,
}

impl ObterCertificadoUseCase {
    pub fn novo(repositorio: Arc<dyn CertificadoRepositorio>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: ObterCertificadoCommand,
    ) -> Result<CertificadoPublico, CertificadoError> {
        let certificado = self
            .repositorio
            .buscar_por_id(comando.escritorio_id, comando.id)?
            .ok_or(CertificadoError::Repositorio(
                crate::application::certificados::ports::RepositorioError::NaoEncontrado,
            ))?;

        Ok((&certificado).into())
    }
}

#[derive(Clone)]
pub struct RevogarCertificadoUseCase {
    repositorio: Arc<dyn CertificadoRepositorio>,
    auditoria: Arc<dyn AuditoriaCertificadoRepositorio>,
}

impl RevogarCertificadoUseCase {
    pub fn novo(
        repositorio: Arc<dyn CertificadoRepositorio>,
        auditoria: Arc<dyn AuditoriaCertificadoRepositorio>,
    ) -> Self {
        Self {
            repositorio,
            auditoria,
        }
    }

    pub async fn executar(
        &self,
        comando: RevogarCertificadoCommand,
    ) -> Result<(), CertificadoError> {
        let certificado = self
            .repositorio
            .buscar_por_id(comando.escritorio_id, comando.id)?
            .ok_or(CertificadoError::Repositorio(
                crate::application::certificados::ports::RepositorioError::NaoEncontrado,
            ))?;

        let valor_anterior = serde_json::to_value(&certificado).unwrap_or_default();

        let revogado = self
            .repositorio
            .revogar(comando.escritorio_id, comando.id)?;

        if !revogado {
            return Err(CertificadoError::Repositorio(
                crate::application::certificados::ports::RepositorioError::NaoEncontrado,
            ));
        }

        self.auditoria.registrar(
            &crate::application::certificados::ports::EventoAuditoriaCertificado {
                escritorio_id: comando.escritorio_id,
                usuario_id: comando.usuario_id,
                acao: "CERTIFICADO_REVOGADO".to_string(),
                entidade: "certificados_a1".to_string(),
                entidade_id: comando.id,
                valor_anterior: Some(valor_anterior),
                valor_novo: None,
                ip: comando.ip,
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::certificados::ports::RepositorioError;
    #[allow(unused_imports)]
    use crate::domain::certificados::FingerprintSha256;
    use std::sync::Arc;

    struct MockRepositorio;
    struct MockAuditoria;
    struct MockCriptografia;

    #[async_trait]
    impl CertificadoRepositorio for MockRepositorio {
        fn criar(
            &self,
            _: EscritorioId,
            _: &crate::domain::certificados::Certificado,
        ) -> Result<(), RepositorioError> {
            Ok(())
        }

        fn buscar_por_id(
            &self,
            _: EscritorioId,
            _: Uuid,
        ) -> Result<Option<crate::domain::certificados::Certificado>, RepositorioError> {
            Ok(None)
        }

        fn buscar_por_fingerprint(
            &self,
            _: EscritorioId,
            _: &str,
        ) -> Result<Option<crate::domain::certificados::Certificado>, RepositorioError> {
            Ok(None)
        }

        fn listar(
            &self,
            _: EscritorioId,
            _: bool,
            _: i64,
            _: i64,
        ) -> Result<ListagemCertificados, RepositorioError> {
            Ok(ListagemCertificados {
                itens: vec![],
                total: 0,
                pagina: 1,
                tamanho: 20,
            })
        }

        fn revogar(&self, _: EscritorioId, _: Uuid) -> Result<bool, RepositorioError> {
            Ok(true)
        }

        fn atualizar(
            &self,
            _: EscritorioId,
            _: &crate::domain::certificados::Certificado,
        ) -> Result<(), RepositorioError> {
            Ok(())
        }
    }

    #[async_trait]
    impl AuditoriaCertificadoRepositorio for MockAuditoria {
        fn registrar(
            &self,
            _: &crate::application::certificados::ports::EventoAuditoriaCertificado,
        ) -> Result<(), RepositorioError> {
            Ok(())
        }
    }

    impl CertificadoCriptografia for MockCriptografia {
        fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CertificadoError> {
            Ok(dados.to_vec())
        }

        fn decifrar(&self, dados_cifrados: &[u8]) -> Result<Vec<u8>, CertificadoError> {
            Ok(dados_cifrados.to_vec())
        }

        fn gerar_fingerprint(&self, _: &[u8]) -> String {
            "a".repeat(64)
        }
    }

    fn criar_use_case() -> CriarCertificadoUseCase {
        CriarCertificadoUseCase::novo(
            Arc::new(MockRepositorio),
            Arc::new(MockAuditoria),
            Arc::new(MockCriptografia),
            PoliticaCertificados::nova(),
        )
    }

    #[tokio::test]
    async fn cria_certificado_valido() {
        let uc = criar_use_case();
        let comando = CriarCertificadoCommand {
            escritorio_id: EscritorioId::de_uuid(Uuid::now_v7()),
            tipo: TipoCertificado::A1,
            titular_cpf_cnpj: "11.222.333/0001-81".to_string(),
            conteudo: vec![1, 2, 3, 4],
            senha: Some("senha123".to_string()),
            algoritmo: "AES-256-GCM".to_string(),
            fingerprint: "a".repeat(64),
            valido_de: Utc::now(),
            valido_ate: Utc::now() + chrono::Duration::days(365),
            ip: None,
            usuario_id: Uuid::now_v7(),
        };

        let resultado = uc.executar(comando).await;
        assert!(resultado.is_ok());
    }
}
