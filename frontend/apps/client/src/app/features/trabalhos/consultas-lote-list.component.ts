import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  ConsultaLoteDto,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-consultas-lote-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consultas-lote-list.component.html',
  styleUrl: './consultas-lote-list.component.css',
})
export class ConsultasLoteListComponent {
  protected readonly trabalhosService = inject(TrabalhosService);
  protected readonly router = inject(Router);

  protected readonly lotes = this.trabalhosService.lotes;
  protected readonly carregando = this.trabalhosService.carregandoLotes;
  protected readonly erro = this.trabalhosService.erroLotes;

  async ngOnInit(): Promise<void> {
    await this.trabalhosService.listarLotes();
  }

  rotuloStatus(status: string): string {
    switch (status) {
      case 'pendente':
        return 'Pendente';
      case 'executando':
        return 'Executando';
      case 'concluido':
        return 'Concluído';
      case 'falhou':
        return 'Falhou';
      case 'parcial':
        return 'Parcial';
      default:
        return status;
    }
  }

  badgeStatus(status: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'concluido':
        return 'success';
      case 'falhou':
        return 'danger';
      case 'executando':
        return 'info';
      case 'parcial':
        return 'warning';
      case 'pendente':
      default:
        return 'warning';
    }
  }

  async abrirCriacao(): Promise<void> {
    await this.router.navigate(['/consultas-lote/novo']);
  }

  async abrirDetalhes(loteId: string): Promise<void> {
    await this.router.navigate(['/consultas-lote', loteId]);
  }
}
