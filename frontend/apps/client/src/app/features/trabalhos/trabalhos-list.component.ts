import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  TrabalhoAgendadoDto,
  TipoTrabalho,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-trabalhos-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './trabalhos-list.component.html',
  styleUrl: './trabalhos-list.component.css',
})
export class TrabalhosListComponent {
  protected readonly trabalhosService = inject(TrabalhosService);
  protected readonly router = inject(Router);

  protected readonly trabalhos = this.trabalhosService.trabalhos;
  protected readonly carregando = this.trabalhosService.carregandoTrabalhos;
  protected readonly erro = this.trabalhosService.erroTrabalhos;

  async ngOnInit(): Promise<void> {
    await this.trabalhosService.listarTrabalhos();
  }

  rotuloTipo(tipo: TipoTrabalho): string {
    switch (tipo) {
      case 'consulta_divida_ativa':
        return 'Consulta Dívida Ativa';
      case 'validar_procuracao':
        return 'Validar Procuração';
      case 'consulta_integra_contador':
        return 'Consulta Integra Contador';
      default:
        return tipo;
    }
  }

  badgeHabilitado(habilitado: boolean): 'success' | 'neutral' {
    return habilitado ? 'success' : 'neutral';
  }

  rotuloHabilitado(habilitado: boolean): string {
    return habilitado ? 'Habilitado' : 'Desabilitado';
  }

  async abrirCriacao(): Promise<void> {
    await this.router.navigate(['/trabalhos/novo']);
  }

  async abrirDetalhes(trabalhoId: string): Promise<void> {
    await this.router.navigate(['/trabalhos', trabalhoId]);
  }

  async alternar(trabalho: TrabalhoAgendadoDto): Promise<void> {
    await this.trabalhosService.alternarTrabalho(trabalho.id, {
      habilitado: !trabalho.habilitado,
    });
  }
}
