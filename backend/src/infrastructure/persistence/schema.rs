// @generated automatically by Diesel CLI.

diesel::table! {
    administradores (id) {
        id -> Uuid,
        email -> Text,
        #[max_length = 11]
        cpf -> Varchar,
        senha_hash -> Text,
        ativo -> Bool,
        ultimo_login_em -> Nullable<Timestamptz>,
        falhas_consecutivas -> Int4,
        bloqueado_ate -> Nullable<Timestamptz>,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    auditoria_eventos (id) {
        id -> Uuid,
        escritorio_id -> Nullable<Uuid>,
        usuario_id -> Nullable<Uuid>,
        administrador_id -> Nullable<Uuid>,
        #[max_length = 64]
        acao -> Varchar,
        #[max_length = 64]
        entidade -> Nullable<Varchar>,
        entidade_id -> Nullable<Uuid>,
        valor_anterior -> Nullable<Jsonb>,
        valor_novo -> Nullable<Jsonb>,
        ip -> Nullable<Inet>,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    certificados_a1 (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        #[max_length = 14]
        titular_cpf_cnpj -> Varchar,
        conteudo_cifrado -> Bytea,
        senha_cifrada -> Nullable<Bytea>,
        #[max_length = 32]
        algoritmo -> Varchar,
        #[max_length = 64]
        fingerprint_sha256 -> Bpchar,
        valido_de -> Timestamptz,
        valido_ate -> Timestamptz,
        ativo -> Bool,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    clientes (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        #[max_length = 2]
        tipo -> Varchar,
        #[max_length = 32]
        porte_regime -> Nullable<Varchar>,
        #[max_length = 14]
        cpf_cnpj -> Varchar,
        nome_razao_social -> Text,
        nome_fantasia -> Nullable<Text>,
        email -> Nullable<Text>,
        telefone -> Nullable<Text>,
        criado_em -> Timestamptz,
        atualizado_em -> Timestamptz,
    }
}

diesel::table! {
    consultas_fiscais (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        cliente_id -> Nullable<Uuid>,
        #[max_length = 32]
        produto -> Varchar,
        #[max_length = 64]
        servico -> Varchar,
        #[max_length = 16]
        verbo -> Nullable<Varchar>,
        #[max_length = 16]
        ambiente -> Varchar,
        #[max_length = 64]
        chave_consulta_hash -> Nullable<Bpchar>,
        #[max_length = 20]
        chave_consulta_mascarada -> Nullable<Varchar>,
        #[max_length = 32]
        fonte_consulta -> Nullable<Varchar>,
        status_code -> Nullable<Int4>,
        #[max_length = 32]
        estado -> Varchar,
        protocolo -> Nullable<Text>,
        request_redigido -> Nullable<Jsonb>,
        response_redigido -> Nullable<Jsonb>,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    credenciais_serpro (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        #[max_length = 32]
        produto -> Varchar,
        #[max_length = 16]
        ambiente -> Varchar,
        consumer_key_cifrado -> Bytea,
        consumer_secret_cifrado -> Bytea,
        criado_em -> Timestamptz,
        atualizado_em -> Timestamptz,
    }
}

diesel::table! {
    escritorios (id) {
        id -> Uuid,
        #[max_length = 40]
        slug -> Varchar,
        nome -> Text,
        #[max_length = 14]
        cnpj -> Nullable<Varchar>,
        modo_trial_serpro -> Bool,
        versao -> Int8,
        atualizado_em -> Timestamptz,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    procuracoes (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        #[max_length = 16]
        ambiente -> Varchar,
        #[max_length = 14]
        contribuinte_cpf_cnpj -> Varchar,
        #[max_length = 14]
        procurador_cpf_cnpj -> Varchar,
        #[max_length = 5]
        codigo_servico -> Varchar,
        #[max_length = 32]
        status -> Varchar,
        valido_de -> Nullable<Timestamptz>,
        valido_ate -> Nullable<Timestamptz>,
        verificado_em -> Timestamptz,
        #[max_length = 32]
        origem -> Varchar,
    }
}

diesel::table! {
    sessoes (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        usuario_id -> Uuid,
        #[max_length = 8]
        audiencia -> Varchar,
        familia_id -> Uuid,
        #[max_length = 64]
        refresh_hash -> Bpchar,
        criado_em -> Timestamptz,
        expira_em -> Timestamptz,
        revogada_em -> Nullable<Timestamptz>,
        ip -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    sessoes_admin (id) {
        id -> Uuid,
        administrador_id -> Uuid,
        familia_id -> Uuid,
        #[max_length = 64]
        refresh_hash -> Bpchar,
        criado_em -> Timestamptz,
        expira_em -> Timestamptz,
        revogada_em -> Nullable<Timestamptz>,
        ip -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    tokens_serpro (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        #[max_length = 32]
        produto -> Varchar,
        #[max_length = 16]
        ambiente -> Varchar,
        #[max_length = 32]
        tipo_token -> Varchar,
        #[max_length = 14]
        autor_pedido_dados -> Varchar,
        token_cifrado -> Bytea,
        escopo -> Nullable<Text>,
        expira_em -> Timestamptz,
        criado_em -> Timestamptz,
    }
}

diesel::table! {
    usuarios (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        email -> Text,
        #[max_length = 11]
        cpf -> Varchar,
        senha_hash -> Text,
        #[max_length = 32]
        papel -> Varchar,
        ativo -> Bool,
        ultimo_login_em -> Nullable<Timestamptz>,
        falhas_consecutivas -> Int4,
        bloqueado_ate -> Nullable<Timestamptz>,
        criado_em -> Timestamptz,
    }
}

diesel::joinable!(auditoria_eventos -> escritorios (escritorio_id));
diesel::joinable!(certificados_a1 -> escritorios (escritorio_id));
diesel::joinable!(clientes -> escritorios (escritorio_id));
diesel::joinable!(consultas_fiscais -> clientes (cliente_id));
diesel::joinable!(consultas_fiscais -> escritorios (escritorio_id));
diesel::joinable!(credenciais_serpro -> escritorios (escritorio_id));
diesel::joinable!(procuracoes -> escritorios (escritorio_id));
diesel::joinable!(sessoes -> escritorios (escritorio_id));
diesel::joinable!(sessoes -> usuarios (usuario_id));
diesel::joinable!(sessoes_admin -> administradores (administrador_id));
diesel::joinable!(tokens_serpro -> escritorios (escritorio_id));
diesel::joinable!(usuarios -> escritorios (escritorio_id));

diesel::allow_tables_to_appear_in_same_query!(
    administradores,
    auditoria_eventos,
    certificados_a1,
    clientes,
    consultas_fiscais,
    credenciais_serpro,
    escritorios,
    procuracoes,
    sessoes,
    sessoes_admin,
    tokens_serpro,
    usuarios,
);
