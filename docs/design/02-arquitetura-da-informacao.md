# Arquitetura da informação e permissões

> **Fase 0 — proposta para aprovação.** As rotas abaixo são caminhos públicos montados pelo Nginx; não implicam criação de novos endpoints de API.

## Canais e mapa de navegação

```text
/  Site institucional
├── /contato
├── /app/login       Plataforma do escritório
└── /admin/login     Acesso discreto do gestor SaaS

/app/                Plataforma do escritório (CLIENTE e OPERADOR)
├── inicio
├── clientes
│   ├── novo
│   └── :id
│       ├── Resumo
│       ├── Consultas fiscais
│       ├── Procurações
│       └── Histórico
├── consultas
│   ├── divida-ativa
│   └── integra-contador       Em breve; sem execução de serviço não habilitado
├── procuracoes
├── certificados
├── equipe                     CLIENTE somente
└── configuracoes/perfil

/admin/             Backoffice SaaS (ADMIN somente)
├── dashboard
├── escritorios
│   ├── novo
│   └── :id
│       ├── Resumo
│       ├── Ambiente SERPRO
│       ├── Credenciais
│       ├── Certificado
│       ├── Usuários
│       └── Auditoria
├── administradores
├── auditoria
└── saude
```

### Regras de navegação

- O Nginx monta o Angular Admin em `/admin/` e o Angular Client em `/app/`. Links, guardas e redirecionamentos precisam respeitar `baseHref`; não usar `/admin/login` como destino para funcionários de escritório.
- `/login` permanece como redirecionamento legado para `/app/login`.
- Ao renomear `/admin/tenants` para `/admin/escritorios`, `/app/divida-ativa` para `/app/consultas/divida-ativa` e `/app/integra-contador` para `/app/consultas/integra-contador`, manter redirecionamentos internos das rotas atuais para evitar links salvos quebrados.
- `/admin/` leva à Visão geral; a lista de escritórios fica em “Escritórios”. A rota legada `/dashboard`, se mantida, deve redirecionar à Visão geral até a migração.
- `/app/` leva a Início. A tela atual de Integra Contador não deve ser interpretada como autorização para ativar serviços adicionais; no desenho novo, o serviço aparece como “Em breve” até confirmação do catálogo/contrato.
- Menu e busca global só revelam dados acessíveis ao usuário atual. Busca do Admin é de tenants; busca do Client é restrita ao próprio escritório.
- Filtros e abas podem ser codificados na URL para permitir link direto e retorno previsível; nunca colocar segredo nem documento fiscal completo na query string.
- Desktop: sidebar colapsável + topbar. Mobile: navegação primária compacta e acesso aos demais destinos por drawer; manter áreas de toque de pelo menos 44 × 44 px.
- Breadcrumbs mostram a hierarquia atual sem substituir o título da página.
- `Ctrl+K`/`⌘K` abre a command palette. Ela oferece apenas rotas/ações permitidas e não executa operação mutável sem confirmar.

## Matriz de permissões

Legenda: **L** ler/consultar; **C** criar; **E** editar; **R** revogar/excluir; **—** sem acesso. A permissão efetiva é sempre decidida no backend.

| Área/ação | ADMIN | CLIENTE (titular) | OPERADOR | Observação |
|---|---:|---:|---:|---|
| Site público e contato | L | L | L | Sem sessão ou dado de tenant |
| Login de backoffice | C | — | — | Somente audiência `admin`/papel `ADMIN` |
| Login da plataforma | — | C | C | Audiência `client`; escritório é informado |
| Início do escritório e selo de ambiente | — | L | L | Ambiente vindo do backend |
| Clientes atendidos: listar e abrir ficha | — | L | L | Sempre restrito ao `escritorio_id` da sessão |
| Clientes atendidos: criar | — | C* | C | API permite CLIENTE/OPERADOR; UI atual restringe a rota ao OPERADOR — resolver na implementação sem alterar contrato silenciosamente |
| Clientes atendidos: editar/excluir | — | —* | E/R | Handlers atuais exigem OPERADOR; mudança de papel requer decisão de produto/API |
| Consultas Dívida Ativa | — | L | L | Consulta somente leitura |
| Integra Contador | — | L* | L* | Somente serviços confirmados; estado “em breve” para capacidade não habilitada |
| Procurações | — | L* | L* | Nenhuma resposta bruta pode conceder autorização na UI |
| Certificados A1: listar/obter/criar | — | L/C | L/C | API existente; arquivo e senha nunca retornam ao front |
| Certificados A1: revogar | — | — | R | Handler atual exige OPERADOR |
| Equipe do escritório | — | L/C/E/R* | — | Acesso visual exclusivo do titular; endpoints de equipe não encontrados na API atual |
| Configurações/perfil próprio | — | L/E* | L/E* | Proposta de UX; confirmar endpoints e campos disponíveis antes de implementar |
| Provisionar escritório | C | — | — | Backoffice; endpoint de criação existente |
| Alterar Trial/Produção | C* | — | — | Apenas ADMIN, com pré-voo e confirmação; produção permanece bloqueada sem condições |
| Administradores SaaS | C/E/R* | — | — | Endpoints de gestão não encontrados na API atual |
| Auditoria global / saúde da plataforma | L | — | — | Rotas de auditoria ainda não encontradas; `/health` existente é diagnóstico técnico |

`*` Proposta sujeita à disponibilidade da API e à decisão de produto. O CLIENTE é o titular do tenant; o desenho não concede permissões que a API atual não oferece. Antes das telas de equipe, perfil ou auditoria, definir contratos e autorização no backend. Até lá, manter item ausente ou explicitamente indisponível, nunca simular sucesso.

## Prioridade e relação com a UI existente

| Destino | Estado no repositório | Direção de IA |
|---|---|---|
| `/app/clientes` e cadastro | Implementados; filtros são select e ficha usa formulário compartilhado | Evoluir para chips sincronizados e ficha de leitura com abas |
| `/app/consultas/divida-ativa` | Consulta e detalhe implementados | Preservar aviso consultivo, resultado Trial e distinção de erros |
| `/app/integra-contador` | Tela autenticada existe | Migrar para “Consultas fiscais”; manter serviços pendentes indisponíveis |
| `/app/inicio` | Saúde, selo e mensagem de base | Evoluir para central de pendências sem inventar métricas |
| `/admin/tenants` | Lista, provisionamento e toggle existem | Apresentar em “Escritórios”; adicionar detalhe e pré-voo |
| `/admin/dashboard` | Componente de dashboard existe, rota não registrada | Definir como Visão geral e corrigir a rota órfã |
| `/app/certificados` | API existente, feature Angular ausente | Implementar após adaptador de contrato documentado |
| `/app/equipe`, `/app/procuracoes`, auditoria e detalhe do escritório | Não há feature completa/endpoints suficientes | Exigir desenho de contrato antes de ativar |

## Critério de “a até dois cliques”

- A partir de qualquer tela autenticada, qualquer destino do menu principal deve exigir no máximo dois cliques.
- As ações contextuais da ficha de cliente (“Consultar Dívida Ativa”, “Verificar procuração”) abrem a consulta já vinculada ao cliente, sem repetir digitação, se o endpoint suportar o vínculo.
- A command palette acelera navegação, mas não contorna permissões, confirmação ou controles do backend.
