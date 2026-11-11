import {
  ChangeDetectionStrategy,
  Component,
  forwardRef,
  input,
  signal,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

let proximoId = 0;

/**
 * Checkbox acessível com `ControlValueAccessor` (valor booleano), label
 * associado, hint e erro.
 *
 * ```html
 * <acc-checkbox label="Li e aceito" formControlName="aceite" />
 * ```
 */
@Component({
  selector: 'acc-checkbox',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccCheckboxComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-checkbox">
      <label class="acc-checkbox__linha" [for]="id">
        <input
          class="acc-checkbox__control"
          type="checkbox"
          [id]="id"
          [checked]="valor()"
          [disabled]="desabilitado()"
          [attr.aria-invalid]="erro() ? 'true' : null"
          [attr.aria-describedby]="erro() ? id + '-erro' : hint() ? id + '-hint' : null"
          (change)="aoMudar($event)"
          (blur)="aoSair()"
        />
        <span class="acc-checkbox__label">{{ label() }}</span>
      </label>
      @if (erro()) {
        <p class="acc-checkbox__erro" [id]="id + '-erro'">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-checkbox__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-checkbox { display: flex; flex-direction: column; gap: 0.25rem; }
    .acc-checkbox__linha {
      display: inline-flex; align-items: center; gap: 0.5rem;
      font-size: 0.875rem; color: var(--acc-color-text-main, #0f172a); cursor: pointer;
    }
    .acc-checkbox__control {
      width: 1rem; height: 1rem; accent-color: var(--acc-color-primary, #1e40af);
      cursor: pointer;
    }
    .acc-checkbox__control:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: 2px; }
    .acc-checkbox__control:disabled { cursor: not-allowed; }
    .acc-checkbox__hint { margin: 0 0 0 1.5rem; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-checkbox__erro { margin: 0 0 0 1.5rem; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccCheckboxComponent implements ControlValueAccessor {
  readonly label = input.required<string>();
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);

  protected readonly id = `acc-checkbox-${++proximoId}`;
  protected readonly valor = signal(false);
  protected readonly desabilitado = signal(false);

  private onChange: (valor: boolean) => void = () => {};
  private onTouched: () => void = () => {};

  protected aoMudar(evento: Event): void {
    const valor = (evento.target as HTMLInputElement).checked;
    this.valor.set(valor);
    this.onChange(valor);
  }

  protected aoSair(): void {
    this.onTouched();
  }

  writeValue(valor: unknown): void {
    this.valor.set(Boolean(valor));
  }

  registerOnChange(fn: (valor: boolean) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(desabilitado: boolean): void {
    this.desabilitado.set(desabilitado);
  }
}
