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
 * Interruptor (toggle) acessível com `ControlValueAccessor` booleano.
 * Usa `role="switch"` e responde a teclado pelo checkbox nativo interno.
 *
 * ```html
 * <acc-switch label="Rotina habilitada" formControlName="habilitado" />
 * ```
 */
@Component({
  selector: 'acc-switch',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccSwitchComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-switch">
      <label class="acc-switch__linha" [for]="id">
        <span class="acc-switch__label">{{ label() }}</span>
        <input
          class="acc-switch__input"
          type="checkbox"
          role="switch"
          [id]="id"
          [checked]="valor()"
          [disabled]="desabilitado()"
          [attr.aria-checked]="valor()"
          [attr.aria-describedby]="hint() ? id + '-hint' : null"
          (change)="aoMudar($event)"
          (blur)="aoSair()"
        />
        <span class="acc-switch__trilho" aria-hidden="true"><span class="acc-switch__bolinha"></span></span>
      </label>
      @if (hint()) {
        <p class="acc-switch__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-switch__linha {
      display: inline-flex; align-items: center; gap: 0.75rem;
      font-size: 0.875rem; color: var(--acc-color-text-main, #0f172a); cursor: pointer;
    }
    .acc-switch__input {
      position: absolute; opacity: 0; width: 1px; height: 1px;
    }
    .acc-switch__trilho {
      position: relative; width: 2.5rem; height: 1.375rem; border-radius: 999px;
      background-color: var(--acc-color-border, #cbd5e1);
      transition: background-color 0.15s ease;
    }
    .acc-switch__bolinha {
      position: absolute; top: 0.125rem; left: 0.125rem;
      width: 1.125rem; height: 1.125rem; border-radius: 50%;
      background-color: #ffffff; box-shadow: 0 1px 2px rgb(15 23 42 / 0.2);
      transition: transform 0.15s ease;
    }
    .acc-switch__input:checked + .acc-switch__trilho { background-color: var(--acc-color-primary, #1e40af); }
    .acc-switch__input:checked + .acc-switch__trilho .acc-switch__bolinha { transform: translateX(1.125rem); }
    .acc-switch__input:focus-visible + .acc-switch__trilho { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: 2px; }
    .acc-switch__input:disabled + .acc-switch__trilho { opacity: 0.5; cursor: not-allowed; }
    .acc-switch__hint { margin: 0.25rem 0 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
  `,
})
export class AccSwitchComponent implements ControlValueAccessor {
  readonly label = input.required<string>();
  readonly hint = input<string | null>(null);

  protected readonly id = `acc-switch-${++proximoId}`;
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
