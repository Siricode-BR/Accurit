import { Component, ChangeDetectionStrategy, input } from '@angular/core';

/**
 * Estado vazio: título, descrição e área livre para ação (ex.: botão).
 * Usado em listas e abas sem dados — texto sempre em pt-BR na tela hospedeira.
 */
@Component({
  selector: 'acc-empty-state',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-empty-state">
      <span class="acc-empty-state__ico" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z"
          />
        </svg>
      </span>
      <p class="acc-empty-state__title">{{ title() }}</p>
      @if (descricao()) {
        <p class="acc-empty-state__description">{{ descricao() }}</p>
      }
      <div class="acc-empty-state__actions">
        <ng-content></ng-content>
      </div>
    </div>
  `,
  styles: `
    .acc-empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 0.375rem;
      padding: 2.5rem 1.5rem;
      text-align: center;
      color: var(--acc-color-text-muted, #64748b);
    }

    .acc-empty-state__ico {
      display: inline-flex;
      width: 2.5rem;
      height: 2.5rem;
      color: var(--acc-color-text-muted, #94a3b8);
    }

    .acc-empty-state__ico svg {
      width: 100%;
      height: 100%;
    }

    .acc-empty-state__title {
      margin: 0.25rem 0 0;
      font-size: 0.9375rem;
      font-weight: 600;
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-empty-state__description {
      margin: 0;
      font-size: 0.8125rem;
      max-width: 34rem;
    }

    .acc-empty-state__actions {
      margin-top: 0.75rem;
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
      justify-content: center;
    }

    .acc-empty-state__actions:empty {
      margin-top: 0;
    }
  `,
})
export class AccEmptyStateComponent {
  readonly title = input.required<string>();
  readonly descricao = input<string | null>(null);
}
