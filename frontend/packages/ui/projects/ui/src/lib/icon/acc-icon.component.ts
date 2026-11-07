import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { ACC_ICON_PATHS } from './acc-icon.paths';
import { AccIconName } from './acc-icon.types';

/** Ícone decorativo do design system. Tamanho via `--acc-icon-size` (padrão 1.25rem). */
@Component({
  selector: 'acc-icon',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-icon.component.html',
  styleUrl: './acc-icon.component.css',
})
export class AccIconComponent {
  readonly name = input.required<AccIconName>();

  protected readonly caminho = computed(() => ACC_ICON_PATHS[this.name()]);
}
