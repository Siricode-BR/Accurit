import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, switchMap, tap, throwError } from 'rxjs';
import {
  EstadoProcuracaoClienteDto,
  ResultadoValidarProcuracaoResponseDto,
  StatusProcuracaoGeral,
} from '@accurit/core';

export type StatusProcuracao = StatusProcuracaoGeral;

/**
 * Serviço de procurações e-CAC (Integra Contador / AUTENTICAPROCURADOR / OBTERPROCURACAO41).
 * O status é persistido no banco de dados e exposto via API REST (ADR-0013 / FASE A).
 * Nenhum token é armazenado no cliente nem exposto pelo backend (ADR-0007 / REGRA SEG).
 */
@Injectable({ providedIn: 'root' })
export class ProcuracaoService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro/integra-contador/procuracoes';

  // Mapa de estados consolidados por cliente lidos do backend
  #estadoPorCliente = signal<Record<string, EstadoProcuracaoClienteDto>>({});
  readonly estadoPorCliente = this.#estadoPorCliente.asReadonly();

  /** Visão pública compatível: mapa clienteId -> status geral */
  readonly statusPorCliente = computed<Record<string, StatusProcuracaoGeral>>(() => {
    const mapa = this.#estadoPorCliente();
    const resultado: Record<string, StatusProcuracaoGeral> = {};
    for (const [id, estado] of Object.entries(mapa)) {
      resultado[id] = estado.status ?? 'PENDENTE';
    }
    return resultado;
  });

  /** Retorna o estado completo da procuração do cliente em cache */
  estadoDe(clienteId: string): EstadoProcuracaoClienteDto | null {
    return this.#estadoPorCliente()[clienteId] ?? null;
  }

  /** Retorna o status geral atual de um cliente (PENDENTE se ainda não carregado ou sem procuração ativa). */
  statusDe(clienteId: string): StatusProcuracaoGeral {
    return this.#estadoPorCliente()[clienteId]?.status ?? 'PENDENTE';
  }

  /**
   * Consulta o estado consolidado da procuração do cliente no backend:
   * status geral, validade, data da última verificação e status por serviço.
   */
  obterPorCliente(clienteId: string): Observable<EstadoProcuracaoClienteDto> {
    return this.http
      .get<EstadoProcuracaoClienteDto>(this.baseUrl, {
        params: { cliente_id: clienteId },
      })
      .pipe(
        tap((estado) => {
          this.#estadoPorCliente.update((mapa) => ({
            ...mapa,
            [clienteId]: estado,
          }));
        }),
      );
  }

  /**
   * Valida a procuração do cliente junto ao SERPRO (com cache de 24h no backend).
   * Atualiza o estado consolidado do cliente após a validação.
   */
  validarProcuracao(clienteId: string): Observable<EstadoProcuracaoClienteDto> {
    return this.http
      .post<ResultadoValidarProcuracaoResponseDto>(`${this.baseUrl}/validar`, { clienteId })
      .pipe(
        switchMap(() => this.obterPorCliente(clienteId)),
        catchError((erro: unknown) => {
          return throwError(() => erro);
        }),
      );
  }

  /**
   * Força uma revalidação imediata no SERPRO (ignora cache no backend).
   */
  revalidarProcuracao(clienteId: string): Observable<EstadoProcuracaoClienteDto> {
    return this.http
      .post<ResultadoValidarProcuracaoResponseDto>(`${this.baseUrl}/revalidar`, { clienteId })
      .pipe(
        switchMap(() => this.obterPorCliente(clienteId)),
        catchError((erro: unknown) => {
          return throwError(() => erro);
        }),
      );
  }
}
