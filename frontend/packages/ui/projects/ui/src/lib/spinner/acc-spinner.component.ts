import { Component, ChangeDetectionStrategy, input } from '@angular/core';

@Component({
  selector: 'acc-spinner',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-spinner" [class]="'acc-spinner--' + size()" role="status" aria-label="Carregando">
      <span class="sr-only">Carregando...</span>
    </div>
  `,
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }

    .acc-spinner {
      border: 2px solid var(--acc-color-border, #e2e8f0);
      border-top-color: var(--acc-color-primary, #1e40af);
      border-radius: 50%;
      animation: acc-spin 0.7s linear infinite;
    }

    .acc-spinner--sm {
      width: 1rem;
      height: 1rem;
      border-width: 2px;
    }

    .acc-spinner--md {
      width: 1.5rem;
      height: 1.5rem;
      border-width: 2.5px;
    }

    .acc-spinner--lg {
      width: 2.5rem;
      height: 2.5rem;
      border-width: 3px;
    }

    .sr-only {
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      margin: -1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
      white-space: nowrap;
      border: 0;
    }

    @keyframes acc-spin {
      to { transform: rotate(360deg); }
    }
  `,
})
export class AccSpinnerComponent {
  readonly size = input<'sm' | 'md' | 'lg'>('md');
}

