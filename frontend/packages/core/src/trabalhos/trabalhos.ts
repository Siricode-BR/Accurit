//! DTOs de trabalhos agendados, execuções e consultas em lote (FASE B).
//! Espelham o backend `presentation/trabalhos/dto.rs`.

export type TipoTrabalho =
  | 'consulta_divida_ativa'
  | 'validar_procuracao'
  | 'consulta_integra_contador';

export type StatusExecucao =
  | 'pendente'
  | 'executando'
  | 'concluido'
  | 'falhou'
  | 'bloqueado';

export type StatusLote =
  | 'pendente'
  | 'executando'
  | 'concluido'
  | 'falhou'
  | 'parcial'
  | 'bloqueado';

/** Resposta de um trabalho agendado (GET/POST /trabalhos). */
export interface TrabalhoAgendadoDto {
  id: string;
  escritorioId: string;
  nome: string;
  tipo: TipoTrabalho;
  produto: string; // 'INTEGRA_CONTADOR' | 'DIVIDA_ATIVA'
  ambiente: string; // 'TRIAL' | 'PRODUCAO'
  cronExpression: string;
  habilitado: boolean;
  proximaExecucao: string | null;
  ultimaExecucao: string | null;
  criadoEm: string;
  atualizadoEm: string;
}

/** Requisição para criar um trabalho agendado. */
export interface CriarTrabalhoRequest {
  nome: string;
  tipo: TipoTrabalho;
  produto: string;
  ambiente: string;
  cronExpression: string;
}

/** Requisição para habilitar/desabilitar um trabalho. */
export interface AlternarTrabalhoRequest {
  habilitado: boolean;
}

/** Resposta de uma execução de trabalho (GET /execucoes, /execucoes/trabalho/{id}). */
export interface ExecucaoTrabalhoDto {
  id: string;
  trabalhoAgendadoId: string;
  escritorioId: string;
  status: StatusExecucao;
  iniciadoEm: string;
  finalizadoEm: string | null;
  erroMensagem: string | null;
  registrosProcessados: number;
  registrosComErro: number;
  tentativas: number;
  criadoEm: string;
}

/** Resposta de uma consulta em lote (GET/POST /consultas-lote). */
export interface ConsultaLoteDto {
  id: string;
  escritorioId: string;
  nome: string;
  produto: string;
  servico: string;
  verbo: string | null;
  status: StatusLote;
  totalItens: number;
  itensConcluidos: number;
  itensComErro: number;
  criadoEm: string;
  iniciadoEm: string | null;
  finalizadoEm: string | null;
}

/** Requisição para criar uma consulta em lote. */
export interface CriarConsultaLoteRequest {
  nome: string;
  produto: string;
  servico: string;
  verbo: string | null;
  documentos: Array<{ documento: string; tipo: string }>;
}

/** Resposta de um item de consulta em lote (GET /consultas-lote/{id}/itens). */
export interface ItemConsultaLoteDto {
  id: string;
  consultaLoteId: string;
  documento: string;
  tipo: string; // 'PF' | 'PJ'
  status: string; // 'pendente' | 'executando' | 'concluido' | 'falhou' | 'bloqueado'
  resultadoRedigido: unknown | null;
  erro: string | null;
  criadoEm: string;
  processadoEm: string | null;
  tentativas: number;
}

/** Opções de filtro para listagem de trabalhos. */
export interface FiltroTrabalhos {
  tipo?: TipoTrabalho;
  habilitado?: boolean;
}

/** Opções de filtro para listagem de lotes. */
export interface FiltroConsultasLote {
  produto?: string;
  servico?: string;
  status?: StatusLote;
}
