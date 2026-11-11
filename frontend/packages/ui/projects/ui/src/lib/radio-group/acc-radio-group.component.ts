import {
  ChangeDetectionStrategy,
  Component,
  forwardRef,
  input,
  signal,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

let proximoId = 0;

export interface AccRadioOption {
  readonly value: string;
  readonly label: string;
  readonly hint?: string;
}

/**
 * Grupo de rádio acessível com `ControlValueAccessor` (valor string),
 * navegável por teclado e com `aria-describedby` para erro/hint.
 *
 * ```html
 * <acc-radio-group label="Tipo" [options]="tipos" formControlName="tipo" />
 * ```
 */
@Component({
  selector: 'acc-radio-group',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccRadioGroupComponent),
      multi: true,
    },
  ],
  template: `
    <fieldset class="acc-radio" [attr.aria-invalid]="erro() ? 'true' : null">
      @if (label()) {
        <legend class="acc-radio__label">{{ label() }}</legend>
      }
      @for (opcao of options(); track opcao.value) {
        <label class="acc-radio__opcao">
          <input
            type="radio"
            [name]="nome"
            [value]="opcao.value"
            [checked]="valor() === opcao.value"
            [disabled]="desabilitado()"
            (change)="aoMudar(opcao.value)"
            (blur)="aoSair()"
          />
          <span>{{ opcao.label }}</span>
          @if (opcao.hint) {
            <small class="acc-radio__hint">{{ opcao.hint }}</small>
          }
        </label>
      }
      @if (erro()) {
        <p class="acc-radio__erro">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-radio__hint">{{ hint() }}</p>
      }
    </fieldset>
  `,
  styles: `
    :host { display: block; }
    .acc-radio { border: 0; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-radio__label {
      padding: 0; font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a);
    }
    .acc-radio__opcao {
      display: inline-flex; align-items: center; gap: 0.5rem;
      font-size: 0.875rem; color: var(--acc-color-text-main, #0f172a); cursor: pointer;
    }
    .acc-radio__opcao input { accent-color: var(--acc-color-primary, #1e40af); }
    .acc-radio__opcao input:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: 2px; }
    .acc-radio__hint { font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-radio__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccRadioGroupComponent implements ControlValueAccessor {
  readonly label = input<string | null>(null);
  readonly options = input<readonly AccRadioOption[]>([]);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);

  protected readonly nome = `acc-radio-${++proximoId}`;
  protected readonly valor = signal('');
  protected readonly desabilitado = signal(false);

  private onChange: (valor: string) => void = () => {};
  private onTouched: () => void = () => {};

  protected aoMudar(valor: string): void {
    this.valor.set(valor);
    this.onChange(valor);
  }

  protected aoSair(): void {
    this.onTouched();
  }

  writeValue(valor: unknown): void {
    this.valor.set(valor === null || valor === undefined ? '' : String(valor));
  }

  registerOnChange(fn: (valor: string) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(desabilitado: boolean): void {
    this.desabilitado.set(desabilitado);
  }
}
