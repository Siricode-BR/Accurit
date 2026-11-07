import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';

export interface AccFilterOption<T extends string = string> {
  value: T;
  label: string;
}

/** Grupo de filtros em "chips" (seleção única) com rolagem horizontal em telas estreitas. */
@Component({
  selector: 'acc-filter-chips',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-chips" role="radiogroup" [attr.aria-label]="ariaLabel()">
      @for (opcao of options(); track opcao.value) {
        <button
          type="button"
          role="radio"
          class="acc-chip"
          [class.acc-chip--active]="opcao.value === value()"
          [attr.aria-checked]="opcao.value === value()"
          (click)="valueChange.emit(opcao.value)"
        >
          {{ opcao.label }}
        </button>
      }
    </div>
  `,
  styles: `
    :host {
      display: block;
      min-width: 0;
    }

    .acc-chips {
      display: flex;
      gap: 0.5rem;
      overflow-x: auto;
      padding: 0.125rem;
      scrollbar-width: none;
    }

    .acc-chips::-webkit-scrollbar {
      display: none;
    }

    .acc-chip {
      flex: 0 0 auto;
      height: 2.25rem;
      padding: 0 0.875rem;
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: 999px;
      background-color: var(--acc-color-surface, #ffffff);
      color: var(--acc-color-text-muted, #475569);
      font: inherit;
      font-size: 0.8125rem;
      font-weight: 500;
      cursor: pointer;
      white-space: nowrap;
      transition: background-color 0.15s ease-in-out, border-color 0.15s ease-in-out, color 0.15s ease-in-out;
    }

    .acc-chip:hover {
      border-color: #cbd5e1;
      color: var(--acc-color-text-main, #0f172a);
      background-color: var(--acc-color-bg, #f8fafc);
    }

    .acc-chip:focus-visible {
      outline: 2px solid var(--acc-color-primary, #1e40af);
      outline-offset: 2px;
    }

    .acc-chip--active,
    .acc-chip--active:hover {
      background-color: var(--acc-color-primary-light, #eff6ff);
      border-color: var(--acc-color-primary, #1e40af);
      color: var(--acc-color-primary, #1e40af);
      font-weight: 600;
    }
  `,
})
export class AccFilterChipsComponent<T extends string = string> {
  readonly options = input.required<readonly AccFilterOption<T>[]>();
  readonly value = input.required<T>();
  readonly ariaLabel = input<string>('Filtros');
  readonly valueChange = output<T>();
}
