import { ChangeDetectionStrategy, Component, computed, input, output } from '@angular/core';

/**
 * Paginação acessível com `page`/`pageSize`/`total`.
 *
 * ```html
 * <acc-pagination [pagina]="pagina()" [tamanho]="20" [total]="total()"
 *                 (paginaChange)="carregar($event)" />
 * ```
 */
@Component({
  selector: 'acc-pagination',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <nav class="acc-pagination" aria-label="Paginação">
      <button
        type="button"
        class="acc-pagination__botao"
        [disabled]="pagina() <= 1"
        (click)="irPara(pagina() - 1)"
      >
        Anterior
      </button>
      <span class="acc-pagination__info" aria-live="polite">
        Página {{ pagina() }} de {{ totalPaginas() }}
        @if (total() > 0) {
          <span class="acc-pagination__total">· {{ total() }} registros</span>
        }
      </span>
      <button
        type="button"
        class="acc-pagination__botao"
        [disabled]="pagina() >= totalPaginas()"
        (click)="irPara(pagina() + 1)"
      >
        Próxima
      </button>
    </nav>
  `,
  styles: `
    :host { display: block; }
    .acc-pagination {
      display: flex; align-items: center; justify-content: flex-end; gap: 0.75rem;
      flex-wrap: wrap; padding-top: 0.75rem; font-size: 0.8125rem;
      color: var(--acc-color-text-muted, #64748b);
    }
    .acc-pagination__botao {
      border: 1px solid var(--acc-color-border, #e2e8f0); background: var(--acc-color-surface, #fff);
      color: var(--acc-color-text-main, #0f172a); padding: 0.375rem 0.75rem;
      border-radius: var(--acc-radius-md, 8px); cursor: pointer; font: inherit;
    }
    .acc-pagination__botao:hover:not(:disabled) { border-color: var(--acc-color-primary, #1e40af); }
    .acc-pagination__botao:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: 2px; }
    .acc-pagination__botao:disabled { opacity: 0.5; cursor: not-allowed; }
    .acc-pagination__total { color: inherit; }
  `,
})
export class AccPaginationComponent {
  readonly pagina = input<number>(1);
  readonly tamanho = input<number>(20);
  readonly total = input<number>(0);

  readonly paginaChange = output<number>();

  protected readonly totalPaginas = computed(() => {
    const tamanho = Math.max(1, this.tamanho());
    return Math.max(1, Math.ceil(this.total() / tamanho));
  });

  protected irPara(pagina: number): void {
    if (pagina < 1 || pagina > this.totalPaginas() || pagina === this.pagina()) {
      return;
    }
    this.paginaChange.emit(pagina);
  }
}
