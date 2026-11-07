/**
 * Histórico e situação fiscal de consultas por cliente (SPEC §9.3).
 *
 * Todos os campos são derivados do formato público redigido do backend
 * (`GET /clientes/{id}/consultas` e `GET /clientes/{id}/situacao-fiscal`) —
 * nunca do payload original da SERPRO.
 */

export type EstadoConsultaFiscal =
  | 'SUCESSO'
  | 'SEM_PENDENCIAS'
  | 'NAO_ENCONTRADO'
  | 'NAO_AUTORIZADO'
  | 'LIMITE_EXCEDIDO'
  | 'INDISPONIVEL'
  | 'RESPOSTA_INVALIDA'
  | 'INDETERMINADO'
  | 'ERRO_INTERNO';

export type FiltroEstadoConsulta = EstadoConsultaFiscal | '';

/** Uma tentativa registrada em `consultas_fiscais` (payload redigido). */
export interface ConsultaFiscalDto {
  id: string;
  produto: string;
  servico: string;
  verbo: string | null;
  ambiente: string;
  estado: EstadoConsultaFiscal;
  fonteConsulta: string | null;
  chaveConsultaMascarada: string | null;
  statusCode: number | null;
  criadoEm: string;
}

export interface ListagemConsultasDto {
  itens: ConsultaFiscalDto[];
  total: number;
  pagina: number;
  tamanho: number;
}

/** Última tentativa de cada serviço fiscal (cartões da tela de detalhes). */
export interface SituacaoFiscalServicoDto {
  servico: string;
  estado: EstadoConsultaFiscal;
  ambiente: string;
  fonteConsulta: string | null;
  chaveConsultaMascarada: string | null;
  statusCode: number | null;
  criadoEm: string;
}

export interface SituacaoFiscalDto {
  aviso: string;
  servicos: SituacaoFiscalServicoDto[];
}
