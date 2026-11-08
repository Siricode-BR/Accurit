import { ChangeDetectionStrategy, Component, computed, effect, inject, untracked } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { AccAppShellComponent, AccShellUsuario } from '@accurit/ui';
import { AuthService } from './core/auth/auth.service';
import { SessionStore } from './core/auth/session.store';
import { NAVEGACAO_CLIENT } from './core/layout/navegacao';
import { ehTitular } from './core/auth/titular.guards';
import { SerproAmbienteService } from './data/serpro.ambiente.service';
import { RotuloAmbiente } from './app.types';

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
  private readonly ambiente = inject(SerproAmbienteService);

  /** Itens de administração (Usuários) só aparecem para o titular do escritório. */
  protected readonly navegacao = computed(() =>
    ehTitular(this.sessao.user()?.role)
      ? NAVEGACAO_CLIENT
      : NAVEGACAO_CLIENT.filter((g) => g.id !== 'administracao'),
  );
  protected readonly autenticado = this.sessao.isAuthenticated;
  protected readonly isTrial = this.ambiente.isTrial;

  protected readonly usuario = computed<AccShellUsuario | null>(() => {
    const u = this.sessao.user();
    return u ? { nome: u.nome || u.email, detalhe: u.role } : null;
  });

  protected readonly rotuloAmbiente = computed<RotuloAmbiente | null>(() => {
    const efetivo = this.ambiente.ambienteEfetivo();
    if (!efetivo) return null;
    return efetivo === 'TRIAL' ? 'Trial (avaliação)' : 'Produção';
  });

  constructor() {
    // Carrega o ambiente SERPRO sempre que houver sessão (troca de usuário invalida o cache).
    effect(() => {
      if (this.autenticado()) {
        untracked(() => this.ambiente.obter().subscribe({ error: () => undefined }));
      }
    });
  }

  protected sair(): void {
    this.auth.logout();
  }
}
