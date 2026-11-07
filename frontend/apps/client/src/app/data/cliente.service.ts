import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import {
  ClienteDto,
  ListagemClientesDto,
  FiltroClientes,
  CreateClienteRequestDto,
  UpdateClienteRequestDto,
} from '@accurit/core';
import { sanitizeDocument } from '@accurit/shared';

interface ClientesState {
  itens: ClienteDto[];
  total: number;
  pagina: number;
  tamanho: number;
  carregando: boolean;
  erro: string | null;
}

@Injectable({ providedIn: 'root' })
export class ClienteService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/clientes';

  #state = signal<ClientesState>({
    itens: [],
    total: 0,
    pagina: 1,
    tamanho: 20,
    carregando: false,
    erro: null,
  });

  // Signals públicos
  readonly clientes = computed(() => this.#state().itens);
  readonly total = computed(() => this.#state().total);
  readonly carregando = computed(() => this.#state().carregando);
  readonly erro = computed(() => this.#state().erro);
  readonly totalPJ = computed(() => this.#state().itens.filter((c) => c.tipo === 'PJ').length);
  readonly totalPF = computed(() => this.#state().itens.filter((c) => c.tipo === 'PF').length);

  listar(filtro: FiltroClientes = 'todos', pagina = 1, tamanho = 20): void {
    this.#state.update((s) => ({ ...s, carregando: true, erro: null }));

    let params = new HttpParams()
      .set('pagina', pagina.toString())
      .set('tamanho', tamanho.toString());

    if (filtro !== 'todos') {
      params = params.set('filtro', filtro);
    }

    this.http.get<ListagemClientesDto>(this.baseUrl, { params }).subscribe({
      next: (res) => {
        this.#state.update((s) => ({
          ...s,
          itens: res.itens,
          total: res.total,
          pagina: res.pagina,
          tamanho: res.tamanho,
          carregando: false,
        }));
      },
      error: (err: { message?: string }) => {
        this.#state.update((s) => ({
          ...s,
          carregando: false,
          erro: err.message ?? 'Falha ao carregar lista de clientes.',
        }));
      },
    });
  }

  criar(payload: CreateClienteRequestDto): Promise<ClienteDto> {
    this.#state.update((s) => ({ ...s, carregando: true, erro: null }));
    return new Promise((resolve, reject) => {
      this.http.post<ClienteDto>(this.baseUrl, payload).subscribe({
        next: (novo) => {
          this.#state.update((s) => ({
            ...s,
            itens: [novo, ...s.itens],
            total: s.total + 1,
            carregando: false,
          }));
          resolve(novo);
        },
        error: (err) => {
          this.#state.update((s) => ({
            ...s,
            carregando: false,
            erro: err.error?.message ?? err.message ?? 'Falha ao cadastrar cliente.',
          }));
          reject(err);
        },
      });
    });
  }

  filtrarPorTextoOuDocumento(termo: string): ClienteDto[] {
    const limpo = sanitizeDocument(termo).toLowerCase();
    const termoStr = termo.toLowerCase().trim();

    return this.clientes().filter((c) => {
      const docLimpo = sanitizeDocument(c.cpfCnpj);
      const matchDoc = limpo ? docLimpo.includes(limpo) : false;
      const matchNome = c.nomeRazaoSocial.toLowerCase().includes(termoStr);
      const matchFantasia = c.nomeFantasia?.toLowerCase().includes(termoStr) ?? false;
      return matchDoc || matchNome || matchFantasia;
    });
  }

  obter(id: string): Promise<ClienteDto> {
    return new Promise((resolve, reject) => {
      this.http.get<ClienteDto>(`${this.baseUrl}/${id}`).subscribe({
        next: (cliente) => resolve(cliente),
        error: (err) =>
          reject(err.error?.message ?? err.message ?? 'Falha ao obter detalhes do cliente.'),
      });
    });
  }

  definirErro(mensagem: string): void {
    this.#state.update((s) => ({ ...s, erro: mensagem }));
  }

  atualizar(id: string, payload: UpdateClienteRequestDto): Promise<ClienteDto> {
    return new Promise((resolve, reject) => {
      this.http.put<ClienteDto>(`${this.baseUrl}/${id}`, payload).subscribe({
        next: (atualizado) => {
          this.#state.update((s) => ({
            ...s,
            itens: s.itens.map((c) => (c.id === id ? atualizado : c)),
            carregando: false,
          }));
          resolve(atualizado);
        },
        error: (err) => {
          this.#state.update((s) => ({
            ...s,
            carregando: false,
            erro: err.error?.message ?? err.message ?? 'Falha ao atualizar cliente.',
          }));
          reject(err);
        },
      });
    });
  }

  excluir(id: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.http.delete(`${this.baseUrl}/${id}`).subscribe({
        next: () => {
          this.#state.update((s) => ({
            ...s,
            itens: s.itens.filter((c) => c.id !== id),
            total: Math.max(0, s.total - 1),
            carregando: false,
          }));
          resolve();
        },
        error: (err) => {
          this.#state.update((s) => ({
            ...s,
            carregando: false,
            erro: err.error?.message ?? err.message ?? 'Falha ao excluir cliente.',
          }));
          reject(err);
        },
      });
    });
  }
}

