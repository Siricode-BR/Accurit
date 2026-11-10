import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  AtualizarUsuarioRequestDto,
  CriarUsuarioRequestDto,
  ListagemUsuariosDto,
  UsuarioDto,
  DefinirAcessoRequestDto,
  DefinirCarteiraRequestDto,
  AcessoUsuarioDto,
} from '@accurit/core';
import { mensagemDeErro } from '../core/http/mensagem-erro';

interface UsuariosState {
  itens: UsuarioDto[];
  total: number;
  pagina: number;
  tamanho: number;
  carregando: boolean;
  erro: string | null;
}

/** Usuários do próprio escritório (somente o titular acessa). */
@Injectable({ providedIn: 'root' })
export class UsuarioService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/usuarios';

  #state = signal<UsuariosState>({
    itens: [],
    total: 0,
    pagina: 1,
    tamanho: 20,
    carregando: false,
    erro: null,
  });

  readonly usuarios = computed(() => this.#state().itens);
  readonly total = computed(() => this.#state().total);
  readonly pagina = computed(() => this.#state().pagina);
  readonly tamanho = computed(() => this.#state().tamanho);
  readonly totalPaginas = computed(() => {
    const total = this.#state().total;
    const tamanho = this.#state().tamanho || 20;
    if (total === 0) return 1;
    return Math.max(1, Math.ceil(total / tamanho));
  });
  readonly carregando = computed(() => this.#state().carregando);
  readonly erro = computed(() => this.#state().erro);

  /**
   * Listagem com paginação e busca no servidor (Fase E).
   * `busca` é filtrada por nome/e-mail/CPF no backend.
   */
  listarPaginado({
    ativo,
    busca,
    pagina = 1,
    tamanho = 20,
  }: {
    ativo?: boolean | null;
    busca?: string | null;
    pagina?: number;
    tamanho?: number;
  } = {}): void {
    this.#state.update((s) => ({ ...s, carregando: true, erro: null }));
    let params = new HttpParams()
      .set('pagina', String(pagina))
      .set('tamanho', String(tamanho));
    if (ativo !== null && ativo !== undefined) {
      params = params.set('ativo', String(ativo));
    }
    if (busca) {
      params = params.set('busca', busca);
    }

    this.http.get<ListagemUsuariosDto>(this.baseUrl, { params }).subscribe({
      next: (res) =>
        this.#state.update((s) => ({
          ...s,
          itens: res.itens,
          total: res.total,
          pagina: res.pagina,
          tamanho: res.tamanho,
          carregando: false,
        })),
      error: (err: { message?: string }) =>
        this.#state.update((s) => ({
          ...s,
          carregando: false,
          erro: mensagemDeErro(err, 'Falha ao carregar a lista de usuários.'),
        })),
    });
  }

  /** Compatibilidade com a chamada anterior (`listar(ativo)`). */
  listar(ativo: boolean | null = null): void {
    this.listarPaginado({ ativo });
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

  // ===== Fase E — função e carteira do usuário =====

  async obterAcesso(usuarioId: string): Promise<AcessoUsuarioDto> {
    return firstValueFrom(
      this.http.get<AcessoUsuarioDto>(`${this.baseUrl}/${usuarioId}/acesso`),
    );
  }

  async definirAcesso(
    usuarioId: string,
    payload: DefinirAcessoRequestDto,
  ): Promise<AcessoUsuarioDto> {
    const acesso = await firstValueFrom(
      this.http.put<AcessoUsuarioDto>(
        `${this.baseUrl}/${usuarioId}/acesso`,
        payload,
      ),
    );
    return acesso;
  }

  async definirCarteira(
    usuarioId: string,
    payload: DefinirCarteiraRequestDto,
  ): Promise<AcessoUsuarioDto> {
    const acesso = await firstValueFrom(
      this.http.put<AcessoUsuarioDto>(
        `${this.baseUrl}/${usuarioId}/carteira`,
        payload,
      ),
    );
    return acesso;
  }
}
