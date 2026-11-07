import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { AccAppShellComponent, AccShellUsuario } from '@accurit/ui';
import { AuthService } from './core/auth/auth.service';
import { SessionStore } from './core/auth/session.store';
import { NAVEGACAO_ADMIN } from './core/layout/navegacao';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, AccAppShellComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App {
  private readonly auth = inject(AuthService);
  private readonly sessao = inject(SessionStore);

  protected readonly navegacao = NAVEGACAO_ADMIN;
  protected readonly autenticado = this.sessao.isAuthenticated;

  protected readonly usuario = computed<AccShellUsuario | null>(() => {
    const u = this.sessao.user();
    return u ? { nome: u.nome || u.email, detalhe: u.role } : null;
  });

  protected sair(): void {
    this.auth.logout();
  }
}
