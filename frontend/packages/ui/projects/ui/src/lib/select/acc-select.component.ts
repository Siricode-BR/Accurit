import { ChangeDetectionStrategy, Component, input, model } from '@angular/core';
import { AccSelectOption } from './acc-select.types';

let proximoId = 0;

/**
 * Select nativo acessível controlado por signal (`[(value)]`).
 * Não depende de `ngModel`, então funciona dentro de qualquer `<form>` sem `name`.
 * Passe `placeholder` = `null` para não exibir a opção vazia.
 */
@Component({
  selector: 'acc-select',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-select.component.html',
  styleUrl: './acc-select.component.css',
})
export class AccSelectComponent {
  readonly label = input.required<string>();
  readonly options = input<readonly AccSelectOption[]>([]);
  readonly value = model<string>('');
  readonly placeholder = input<string | null>('Selecione');
  readonly disabled = input<boolean>(false);
  readonly hint = input<string | null>(null);

  protected readonly selectId = `acc-select-${++proximoId}`;

  protected aoMudar(event: Event): void {
    this.value.set((event.target as HTMLSelectElement).value);
  }
}
