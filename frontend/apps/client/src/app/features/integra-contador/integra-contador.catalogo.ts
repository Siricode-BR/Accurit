import { ServicoCatalogo, VerboIntegraContador, VerboOpcao } from './integra-contador.types';

/** Verbo selecionado por padrão ao abrir a consulta. */
export const VERBO_PADRAO: VerboIntegraContador = 'Consultar';

/**
 * Catálogo de referência do Integra Contador (espelha o backend
 * `domain/integra_contador/servicos.rs:catalogo_de_referencia`).
 */
export const CATALOGO: readonly ServicoCatalogo[] = [
  { idSistema: 'SICALC', idServico: 'CONSULTAAPOIORECEITAS52', versaoSistema: null, verbo: 'Apoiar' },
  { idSistema: 'PROCURACOES', idServico: 'OBTERPROCURACAO41', versaoSistema: '1', verbo: 'Consultar' },
  { idSistema: 'SITFIS', idServico: 'RELATORIOSITFIS92', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'PGDASD', idServico: 'CONSDECLARACAO13', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PGDASD', idServico: 'TRANSDECLARACAO11', versaoSistema: null, verbo: 'Declarar' },
  { idSistema: 'DCTFWEB', idServico: 'GERARGUIA31', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'CAIXAPOSTAL', idServico: 'INNOVAMSG63', versaoSistema: null, verbo: 'Monitorar' },
  { idSistema: 'AUTENTICAPROCURADOR', idServico: 'ENVIOXMLASSINADO81', versaoSistema: '1.0', verbo: 'Apoiar' },
];

export const VERBOS: readonly VerboOpcao[] = [
  { valor: 'Consultar', label: 'Consultar' },
  { valor: 'Declarar', label: 'Declarar' },
  { valor: 'Emitir', label: 'Emitir' },
  { valor: 'Apoiar', label: 'Apoiar' },
  { valor: 'Monitorar', label: 'Monitorar' },
];
