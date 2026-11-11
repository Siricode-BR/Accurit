#!/usr/bin/env node
/**
 * Guarda de regressão do design system (@accurit/ui).
 *
 * Falha quando uma tela de apps/client ou apps/admin usa elementos crus de
 * formulário/tabela/botão fora do pacote ui:
 *   - <input> (a única exceção é <input type="file"> dentro de acc-file-field);
 *   - <select>, <textarea>, <table>;
 *   - <button> sem atributo acc-* (acc-button/acc-cta-button/acc-copy-button).
 *
 * A lista LEGADO abaixo é uma allowlist explícita e justificada de telas que
 * ainda não foram migradas. Ela deve encolher a cada tela migrada e jamais
 * crescer.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const raiz = new URL('..', import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1');
const alvos = ['apps/client/src', 'apps/admin/src'].map((p) => join(raiz, p));

const LEGADO = new Set([
  // Clientes
  'app/features/clientes/cliente-list.component.html',
  'app/features/clientes/cliente-form.component.html',
  'app/features/clientes/cliente-editar.page.html',
  'app/features/clientes/cliente-detalhes.page.html',
  // Usuários
  'app/features/usuarios/usuario-list.component.html',
  'app/features/usuarios/usuario-form.component.ts',
  // Trabalhos / guias / relatórios / consumo / notificações
  'app/features/trabalhos/trabalho-criar.component.html',
  'app/features/trabalhos/trabalho-detalhes.component.html',
  'app/features/trabalhos/trabalhos-list.component.html',
  'app/features/trabalhos/consultas-lote-list.component.html',
  'app/features/guias/guia-emitir.component.html',
  'app/features/relatorios/relatorios-list.component.ts',
  'app/features/relatorios/relatorios-list.component.html',
  'app/features/consumo/consumo.page.ts',
  'app/features/notificacoes/notificacoes-list.component.html',
  // Admin
  'app/features/administradores/administradores-list.component.ts',
  'app/features/consumo/consumo-admin.page.ts',
  'app/features/tenants/tenant-list.component.ts',
  'app/features/tenants/tenant-detalhes.page.ts',
  'app/features/integra-contador/catalogo-page.component.ts',
  // Demais telas ainda não migradas (Etapa 4 incremental)
  'app/features/auth/login.component.html',
  'app/features/auth/login.component.ts',
  'app/features/divida-ativa/components/divida-ativa-consulta.component.html',
  'app/features/integra-contador/components/integra-contador-consulta.component.html',
  'app/features/guias/guias-list.component.html',
  'app/features/serpro/trial-mock-panel.component.ts',
  'app/features/usuarios/usuario-edicao.component.ts',
  'app/features/usuarios/usuario-senha.component.ts',
  'app/features/integra-contador/catalogo.page.ts',
  'app/features/tenants/tenant-list.component.html',
]);

const elementos = [
  { nome: 'input', regex: /<input\b(?![^>]*\btype\s*=\s*["']file["'])/gi },
  { nome: 'select', regex: /<select\b/gi },
  { nome: 'textarea', regex: /<textarea\b/gi },
  { nome: 'table', regex: /<table\b/gi },
  {
    nome: 'button sem acc-*',
    regex: /<button\b(?![^>]*\bacc-[a-z-]+)(?![^>]*\bacc[A-Z])/gi,
  },
];

function arquivos(diretorio) {
  const saida = [];
  for (const entrada of readdirSync(diretorio)) {
    const caminho = join(diretorio, entrada);
    const info = statSync(caminho);
    if (info.isDirectory()) {
      saida.push(...arquivos(caminho));
    } else if (/\.(html|ts)$/.test(entrada) && !entrada.endsWith('.spec.ts')) {
      saida.push(caminho);
    }
  }
  return saida;
}

const violacoes = [];
let ignorados = 0;

for (const alvo of alvos) {
  for (const arquivo of arquivos(alvo)) {
    const relativoApp = relative(alvo, arquivo).replaceAll('\\', '/');
    const conteudo = readFileSync(arquivo, 'utf8');
    const encontrados = [];
    for (const elemento of elementos) {
      const total = [...conteudo.matchAll(elemento.regex)].length;
      if (total > 0) {
        encontrados.push(`${elemento.nome} (${total})`);
      }
    }
    if (encontrados.length === 0) {
      continue;
    }
    if (LEGADO.has(relativoApp)) {
      ignorados += 1;
      continue;
    }
    violacoes.push(`${relative(raiz, arquivo)}: ${encontrados.join(', ')}`);
  }
}

if (violacoes.length > 0) {
  console.error('✖ Elementos crus fora do @accurit/ui encontrados:\n');
  for (const violacao of violacoes) {
    console.error(`  - ${violacao}`);
  }
  console.error(
    '\nUse acc-input/acc-select/acc-textarea/acc-checkbox/acc-switch/acc-alert/acc-table ' +
      'e acc-button (ou atualize a allowlist LEGADO apenas ao migrar uma tela).',
  );
  process.exit(1);
}

console.log(
  `✔ check:ui ok — ${ignorados} tela(s) legadas na allowlist, nenhuma regressão nova.`,
);
