import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { mensagemDeErro } from '../core/http/mensagem-erro';

export interface ConsultaDevedorResponse {
  ambiente: string;
  estado: string;
  fonteConsulta: string;
  chaveMascarada: string;
  horarioConsulta: string;
  tentativas: number;
  aviso: string;
  inscricoes: Array<{
    numeroInscricao: string;
    numeroProcesso?: string | null;
    situacaoDescricao: string;
    nomeDevedor?: string | null;
    tipoDevedor: string;
    valorTotalConsolidado: { valor: string; remarca: string };
  }>;
}

export interface ConsultaInscricaoResponse {
  ambiente: string;
  estado: string;
  fonteConsulta: string | null;
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
  valorTotalInscrito?: { valor: string; remarca: string } | null;
  valorTotalConsolidado?: { valor: string; remarca: string } | null;
  valorRemanescente?: { valor: string; remarca: string } | null;
  indicadorPrescricao?: boolean | null;
  nomeUnidade?: string | null;
}

@Injectable({ providedIn: 'root' })
export class DividaAtivaService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/divida-ativa';

  #carregando = signal(false);
  #erro = signal<string | null>(null);
  #ultimoDevedor = signal<ConsultaDevedorResponse | null>(null);
  #ultimaInscricao = signal<ConsultaInscricaoResponse | null>(null);

  readonly carregando = this.#carregando.asReadonly();
  readonly erro = this.#erro.asReadonly();
  readonly ultimoDevedor = this.#ultimoDevedor.asReadonly();
  readonly ultimaInscricao = this.#ultimaInscricao.asReadonly();

  async consultarDevedor(cpfCnpj: string): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);

    try {
      const resposta = await firstValueFrom(
        this.http.get<ConsultaDevedorResponse>(
          `${this.baseUrl}/devedores/${encodeURIComponent(cpfCnpj)}`
        )
      );
      this.#ultimoDevedor.set(resposta);
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao consultar devedor na Dívida Ativa.')
      );
      throw err;
    } finally {
      this.#carregando.set(false);
    }
  }

  async consultarInscricao(numeroInscricao: string): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);

    try {
      const resposta = await firstValueFrom(
        this.http.get<ConsultaInscricaoResponse>(
          `${this.baseUrl}/inscricoes/${encodeURIComponent(numeroInscricao)}`
        )
      );
      this.#ultimaInscricao.set(resposta);
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao consultar inscrição na Dívida Ativa.')
      );
      throw err;
    } finally {
      this.#carregando.set(false);
    }
  }

  limpar(): void {
    this.#ultimoDevedor.set(null);
    this.#ultimaInscricao.set(null);
    this.#erro.set(null);
  }
}
