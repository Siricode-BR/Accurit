import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { AccIconComponent } from '../icon/acc-icon.component';
import { AccIconName } from '../icon/acc-icon.types';

/**
 * Botão de chamada principal da página (ex.: "Novo cliente", "Novo escritório").
 * Pílula com gradiente, ícone em círculo e elevação no hover.
 */
@Component({
  selector: 'button[acc-cta], a[acc-cta]',
  standalone: true,
  imports: [AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <span class="acc-cta__icone" aria-hidden="true"><acc-icon [name]="icon()" /></span>
    <span class="acc-cta__texto"><ng-content></ng-content></span>
  `,
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      gap: 0.625rem;
      height: 2.75rem;
      padding: 0 1.25rem 0 0.5rem;
      border: 0;
      border-radius: 999px;
      background: linear-gradient(
        135deg,
        var(--acc-color-primary-hover, #1d4ed8) 0%,
        var(--acc-color-primary, #1e40af) 100%
      );
      color: #ffffff;
      font: inherit;
      font-size: 0.875rem;
      font-weight: 600;
      text-decoration: none;
      white-space: nowrap;
      cursor: pointer;
      box-shadow: 0 1px 2px rgb(30 64 175 / 0.3), 0 6px 14px -4px rgb(30 64 175 / 0.45);
      transition: transform 0.15s ease, box-shadow 0.15s ease, filter 0.15s ease;
    }

    :host(:hover) {
      transform: translateY(-1px);
      filter: brightness(1.06);
      box-shadow: 0 2px 4px rgb(30 64 175 / 0.3), 0 10px 20px -6px rgb(30 64 175 / 0.55);
    }

    :host(:active) {
      transform: translateY(0);
    }

    :host(:focus-visible) {
      outline: 3px solid rgb(30 64 175 / 0.35);
      outline-offset: 2px;
    }

    .acc-cta__icone {
      --acc-icon-size: 1.125rem;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 2rem;
      height: 2rem;
      border-radius: 50%;
      background-color: rgb(255 255 255 / 0.18);
    }
  `,
})
export class AccCtaButtonComponent {
  readonly icon = input<AccIconName>('plus');
}
