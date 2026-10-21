import { Component, computed, inject, signal } from '@angular/core';
import { SaudeService } from '../../data/saude.service';

type EstadoSaude = 'carregando' | 'ok' | 'indisponivel';

@Component({
  selector: 'app-dashboard',
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.css',
})
export class Dashboard {
  private readonly saude = inject(SaudeService);

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
}
