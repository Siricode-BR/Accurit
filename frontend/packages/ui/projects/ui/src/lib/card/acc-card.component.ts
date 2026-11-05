import { Component, ChangeDetectionStrategy } from '@angular/core';

@Component({
  selector: 'acc-card',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-card">
      <ng-content select="[acc-card-header]"></ng-content>
      <div class="acc-card__body">
        <ng-content></ng-content>
      </div>
      <ng-content select="[acc-card-footer]"></ng-content>
    </div>
  `,
  styles: `
    .acc-card {
      background: var(--acc-color-surface, #ffffff);
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-lg, 12px);
      box-shadow: var(--acc-shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }

    .acc-card__body {
      padding: 1.5rem;
    }

    ::ng-deep [acc-card-header] {
      padding: 1.25rem 1.5rem;
      border-bottom: 1px solid var(--acc-color-border, #e2e8f0);
      font-weight: 600;
      font-size: 1.125rem;
    }

    ::ng-deep [acc-card-footer] {
      padding: 1rem 1.5rem;
      border-top: 1px solid var(--acc-color-border, #e2e8f0);
      background-color: var(--acc-color-bg, #f8fafc);
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 0.75rem;
    }
  `,
})
export class AccCardComponent {}

