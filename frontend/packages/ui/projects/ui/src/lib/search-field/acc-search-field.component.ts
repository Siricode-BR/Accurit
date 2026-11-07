import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  input,
  output,
  viewChild,
} from '@angular/core';
import { formatDocument } from '@accurit/shared';
import { AccIconComponent } from '../icon/acc-icon.component';

/**
 * Campo de busca com ícone e botão de limpar.
 *
 * Com `documentMask`, o texto é formatado como CPF/CNPJ somente enquanto contém
 * apenas dígitos e pontuação de documento — buscas por nome continuam livres.
 */
@Component({
  selector: 'acc-search-field',
  standalone: true,
  imports: [AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-search" [class.acc-search--invalid]="invalid()">
      <acc-icon class="acc-search__icon" name="search" />
      <input
        #campo
        type="search"
        class="acc-search__input"
        autocomplete="off"
        [attr.aria-label]="ariaLabel() || placeholder()"
        [placeholder]="placeholder()"
        [value]="value()"
        (input)="aoDigitar($event)"
      />
      @if (value()) {
        <button type="button" class="acc-search__clear" aria-label="Limpar busca" (click)="limpar()">
          <acc-icon name="close" />
        </button>
      }
    </div>
  `,
  styles: `
    :host {
      display: block;
    }

    .acc-search {
      position: relative;
      display: flex;
      align-items: center;
      height: 2.75rem;
      background-color: var(--acc-color-surface, #ffffff);
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-lg, 12px);
      transition: border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;
    }

    .acc-search:hover {
      border-color: #cbd5e1;
    }

    .acc-search:focus-within {
      border-color: var(--acc-color-primary, #1e40af);
      box-shadow: 0 0 0 3px rgb(30 64 175 / 0.14);
    }

    .acc-search--invalid,
    .acc-search--invalid:focus-within {
      border-color: var(--acc-color-warning, #b45309);
      box-shadow: 0 0 0 3px rgb(180 83 9 / 0.12);
    }

    .acc-search__icon {
      --acc-icon-size: 1.125rem;
      position: absolute;
      left: 0.875rem;
      color: var(--acc-color-text-muted, #64748b);
      pointer-events: none;
    }

    .acc-search__input {
      flex: 1;
      min-width: 0;
      height: 100%;
      padding: 0 2.5rem 0 2.75rem;
      border: 0;
      outline: none;
      background: transparent;
      font: inherit;
      font-size: 0.875rem;
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-search__input::placeholder {
      color: var(--acc-color-text-muted, #94a3b8);
    }

    .acc-search__input::-webkit-search-cancel-button {
      display: none;
    }

    .acc-search__clear {
      --acc-icon-size: 0.875rem;
      position: absolute;
      right: 0.5rem;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 1.75rem;
      height: 1.75rem;
      border: 0;
      border-radius: 999px;
      background: transparent;
      color: var(--acc-color-text-muted, #64748b);
      cursor: pointer;
    }

    .acc-search__clear:hover {
      background-color: var(--acc-color-bg, #f1f5f9);
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-search__clear:focus-visible {
      outline: 2px solid var(--acc-color-primary, #1e40af);
    }
  `,
})
export class AccSearchFieldComponent {
  readonly value = input<string>('');
  readonly placeholder = input<string>('Buscar...');
  readonly ariaLabel = input<string>('');
  /** Formata CPF/CNPJ enquanto o termo digitado parecer um documento. */
  readonly documentMask = input<boolean>(false);
  /** Destaca o campo em âmbar (ex.: documento incompleto). */
  readonly invalid = input<boolean>(false);

  readonly valueChange = output<string>();

  private readonly campo = viewChild.required<ElementRef<HTMLInputElement>>('campo');

  protected aoDigitar(event: Event): void {
    const el = event.target as HTMLInputElement;
    let texto = el.value;
    if (this.documentMask() && /^[\d.\-/\s]+$/.test(texto) && /\d/.test(texto)) {
      texto = formatDocument(texto.slice(0, 18));
      if (texto !== el.value) el.value = texto;
    }
    this.valueChange.emit(texto);
  }

  protected limpar(): void {
    this.campo().nativeElement.value = '';
    this.valueChange.emit('');
    this.campo().nativeElement.focus();
  }
}
