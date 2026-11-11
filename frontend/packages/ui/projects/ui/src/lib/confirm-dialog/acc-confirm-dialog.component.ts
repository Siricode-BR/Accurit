import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { AccModalComponent } from '../modal/acc-modal.component';
import { AccButtonComponent } from '../button/acc-button.component';

/**
 * Diálogo de confirmação para ações destrutivas ou ativar/desativar,
 * construído sobre `acc-modal`. Renderize com `@if` (existir = aberto).
 *
 * ```html
 * @if (confirmando()) {
 *   <acc-confirm-dialog titulo="Desativar usuário" mensagem="Esta ação revoga as sessões."
 *     variante="danger" (confirmar)="desativar()" (cancelar)="confirmando.set(false)" />
 * }
 * ```
 */
@Component({
  selector: 'acc-confirm-dialog',
  standalone: true,
  imports: [AccModalComponent, AccButtonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <acc-modal [title]="titulo()" [subtitle]="subtitulo()" (fechar)="cancelado.emit()">
      <p class="acc-confirm-dialog__mensagem">{{ mensagem() }}</p>
      <div class="acc-confirm-dialog__acoes">
        <button acc-button type="button" variant="outline" (click)="cancelado.emit()">
          {{ cancelarTexto() }}
        </button>
        <button
          acc-button
          type="button"
          [variant]="variante() === 'danger' ? 'danger' : 'primary'"
          [loading]="processando()"
          (click)="confirmar.emit()"
        >
          {{ confirmarTexto() }}
        </button>
      </div>
    </acc-modal>
  `,
  styles: `
    .acc-confirm-dialog__mensagem {
      margin: 0 0 1.25rem; font-size: 0.875rem; color: var(--acc-color-text-main, #0f172a);
      white-space: pre-line;
    }
    .acc-confirm-dialog__acoes { display: flex; justify-content: flex-end; gap: 0.5rem; flex-wrap: wrap; }
  `,
})
export class AccConfirmDialogComponent {
  readonly titulo = input.required<string>();
  readonly subtitulo = input<string | null>(null);
  readonly mensagem = input.required<string>();
  readonly confirmarTexto = input<string>('Confirmar');
  readonly cancelarTexto = input<string>('Cancelar');
  readonly variante = input<'danger' | 'primary'>('danger');
  readonly processando = input<boolean>(false);

  readonly confirmar = output<void>();
  readonly cancelado = output<void>();
}
