# ADR-0001 — Escolha da Stack Tecnológica

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Backend, frontends, banco de dados, proxy e orquestração
- **Relaciona-se com:** [ADR-0002](0002-escopo-consultivo-divida-ativa.md), [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md), [ADR-0005](0005-alternador-modo-trial-serpro.md), [ADR-0006](0006-migracao-automatica-banco-dados.md)

## Contexto

A Accurit é um SaaS B2B multi-tenant para escritórios de contabilidade. A plataforma precisa conciliar:

- isolamento rígido de dados por `escritorio_id`;
- integração segura com APIs fiscais da SERPRO (OAuth2, mTLS, certificado A1, assinatura XMLDSig e procurações e-CAC);
- processamento de payloads fiscais com precisão decimal e higienização de CPF/CNPJ;
- auditoria de consultas e ações administrativas;
- operação local e em produção com baixo atrito, incluindo migrações automáticas;
- três experiências públicas distintas: site institucional, painel administrativo e portal do cliente.

O produto também tem requisitos não funcionais relevantes: confiabilidade, rastreabilidade, segurança de segredos, desempenho previsível e facilidade de evolução por uma equipe pequena. A escolha da stack precisa equilibrar produtividade, segurança de tipos e custo operacional, sem introduzir complexidade desnecessária.

## Decisão

Adotar a seguinte stack como padrão para toda a plataforma:

| Camada | Tecnologia | Responsabilidade |
|---|---|---|
| Backend | Rust 2024, Axum 0.8, Tokio, Diesel 2.2+ | API REST, regras de negócio, integrações SERPRO, assinatura XML e auditoria |
| Banco de dados | PostgreSQL 16 (`postgres:16-alpine`) | Persistência relacional multi-tenant, restrições, índices e migrações |
| Site institucional (`/`) | Next.js (App Router, SSG/SSR) servido por `nginx:alpine` | SEO, conteúdo institucional e captação de leads |
| Painel administrativo (`/admin`) | Angular 21 (Signals, Standalone Components, SPA/CSR) | Gestão de clientes, configuração SERPRO e alternância Trial/Produção |
| Portal do cliente (`/app` e `/login`) | Angular 21 (Signals, Standalone Components, SPA/CSR) | Autenticação e experiência do cliente do escritório |
| Proxy reverso | Nginx (`nginx:alpine`) | TLS, roteamento, compressão, cache de estáticos e ponto único de entrada |
| Orquestração | Docker Compose | Build e execução integrada de todos os serviços |

### Decisões complementares

1. **Backend em Rust com Axum e Diesel.**
   - Axum fornece um modelo de composição de rotas e middlewares compatível com Tokio, com baixo overhead e integração natural com o ecossistema assíncrono.
   - Diesel é usado como ORM e query builder tipado, com validação de SQL em tempo de compilação e suporte a migrações embutidas.
   - O backend segue Clean Architecture, DDD e SOLID, com as camadas `domain`, `application`, `infrastructure` e `presentation`. O domínio não depende de Axum, Diesel ou detalhes de transporte.
2. **PostgreSQL 16 como única fonte de verdade.**
   - O modelo é relacional, com chaves estrangeiras, constraints de integridade, `UNIQUE` por tenant e índices compostos iniciados por `escritorio_id`.
   - Tipos como `UUID`, `TIMESTAMPTZ`, `NUMERIC`/`DECIMAL` e `JSONB` atendem ao domínio fiscal e à auditoria.
   - Não há banco NoSQL ou cache externo obrigatório no caminho crítico. Cache de token e de procuração é uma otimização, não a fonte de verdade.
3. **Angular 21 para Admin e Cliente.**
   - As duas aplicações são SPAs/CSR separadas, com Signals para estado de tela e `computed()` para valores derivados.
   - Componentes standalone, guards por papel e organização por feature reduzem acoplamento e permitem evolução independente do Admin e do portal do cliente.
   - A separação física evita que o bundle administrativo seja entregue a usuários do portal.
4. **Next.js para o site institucional (`web`).**
   - O App Router atende SEO, SSG/SSR e páginas institucionais.
   - A publicação ocorre atrás do proxy Nginx. O artefato estático pode ser servido por `nginx:alpine`; quando SSR for necessário, o runtime Node correspondente permanece interno, sem exposição direta ao host.
5. **Nginx como único ponto de entrada público.**
   - O proxy publica `/` (`web`), `/admin/`, `/app/`, `/login` e `/api/`.
   - Backend, PostgreSQL, Admin e Cliente não publicam portas diretamente em produção.
   - O proxy repassa `Authorization`, remove `x-cpf-usuario` enviado pelo cliente e valida o
     `X-Request-Id`; não há cache no caminho `/api/`.
6. **Docker Compose como ambiente de referência.**
   - O fluxo suportado é `docker compose up --build`, com healthchecks, rede interna e volume persistente do PostgreSQL.
   - O backend aplica migrações no startup, conforme a [ADR-0006](0006-migracao-automatica-banco-dados.md).

### Alternativas consideradas

| Alternativa | Motivo da não adoção |
|---|---|
| Backend em Node.js/NestJS | Menor segurança de tipos em regras fiscais e maior risco de erros em cálculos e contratos; o custo de Rust é compensado pela criticidade do domínio. |
| Backend em Go | Boa performance e simplicidade, mas o ecossistema de tipagem do domínio e de queries SQL não foi considerado superior ao conjunto Rust + Diesel para este projeto. |
| SPA única em Angular para Admin e Cliente | Mistura bundles e permissões; aumenta o risco de exposição de funcionalidades administrativas e dificulta a evolução independente. |
| React/Next.js também para Admin e Cliente | Reduziria a variedade de frameworks, mas o projeto optou por Angular 21 para os painéis por sua estrutura de aplicação, guards e organização por features. |
| Banco NoSQL | O domínio exige relações, constraints e consultas transacionais; um banco relacional é mais adequado à integridade fiscal e ao isolamento por tenant. |

