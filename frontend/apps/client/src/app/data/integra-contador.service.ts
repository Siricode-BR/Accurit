import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { mensagemDeErro } from '../core/http/mensagem-erro';

export type VerboIntegraContador = 'Consultar' | 'Declarar' | 'Emitir' | 'Apoiar' | 'Monitorar';

export interface PedidoDados {
  idSistema: string;
  idServico: string;
  versaoSistema: string;
  dados: string;
}

export interface ExecucaoIntegraContadorRequest {
  documentoContribuinte: string;
  verbo: VerboIntegraContador;
  pedidoDados: PedidoDados;
  confirmacaoExplicita?: boolean;
  chaveIdempotencia?: string;
}

export interface ResultadoIntegraContadorResponse {
  ambiente: 'TRIAL' | 'PRODUCAO';
  verbo: VerboIntegraContador;
  idSistema: string;
  idServico: string;
  estado: string;
  statusSerpro: number;
  mensagem: string;
  referenciaResposta?: string | null;
  tempoEsperaSegundos?: number | null;
  etag?: string | null;
  expires?: string | null;
  codigosMensagem: string[];
}

@Injectable({ providedIn: 'root' })
export class IntegraContadorService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro/integra-contador';

  #carregando = signal(false);
  #erro = signal<string | null>(null);
  #ultimoResultado = signal<ResultadoIntegraContadorResponse | null>(null);

  readonly carregando = this.#carregando.asReadonly();
  readonly erro = this.#erro.asReadonly();
  readonly ultimoResultado = this.#ultimoResultado.asReadonly();

  async executar(request: ExecucaoIntegraContadorRequest): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);

    try {
      const resposta = await firstValueFrom(
        this.http.post<ResultadoIntegraContadorResponse>(
          `${this.baseUrl}/executar`,
          request
        )
      );
      this.#ultimoResultado.set(resposta);
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao executar Integra Contador.')
      );
      throw err;
    } finally {
      this.#carregando.set(false);
    }
  }

  limparResultado(): void {
    this.#ultimoResultado.set(null);
    this.#erro.set(null);
  }
}
