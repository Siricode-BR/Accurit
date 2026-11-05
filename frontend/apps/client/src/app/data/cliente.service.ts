import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import {
  ClienteDto,
  ListagemClientesDto,
  FiltroClientes,
  CreateClienteRequestDto,
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
    return new Promise((resolve, reject) => {
      this.http.post<ClienteDto>(this.baseUrl, payload).subscribe({
        next: (novo) => {
          this.#state.update((s) => ({
            ...s,
            itens: [novo, ...s.itens],
            total: s.total + 1,
          }));
          resolve(novo);
        },
        error: (err) => reject(err),
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
}

