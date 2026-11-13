import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
  OnInit,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  TrabalhoAgendadoDto,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import {
  AccAlertComponent,
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccPageHeaderComponent,
  AccSpinnerComponent,
  AccStatCardComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-trabalho-detalhes',
  standalone: true,
  imports: [
    CommonModule,
    AccAlertComponent,
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccPageHeaderComponent,
    AccSpinnerComponent,
    AccStatCardComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './trabalho-detalhes.component.html',
  styleUrl: './trabalho-detalhes.component.css',
})
export class TrabalhoDetalhesComponent implements OnInit {
  protected readonly route = inject(ActivatedRoute);
  protected readonly router = inject(Router);
  protected readonly trabalhosService = inject(TrabalhosService);

  protected readonly trabalho = signal<TrabalhoAgendadoDto | null>(null);
  protected readonly execucoes = this.trabalhosService.execucoes;
  protected readonly carregandoTrabalho = signal(false);
  protected readonly carregandoExecucoes = this.trabalhosService.carregandoExecucoes;
  protected readonly erro = signal<string | null>(null);

  async ngOnInit(): Promise<void> {
    const id = this.route.snapshot.paramMap.get('id');
    if (!id) {
      this.erro.set('ID do trabalho não informado.');
      return;
    }
    await this.carregarTrabalho(id);
    await this.trabalhosService.listarExecucoesPorTrabalho(id);
  }

  private async carregarTrabalho(id: string): Promise<void> {
    this.carregandoTrabalho.set(true);
    this.erro.set(null);
    try {
      const t = await this.trabalhosService.buscarTrabalho(id);
      this.trabalho.set(t);
    } catch (err: any) {
      this.erro.set(mensagemDeErro(err, 'Falha ao carregar o trabalho.'));
    } finally {
      this.carregandoTrabalho.set(false);
    }
  }

  protected rotuloTipo(tipo: string): string {
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

  protected rotuloStatus(status: string): string {
    switch (status) {
      case 'pendente':
        return 'Pendente';
      case 'executando':
        return 'Executando';
      case 'concluido':
        return 'Concluído';
      case 'falhou':
        return 'Falhou';
      case 'bloqueado':
        return 'Bloqueado';
      default:
        return status;
    }
  }

  protected badgeStatus(status: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'concluido':
        return 'success';
      case 'falhou':
        return 'danger';
      case 'executando':
        return 'info';
      case 'bloqueado':
      case 'pendente':
      default:
        return 'warning';
    }
  }

  protected rotuloAmbiente(ambiente: string): string {
    return ambiente === 'TRIAL' ? 'Trial' : 'Produção';
  }

  protected async alternar(): Promise<void> {
    const t = this.trabalho();
    if (!t) return;
    await this.trabalhosService.alternarTrabalho(t.id, {
      habilitado: !t.habilitado,
    });
    await this.carregarTrabalho(t.id);
  }

  protected async voltar(): Promise<void> {
    await this.router.navigate(['/trabalhos']);
  }
}
