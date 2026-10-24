import { Component, computed, inject, signal } from '@angular/core';

import { AmbienteSerproInfo, SerproAmbienteService } from '../../data/serpro.ambiente.service';

type EstadoSelo = 'carregando' | 'trial' | 'producao' | 'desconhecido';

/** Selo compartilhado do ambiente SERPRO ativo do escritório (D8, SPEC §8).
 * Consome o `SerproAmbienteService` (cache/single-flight por sessão) e
 * comunica o ambiente efetivo dos serviços do escritório ao usuário. */
@Component({
  selector: 'app-selo-ambiente',
  standalone: true,
  template: `
    @if (estado() !== 'desconhecido') {
      <div
        class="selo"
        [class]="'selo selo--' + estado()"
        title="Ambiente dos serviços SERPRO de uso no seu escritório"
      >
        {{ rotulo() }}
      </div>
    }
  `,
  styles: `
    :host {
      display: inline-block;
    }
  `,
})
export class SeloAmbienteComponent {
  private readonly serpro = inject(SerproAmbienteService);

  protected readonly estado = signal<EstadoSelo>('carregando');

  protected readonly rotulo = computed(() => {
    switch (this.estado()) {
      case 'trial':
        return 'SERPRO: Trial';
      case 'producao':
        return 'SERPRO: Produção';
      default:
        return 'SERPRO: verificando…';
    }
  });

  constructor() {
    this.serpro.obter().subscribe({
      next: (info) => this.estado.set(estadoDe(info)),
      error: () => this.estado.set('desconhecido'),
    });
  }
}

const estadoDe = (info: AmbienteSerproInfo): EstadoSelo =>
  info.ambienteEfetivo === 'PRODUCAO' ? 'producao' : 'trial';
