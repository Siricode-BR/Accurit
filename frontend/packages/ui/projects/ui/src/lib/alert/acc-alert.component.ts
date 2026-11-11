import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';

export type VarianteAlerta = 'info' | 'sucesso' | 'aviso' | 'erro';

/**
 * Alerta inline do design system; `erro` usa `role="alert"` (assertivo),
 * os demais `role="status"`. Pode ser descartável.
 *
 * ```html
 * <acc-alert variante="erro" titulo="Falha" [descartavel]="true" (fechado)="limpar()">
 *   Não foi possível salvar. Código de suporte: {{ requestId }}
 * </acc-alert>
 * ```
 */
@Component({
  selector: 'acc-alert',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-alert" [class]="'acc-alert--' + variante()" [attr.role]="variante() === 'erro' ? 'alert' : 'status'">
      <div class="acc-alert__conteudo">
        @if (titulo()) {
          <p class="acc-alert__titulo">{{ titulo() }}</p>
        }
        <div class="acc-alert__texto"><ng-content></ng-content></div>
      </div>
      @if (descartavel()) {
        <button type="button" class="acc-alert__fechar" aria-label="Fechar alerta" (click)="fechado.emit()">
          ×
        </button>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-alert {
      display: flex; align-items: flex-start; justify-content: space-between; gap: 0.75rem;
      padding: 0.75rem 1rem; border: 1px solid transparent;
      border-radius: var(--acc-radius-md, 8px); font-size: 0.875rem;
    }
    .acc-alert--info { background: #eff6ff; border-color: #bfdbfe; color: #1e3a8a; }
    .acc-alert--sucesso { background: #ecfdf5; border-color: #a7f3d0; color: #065f46; }
    .acc-alert--aviso { background: #fffbeb; border-color: #fde68a; color: #92400e; }
    .acc-alert--erro { background: #fef2f2; border-color: #fecaca; color: #991b1b; }
    .acc-alert__titulo { margin: 0 0 0.25rem; font-weight: 600; }
    .acc-alert__texto { display: block; }
    .acc-alert__fechar {
      border: 0; background: transparent; color: inherit; cursor: pointer;
      font-size: 1.125rem; line-height: 1; padding: 0;
    }
    .acc-alert__fechar:focus-visible { outline: 2px solid currentColor; border-radius: 4px; }
  `,
})
export class AccAlertComponent {
  readonly variante = input<VarianteAlerta>('info');
  readonly titulo = input<string | null>(null);
  readonly descartavel = input<boolean>(false);
  readonly fechado = output<void>();
}
