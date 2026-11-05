import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import {
  TenantDto,
  TenantAlteradoDto,
  CriarTenantRequestDto,
  CriarTenantResponseDto,
  AmbienteSerpro,
} from '@accurit/core';

interface TenantsState {
  itens: TenantDto[];
  total: number;
  pagina: number;
  tamanho: number;
  carregando: boolean;
  erro: string | null;
}

@Injectable({ providedIn: 'root' })
export class TenantService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin/escritorios';

  #state = signal<TenantsState>({
    itens: [],
    total: 0,
    pagina: 1,
    tamanho: 20,
    carregando: false,
    erro: null,
  });

  // Signals públicos
  readonly tenants = computed(() => this.#state().itens);
  readonly total = computed(() => this.#state().total);
  readonly carregando = computed(() => this.#state().carregando);
  readonly erro = computed(() => this.#state().erro);

  readonly totalTrial = computed(
    () => this.#state().itens.filter((t) => t.modoTrialSerpro).length
  );
  readonly totalProducao = computed(
    () => this.#state().itens.filter((t) => !t.modoTrialSerpro).length
  );

  listar(pagina = 1, tamanho = 20): void {
    this.#state.update((s) => ({ ...s, carregando: true, erro: null }));

    const params = new HttpParams()
      .set('page', pagina.toString())
      .set('pageSize', tamanho.toString());

    this.http.get<{ itens: TenantDto[]; total: number; page: number; pageSize: number }>(
      this.baseUrl,
      { params }
    ).subscribe({
      next: (res) => {
        this.#state.update((s) => ({
          ...s,
          itens: res.itens,
          total: res.total,
          pagina: res.page,
          tamanho: res.pageSize,
          carregando: false,
        }));
      },
      error: (err: { message?: string }) => {
        this.#state.update((s) => ({
          ...s,
          carregando: false,
          erro: err.message ?? 'Falha ao carregar lista de escritórios.',
        }));
      },
    });
  }

  criar(payload: CriarTenantRequestDto): Promise<CriarTenantResponseDto> {
    return new Promise((resolve, reject) => {
      this.http.post<CriarTenantResponseDto>(this.baseUrl, payload).subscribe({
        next: (novo) => {
          this.listar();
          resolve(novo);
        },
        error: (err) => reject(err),
      });
    });
  }

  alternarModoSerpro(tenantId: string, modoTrial: boolean, versao: number): Promise<TenantAlteradoDto> {
    return new Promise((resolve, reject) => {
      const url = `${this.baseUrl}/${tenantId}/serpro/trial`;
      this.http.patch<TenantAlteradoDto>(url, { modoTrial, versaoEsperada: versao }).subscribe({
        next: (alt) => {
          this.#state.update((s) => ({
            ...s,
            itens: s.itens.map((t) =>
              t.id === tenantId
                ? {
                    ...t,
                    modoTrialSerpro: alt.modoTrialSerpro,
                    versao: alt.versaoAtual,
                    atualizadoEm: alt.atualizadoEm,
                  }
                : t
            ),
          }));
          resolve(alt);
        },
        error: (err) => reject(err),
      });
    });
  }
}

