import { Component, ChangeDetectionStrategy, input } from '@angular/core';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'outline' | 'ghost';
export type ButtonSize = 'sm' | 'md' | 'lg';

@Component({
  selector: 'button[acc-button], a[acc-button]',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (loading()) {
      <span class="acc-btn__spinner" aria-hidden="true"></span>
    }
    <span class="acc-btn__content" [class.acc-btn__content--loading]="loading()">
      <ng-content></ng-content>
    </span>
  `,
  host: {
    '[class]': '"acc-btn acc-btn--" + variant() + " acc-btn--" + size()',
    '[class.acc-btn--loading]': 'loading()',
    '[attr.disabled]': 'disabled() || loading() ? true : null',
    '[attr.aria-busy]': 'loading()',
  },
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      font-weight: 500;
      border-radius: var(--acc-radius-md, 6px);
      cursor: pointer;
      transition: all 0.15s ease-in-out;
      border: 1px solid transparent;
      outline: none;
      user-select: none;
      text-decoration: none;
      font-family: inherit;
    }

    :host(:disabled), :host(.acc-btn--loading) {
      opacity: 0.65;
      cursor: not-allowed;
    }

    /* Sizes */
    .acc-btn--sm {
      padding: 0.25rem 0.625rem;
      font-size: 0.8125rem;
      line-height: 1.25rem;
    }
    .acc-btn--md {
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
      line-height: 1.25rem;
    }
    .acc-btn--lg {
      padding: 0.625rem 1.25rem;
      font-size: 1rem;
      line-height: 1.5rem;
    }

    /* Variants */
    .acc-btn--primary {
      background-color: var(--acc-color-primary, #1e40af);
      color: #ffffff;
    }
    .acc-btn--primary:hover:not(:disabled) {
      background-color: var(--acc-color-primary-hover, #1d4ed8);
    }

    .acc-btn--secondary {
      background-color: var(--acc-color-secondary, #0f172a);
      color: #ffffff;
    }

    .acc-btn--danger {
      background-color: var(--acc-color-danger, #b91c1c);
      color: #ffffff;
    }

    .acc-btn--outline {
      background-color: transparent;
      border-color: var(--acc-color-border, #e2e8f0);
      color: var(--acc-color-text-main, #0f172a);
    }
    .acc-btn--outline:hover:not(:disabled) {
      background-color: var(--acc-color-bg, #f8fafc);
    }

    .acc-btn--ghost {
      background-color: transparent;
      color: var(--acc-color-text-muted, #64748b);
    }
    .acc-btn--ghost:hover:not(:disabled) {
      background-color: var(--acc-color-bg, #f8fafc);
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-btn__spinner {
      width: 1rem;
      height: 1rem;
      border: 2px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: acc-spin 0.6s linear infinite;
    }

    @keyframes acc-spin {
      to { transform: rotate(360deg); }
    }
  `,
})
export class AccButtonComponent {
  readonly variant = input<ButtonVariant>('primary');
  readonly size = input<ButtonSize>('md');
  readonly disabled = input<boolean>(false);
  readonly loading = input<boolean>(false);
}

