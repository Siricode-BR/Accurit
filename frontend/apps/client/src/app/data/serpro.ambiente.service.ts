import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ReplaySubject, Observable } from 'rxjs';
import { AmbienteSerproInfoDto, EstadoAmbiente } from '@accurit/core';
import { SessionStore } from '../core/auth/session.store';

@Injectable({ providedIn: 'root' })
export class SerproAmbienteService {
  private readonly http = inject(HttpClient);
  private readonly sessao = inject(SessionStore);
  private readonly baseUrl = '/api/v1/serpro/ambiente';

  // Signals internos de estado
  #info = signal<AmbienteSerproInfoDto | null>(null);
  #estado = signal<EstadoAmbiente>('indefinido');

  // Memoização single-flight: invalida ao trocar usuário
  private usuarioNoCache: unknown = null;
  private carregamento$: Observable<AmbienteSerproInfoDto> | null = null;

  /** Dados derivados públicos — somente leitura */
  readonly leitura = computed(() => ({
    estado: this.#estado(),
    ambiente: this.#info(),
  }));

  readonly ambienteEfetivo = computed(() => this.#info()?.ambienteEfetivo ?? null);

  readonly isTrial = computed(() => this.#info()?.ambienteEfetivo === 'TRIAL');

  /**
   * Retorna um Observable single-flight para o ambiente SERPRO do escritório.
   * Reutiliza a requisição em curso ou o cache da sessão atual.
   * Uso: RxJS necessário aqui pois o consumer precisa de streaming/completions.
   */
  obter(): Observable<AmbienteSerproInfoDto> {
    const usuario = this.sessao.user();

    if (this.usuarioNoCache !== usuario) {
      this.usuarioNoCache = usuario;
      this.#info.set(null);
      this.#estado.set('indefinido');
      this.carregamento$ = null;
    }

    if (this.#info()) {
      // Já carregado — emite imediatamente
      const cached = this.#info()!;
      const replay$ = new ReplaySubject<AmbienteSerproInfoDto>(1);
      replay$.next(cached);
      replay$.complete();
      return replay$.asObservable();
    }

    if (!this.carregamento$) {
      this.#estado.set('carregando');
      const replay$ = new ReplaySubject<AmbienteSerproInfoDto>(1);

      this.http.get<AmbienteSerproInfoDto>(this.baseUrl).subscribe({
        next: (info) => {
          this.#info.set(info);
          this.#estado.set('pronto');
          replay$.next(info);
          replay$.complete();
        },
        error: () => {
          this.#estado.set('indisponivel');
          this.carregamento$ = null;
          replay$.error(new Error('Ambiente SERPRO indisponível'));
        },
      });

      this.carregamento$ = replay$.asObservable();
    }

    return this.carregamento$;
  }
}

