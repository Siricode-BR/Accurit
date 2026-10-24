import { HttpClient } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';
import { Observable, ReplaySubject } from 'rxjs';

import { API_BASE_URL } from '../core/api.config';
import { SessionStore } from '../core/auth/session.store';

export type AmbienteSerpro = 'TRIAL' | 'PRODUCAO';

/** Resposta enxuta de GET /api/v1/serpro/ambiente (D8 de
 * DECISOES-PENDENTES): apenas o ambiente efetivo e quando ele mudou. */
export interface AmbienteSerproInfo {
  ambienteEfetivo: AmbienteSerpro;
  atualizadoEm: string;
}

export type EstadoAmbiente = 'indefinido' | 'carregando' | 'pronto' | 'indisponivel';

/** Camada de dados do selo de ambiente SERPRO. O backend é a autoridade:
 * leitura única do ambiente do escritório do token do portal (SPEC §8). A
 * leitura em curso é compartilhada (single-flight) e o cache é invalidado
 * quando o usuário da sessão muda. */
@Injectable({ providedIn: 'root' })
export class SerproAmbienteService {
  private readonly http = inject(HttpClient);
  private readonly sessao = inject(SessionStore);
  private readonly baseUrl = inject(API_BASE_URL);

  private readonly info = signal<AmbienteSerproInfo | null>(null);
  private readonly estado = signal<EstadoAmbiente>('indefinido');

  private usuarioNoCache: unknown = null;
  private carregamento: Observable<AmbienteSerproInfo> | null = null;

  readonly leitura = computed(() => ({
    estado: this.estado(),
    ambiente: this.info(),
  }));

  /** Fluxo da leitura do ambiente do escritório do token. Reaproveita a
   * requisição em curso ou o cache da sessão atual. */
  obter(): Observable<AmbienteSerproInfo> {
    const usuario = this.sessao.usuario();
    if (this.usuarioNoCache !== usuario) {
      this.usuarioNoCache = usuario;
      this.info.set(null);
      this.estado.set('indefinido');
      this.carregamento = null;
    }

    const cache = this.info();
    if (cache !== null) {
      return new Observable<AmbienteSerproInfo>((inscricao) => {
        inscricao.next(cache);
        inscricao.complete();
      });
    }

    if (this.carregamento === null) {
      this.estado.set('carregando');
      const sujeto = new ReplaySubject<AmbienteSerproInfo>(1);
      this.carregamento = sujeto;
      this.http
        .get<AmbienteSerproInfo>(`${this.baseUrl}/serpro/ambiente`)
        .subscribe({
          next: (resposta) => {
            this.info.set(resposta);
            this.estado.set('pronto');
            sujeto.next(resposta);
            sujeto.complete();
            this.carregamento = null;
          },
          error: (erro) => {
            this.estado.set('indisponivel');
            sujeto.error(erro);
            this.carregamento = null;
          },
        });
      return sujeto;
    }

    return this.carregamento;
  }
}
