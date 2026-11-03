import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export type AmbienteIntegraContador = 'TRIAL' | 'PRODUCAO';
export type VerboIntegraContador = 'Apoiar' | 'Consultar' | 'Declarar' | 'Emitir' | 'Monitorar';
export type CategoriaErroIntegraContador = 'autorizacao' | 'transporte' | 'negocio';

export interface ServicoIntegraContador {
  idSistema: string;
  idServico: string;
  versaoSistema: string | null;
  verbo: VerboIntegraContador;
  habilitado: boolean;
  requerConfirmacao: boolean;
}

export interface StatusIntegraContador {
  ambienteEfetivo: AmbienteIntegraContador;
  producaoHabilitada: boolean;
  assinaturaHabilitada: boolean;
  motivoBloqueioProducao: string;
  motivoBloqueioAssinatura: string;
  servicos: ServicoIntegraContador[];
}

export interface ResultadoIntegraContador {
  ambiente: AmbienteIntegraContador;
  verbo: VerboIntegraContador;
  idSistema: string;
  idServico: string;
  estado: string;
  statusSerpro: number;
  mensagem: string;
  referenciaResposta: string | null;
  tempoEsperaSegundos: number | null;
  etag: string | null;
  expires: string | null;
  codigosMensagem: string[];
}

export interface PedidoDadosIntegraContador {
  idSistema: string;
  idServico: string;
  versaoSistema: string;
  dados: string;
}

export interface RequisicaoExecucaoIntegraContador {
  documentoContribuinte: string;
  verbo: VerboIntegraContador;
  pedidoDados: PedidoDadosIntegraContador;
  confirmacaoExplicita: boolean;
  chaveIdempotencia?: string;
}

@Injectable({ providedIn: 'root' })
export class IntegraContadorService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro/integra-contador';

  consultarStatus(): Observable<StatusIntegraContador> {
    return this.http.get<StatusIntegraContador>(`${this.baseUrl}/status`);
  }

  consultarProcuracoes(documentoContribuinte: string): Observable<ResultadoIntegraContador> {
    return this.http.post<ResultadoIntegraContador>(`${this.baseUrl}/procuracoes/consultar`, {
      documentoContribuinte,
    });
  }

  executarOperacao(
    requisicao: RequisicaoExecucaoIntegraContador,
  ): Observable<ResultadoIntegraContador> {
    return this.http.post<ResultadoIntegraContador>(`${this.baseUrl}/executar`, requisicao);
  }
}

export function categoriaErroIntegraContador(error: unknown): CategoriaErroIntegraContador {
  if (!(error instanceof HttpErrorResponse)) {
    return 'transporte';
  }

  const code = error.error?.error?.code as string | undefined;
  if (
    code === 'ACESSO_NEGADO' ||
    code === 'SERPRO_NAO_AUTORIZADO' ||
    error.status === 401
  ) {
    return 'autorizacao';
  }
  if (error.status === 0 || error.status >= 500) {
    return 'transporte';
  }
  return 'negocio';
}

export function mensagemErroIntegraContador(error: unknown): string {
  if (!(error instanceof HttpErrorResponse)) {
    return 'Não foi possível concluir a operação.';
  }

  const code = error.error?.error?.code as string | undefined;
  switch (code) {
    case 'ACESSO_NEGADO':
    case 'SERPRO_NAO_AUTORIZADO':
      return 'A autorização não foi confirmada. Verifique a procuração e tente novamente.';
    case 'ESTADO_INDETERMINADO':
      return 'A SERPRO não confirmou o resultado. Consulte o estado antes de qualquer repetição.';
    case 'PRODUCAO_BLOQUEADA':
      return 'A produção permanece bloqueada até a confirmação contratual e aprovação D5.';
    case 'SERVICO_NAO_HABILITADO':
      return 'Este serviço ainda não está habilitado no catálogo confirmado.';
    case 'VERSAO_NAO_CONFIRMADA':
      return 'A versão deste serviço ainda não foi confirmada pela SERPRO.';
    case 'LIMITE_SERPRO':
      return 'Limite de chamadas atingido. Aguarde antes de consultar novamente.';
    case 'VALIDACAO':
      return 'A solicitação não corresponde ao contrato habilitado.';
  }

  if (error.status === 401 || error.status === 403) {
    return 'A autorização para esta operação foi negada.';
  }
  if (error.status === 429) {
    return 'Limite de chamadas atingido. Aguarde antes de consultar novamente.';
  }
  if (error.status === 0 || error.status >= 500) {
    return 'Falha de comunicação com o serviço. O estado pode estar indeterminado.';
  }

  const apiMessage = error.error?.error?.message;
  return typeof apiMessage === 'string' && apiMessage.length > 0
    ? apiMessage
    : 'A solicitação foi rejeitada.';
}
