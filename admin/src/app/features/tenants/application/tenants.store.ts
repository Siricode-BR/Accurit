import { HttpErrorResponse } from '@angular/common/http';
import { computed, inject, Injectable, signal } from '@angular/core';
import { firstValueFrom } from 'rxjs';

import { Tenant, TenantAlterado, ResultadoTroca, Pagina } from '../domain/tenant';
import { TenantService } from '../data/tenant.service';

/** Camada de aplicação: estado e fluxo da lista de escritórios.
 *
 * O toggle é **pessimista** (SPEC §8): a UI só muda depois que o PATCH
 * confirma; nenhum clique em curso é perdido (trocas simultâneas na mesma
 * linha são ignoradas) e um 409 de versão divergente recarrega a página
 * atual automaticamente. */
@Injectable({ providedIn: 'root' })
export class TenantsStore {
  private readonly service = inject(TenantService);

  private readonly estado = signal<EstadoPagina>({ fase: 'carregando' });
  private readonly trocando = signal<ReadonlySet<string>>(new Set());

  readonly carregando = computed(() => this.estado().fase === 'carregando');
  readonly indisponivel = computed(() => this.estado().fase === 'indisponivel');

  readonly tenants = computed(() => {
    const atual = this.estado();
    return atual.fase === 'pronta' ? atual.pagina.itens : [];
  });
  readonly paginaAtual = computed(() => {
    const atual = this.estado();
    return atual.fase === 'pronta' ? atual.pagina.page : 1;
  });
  readonly porPagina = computed(() => {
    const atual = this.estado();
    return atual.fase === 'pronta' ? atual.pagina.pageSize : 0;
  });
  readonly total = computed(() => {
    const atual = this.estado();
    return atual.fase === 'pronta' ? atual.pagina.total : 0;
  });
  readonly paginas = computed(() => {
    const limite = this.porPagina();
    return limite > 0 ? Math.max(1, Math.ceil(this.total() / limite)) : 1;
  });

  protected static readonly TAMANHO_PADRAO = 20;

  listarTrocas(): ReadonlySet<string> {
    return this.trocando();
  }

  trocaEmAndamento(id: string): boolean {
    return this.trocando().has(id);
  }

  /** Carrega a página pedida (1-based). Falhas da rede deixam o estado
   * `indisponivel` para a tela mostrar recarregar. */
  async carregar(pagina = 1): Promise<void> {
    this.estado.set({ fase: 'carregando' });
    try {
      const tulha = await firstValueFrom(
        this.service.listar(pagina, TenantsStore.TAMANHO_PADRAO),
      );
      this.estado.set({ fase: 'pronta', pagina: tulha });
    } catch {
      this.estado.set({ fase: 'indisponivel' });
    }
  }

  /** Troca o modo de um escritório com concorrência otimista. Em conflito
   * (409) a página atual é recarregada antes de devolver o resultado. */
  async alternarTrial(entidade: Tenant, novoModo: boolean): Promise<ResultadoTroca> {
    if (this.trocaEmAndamento(entidade.id)) {
      return { tipo: 'ok' };
    }
    this.trocando.update((emCurso) => new Set(emCurso).add(entidade.id));
    try {
      const resposta = await firstValueFrom(
        this.service.alterarTrial(entidade.id, {
          modoTrialSerpro: novoModo,
          versao: entidade.versao,
        }),
      );
      this.aplicarResposta(entidade.id, resposta);
      return { tipo: 'ok' };
    } catch (erro) {
      return await this.classificar(erro);
    } finally {
      this.trocando.update((emCurso) => {
        const copia = new Set(emCurso);
        copia.delete(entidade.id);
        return copia;
      });
    }
  }

  private aplicarResposta(id: string, resposta: TenantAlterado): void {
    const estado = this.estado();
    if (estado.fase !== 'pronta') return;
    this.estado.set({
      fase: 'pronta',
      pagina: {
        ...estado.pagina,
        itens: estado.pagina.itens.map((entidade) =>
          entidade.id === id
            ? {
                ...entidade,
                modoTrialSerpro: resposta.modoTrialSerpro,
                versao: resposta.versaoAtual,
                atualizadoEm: resposta.atualizadoEm,
              }
            : entidade,
        ),
      },
    });
  }

  private async classificar(erro: unknown): Promise<ResultadoTroca> {
    const resposta = erro instanceof HttpErrorResponse ? erro : null;
    // 409: versão divergiu — recarregamos a página efetiva (SPEC §8).
    if (resposta?.status === 409) {
      await this.carregar(this.paginaAtual());
      return { tipo: 'conflito' };
    }
    switch (resposta?.status) {
      case 400:
        return { tipo: 'bloqueado', mensagem: this.mensagem(resposta) };
      case 404:
        return { tipo: 'naoEncontrado' };
      case 503:
        return { tipo: 'indisponivel' };
      default:
        if (!resposta || resposta.status === 0) {
          return { tipo: 'indisponivel' };
        }
        return { tipo: 'bloqueado', mensagem: this.mensagem(resposta) };
    }
  }

  private mensagem(erro: HttpErrorResponse): string {
    const corpo = erro.error as { error?: { message?: string } } | null;
    return corpo?.error?.message ?? 'Não foi possível alterar o ambiente.';
  }
}

type EstadoPagina =
  | { fase: 'carregando' }
  | { fase: 'pronta'; pagina: Pagina<Tenant> }
  | { fase: 'indisponivel' };
