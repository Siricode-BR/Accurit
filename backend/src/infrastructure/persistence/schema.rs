// @generated automatically by Diesel CLI.

diesel::table! {
    auditoria_eventos (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        usuario_id -> Nullable<Uuid>,
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
    usuarios (id) {
        id -> Uuid,
        escritorio_id -> Uuid,
        email -> Text,
        #[max_length = 11]
        cpf -> Nullable<Varchar>,
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
diesel::joinable!(sessoes -> escritorios (escritorio_id));
diesel::joinable!(sessoes -> usuarios (usuario_id));
diesel::joinable!(usuarios -> escritorios (escritorio_id));

diesel::allow_tables_to_appear_in_same_query!(auditoria_eventos, escritorios, sessoes, usuarios,);
