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
  ConsultaLoteDto,
  ItemConsultaLoteDto,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-consulta-lote-detalhes',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consulta-lote-detalhes.component.html',
  styleUrl: './consulta-lote-detalhes.component.css',
})
export class ConsultaLoteDetalhesComponent implements OnInit {
  protected readonly route = inject(ActivatedRoute);
  protected readonly router = inject(Router);
  protected readonly trabalhosService = inject(TrabalhosService);

  protected readonly lote = signal<ConsultaLoteDto | null>(null);
  protected readonly itens = signal<ItemConsultaLoteDto[]>([]);
  protected readonly carregandoLote = signal(false);
  protected readonly carregandoItens = signal(false);
  protected readonly erro = signal<string | null>(null);

  async ngOnInit(): Promise<void> {
    const id = this.route.snapshot.paramMap.get('id');
    if (!id) {
      this.erro.set('ID do lote não informado.');
      return;
    }
    await this.carregarLote(id);
    await this.carregarItens(id);
  }

  private async carregarLote(id: string): Promise<void> {
    this.carregandoLote.set(true);
    this.erro.set(null);
    try {
      const l = await this.trabalhosService.buscarLote(id);
      this.lote.set(l);
    } catch (err: any) {
      this.erro.set(mensagemDeErro(err, 'Falha ao carregar o lote.'));
    } finally {
      this.carregandoLote.set(false);
    }
  }

  private async carregarItens(id: string): Promise<void> {
    this.carregandoItens.set(true);
    try {
      const itensLote = await this.trabalhosService.listarItensLote(id);
      this.itens.set(itensLote);
    } catch (err: any) {
      this.erro.set(mensagemDeErro(err, 'Falha ao carregar itens do lote.'));
    } finally {
      this.carregandoItens.set(false);
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
      case 'parcial':
        return 'Parcial';
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
      case 'parcial':
        return 'warning';
      case 'pendente':
      default:
        return 'warning';
    }
  }

  protected badgeItemStatus(status: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'concluido':
        return 'success';
      case 'falhou':
        return 'danger';
      case 'pendente':
      default:
        return 'warning';
    }
  }

  protected rotuloItemStatus(status: string): string {
    switch (status) {
      case 'concluido':
        return 'Concluído';
      case 'falhou':
        return 'Falhou';
      case 'pendente':
      default:
        return 'Pendente';
    }
  }

  protected tipoDocumento(documento: string): string {
    return documento.length === 11 ? 'PF' : 'PJ';
  }

  protected async voltar(): Promise<void> {
    await this.router.navigate(['/consultas-lote']);
  }
}
