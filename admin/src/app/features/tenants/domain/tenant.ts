/* Camada de domínio da feature de tenants: tipos puros, sem dependências
 * de HTTP ou framework (SPEC §10.1 — contrato camelCase da API de admin). */

export type AmbienteSerpro = 'TRIAL' | 'PRODUCAO';

/** Resumo do escritório em GET /api/v1/admin/escritorios (paginado). */
export interface Tenant {
  id: string;
  slug: string;
  nome: string;
  modoTrialSerpro: boolean;
  /** Versão otimista para PATCH com concorrência. */
  versao: number;
  atualizadoEm: string;
}

/** Página efetiva devolvida pelo backend (page/pageSize já clampados). */
export interface Pagina<T> {
  itens: T[];
  page: number;
  pageSize: number;
  total: number;
}

/** Resposta do PATCH /admin/escritorios/{id}/serpro/trial. */
export interface TenantAlterado {
  modoTrialSerpro: boolean;
  ambienteEfetivo: AmbienteSerpro;
  versaoAtual: number;
  atualizadoEm: string;
}

export interface CriarTenantRequest {
  slug: string;
  nome: string;
  emailTitular: string;
  cpfTitular: string;
  senhaTitular: string;
}

export interface CriarTenantResponse {
  escritorioId: string;
  slug: string;
  titularId: string;
  emailTitular: string;
}

/** Classificação do fim da troca de modo (sem acoplar a HTTP). */
export type ResultadoTroca =
  | { tipo: 'ok' }
  | { tipo: 'conflito' }
  | { tipo: 'bloqueado'; mensagem: string }
  | { tipo: 'naoEncontrado' }
  | { tipo: 'indisponivel' };
