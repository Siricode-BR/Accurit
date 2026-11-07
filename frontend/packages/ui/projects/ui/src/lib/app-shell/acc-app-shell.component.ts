import { ChangeDetectionStrategy, Component, computed, input, output, signal } from '@angular/core';
import { AccIconComponent } from '../icon/acc-icon.component';
import { AccSidebarComponent } from '../sidebar/acc-sidebar.component';
import { AccNavGroup } from '../sidebar/acc-sidebar.types';
import { AccShellUsuario } from './acc-app-shell.types';

/**
 * Casca da área autenticada: sidebar recolhível (desktop) / drawer (mobile) + topbar.
 *
 * Slots: `[acc-shell-topbar]` (selos/ações na barra superior) e conteúdo padrão (página).
 * Reutilizável pelos apps Angular (client e admin) — cada app só informa `groups`.
 */
@Component({
  selector: 'acc-app-shell',
  standalone: true,
  imports: [AccSidebarComponent, AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-app-shell.component.html',
  styleUrl: './acc-app-shell.component.css',
})
export class AccAppShellComponent {
  readonly groups = input.required<readonly AccNavGroup[]>();
  readonly brand = input<string>('Accurit');
  readonly portal = input<string>('');
  readonly usuario = input<AccShellUsuario | null>(null);
  readonly sair = output<void>();

  protected readonly recolhida = signal(false);
  protected readonly menuMobileAberto = signal(false);

  protected readonly iniciais = computed(() => {
    const nome = this.usuario()?.nome.trim() ?? '';
    return nome ? nome.slice(0, 2).toUpperCase() : '?';
  });

  protected fecharMenuMobile(): void {
    this.menuMobileAberto.set(false);
  }
}
