import { Injectable, inject, computed, signal } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  EstadoConsultaFiscal,
  ListagemConsultasDto,
  SituacaoFiscalDto,
} from '@accurit/core';
import { mensagemDeErro } from '../core/http/mensagem-erro';

export interface CarregarConsultasOpcoes {
  pagina?: number;
  estado?: EstadoConsultaFiscal;
}

interface ConsultasState {
  clienteId: string | null;
  historico: ListagemConsultasDto | null;
  situacao: SituacaoFiscalDto | null;
  carregando: boolean;
  erro: string | null;
  /** `NAO_ENCONTRADO` distingue cliente inexistente/alheio de falha de rede. */
  codigoErro: string | null;
  filtroEstado: EstadoConsultaFiscal | null;
  carregandoSituacao: boolean;
  /** Falha do resumo (cartões) não prende o histórico — só esconde a visão. */
  erroSituacao: string | null;
}

const ESTADO_INICIAL: ConsultasState = {
  clienteId: null,
  historico: null,
  situacao: null,
  carregando: false,
  erro: null,
  codigoErro: null,
  filtroEstado: null,
  carregandoSituacao: false,
  erroSituacao: null,
};

function codigoDeErro(erro: unknown): string | null {
  return (erro as { error?: { code?: string } })?.error?.code ?? null;
}

/**
 * Leitura do histórico e da situação fiscal de UM cliente
 * (`GET /clientes/{id}/consultas` e `GET /clientes/{id}/situacao-fiscal`,
 * SPEC §9.3). Não executa consultas — o disparo acontece na tela
 * `/divida-ativa` (ADR-0002: Dívida Ativa é consultiva).
 *
 * Padrão do pacote (`ClienteService`, `DividaAtivaService`): estado em
 * signals públicos somente leitura; endpoints por requisição, sem cache.
 */
@Injectable({ providedIn: 'root' })
export class ConsultasFiscaisService {
  private readonly http = inject(HttpClient);

  #state = signal<ConsultasState>(ESTADO_INICIAL);

  readonly historico = computed(() => this.#state().historico);
  readonly situacao = computed(() => this.#state().situacao);
  readonly itens = computed(() => this.#state().historico?.itens ?? []);
  readonly total = computed(() => this.#state().historico?.total ?? 0);
  readonly pagina = computed(() => this.#state().historico?.pagina ?? 1);
  readonly tamanho = computed(() => this.#state().historico?.tamanho ?? 20);
  readonly filtroEstado = computed(() => this.#state().filtroEstado);
  readonly clienteIdCarregado = computed(() => this.#state().clienteId);

  readonly carregando = computed(() => this.#state().carregando);
  readonly pronto = computed(
    () => !this.#state().carregando && this.#state().historico !== null && this.#state().erro === null,
  );
  readonly erroCarregamento = computed(() => this.#state().erro);
  readonly naoEncontrado = computed(() => this.#state().codigoErro === 'NAO_ENCONTRADO');
  readonly temErro = computed(() => this.#state().erro !== null);

  readonly situacaoCarregando = computed(() => this.#state().carregandoSituacao);
  readonly erroSituacao = computed(() => this.#state().erroSituacao);
  /** Aviso permanente de leitura (ADR-0002): vem do backend e nunca muda. */
  readonly avisoConsultivo = computed(() => this.#state().situacao?.aviso ?? null);

  /**
   * Carrega histórico e situação fiscal do cliente. Troca de cliente
   * reseta tudo (nunca exibe dados do cliente anterior entre telas).
   */
  async carregar(
    clienteId: string,
    opcoes: CarregarConsultasOpcoes = {},
  ): Promise<void> {
    const clienteMudou = this.#state().clienteId !== clienteId;
    const filtroEstado = opcoes.estado ?? null;
    const pagina = opcoes.pagina ?? (filtroEstado ? 1 : this.#state().historico?.pagina ?? 1);

    this.#state.update((s) => ({
      ...s,
      clienteId,
      historico: clienteMudou ? null : s.historico,
      situacao: clienteMudou ? null : s.situacao,
      carregando: true,
      erro: null,
      codigoErro: null,
      filtroEstado,
      erroSituacao: null,
    }));

    const params = new HttpParams().set('pagina', String(pagina));
    const paramsFiltrados = filtroEstado
      ? params.set('estado', filtroEstado)
      : params;

    try {
      const resposta = await firstValueFrom(
        this.http.get<ListagemConsultasDto>(
          `/api/v1/clientes/${encodeURIComponent(clienteId)}/consultas`,
          { params: paramsFiltrados },
        ),
      );

      // resposta do MESMO cliente ainda? (evita sobrescrever troca em curso)
      if (this.#state().clienteId !== clienteId) {
        return;
      }

      this.#state.update((s) => ({
        ...s,
        historico: resposta,
        carregando: false,
        erro: null,
        codigoErro: null,
      }));
    } catch (erro) {
      if (this.#state().clienteId !== clienteId) {
        return;
      }
      this.#state.update((s) => ({
        ...s,
        historico: null,
        carregando: false,
        erro: mensagemDeErro(erro, 'Falha ao carregar o histórico de consultas.'),
        codigoErro: codigoDeErro(erro),
      }));
      return;
    }

    void this.carregarSituacao(clienteId);
  }

  /** Resumo por serviço (cartões). Erros não cancelam o histórico. */
  async carregarSituacao(clienteId: string): Promise<void> {
    this.#state.update((s) => ({ ...s, carregandoSituacao: true, erroSituacao: null }));

    try {
      const resposta = await firstValueFrom(
        this.http.get<SituacaoFiscalDto>(
          `/api/v1/clientes/${encodeURIComponent(clienteId)}/situacao-fiscal`,
        ),
      );

      if (this.#state().clienteId !== clienteId) {
        return;
      }

      this.#state.update((s) => ({
        ...s,
        situacao: resposta,
        carregandoSituacao: false,
        erroSituacao: null,
      }));
    } catch (erro) {
      if (this.#state().clienteId !== clienteId) {
        return;
      }
      this.#state.update((s) => ({
        ...s,
        situacao: null,
        carregandoSituacao: false,
        erroSituacao: mensagemDeErro(erro, 'Falha ao carregar a situação fiscal.'),
      }));
    }
  }

  limpar(): void {
    this.#state.set(ESTADO_INICIAL);
  }
}