## Consequências

### Positivas

- **Segurança de tipos no backend:** o compilador Rust e as queries tipadas do Diesel reduzem classes de erro em regras fiscais, contratos e isolamento de tenant.
- **Desempenho previsível:** Axum/Tokio oferecem baixo overhead para chamadas HTTP e para o volume esperado de consultas fiscais.
- **Integridade relacional:** PostgreSQL 16 fornece constraints, transações, índices e tipos adequados a valores monetários e auditoria.
- **SEO e performance no site institucional:** Next.js com SSG/SSR atende aos requisitos de SEO e performance do site.
- **Separação de responsabilidades:** Admin e Cliente evoluem como aplicações independentes, com limites de acesso claros.
- **Operação reproduzível:** Docker Compose, healthchecks e migrações embutidas tornam o ambiente local e o deploy previsíveis.
- **Evolução disciplinada:** a combinação de Clean Architecture, DDD e SOLID mantém regras de negócio isoladas de frameworks e de detalhes de infraestrutura.

### Negativas

- **Complexidade poliglota:** Rust, Angular/TypeScript, Next.js/React, SQL e Nginx exigem competências distintas e aumentam o custo de contratação e de manutenção.
- **Curva de aprendizado em Rust:** concorrência, ownership e lifetimes podem reduzir a velocidade inicial da equipe.
- **Tempo de compilação:** builds Rust são mais lentos que os de stacks interpretadas, especialmente em pipelines sem cache.
- **Acoplamento ao Diesel:** mudanças de schema exigem atualização de tipos e podem gerar ciclos de compilação mais longos.
- **Duas aplicações Angular:** há duplicação de infraestrutura de frontend, componentes compartilhados e pipelines de build.
- **Next.js em container:** o modo SSR adiciona um runtime Node e mais uma superfície operacional; o proxy precisa ser configurado corretamente para assets e rotas.
- **Imagens Docker maiores:** Rust e Node produzem imagens de build pesadas; é necessário multi-stage build e imagem final enxuta.

### Mitigações

- Fixar versões de toolchains, dependências e imagens Docker; atualizar de forma controlada.
- Usar multi-stage builds, cache de dependências e imagens finais mínimas.
- Manter uma biblioteca compartilhada de UI e contratos para os dois frontends Angular.
- Automatizar lint, formatação, testes, `cargo audit` e `npm audit` no CI.
- Documentar o fluxo de desenvolvimento e os limites entre camadas no backend.
- Monitorar latência, erros e saturação dos serviços desde a primeira fase.

## Conformidade e Regras de Negócio

1. **Versões de referência:** backend em Rust 2024, Axum 0.8, Diesel 2.2+, PostgreSQL 16; frontends em Angular 21; site institucional (`web`) em Next.js 16 com App Router. Alterações de versão principal exigem nova ADR.
2. **Separação de camadas:** o domínio não pode importar Axum, Diesel, `reqwest` ou tipos de transporte. Casos de uso dependem de traits/ports; a infraestrutura implementa esses ports.
3. **Multi-tenancy:** todo dado de negócio é isolado por `escritorio_id`. O tenant é derivado do contexto autenticado (JWT + middleware), nunca de um campo livre do corpo ou da query.
4. **Frontends sem acesso direto ao banco:** toda leitura e escrita passa pela API REST do backend. Nenhum frontend recebe credenciais de banco, certificados A1 ou tokens SERPRO.
5. **Admin e Cliente separados:** funcionalidades administrativas não podem ser incluídas no bundle do portal do cliente. Guards de rota e autorização no backend são obrigatórios.
6. **Proxy único:** em produção, apenas o Nginx publica portas. Backend, PostgreSQL, Admin e Cliente permanecem em rede interna.
7. **Configuração por ambiente:** conexão de banco, segredos JWT, chave de criptografia, credenciais SERPRO e modo Trial são fornecidos por variáveis de ambiente/secret manager, nunca versionados.
8. **Precisão financeira:** valores monetários usam `Decimal`/`NUMERIC` ou string no transporte; é proibido usar `f32`/`f64` para cálculos financeiros.
9. **Migrações:** o schema é versionado em SQL e aplicado automaticamente pelo backend, conforme a [ADR-0006](0006-migracao-automatica-banco-dados.md). O fluxo normal não exige `diesel_cli` no host.
10. **Escopo fiscal:** a plataforma não pode expor fluxos de negociação, parcelamento ou liquidação de dívida ativa, conforme a [ADR-0002](0002-escopo-consultivo-divida-ativa.md).

## Critérios de verificação

- `docker compose up --build` sobe PostgreSQL, backend, web, Admin, Cliente e proxy de forma integrada.
- O backend responde em `/api/v1/health` e aplica migrações no startup.
- `/`, `/admin/`, `/app/`, `/login` e `/api/` são roteados pelo Nginx.
- O Admin e o Cliente são aplicações Angular independentes, com Signals e componentes standalone.
- O domínio do backend não contém dependências de framework ou de banco.
- Nenhum segredo, token, certificado ou dado pessoal é versionado ou enviado ao frontend.

## Referências

- [README](../../README.md)
- [SPEC](../../SPEC.md)
- [ROADMAP](../../ROADMAP.md)
- [INSTRUCTIONS](../../INSTRUCTIONS.md)
- [ADR-0002 — Escopo consultivo da Dívida Ativa](0002-escopo-consultivo-divida-ativa.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
- [ADR-0006 — Migração automática do banco de dados](0006-migracao-automatica-banco-dados.md)
