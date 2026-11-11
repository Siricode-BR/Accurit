import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/**
 * Wrapper de layout para um campo de formulário: label associado,
 * obrigatoriedade visual, hint e erro padronizados. Use quando o campo
 * precisa de composição própria (ex.: grupo de inputs).
 *
 * ```html
 * <acc-form-field label="Período" hint="Competência mensal" [obrigatorio]="true">
 *   <acc-month-field formControlName="competencia" />
 * </acc-form-field>
 * ```
 */
@Component({
  selector: 'acc-form-field',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-form-field">
      @if (label()) {
        <span class="acc-form-field__label">
          {{ label() }}
          @if (obrigatorio()) {
            <span class="acc-form-field__obrigatorio" aria-hidden="true">*</span>
          }
        </span>
      }
      <ng-content></ng-content>
      @if (erro()) {
        <p class="acc-form-field__erro" role="alert">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-form-field__hint">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; min-width: 0; }
    .acc-form-field { display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-form-field__label {
      font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a);
    }
    .acc-form-field__obrigatorio { color: var(--acc-color-danger, #dc2626); margin-left: 0.125rem; }
    .acc-form-field__hint { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-form-field__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccFormFieldComponent {
  readonly label = input<string | null>(null);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);
  readonly obrigatorio = input<boolean>(false);
}
