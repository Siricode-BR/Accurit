import { ServicoCatalogo, VerboIntegraContador, VerboOpcao } from './integra-contador.types';

/** Verbo selecionado por padrão ao abrir a consulta. */
export const VERBO_PADRAO: VerboIntegraContador = 'Consultar';

/**
 * Catálogo de referência do Integra Contador (espelha o backend
 * `domain/integra_contador/servicos.rs:catalogo_de_referencia` e a migração
 * `20261102140100_create_catalogo_servicos_integra/up.sql`).
 */
export const CATALOGO: readonly ServicoCatalogo[] = [
  { idSistema: 'PROCURACOES', idServico: 'OBTERPROCURACAO41', versaoSistema: '1', verbo: 'Consultar' },
  { idSistema: 'AUTENTICAPROCURADOR', idServico: 'ENVIOXMLASSINADO81', versaoSistema: '1.0', verbo: 'Apoiar' },
  { idSistema: 'SITFIS', idServico: 'RELATORIOSITFIS92', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'SITFIS', idServico: 'SOLICITARPROTOCOLO91', versaoSistema: null, verbo: 'Apoiar' },
  { idSistema: 'PGDASD', idServico: 'CONSDECLARACAO13', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PGDASD', idServico: 'GERARDAS12', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'CAIXAPOSTAL', idServico: 'INNOVAMSG63', versaoSistema: null, verbo: 'Monitorar' },
  { idSistema: 'CAIXAPOSTAL', idServico: 'MSGCONTRIBUINTE61', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'CAIXAPOSTAL', idServico: 'MSGDETALHAMENTO62', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PGMEI', idServico: 'GERARDASPDF21', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'PGMEI', idServico: 'GERARDASCODBARRA22', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'DCTFWEB', idServico: 'GERARGUIA31', versaoSistema: null, verbo: 'Emitir' },
  { idSistema: 'PARCSN', idServico: 'PEDIDOSPARC163', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PARCSN', idServico: 'OBTERPARC164', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PARCSN', idServico: 'PARCELASPARAGERAR162', versaoSistema: null, verbo: 'Consultar' },
  { idSistema: 'PARCSN', idServico: 'DETPAGTOPARC165', versaoSistema: null, verbo: 'Consultar' },
];

export const VERBOS: readonly VerboOpcao[] = [
  { valor: 'Consultar', label: 'Consultar' },
  { valor: 'Declarar', label: 'Declarar' },
  { valor: 'Emitir', label: 'Emitir' },
  { valor: 'Apoiar', label: 'Apoiar' },
  { valor: 'Monitorar', label: 'Monitorar' },
];
