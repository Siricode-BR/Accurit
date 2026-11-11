import {
  ChangeDetectionStrategy,
  Component,
  forwardRef,
  input,
  signal,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

let proximoId = 0;

export type TipoInput =
  | 'text'
  | 'email'
  | 'password'
  | 'number'
  | 'tel'
  | 'search'
  | 'date'
  | 'month';

/**
 * Campo de texto do design system, com `ControlValueAccessor`
 * (`formControl`/`ngModel`), label associado, hint, erro acessível
 * (`aria-invalid`/`aria-describedby`), prefixo/sufixo, estados
 * disabled/readonly/loading e toggle de visibilidade da senha.
 *
 * ```html
 * <acc-input label="E-mail" type="email" formControlName="email" />
 * <acc-input label="Senha" type="password" formControlName="senha" />
 * ```
 */
@Component({
  selector: 'acc-input',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccInputComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-input" [class.acc-input--invalid]="!!erro()">
      @if (label()) {
        <label class="acc-input__label" [for]="id">{{ label() }}</label>
      }
      <div class="acc-input__wrapper">
        @if (prefixo()) {
          <span class="acc-input__affix" aria-hidden="true">{{ prefixo() }}</span>
        }
        <input
          class="acc-input__control"
          [id]="id"
          [type]="tipoEfetivo()"
          [placeholder]="placeholder() ?? ''"
          [attr.inputmode]="inputmode()"
          [attr.autocomplete]="autocomplete()"
          [disabled]="desabilitado()"
          [readOnly]="somenteLeitura()"
          [attr.aria-invalid]="erro() ? 'true' : null"
          [attr.aria-describedby]="descritoPor()"
          [value]="valor()"
          (input)="aoEntrada($event)"
          (blur)="aoSair()"
        />
        @if (tipo() === 'password') {
          <button
            type="button"
            class="acc-input__toggle"
            [attr.aria-label]="visivel() ? 'Ocultar senha' : 'Mostrar senha'"
            [attr.aria-pressed]="visivel()"
            (click)="alternarVisibilidade()"
          >
            {{ visivel() ? 'Ocultar' : 'Mostrar' }}
          </button>
        }
        @if (sufixo()) {
          <span class="acc-input__affix" aria-hidden="true">{{ sufixo() }}</span>
        }
        @if (carregando()) {
          <span class="acc-input__loading" aria-hidden="true"></span>
        }
      </div>
      @if (erro()) {
        <p class="acc-input__erro" [id]="id + '-erro'">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-input__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; min-width: 0; }
    .acc-input { display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-input__label {
      font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a);
    }
    .acc-input__wrapper {
      display: flex; align-items: center; gap: 0.5rem;
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-md, 8px);
      background-color: var(--acc-color-surface, #ffffff);
      padding: 0 0.875rem;
    }
    .acc-input__wrapper:focus-within {
      border-color: var(--acc-color-primary, #1e40af);
      box-shadow: 0 0 0 3px rgb(30 64 175 / 0.15);
    }
    .acc-input--invalid .acc-input__wrapper { border-color: var(--acc-color-danger, #dc2626); }
    .acc-input__control {
      flex: 1; min-width: 0; border: 0; outline: none; background: transparent;
      padding: 0.625rem 0; font-family: inherit; font-size: 0.875rem;
      color: var(--acc-color-text-main, #0f172a);
    }
    .acc-input__control:disabled { color: var(--acc-color-text-muted, #64748b); cursor: not-allowed; }
    .acc-input__affix { font-size: 0.8125rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-input__toggle {
      border: 0; background: transparent; padding: 0; cursor: pointer;
      font-size: 0.75rem; font-weight: 600; color: var(--acc-color-primary, #1e40af);
    }
    .acc-input__toggle:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); border-radius: 4px; }
    .acc-input__loading {
      width: 1rem; height: 1rem; border-radius: 50%;
      border: 2px solid var(--acc-color-border, #e2e8f0);
      border-top-color: var(--acc-color-primary, #1e40af);
      animation: acc-input-spin 0.8s linear infinite;
    }
    .acc-input__hint { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-input__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
    @keyframes acc-input-spin { to { transform: rotate(360deg); } }
  `,
})
export class AccInputComponent implements ControlValueAccessor {
  readonly label = input<string | null>(null);
  readonly tipo = input<TipoInput>('text');
  readonly placeholder = input<string | null>(null);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);
  readonly prefixo = input<string | null>(null);
  readonly sufixo = input<string | null>(null);
  readonly inputmode = input<string | null>(null);
  readonly autocomplete = input<string | null>(null);
  readonly somenteLeitura = input<boolean>(false);
  readonly carregando = input<boolean>(false);

  protected readonly id = `acc-input-${++proximoId}`;
  protected readonly valor = signal('');
  protected readonly desabilitado = signal(false);
  protected readonly visivel = signal(false);

  private onChange: (valor: string) => void = () => {};
  private onTouched: () => void = () => {};

  protected tipoEfetivo(): string {
    if (this.tipo() !== 'password') {
      return this.tipo();
    }
    return this.visivel() ? 'text' : 'password';
  }

  protected descritoPor(): string | null {
    if (this.erro()) {
      return `${this.id}-erro`;
    }
    return this.hint() ? `${this.id}-hint` : null;
  }

  protected aoEntrada(evento: Event): void {
    const valor = (evento.target as HTMLInputElement).value;
    this.valor.set(valor);
    this.onChange(valor);
  }

  protected aoSair(): void {
    this.onTouched();
  }

  protected alternarVisibilidade(): void {
    this.visivel.update((atual) => !atual);
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
