use std::fs;
use std::path::Path;

const OPERACOES_PROIBIDAS: &[&str] = &[
    "alter table",
    "add column",
    "drop column",
    "rename",
    "alter type",
    "alter index",
];

fn migrations_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

#[test]
fn nenhuma_migration_altera_tabela_existente() {
    let mut verificadas = 0;

    for entrada in fs::read_dir(migrations_dir()).expect("diretório de migrations deve existir") {
        let entrada = entrada.expect("entrada legível");
        if !entrada.file_type().expect("tipo de arquivo").is_dir() {
            continue;
        }

        for arquivo in ["up.sql", "down.sql"] {
            let caminho = entrada.path().join(arquivo);
            let conteudo = fs::read_to_string(&caminho).expect("migration legível");
            let minúsculo = conteudo.to_lowercase();

            for proibida in OPERACOES_PROIBIDAS {
                assert!(
                    !minúsculo.contains(proibida),
                    "{} contém operação proibida antes do MVP: {proibida}",
                    caminho.display()
                );
            }

            verificadas += 1;
        }
    }

    assert!(
        verificadas >= 8,
        "deve haver pelo menos quatro migrations completas"
    );
}

#[test]
fn nomes_de_migration_seguem_o_padrao_timestamp_e_snake_case() {
    for entrada in fs::read_dir(migrations_dir()).expect("diretório de migrations deve existir") {
        let entrada = entrada.expect("entrada legível");
        if !entrada.file_type().expect("tipo de arquivo").is_dir() {
            continue;
        }

        let nome = entrada.file_name().to_string_lossy().to_string();
        let (versao, descricao) = nome
            .split_once('_')
            .unwrap_or_else(|| panic!("migration sem timestamp: {nome}"));

        assert_eq!(versao.len(), 14, "timestamp deve ter 14 dígitos: {nome}");
        assert!(
            versao.bytes().all(|byte| byte.is_ascii_digit()),
            "timestamp deve ser numérico: {nome}"
        );
        assert!(!descricao.is_empty(), "descrição ausente: {nome}");
        assert!(
            descricao
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "descrição deve ser snake_case: {nome}"
        );
    }
}

#[test]
fn toda_migration_tem_up_e_down_nao_vazios() {
    for entrada in fs::read_dir(migrations_dir()).expect("diretório de migrations deve existir") {
        let entrada = entrada.expect("entrada legível");
        if !entrada.file_type().expect("tipo de arquivo").is_dir() {
            continue;
        }

        for arquivo in ["up.sql", "down.sql"] {
            let caminho = entrada.path().join(arquivo);
            let conteudo = fs::read_to_string(&caminho).expect("migration legível");
            assert!(
                !conteudo.trim().is_empty(),
                "{} não pode estar vazio",
                caminho.display()
            );
        }

        let down = fs::read_to_string(entrada.path().join("down.sql")).expect("down.sql legível");
        assert!(
            down.trim_start().to_lowercase().starts_with("drop table"),
            "down.sql deve apenas remover a tabela criada: {}",
            entrada.path().display()
        );
    }
}
