import { Component, computed, inject, signal } from '@angular/core';
import { Router } from '@angular/router';

import { AuthService } from '../../core/auth/auth.service';
import { SessionStore } from '../../core/auth/session.store';
import { SaudeService } from '../../data/saude.service';
import { SerproConfiguracaoService } from '../../data/serpro.configuracao.service';

type EstadoSaude = 'carregando' | 'ok' | 'indisponivel';
type EstadoSerpro = 'carregando' | 'trial' | 'producao' | 'desconhecido';

@Component({
  selector: 'app-inicio',
  templateUrl: './inicio.html',
  styleUrl: './inicio.css',
})
export class Inicio {
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
        return 'Plataforma disponível';
      case 'indisponivel':
        return 'Plataforma indisponível';
      default:
        return 'Verificando plataforma';
    }
  });
  protected readonly descricao = computed(() => {
    switch (this.estado()) {
      case 'ok':
        return 'Conexão com a API e o banco de dados confirmada.';
      case 'indisponivel':
        return 'Não foi possível confirmar a conexão com a plataforma.';
      default:
        return 'Consultando o estado atual da plataforma.';
    }
  });

  private readonly serpro = inject(SerproConfiguracaoService);

  protected readonly estadoSerpro = signal<EstadoSerpro>('carregando');
  protected readonly rotuloSerpro = computed(() => {
    switch (this.estadoSerpro()) {
      case 'trial':
        return 'SERPRO: Trial';
      case 'producao':
        return 'SERPRO: Produção';
      default:
        return 'SERPRO: verificando…';
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
    this.serpro.obterConfiguracao().subscribe({
      next: (configuracao) =>
        this.estadoSerpro.set(configuracao.ambienteEfetivo === 'PRODUCAO' ? 'producao' : 'trial'),
      error: () => this.estadoSerpro.set('desconhecido'),
    });
  }

  protected sair(): void {
    this.auth.logout();
    void this.router.navigateByUrl('/login');
  }
}
