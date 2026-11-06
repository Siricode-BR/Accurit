import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, tap, throwError } from 'rxjs';
import {
  AutenticarProcuradorRequestDto,
  AutenticarProcuradorResponseDto,
} from '@accurit/core';

export type StatusProcuracao = 'AUTORIZADA' | 'PENDENTE';

/**
 * Valida a procuração e-CAC (endpoint AUTENTICAPROCURADOR do SERPRO) para um
 * cliente do escritório. O backend assina o Termo de Autorização com o
 * certificado A1 do escritório e guarda o jwt_token **cifrado** por cliente —
 * o token nunca chega ao navegador (ADR-0007 / REGRA SEG); este serviço
 * mantém apenas o status de autorização por cliente para exibição na UI.
 */
@Injectable({ providedIn: 'root' })
export class ProcuracaoService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro/integra-contador/procuracoes';

  // Status por cliente dentro da sessão (ausente = PENDENTE). Nenhum
  // token é armazenado — apenas o resultado da validação.
  #status = signal<Record<string, StatusProcuracao>>({});

  /** Visão pública somente-leitura do mapa status-por-cliente. */
  readonly statusPorCliente = this.#status.asReadonly();

  /** Status atual de um cliente (PENDENTE enquanto não autorizado). */
  statusDe(clienteId: string): StatusProcuracao {
    return this.#status()[clienteId] ?? 'PENDENTE';
  }

  /**
   * Dispara a autenticação do procurador para o cliente e atualiza o
   * status local na resposta (AUTORIZADA) ou em caso de erro (PENDENTE).
   */
  validarProcuracao(clienteId: string): Observable<AutenticarProcuradorResponseDto> {
    const request: AutenticarProcuradorRequestDto = { clienteId };
    return this.http
      .post<AutenticarProcuradorResponseDto>(`${this.baseUrl}/autenticar`, request)
      .pipe(
        tap((resposta) => {
          this.#status.update((mapa) => ({
            ...mapa,
            [resposta.clienteId]: 'AUTORIZADA',
          }));
        }),
        catchError((erro: unknown) => {
          this.#status.update((mapa) => ({ ...mapa, [clienteId]: 'PENDENTE' }));
          return throwError(() => erro);
        }),
      );
  }
}
