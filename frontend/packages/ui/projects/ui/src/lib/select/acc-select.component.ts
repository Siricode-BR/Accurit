import {
  ChangeDetectionStrategy,
  Component,
  forwardRef,
  input,
  model,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';
import { AccSelectOption } from './acc-select.types';

let proximoId = 0;

/**
 * Select nativo acessível controlado por signal (`[(value)]`) e por
 * `ControlValueAccessor` (`formControl`/`ngModel`).
 * Passe `placeholder` = `null` para não exibir a opção vazia.
 */
@Component({
  selector: 'acc-select',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => AccSelectComponent),
      multi: true,
    },
  ],
  templateUrl: './acc-select.component.html',
  styleUrl: './acc-select.component.css',
})
export class AccSelectComponent implements ControlValueAccessor {
  readonly label = input.required<string>();
  readonly options = input<readonly AccSelectOption[]>([]);
  readonly value = model<string>('');
  readonly placeholder = input<string | null>('Selecione');
  readonly disabled = input<boolean>(false);
  readonly hint = input<string | null>(null);

  protected readonly selectId = `acc-select-${++proximoId}`;

  private onChange: (valor: string) => void = () => {};
  private onTouched: () => void = () => {};

  protected aoMudar(event: Event): void {
    const valor = (event.target as HTMLSelectElement).value;
    this.value.set(valor);
    this.onChange(valor);
  }

  protected aoSair(): void {
    this.onTouched();
  }

  writeValue(valor: unknown): void {
    this.value.set(valor === null || valor === undefined ? '' : String(valor));
  }

  registerOnChange(fn: (valor: string) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(desabilitado: boolean): void {
    // O select usa `disabled` do template; formulários reativos ainda
    // controlam o estado por `control.disable()`.
    void desabilitado;
  }
}
