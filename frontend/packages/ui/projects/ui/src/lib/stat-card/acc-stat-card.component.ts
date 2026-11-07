import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { AccStatTone } from './acc-stat-card.types';

/** Indicador resumido (rótulo + valor) para dashboards. Não inventa valores: exiba só dados reais. */
@Component({
  selector: 'acc-stat-card',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-stat-card.component.html',
  styleUrl: './acc-stat-card.component.css',
})
export class AccStatCardComponent {
  readonly label = input.required<string>();
  readonly value = input<string | number>('—');
  readonly hint = input<string | null>(null);
  readonly tone = input<AccStatTone>('default');
  readonly loading = input<boolean>(false);
}
