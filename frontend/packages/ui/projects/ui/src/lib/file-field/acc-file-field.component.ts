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
 * Campo de arquivo do design system (`ControlValueAccessor<File | null>`),
 * o único lugar onde um `<input type="file">` cru é permitido nas telas.
 *
 * ```html
 * <acc-file-field label="Planilha" accept=".csv,.xlsx" formControlName="arquivo" />
 * ```
 */
@Component({
  selector: 'acc-file-field',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccFileFieldComponent),
      multi: true,
    },
  ],
  template: `
    <div class="acc-file" [class.acc-file--invalid]="!!erro()">
      @if (label()) {
        <label class="acc-file__label" [for]="id">{{ label() }}</label>
      }
      <div class="acc-file__linha">
        <input
          class="acc-file__input"
          type="file"
          [id]="id"
          [accept]="accept()"
          [disabled]="desabilitado()"
          [attr.aria-invalid]="erro() ? 'true' : null"
          [attr.aria-describedby]="erro() ? id + '-erro' : hint() ? id + '-hint' : null"
          (change)="aoSelecionar($event)"
          (blur)="aoSair()"
        />
        @if (arquivo()) {
          <button type="button" class="acc-file__limpar" (click)="limpar()">Remover</button>
        }
      </div>
      <p class="acc-file__nome">{{ arquivo()?.name ?? 'Nenhum arquivo selecionado' }}</p>
      @if (erro()) {
        <p class="acc-file__erro" [id]="id + '-erro'">{{ erro() }}</p>
      } @else if (hint()) {
        <p class="acc-file__hint" [id]="id + '-hint'">{{ hint() }}</p>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .acc-file { display: flex; flex-direction: column; gap: 0.375rem; }
    .acc-file__label { font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main, #0f172a); }
    .acc-file__linha { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
    .acc-file__input { font-size: 0.875rem; color: var(--acc-color-text-main, #0f172a); }
    .acc-file__input:focus-visible { outline: 2px solid var(--acc-color-primary, #1e40af); outline-offset: 2px; }
    .acc-file--invalid .acc-file__input { color: var(--acc-color-danger, #dc2626); }
    .acc-file__limpar {
      border: 1px solid var(--acc-color-border, #e2e8f0); background: transparent;
      color: var(--acc-color-text-muted, #64748b); border-radius: var(--acc-radius-md, 8px);
      padding: 0.25rem 0.5rem; font-size: 0.75rem; cursor: pointer;
    }
    .acc-file__nome { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-file__hint { margin: 0; font-size: 0.75rem; color: var(--acc-color-text-muted, #64748b); }
    .acc-file__erro { margin: 0; font-size: 0.75rem; color: var(--acc-color-danger, #dc2626); }
  `,
})
export class AccFileFieldComponent implements ControlValueAccessor {
  readonly label = input<string | null>(null);
  readonly accept = input<string | null>(null);
  readonly hint = input<string | null>(null);
  readonly erro = input<string | null>(null);

  protected readonly id = `acc-file-${++proximoId}`;
  protected readonly arquivo = signal<File | null>(null);
  protected readonly desabilitado = signal(false);

  private onChange: (valor: File | null) => void = () => {};
  private onTouched: () => void = () => {};

  protected aoSelecionar(evento: Event): void {
    const arquivo = (evento.target as HTMLInputElement).files?.[0] ?? null;
    this.arquivo.set(arquivo);
    this.onChange(arquivo);
  }

  protected limpar(): void {
    this.arquivo.set(null);
    this.onChange(null);
  }

  protected aoSair(): void {
    this.onTouched();
  }

  writeValue(valor: unknown): void {
    this.arquivo.set(valor instanceof File ? valor : null);
  }

  registerOnChange(fn: (valor: File | null) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(desabilitado: boolean): void {
    this.desabilitado.set(desabilitado);
  }
}
