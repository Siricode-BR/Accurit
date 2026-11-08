import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  AtualizarUsuarioRequestDto,
  CriarUsuarioRequestDto,
  ListagemUsuariosDto,
  UsuarioDto,
} from '@accurit/core';

interface UsuariosState {
  itens: UsuarioDto[];
  total: number;
  carregando: boolean;
  erro: string | null;
}

/** Usuários do próprio escritório (somente o titular acessa). */
@Injectable({ providedIn: 'root' })
export class UsuarioService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/usuarios';

  #state = signal<UsuariosState>({ itens: [], total: 0, carregando: false, erro: null });

  readonly usuarios = computed(() => this.#state().itens);
  readonly total = computed(() => this.#state().total);
  readonly carregando = computed(() => this.#state().carregando);
  readonly erro = computed(() => this.#state().erro);

  /** `ativo`: `null` lista todos; a busca textual é aplicada na tela. */
  listar(ativo: boolean | null = null): void {
    this.#state.update((s) => ({ ...s, carregando: true, erro: null }));
    let params = new HttpParams();
    if (ativo !== null) params = params.set('ativo', String(ativo));

    this.http.get<ListagemUsuariosDto>(this.baseUrl, { params }).subscribe({
      next: (res) =>
        this.#state.update((s) => ({ ...s, itens: res.itens, total: res.total, carregando: false })),
      error: (err: { message?: string }) =>
        this.#state.update((s) => ({
          ...s,
          carregando: false,
          erro: err.message ?? 'Falha ao carregar a lista de usuários.',
        })),
    });
  }

  async criar(payload: CriarUsuarioRequestDto): Promise<UsuarioDto> {
    const novo = await firstValueFrom(this.http.post<UsuarioDto>(this.baseUrl, payload));
    this.#state.update((s) => ({ ...s, itens: [novo, ...s.itens], total: s.total + 1 }));
    return novo;
  }

  async atualizar(id: string, patch: AtualizarUsuarioRequestDto): Promise<UsuarioDto> {
    const atualizado = await firstValueFrom(
      this.http.patch<UsuarioDto>(`${this.baseUrl}/${id}`, patch),
    );
    this.#state.update((s) => ({
      ...s,
      itens: s.itens.map((u) => (u.id === id ? atualizado : u)),
    }));
    return atualizado;
  }

  async redefinirSenha(id: string, senha: string): Promise<void> {
    await firstValueFrom(this.http.post<void>(`${this.baseUrl}/${id}/senha`, { senha }));
  }
}
