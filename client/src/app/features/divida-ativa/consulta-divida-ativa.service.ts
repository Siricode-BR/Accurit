import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export type AmbienteConsulta = 'TRIAL' | 'PRODUCAO';
export type EstadoConsulta = 'COM_PENDENCIAS' | 'SEM_PENDENCIAS';
export type FonteConsulta = 'CPF_CNPJ' | 'CODIGO_CONTRIBUINTE';

export interface ValorConsulta {
  valor: string;
  remarca: string;
}

export interface InscricaoResumo {
  numeroInscricao: string;
  numeroProcesso?: string | null;
  situacaoDescricao: string;
  nomeDevedor?: string | null;
  tipoDevedor: string;
  valorTotalConsolidado: ValorConsulta;
}

export interface ConsultaDevedor {
  ambiente: AmbienteConsulta;
  estado: EstadoConsulta;
  fonteConsulta: FonteConsulta;
  chaveMascarada: string;
  horarioConsulta: string;
  tentativas: number;
  aviso: string;
  inscricoes: InscricaoResumo[];
}

export interface ConsultaInscricao {
  ambiente: AmbienteConsulta;
  estado: EstadoConsulta;
  fonteConsulta: FonteConsulta | null;
  chaveMascarada: string;
  horarioConsulta: string;
  tentativas: number;
  aviso: string;
  numeroInscricao: string;
  processoAdministrativo?: string | null;
  situacao: string;
  dataInscricao?: string | null;
  orgaoOrigem?: string | null;
  codigoNaturezaReceita?: string | null;
  nomeReceita?: string | null;
  valorTotalInscrito?: ValorConsulta | null;
  valorTotalConsolidado?: ValorConsulta | null;
  valorRemanescente?: ValorConsulta | null;
  indicadorPrescricao?: boolean | null;
  nomeUnidade?: string | null;
}

@Injectable({ providedIn: 'root' })
export class ConsultaDividaAtivaService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/divida-ativa';

  consultarDevedor(documento: string): Observable<ConsultaDevedor> {
    const digitos = documento.replace(/\D/g, '');
    return this.http.get<ConsultaDevedor>(
      `${this.baseUrl}/devedores/${encodeURIComponent(digitos)}`,
    );
  }

  consultarInscricao(numero: string): Observable<ConsultaInscricao> {
    const digitos = numero.replace(/\D/g, '');
    return this.http.get<ConsultaInscricao>(
      `${this.baseUrl}/inscricoes/${encodeURIComponent(digitos)}`,
    );
  }
}

export function mensagemErroConsulta(error: unknown): string {
  if (!(error instanceof HttpErrorResponse)) {
    return 'Não foi possível concluir a consulta. Tente novamente.';
  }

  const code = error.error?.error?.code as string | undefined;
  switch (code) {
    case 'SERPRO_PROD_NOT_CONFIGURED':
      return 'A consulta em produção ainda não está habilitada para este serviço.';
    case 'SERPRO_TRIAL_NOT_CONFIGURED':
      return 'O ambiente de demonstração está temporariamente indisponível.';
    case 'SERPRO_UNAUTHORIZED':
    case 'SERPRO_PROXY_DENIED':
      return 'A integração não autorizou a consulta. Verifique a configuração do serviço.';
    case 'SERPRO_RATE_LIMITED':
    case 'LIMITE_REQUISICOES':
    case 'MUITAS_REQUISICOES':
      return 'Limite de consultas atingido. Aguarde um momento antes de tentar novamente.';
    case 'SERPRO_NOT_FOUND':
      return 'A SERPRO não localizou a chave informada. Isso não confirma ausência de pendências.';
    case 'SERPRO_BAD_RESPONSE':
      return 'A integração recebeu uma resposta inválida. Tente novamente mais tarde.';
    case 'SERPRO_INDETERMINATE':
      return 'A SERPRO não confirmou o resultado. Confirme o status antes de tentar novamente.';
    case 'SERPRO_UNAVAILABLE':
      return 'O serviço da SERPRO está indisponível. Tente novamente mais tarde.';
  }

  if (error.status === 401 || error.status === 403) {
    return 'Sua sessão não está autorizada para consultar este serviço.';
  }
  if (error.status === 429) {
    return 'Limite de consultas atingido. Aguarde um momento antes de tentar novamente.';
  }
  if (error.status >= 500) {
    return 'Não foi possível comunicar com o serviço de consulta. Tente novamente mais tarde.';
  }

  const apiMessage = error.error?.error?.message;
  return typeof apiMessage === 'string' && apiMessage.length > 0
    ? apiMessage
    : 'Não foi possível concluir a consulta. Verifique os dados informados.';
}
