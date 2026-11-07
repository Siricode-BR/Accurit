import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { AccIconComponent } from '../icon/acc-icon.component';
import { AccNavGroup } from './acc-sidebar.types';

/** Lista de navegação primária agrupada. Usada dentro do `acc-app-shell`. */
@Component({
  selector: 'acc-sidebar',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-sidebar.component.html',
  styleUrl: './acc-sidebar.component.css',
})
export class AccSidebarComponent {
  readonly groups = input.required<readonly AccNavGroup[]>();
  readonly collapsed = input<boolean>(false);
  /** Emitido ao clicar em um destino (o shell usa para fechar o drawer mobile). */
  readonly navegou = output<void>();
}
