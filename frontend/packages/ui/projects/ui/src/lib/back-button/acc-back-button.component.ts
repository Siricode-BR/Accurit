import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { AccIconComponent } from '../icon/acc-icon.component';

/**
 * Botão "voltar" circular com ícone de seta. Posicionado sempre à esquerda do título da página
 * (`<acc-page-header>` projeta qualquer `[acc-back]` nessa posição).
 *
 * Uso: `<button acc-back label="Voltar para clientes" (click)="voltar()"></button>`
 * ou `<a acc-back routerLink="/clientes" label="Voltar para clientes"></a>`.
 */
@Component({
  selector: 'button[acc-back], a[acc-back]',
  standalone: true,
  imports: [AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '[attr.aria-label]': 'label()',
    '[attr.title]': 'label()',
  },
  template: `<acc-icon class="acc-back__icone" name="chevron-left" />`,
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex: 0 0 auto;
      width: 2.5rem;
      height: 2.5rem;
      padding: 0;
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: 999px;
      background-color: var(--acc-color-surface, #ffffff);
      color: var(--acc-color-text-muted, #475569);
      text-decoration: none;
      cursor: pointer;
      box-shadow: 0 1px 2px rgb(15 23 42 / 0.06);
      transition: border-color 0.15s ease, color 0.15s ease, box-shadow 0.15s ease, background-color 0.15s ease;
    }

    :host(:hover) {
      border-color: var(--acc-color-primary, #1e40af);
      background-color: var(--acc-color-primary-light, #eff6ff);
      color: var(--acc-color-primary, #1e40af);
      box-shadow: 0 2px 6px rgb(30 64 175 / 0.18);
    }

    :host(:active) {
      box-shadow: none;
    }

    :host(:focus-visible) {
      outline: 3px solid rgb(30 64 175 / 0.35);
      outline-offset: 2px;
    }

    .acc-back__icone {
      --acc-icon-size: 1.25rem;
      display: inline-flex;
      transition: transform 0.15s ease;
    }

    :host(:hover) .acc-back__icone {
      transform: translateX(-2px);
    }
  `,
})
export class AccBackButtonComponent {
  readonly label = input<string>('Voltar');
}
