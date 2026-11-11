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
 * Campo de data (`input type="date"`, valor ISO `yyyy-mm-dd`),
 * com `ControlValueAccessor`, label, hint e erro.
 */
@Component({
  selector: 'acc-date-field',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccDateFieldComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-date" [class.acc-date--invalid]="!!erro()">
      @if (label()) {
        <label class="acc-date__label" [for]="id">{{ label() }}</label>
      }
      <input
        class="acc-date__control"
        type="date"
        [id]="id"
        [value]="valor()"
        [disabled]="desabilitado()"
        [attr.aria-invalid]="erro() ? 'true' : null"
        [attr.aria-describedby]="erro() ? id + '-erro' : hint() ? id + '-hint' : null"
        (input)="aoEntrada($event)"
        (blur)="aoSair()"
      />
      @if (erro()) {
        <p class="acc-date__erro" [id]="id + '-erro'">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-date__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-date { display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-date__label { font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a); }
    .acc-date__control {
      width: 100%; box-sizing: border-box; padding: 0.625rem 0.875rem;
      border: 1px solid var(--acc-color-border, #e2e8f0); border-radius: var(--acc-radius-md, 8px);
      background: var(--acc-color-surface, #fff); color: var(--acc-color-text-main, #0f172a);
      font-family: inherit; font-size: 0.875rem; outline: none;
    }
    .acc-date__control:focus-visible {
      border-color: var(--acc-color-primary, #1e40af); box-shadow: 0 0 0 3px rgb(30 64 175 / 0.15);
    }
    .acc-date--invalid .acc-date__control { border-color: var(--acc-color-danger, #dc2626); }
    .acc-date__hint { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-date__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccDateFieldComponent implements ControlValueAccessor {
  readonly label = input<string | null>(null);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);

  protected readonly id = `acc-date-${++proximoId}`;
  protected readonly valor = signal('');
  protected readonly desabilitado = signal(false);

  private onChange: (valor: string) => void = () => {};
  private onTouched: () => void = () => {};

  protected aoEntrada(evento: Event): void {
    const valor = (evento.target as HTMLInputElement).value;
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
