import {
  ChangeDetectionStrategy,
  Component,
  DOCUMENT,
  ElementRef,
  OnDestroy,
  afterNextRender,
  inject,
  input,
  output,
  viewChild,
} from '@angular/core';
import { AccIconComponent } from '../icon/acc-icon.component';

let proximoId = 0;

/**
 * Janela modal (popup) com título, botão de fechar, `Esc` e clique no fundo.
 * Renderize-a com `@if` no componente hospedeiro: existir = aberta.
 * Bloqueia a rolagem da página e devolve o foco ao elemento anterior ao fechar.
 */
@Component({
  selector: 'acc-modal',
  standalone: true,
  imports: [AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { '(document:keydown.escape)': 'fechar.emit()' },
  template: `
    <div class="acc-modal__backdrop" (mousedown)="aoClicarNoFundo($event)">
      <div
        #painel
        class="acc-modal__panel"
        [class.acc-modal__panel--lg]="size() === 'lg'"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        [attr.aria-labelledby]="idTitulo"
      >
        <header class="acc-modal__header">
          <div>
            <h2 class="acc-modal__title" [id]="idTitulo">{{ title() }}</h2>
            @if (subtitle()) {
              <p class="acc-modal__subtitle">{{ subtitle() }}</p>
            }
          </div>
          <button type="button" class="acc-modal__close" aria-label="Fechar" (click)="fechar.emit()">
            <acc-icon name="close" />
          </button>
        </header>
        <div class="acc-modal__body">
          <ng-content></ng-content>
        </div>
      </div>
    </div>
  `,
  styles: `
    .acc-modal__backdrop {
      position: fixed;
      inset: 0;
      z-index: 1000;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 1rem;
      background-color: rgb(15 23 42 / 0.5);
      backdrop-filter: blur(2px);
      animation: acc-modal-fade 0.15s ease-out;
    }

    .acc-modal__panel {
      display: flex;
      flex-direction: column;
      width: 100%;
      max-width: 36rem;
      max-height: calc(100dvh - 2rem);
      background-color: var(--acc-color-surface, #ffffff);
      border: 1px solid var(--acc-color-border, #e2e8f0);
      border-radius: var(--acc-radius-lg, 12px);
      box-shadow: 0 24px 48px -12px rgb(15 23 42 / 0.3);
      outline: none;
      animation: acc-modal-up 0.2s ease-out;
    }

    .acc-modal__panel--lg {
      max-width: 46rem;
    }

    .acc-modal__header {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: 1rem;
      padding: 1.25rem 1.5rem 1rem;
      border-bottom: 1px solid var(--acc-color-border, #e2e8f0);
    }

    .acc-modal__title {
      margin: 0;
      font-size: 1.125rem;
      font-weight: 700;
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-modal__subtitle {
      margin: 0.25rem 0 0;
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted, #64748b);
    }

    .acc-modal__close {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex: 0 0 auto;
      width: 2.25rem;
      height: 2.25rem;
      border: 0;
      border-radius: var(--acc-radius-md, 8px);
      background: transparent;
      color: var(--acc-color-text-muted, #64748b);
      cursor: pointer;
    }

    .acc-modal__close:hover {
      background-color: var(--acc-color-bg, #f1f5f9);
      color: var(--acc-color-text-main, #0f172a);
    }

    .acc-modal__close:focus-visible {
      outline: 2px solid var(--acc-color-primary, #1e40af);
    }

    .acc-modal__body {
      padding: 1.25rem 1.5rem 1.5rem;
      overflow-y: auto;
    }

    @keyframes acc-modal-fade {
      from { opacity: 0; }
    }

    @keyframes acc-modal-up {
      from { opacity: 0; transform: translateY(8px) scale(0.98); }
    }
  `,
})
export class AccModalComponent implements OnDestroy {
  readonly title = input.required<string>();
  readonly subtitle = input<string | null>(null);
  readonly size = input<'md' | 'lg'>('md');
  readonly fechar = output<void>();

  protected readonly idTitulo = `acc-modal-titulo-${++proximoId}`;

  private readonly doc = inject(DOCUMENT);
  private readonly painel = viewChild.required<ElementRef<HTMLElement>>('painel');
  private readonly focoAnterior = this.doc.activeElement as HTMLElement | null;
  private readonly overflowAnterior = this.doc.body.style.overflow;

  constructor() {
    this.doc.body.style.overflow = 'hidden';
    afterNextRender(() => {
      const painel = this.painel().nativeElement;
      const primeiro = painel.querySelector<HTMLElement>(
        'input:not([type=hidden]):not([disabled]), select:not([disabled]), textarea:not([disabled])',
      );
      (primeiro ?? painel).focus();
    });
  }

  ngOnDestroy(): void {
    this.doc.body.style.overflow = this.overflowAnterior;
    this.focoAnterior?.focus?.();
  }

  protected aoClicarNoFundo(event: MouseEvent): void {
    if (event.target === event.currentTarget) this.fechar.emit();
  }
}
