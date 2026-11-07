import { Component, ChangeDetectionStrategy, computed, input, model } from '@angular/core';

export interface AccTab {
  /** Valor usado em `?aba=` (query param) e para casar com o panel externo. */
  readonly id: string;
  readonly label: string;
  readonly disabled?: boolean;
  /** Aba de preparação (fase 2/3): visível, mas sem conteúdo e com aviso. */
  readonly emBreve?: boolean;
}

/**
 * Abas acessíveis (WAI-ARIA tabs): navegação por teclado com tabindex
 * móvel (ArrowLeft/ArrowRight/Home/End), estados `aria-selected`,
 * `aria-disabled` e foco visível. O conteúdo das abas fica fora
 * (branch por `active()` na tela hospedeira), mantendo o componente
 * livre de depender de rotas ou query params.
 */
@Component({
  selector: 'acc-tabs',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-tabs" role="tablist" aria-label="Seções do cliente">
      @for (tab of tabs(); track tab.id; let indice = $index) {
        <button
          #botao
          type="button"
          role="tab"
          class="acc-tabs__tab"
          [class.acc-tabs__tab--active]="tab.id === active()"
          [class.acc-tabs__tab--em-breve]="tab.emBreve"
          [attr.aria-selected]="tab.id === active()"
          [attr.aria-disabled]="tab.disabled || null"
          [attr.tabindex]="(tab.id === active() || indicesAtivos().includes(indice)) ? 0 : -1"
          [disabled]="tab.disabled"
          (click)="selecionar(tab, botao)"
          (keydown)="aoPressionar($event, indice)"
        >
          {{ tab.label }}
          @if (tab.emBreve) {
            <span class="acc-tabs__soon">em breve</span>
          }
        </button>
      }
    </div>
  `,
  styles: `
    .acc-tabs {
      display: flex;
      gap: 0.25rem;
      padding: 0.25rem;
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-lg, 12px);
      background: var(--acc-color-bg, #f8fafc);
      overflow-x: auto;
      scrollbar-width: none;
    }

    .acc-tabs::-webkit-scrollbar {
      display: none;
    }

    .acc-tabs__tab {
      display: inline-flex;
      align-items: center;
      gap: 0.375rem;
      flex: 0 0 auto;
      padding: 0.5rem 0.875rem;
      font-size: 0.875rem;
      font-weight: 600;
      font-family: inherit;
      line-height: 1.25rem;
      color: var(--acc-color-text-muted, #64748b);
      background: transparent;
      border: none;
      border-radius: var(--acc-radius-md, 8px);
      cursor: pointer;
      transition: background-color 0.15s ease, color 0.15s ease;
    }

    .acc-tabs__tab:focus-visible {
      outline: 2px solid var(--acc-color-primary, #1e40af);
      outline-offset: 1px;
    }

    .acc-tabs__tab:hover:not(:disabled) {
      color: var(--acc-color-text-main, #0f172a);
      background: rgba(148, 163, 184, 0.18);
    }

    .acc-tabs__tab--active {
      color: var(--acc-color-primary, #1e40af);
      background: var(--acc-color-surface, #ffffff);
      box-shadow: var(--acc-shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
    }

    .acc-tabs__tab:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .acc-tabs__soon {
      padding: 0.0625rem 0.375rem;
      border-radius: 9999px;
      font-size: 0.625rem;
      font-weight: 700;
      line-height: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      background: var(--acc-color-info-bg, #f0f9ff);
      color: var(--acc-color-info, #0369a1);
    }
  `,
})
export class AccTabsComponent {
  readonly tabs = input.required<readonly AccTab[]>();
  readonly active = model.required<string>();

  /** índices selecionáveis (não-desabilitadas) para navegação por teclado */
  readonly indicesAtivos = computed(() =>
    this.tabs()
      .map((tab, indice) => (tab.disabled ? -1 : indice))
      .filter((indice) => indice >= 0),
  );

  selecionar(tab: AccTab, origem: HTMLElement): void {
    if (tab.disabled) {
      return;
    }
    this.active.set(tab.id);
    origem.focus();
  }

  aoPressionar(event: KeyboardEvent, indice: number): void {
    const ativos = this.indicesAtivos();
    if (ativos.length < 2) {
      return;
    }
    const posicaoAtual = ativos.indexOf(indice);
    let proximo: number | null = null;

    switch (event.key) {
      case 'ArrowRight': {
        proximo = ativos[(posicaoAtual + 1) % ativos.length] ?? null;
        break;
      }
      case 'ArrowLeft': {
        proximo = ativos[(posicaoAtual - 1 + ativos.length) % ativos.length] ?? null;
        break;
      }
      case 'Home': {
        proximo = ativos[0] ?? null;
        break;
      }
      case 'End': {
        proximo = ativos[ativos.length - 1] ?? null;
        break;
      }
      default:
        return;
    }

    event.preventDefault();
    if (proximo === null || proximo === indice) {
      return;
    }

    const tab = this.tabs()[proximo];
    if (!tab) {
      return;
    }
    this.active.set(tab.id);
    const alvo = (event.target as HTMLElement).parentElement?.children[proximo];
    if (alvo instanceof HTMLElement) {
      alvo.focus();
    }
  }
}
