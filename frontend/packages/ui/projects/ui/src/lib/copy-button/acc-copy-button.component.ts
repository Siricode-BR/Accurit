import { Component, ChangeDetectionStrategy, OnDestroy, inject, input, signal } from '@angular/core';

/**
 * Botão de copiar (detector): copia texto cru (nunca HTML) e mostra
 * um breve "copiado" com anúncio em `aria-live`. Fallback para
 * `document.execCommand` em navegadores sem a Clipboard API.
 */
@Component({
  selector: 'acc-copy-button',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button
      type="button"
      class="acc-copy-button"
      (click)="copiar()"
      aria-label="Copiar valor"
    >
      @if (copiado()) {
        <svg class="acc-copy-button__ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
        </svg>
      } @else {
        <svg class="acc-copy-button__ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.75"
          />
        </svg>
      }
      <span aria-live="polite">{{ copiado() ? 'Copiado' : 'Copiar' }}</span>
    </button>
  `,
  styles: `
    .acc-copy-button {
      display: inline-flex;
      align-items: center;
      gap: 0.25rem;
      padding: 0.25rem 0.5rem;
      font-size: 0.75rem;
      font-weight: 600;
      font-family: inherit;
      color: var(--acc-color-text-muted, #64748b);
      background: transparent;
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-md, 8px);
      cursor: pointer;
      transition: color 0.15s ease, background-color 0.15s ease;
    }

    .acc-copy-button:focus-visible {
      outline: 2px solid var(--acc-color-primary, #1e40af);
      outline-offset: 1px;
    }

    .acc-copy-button:hover {
      color: var(--acc-color-text-main, #0f172a);
      background: var(--acc-color-bg, #f8fafc);
    }

    .acc-copy-button__ico {
      width: 0.75rem;
      height: 0.75rem;
    }
  `,
})
export class AccCopyButtonComponent implements OnDestroy {
  /** valor cru a copiar (ex.: CNPJ apenas com dígitos) */
  readonly valor = input.required<string>();

  #temporizador: ReturnType<typeof setTimeout> | null = null;
  #copiadoAberto = signal(false);

  /** estado transitório exibido por ~1,6 s após copiar */
  readonly copiado = this.#copiadoAberto.asReadonly();

  async copiar(): Promise<void> {
    const texto = this.valor();
    try {
      if (globalThis.navigator?.clipboard?.writeText) {
        await globalThis.navigator.clipboard.writeText(texto);
      } else {
        const area = document.createElement('textarea');
        area.value = texto;
        area.setAttribute('readonly', 'true');
        area.style.position = 'absolute';
        area.style.left = '-9999px';
        document.body.appendChild(area);
        area.select();
        document.execCommand('copy');
        area.remove();
      }
      this.#copiadoAberto.set(true);
      this.#agendar();
    } catch {
      // copia indisponível (permissão/ambiente) — seguir silenciosamente
    }
  }

  #agendar(): void {
    if (this.#temporizador) {
      clearTimeout(this.#temporizador);
    }
    this.#temporizador = setTimeout(() => this.#copiadoAberto.set(false), 1600);
  }

  ngOnDestroy(): void {
    if (this.#temporizador) {
      clearTimeout(this.#temporizador);
    }
  }
}
