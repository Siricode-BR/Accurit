import { Component, ChangeDetectionStrategy, input } from '@angular/core';

export type AccSkeletonVariante = 'texto' | 'linha' | 'card' | 'circulo';

/**
 * Placeholder de carregamento com brilho animado. A quantidade de
 * repetição fica em composição na tela hospedeira (`@for`).
 */
@Component({
  selector: 'acc-skeleton',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<div class="acc-skeleton__bloco"></div>`,
  host: {
    '[class]': '"acc-skeleton acc-skeleton--" + variante()',
    '[style.width]': 'largura()',
    '[style.height]': 'altura()',
  },
  styles: `
    :host {
      display: block;
    }

    .acc-skeleton__bloco {
      width: 100%;
      height: 100%;
      border-radius: var(--acc-radius-md, 8px);
      background: linear-gradient(
        100deg,
        var(--acc-color-skeleton, #e2e8f0) 40%,
        #f1f5f9 50%,
        var(--acc-color-skeleton, #e2e8f0) 60%
      );
      background-size: 200% 100%;
      animation: acc-skeleton-brilho 1.4s ease-in-out infinite;
    }

    :host(.acc-skeleton--circulo) .acc-skeleton__bloco {
      border-radius: 9999px;
    }

    @keyframes acc-skeleton-brilho {
      0% {
        background-position: 120% 0;
      }
      100% {
        background-position: -80% 0;
      }
    }

    @media (prefers-reduced-motion: reduce) {
      .acc-skeleton__bloco {
        animation: none;
      }
    }
  `,
})
export class AccSkeletonComponent {
  readonly variante = input<AccSkeletonVariante>('texto');
  /** dimensões explícitas (ex.: '3rem', '100%'); as variantes têm default */
  readonly largura = input<string | null>(null);
  readonly altura = input<string | null>(null);
}
