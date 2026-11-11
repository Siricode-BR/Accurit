import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { AccToastService } from './acc-toast.service';

/**
 * Região fixa de toasts (`aria-live="polite"`; erros usam `role="alert"`).
 * Renderize uma única vez na casca autenticada (`acc-app-shell`).
 */
@Component({
  selector: 'acc-toast-host',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="acc-toasts" aria-live="polite" aria-atomic="false">
      @for (toast of toasts(); track toast.id) {
        <div
          class="acc-toast"
          [class]="'acc-toast--' + toast.variante"
          [attr.role]="toast.variante === 'erro' ? 'alert' : 'status'"
        >
          <p class="acc-toast__mensagem">{{ toast.mensagem }}</p>
          <button
            type="button"
            class="acc-toast__fechar"
            aria-label="Fechar notificação"
            (click)="servico.fechar(toast.id)"
          >
            ×
          </button>
        </div>
      }
    </div>
  `,
  styles: `
    .acc-toasts {
      position: fixed; right: 1rem; bottom: 1rem; z-index: 1100;
      display: flex; flex-direction: column; gap: 0.5rem; max-width: min(24rem, calc(100vw - 2rem));
    }
    .acc-toast {
      display: flex; align-items: flex-start; justify-content: space-between; gap: 0.75rem;
      padding: 0.75rem 1rem; border: 1px solid transparent; border-radius: var(--acc-radius-md, 8px);
      font-size: 0.875rem; box-shadow: 0 10px 24px -12px rgb(15 23 42 / 0.4);
      background: var(--acc-color-surface, #fff);
    }
    .acc-toast--sucesso { background: #ecfdf5; border-color: #a7f3d0; color: #065f46; }
    .acc-toast--erro { background: #fef2f2; border-color: #fecaca; color: #991b1b; }
    .acc-toast--aviso { background: #fffbeb; border-color: #fde68a; color: #92400e; }
    .acc-toast--info { background: #eff6ff; border-color: #bfdbfe; color: #1e3a8a; }
    .acc-toast__mensagem { margin: 0; }
    .acc-toast__fechar {
      border: 0; background: transparent; color: inherit; cursor: pointer;
      font-size: 1.125rem; line-height: 1; padding: 0;
    }
    .acc-toast__fechar:focus-visible { outline: 2px solid currentColor; border-radius: 4px; }
  `,
})
export class AccToastHostComponent {
  private readonly toastService = inject(AccToastService);
  protected readonly servico = this.toastService;
  protected readonly toasts = this.toastService.toasts;
}
