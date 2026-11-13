import {
  Component,
  ChangeDetectionStrategy,
  computed,
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
  AccAlertComponent,
  AccButtonComponent,
  AccCardComponent,
  AccPageHeaderComponent,
  AccSpinnerComponent,
  AccStatCardComponent,
  AccTableComponent,
} from '@accurit/ui';
import { AccColunaTabela } from '@accurit/ui';

const COLUNAS_ITENS: AccColunaTabela<Record<string, unknown>>[] = [
  { campo: 'documento', titulo: 'Documento' },
  { campo: 'tipo', titulo: 'Tipo' },
  { campo: 'status', titulo: 'Status' },
  { campo: 'tentativas', titulo: 'Tentativas', alinhamento: 'right' },
  { campo: 'erro', titulo: 'Erro' },
];

@Component({
  selector: 'app-consulta-lote-detalhes',
  standalone: true,
  imports: [
    CommonModule,
    AccAlertComponent,
    AccButtonComponent,
    AccCardComponent,
    AccPageHeaderComponent,
    AccSpinnerComponent,
    AccStatCardComponent,
    AccTableComponent,
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
  protected readonly colunasItens = COLUNAS_ITENS;
  protected readonly itensTabela = computed<Record<string, unknown>[]>(() =>
    this.itens().map((item) => ({
      documento: item.documento,
      tipo: item.tipo || this.tipoDocumento(item.documento),
      status: this.rotuloItemStatus(item.status),
      tentativas: item.tentativas,
      erro: item.erro ?? '—',
    })),
  );
  protected readonly progresso = computed(() => {
    const l = this.lote();
    if (!l || !l.totalItens) return 0;
    return Math.round((l.itensConcluidos / l.totalItens) * 100);
  });
  protected readonly carregandoLote = signal(false);
  protected readonly carregandoItens = signal(false);
  protected readonly reexecutando = signal(false);
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
      case 'pendente': return 'Pendente';
      case 'executando': return 'Executando';
      case 'concluido': return 'Concluído';
      case 'falhou': return 'Falhou';
      case 'parcial': return 'Parcial';
      case 'bloqueado': return 'Bloqueado';
      default: return status;
    }
  }

  protected badgeStatus(status: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'concluido': return 'success';
      case 'falhou': return 'danger';
      case 'executando': return 'info';
      default: return 'warning';
    }
  }

  protected rotuloItemStatus(status: string): string {
    switch (status) {
      case 'concluido': return 'Concluído';
      case 'falhou': return 'Falhou';
      case 'executando': return 'Executando';
      case 'bloqueado': return 'Bloqueado';
      case 'pendente': default: return 'Pendente';
    }
  }

  protected podeReexecutar(): boolean {
    const status = this.lote()?.status;
    return status === 'falhou' || status === 'parcial' || status === 'bloqueado';
  }

  protected async reexecutar(): Promise<void> {
    const lote = this.lote();
    if (!lote || this.reexecutando()) return;
    this.reexecutando.set(true);
    this.erro.set(null);
    try {
      await this.trabalhosService.reexecutarLote(lote.id);
      await this.carregarLote(lote.id);
      await this.carregarItens(lote.id);
    } catch (err: any) {
      this.erro.set(mensagemDeErro(err, 'Falha ao reexecutar a consulta em lote.'));
    } finally {
      this.reexecutando.set(false);
    }
  }

  protected tipoDocumento(documento: string): string {
    return documento.length === 11 ? 'PF' : 'PJ';
  }

  protected async voltar(): Promise<void> {
    await this.router.navigate(['/consultas-lote']);
  }
}
