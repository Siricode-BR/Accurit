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
 * Campo de competência (`input type="month"`, valor ISO `yyyy-mm`),
 * com `ControlValueAccessor`, label, hint e erro.
 */
@Component({
  selector: 'acc-month-field',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccMonthFieldComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-month" [class.acc-month--invalid]="!!erro()">
      @if (label()) {
        <label class="acc-month__label" [for]="id">{{ label() }}</label>
      }
      <input
        class="acc-month__control"
        type="month"
        [id]="id"
        [value]="valor()"
        [disabled]="desabilitado()"
        [attr.aria-invalid]="erro() ? 'true' : null"
        [attr.aria-describedby]="erro() ? id + '-erro' : hint() ? id + '-hint' : null"
        (input)="aoEntrada($event)"
        (blur)="aoSair()"
      />
      @if (erro()) {
        <p class="acc-month__erro" [id]="id + '-erro'">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-month__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-month { display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-month__label { font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a); }
    .acc-month__control {
      width: 100%; box-sizing: border-box; padding: 0.625rem 0.875rem;
      border: 1px solid var(--acc-color-border, #e2e8f0); border-radius: var(--acc-radius-md, 8px);
      background: var(--acc-color-surface, #fff); color: var(--acc-color-text-main, #0f172a);
      font-family: inherit; font-size: 0.875rem; outline: none;
    }
    .acc-month__control:focus-visible {
      border-color: var(--acc-color-primary, #1e40af); box-shadow: 0 0 0 3px rgb(30 64 175 / 0.15);
    }
    .acc-month--invalid .acc-month__control { border-color: var(--acc-color-danger, #dc2626); }
    .acc-month__hint { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-month__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccMonthFieldComponent implements ControlValueAccessor {
  readonly label = input<string | null>(null);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);

  protected readonly id = `acc-month-${++proximoId}`;
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
