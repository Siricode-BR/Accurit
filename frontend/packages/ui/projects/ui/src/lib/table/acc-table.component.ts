import { NgTemplateOutlet } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  input,
  output,
  signal,
} from '@angular/core';
import { AccEmptyStateComponent } from '../empty-state/acc-empty-state.component';
import { AccSkeletonComponent } from '../skeleton/acc-skeleton.component';
import { AccColunaTabela, AccOrdenacaoTabela } from './acc-table.types';

/**
 * Tabela declarativa do design system.
 *
 * - colunas via `[colunas]`, com `[coluna.template]` para célula customizada;
 * - ordenação local opcional (`ordenavel`), com `(ordenacaoChange)`;
 * - loading com skeleton, vazio com `acc-empty-state` e erro com retry;
 * - linhas clicáveis acessíveis (teclado) quando `[linhaClicavel]="true"`;
 * - overflow horizontal em telas pequenas.
 *
 * ```html
 * <acc-table [colunas]="colunas" [linhas]="clientes()" [linhaClicavel]="true"
 *            (linhaSelecionada)="abrir($event)" [carregando]="carregando()"
 *            [erro]="erro()" (tentarNovamente)="recarregar()" />
 * ```
 */
@Component({
  selector: 'acc-table',
  standalone: true,
  imports: [NgTemplateOutlet, AccEmptyStateComponent, AccSkeletonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-table__wrapper">
      @if (erro()) {
        <div class="acc-table__erro" role="alert">
          <p>{{ erro() }}</p>
          <button type="button" class="acc-table__retry" (click)="tentarNovamente.emit()">
            Tentar novamente
          </button>
        </div>
      } @else if (carregando()) {
        <div class="acc-table__skeleton" aria-busy="true" aria-live="polite">
          @for (linha of esqueletos; track linha) {
            <acc-skeleton variante="linha" altura="2.5rem" />
          }
        </div>
      } @else if (linhasOrdenadas().length === 0) {
        <acc-empty-state [title]="vazio()" [descricao]="descricaoVazio()" />
      } @else {
        <table class="acc-table">
          <thead>
            <tr>
              @for (coluna of colunas(); track coluna.campo) {
                <th
                  [style.width]="coluna.largura ?? null"
                  [style.text-align]="coluna.alinhamento ?? 'left'"
                  [attr.aria-sort]="ariaSort(coluna)"
                >
                  @if (coluna.ordenavel) {
                    <button type="button" class="acc-table__ordenar" (click)="ordenar(coluna.campo)">
                      {{ coluna.titulo }}
                      <span aria-hidden="true">{{ seta(coluna.campo) }}</span>
                    </button>
                  } @else {
                    {{ coluna.titulo }}
                  }
                </th>
              }
            </tr>
          </thead>
          <tbody>
            @for (linha of linhasOrdenadas(); track idDaLinha(linha, $index)) {
              <tr
                [class.acc-table__linha--clicavel]="linhaClicavel()"
                [attr.tabindex]="linhaClicavel() ? 0 : null"
                [attr.role]="linhaClicavel() ? 'button' : null"
                (click)="selecionar(linha)"
                (keydown.enter)="selecionar(linha)"
                (keydown.space)="selecionar(linha); $event.preventDefault()"
              >
                @for (coluna of colunas(); track coluna.campo) {
                  <td [style.text-align]="coluna.alinhamento ?? 'left'">
                    @if (coluna.template) {
                      <ng-container
                        [ngTemplateOutlet]="coluna.template"
                        [ngTemplateOutletContext]="{ $implicit: linha, linha: linha, valor: valor(linha, coluna.campo) }"
                      />
                    } @else {
                      {{ exibir(valor(linha, coluna.campo)) }}
                    }
                  </td>
                }
              </tr>
            }
          </tbody>
        </table>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-table__wrapper { width: 100%; overflow-x: auto; }
    .acc-table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
    .acc-table th, .acc-table td {
      padding: 0.625rem 0.75rem; border-bottom: 1px solid var(--acc-color-border, #e2e8f0);
      text-align: left; white-space: nowrap;
    }
    .acc-table th {
      font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em;
      color: var(--acc-color-text-muted, #64748b); font-weight: 600;
    }
    .acc-table tbody tr:hover { background-color: var(--acc-color-bg, #f8fafc); }
    .acc-table__linha--clicavel { cursor: pointer; }
    .acc-table__linha--clicavel:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: -2px; }
    .acc-table__ordenar {
      display: inline-flex; align-items: center; gap: 0.25rem;
      border: 0; background: transparent; padding: 0; cursor: pointer;
      font: inherit; color: inherit; text-transform: inherit; letter-spacing: inherit;
    }
    .acc-table__ordenar:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); border-radius: 4px; }
    .acc-table__skeleton { display: grid; gap: 0.5rem; padding: 0.5rem 0; }
    .acc-table__erro { display: grid; gap: 0.5rem; justify-items: start; padding: 1rem; color: var(--acc-color-danger, #dc2626); }
    .acc-table__retry {
      border: 1px solid currentColor; background: transparent; color: inherit;
      padding: 0.375rem 0.75rem; border-radius: var(--acc-radius-md, 8px); cursor: pointer;
    }
  `,
})
export class AccTableComponent<T extends Record<string, unknown> = Record<string, unknown>> {
  readonly colunas = input.required<readonly AccColunaTabela<T>[]>();
  readonly linhas = input<readonly T[]>([]);
  readonly carregando = input<boolean>(false);
  readonly erro = input<string | null>(null);
  readonly vazio = input<string>('Nenhum registro encontrado.');
  readonly descricaoVazio = input<string | null>(null);
  readonly linhaClicavel = input<boolean>(false);
  readonly campoId = input<string>('id');

  readonly linhaSelecionada = output<T>();
  readonly tentarNovamente = output<void>();
  readonly ordenacaoChange = output<AccOrdenacaoTabela | null>();

  protected readonly esqueletos = [0, 1, 2, 3, 4];
  protected readonly ordenacao = signal<AccOrdenacaoTabela | null>(null);

  protected readonly linhasOrdenadas = computed(() => {
    const ordenacao = this.ordenacao();
    const linhas = [...this.linhas()];
    if (!ordenacao) {
      return linhas;
    }
    return linhas.sort((a, b) => {
      const va = a[ordenacao.campo];
      const vb = b[ordenacao.campo];
      const comparacao = String(va ?? '').localeCompare(String(vb ?? ''), 'pt-BR', {
        numeric: true,
        sensitivity: 'base',
      });
      return ordenacao.direcao === 'asc' ? comparacao : -comparacao;
    });
  });

  protected ordenar(campo: string): void {
    const atual = this.ordenacao();
    const proxima: AccOrdenacaoTabela =
      atual?.campo === campo && atual.direcao === 'asc'
        ? { campo, direcao: 'desc' }
        : { campo, direcao: 'asc' };
    this.ordenacao.set(proxima);
    this.ordenacaoChange.emit(proxima);
  }

  protected seta(campo: string): string {
    const atual = this.ordenacao();
    if (atual?.campo !== campo) {
      return '↕';
    }
    return atual.direcao === 'asc' ? '↑' : '↓';
  }

  protected ariaSort(coluna: AccColunaTabela<T>): string | null {
    if (!coluna.ordenavel) {
      return null;
    }
    const atual = this.ordenacao();
    if (atual?.campo !== coluna.campo) {
      return 'none';
    }
    return atual.direcao === 'asc' ? 'ascending' : 'descending';
  }

  protected valor(linha: T, campo: string): unknown {
    return linha[campo];
  }

  protected exibir(valor: unknown): string {
    if (valor === null || valor === undefined || valor === '') {
      return '—';
    }
    return String(valor);
  }

  protected idDaLinha(linha: T, indice: number): unknown {
    return linha[this.campoId()] ?? indice;
  }

  protected selecionar(linha: T): void {
    if (this.linhaClicavel()) {
      this.linhaSelecionada.emit(linha);
    }
  }
}
