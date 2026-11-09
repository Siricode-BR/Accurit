import {
  Component,
  OnInit,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  RelatorioDto,
  RelatorioResumoDto,
  CriarRelatorioRequestDto,
  TipoRelatorio,
  FormatoRelatorio,
  LinkCompartilhamentoDto,
} from '@accurit/core';
import { RelatoriosService } from '../../data/relatorios.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
  AccCtaButtonComponent,
  AccIconComponent,
  AccModalComponent,
} from '@accurit/ui';

type TipoRelatorioOpcao = { value: TipoRelatorio; label: string };
type FormatoRelatorioOpcao = { value: FormatoRelatorio; label: string };

const TIPOS_RELATORIO: readonly TipoRelatorioOpcao[] = [
  { value: 'situacao_fiscal', label: 'Situação Fiscal' },
  { value: 'historico_consultas', label: 'Histórico de Consultas' },
  { value: 'pendencias_carteira', label: 'Pendências por Carteira' },
];

const FORMATOS_RELATORIO: readonly FormatoRelatorioOpcao[] = [
  { value: 'PDF', label: 'PDF' },
  { value: 'XLSX', label: 'XLSX' },
];

@Component({
  selector: 'app-relatorios-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
    AccCtaButtonComponent,
    AccIconComponent,
    AccModalComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './relatorios-list.component.html',
  styleUrl: './relatorios-list.component.css',
})
export class RelatoriosListComponent implements OnInit {
  protected readonly service = inject(RelatoriosService);
  protected readonly router = inject(Router);

  protected readonly relatorios = this.service.relatorios;
  protected readonly carregando = this.service.carregando;
  protected readonly erro = this.service.erro;

  protected readonly modalGerarAberto = signal(false);
  protected readonly tipoSelecionado = signal<TipoRelatorio>('situacao_fiscal');
  protected readonly formatoSelecionado = signal<FormatoRelatorio>('PDF');

  protected readonly linkCompartilhamento = signal<LinkCompartilhamentoDto | null>(null);

  protected readonly tiposRelatorio = TIPOS_RELATORIO;
  protected readonly formatosRelatorio = FORMATOS_RELATORIO;

  ngOnInit(): void {
    void this.service.listar();
  }

  protected rotuloTipo(tipo: string): string {
    const opcao = TIPOS_RELATORIO.find((t) => t.value === tipo);
    return opcao?.label ?? tipo;
  }

  protected rotuloFormato(formato: string): string {
    const opcao = FORMATOS_RELATORIO.find((f) => f.value === formato);
    return opcao?.label ?? formato;
  }

  protected rotuloStatus(status: string): string {
    switch (status) {
      case 'concluido':
        return 'Concluído';
      case 'gerando':
        return 'Gerando';
      case 'falhou':
        return 'Falhou';
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
      case 'gerando':
        return 'info';
      default:
        return 'warning';
    }
  }

  protected async gerarRelatorio(): Promise<void> {
    const requisicao: CriarRelatorioRequestDto = {
      tipoRelatorio: this.tipoSelecionado(),
      formato: this.formatoSelecionado(),
    };
    try {
      await this.service.gerar(requisicao);
      this.fecharModalGerar();
    } catch {
      // erro já exibido pelo serviço
    }
  }

  protected abrirModalGerar(): void {
    this.modalGerarAberto.set(true);
  }

  protected fecharModalGerar(): void {
    this.modalGerarAberto.set(false);
  }

  protected selecionarTipo(valor: string): void {
    this.tipoSelecionado.set(valor as TipoRelatorio);
  }

  protected selecionarFormato(valor: string): void {
    this.formatoSelecionado.set(valor as FormatoRelatorio);
  }

  protected async baixarRelatorio(id: string): Promise<void> {
    try {
      const blob = await this.service.baixar(id);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'relatorio';
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch {
      // erro já exibido pelo serviço
    }
  }

  protected async abrirModalCompartilhamento(id: string): Promise<void> {
    try {
      const link = await this.service.criarLinkCompartilhamento(id, {
        expiraEm: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(),
      });
      this.linkCompartilhamento.set(link);
    } catch {
      // erro já exibido pelo serviço
    }
  }

  protected fecharModalCompartilhamento(): void {
    this.linkCompartilhamento.set(null);
  }

  protected limparErro(): void {
    this.service.limpar();
  }

  protected copiarLink(token: string): void {
    const url = this.urlCompartilhamento(token);
    navigator.clipboard.writeText(url).then(() => {
      // sucesso silencioso
    });
  }

  protected urlCompartilhamento(token: string): string {
    return (
      this.router.serializeUrl(
        this.router.createUrlTree(['/relatorios/compartilhado', token]),
      ) || window.location.origin + '/relatorios/compartilhado/' + token
    );
  }
}
