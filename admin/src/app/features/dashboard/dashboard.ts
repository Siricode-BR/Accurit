import { Component, computed, inject, signal } from '@angular/core';
import { Router } from '@angular/router';

import { AuthService } from '../../core/auth/auth.service';
import { SessionStore } from '../../core/auth/session.store';
import { SaudeService } from '../../data/saude.service';

type EstadoSaude = 'carregando' | 'ok' | 'indisponivel';

@Component({
  selector: 'app-dashboard',
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.css',
})
export class Dashboard {
  private readonly saude = inject(SaudeService);
  private readonly auth = inject(AuthService);
  private readonly sessao = inject(SessionStore);
  private readonly router = inject(Router);

  protected readonly usuario = this.sessao.usuario;
  protected readonly email = computed(() => this.usuario()?.email ?? '');

  protected readonly estado = signal<EstadoSaude>('carregando');
  protected readonly rotulo = computed(() => {
    switch (this.estado()) {
      case 'ok':
        return 'Serviços operacionais';
      case 'indisponivel':
        return 'Serviços indisponíveis';
      default:
        return 'Verificando serviços';
    }
  });
  protected readonly descricao = computed(() => {
    switch (this.estado()) {
      case 'ok':
        return 'API e banco de dados respondendo normalmente.';
      case 'indisponivel':
        return 'A API ou o banco de dados não respondeu à verificação.';
      default:
        return 'Consultando o estado atual da plataforma.';
    }
  });

  constructor() {
    this.saude.verificar().subscribe({
      next: (resposta) => {
        this.estado.set(
          resposta.status === 'ok' && resposta.db === 'ok' ? 'ok' : 'indisponivel',
        );
      },
      error: () => this.estado.set('indisponivel'),
    });
  }

  protected sair(): void {
    this.auth.logout();
    void this.router.navigateByUrl('/login');
  }
}
