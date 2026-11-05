import { Component, ChangeDetectionStrategy, input } from '@angular/core';

export type BadgeStatus = 'success' | 'warning' | 'danger' | 'info' | 'neutral';

@Component({
  selector: 'acc-badge',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <span class="acc-badge__dot"></span>
    <span class="acc-badge__label">
      <ng-content></ng-content>
    </span>
  `,
  host: {
    '[class]': '"acc-badge acc-badge--" + status()',
  },
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      gap: 0.375rem;
      padding: 0.125rem 0.5rem;
      font-size: 0.75rem;
      font-weight: 600;
      line-height: 1rem;
      border-radius: 9999px;
      width: fit-content;
      font-family: inherit;
    }

    .acc-badge__dot {
      width: 0.375rem;
      height: 0.375rem;
      border-radius: 50%;
      background-color: currentColor;
    }

    .acc-badge--success {
      background-color: var(--acc-color-success-bg, #f0fdf4);
      color: var(--acc-color-success, #15803d);
    }

    .acc-badge--warning {
      background-color: var(--acc-color-warning-bg, #fffbeb);
      color: var(--acc-color-warning, #b45309);
    }

    .acc-badge--danger {
      background-color: var(--acc-color-danger-bg, #fef2f2);
      color: var(--acc-color-danger, #b91c1c);
    }

    .acc-badge--info {
      background-color: var(--acc-color-info-bg, #f0f9ff);
      color: var(--acc-color-info, #0369a1);
    }

    .acc-badge--neutral {
      background-color: #f1f5f9;
      color: #475569;
    }
  `,
})
export class AccBadgeComponent {
  readonly status = input<BadgeStatus>('neutral');
}

