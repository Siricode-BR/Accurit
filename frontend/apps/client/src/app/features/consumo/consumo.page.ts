import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { DatePipe, DecimalPipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RetencaoLgpdDto } from '@accurit/core';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccEmptyStateComponent,
  AccPageHeaderComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { ConsumoService } from '../../data/consumo.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';

@Component({
  selector: 'app-consumo-page',
  standalone: true,
  imports: [
    DatePipe,
    DecimalPipe,
    FormsModule,
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccEmptyStateComponent,
    AccPageHeaderComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-container">
      <acc-page-header
        title="Consumo SERPRO"
        subtitle="Chamadas do mês corrente, limites do plano e histórico por cliente."
      />

      @if (carregando()) {
        <acc-spinner />
      } @else if (erro()) {
        <acc-empty-state title="Consumo indisponível" [descricao]="erro()!" />
        <button acc-button variant="outline" (click)="recarregar()">Tentar novamente</button>
      } @else if (consumo(); as c) {
        @if (c.avisoConsumo) {
          <div class="aviso" [class.aviso--critico]="c.percentualUtilizacao >= 100" role="status">
            {{ c.avisoConsumo }}
          </div>
        }

        <div class="cards">
          <acc-card>
            <h2>Chamadas no mês</h2>
            <p class="numero">{{ c.chamadasUsadasMes | number }} / {{ c.chamadasMesLimite | number }}</p>
            <div class="barra" role="progressbar" [attr.aria-valuenow]="c.percentualUtilizacao" aria-valuemin="0" aria-valuemax="100">
              <span [style.width.%]="min(c.percentualUtilizacao, 100)"></span>
            </div>
            <p class="legenda">{{ c.percentualUtilizacao | number: '1.0-1' }}% do limite</p>
          </acc-card>
          <acc-card>
            <h2>Limites do plano</h2>
            <p>Clientes: <strong>{{ c.clientesLimite | number }}</strong></p>
            <p>Usuários: <strong>{{ c.usuariosLimite | number }}</strong></p>
            <p>Bloqueio ao exceder: <acc-badge [status]="c.bloqueioAoExceder ? 'warning' : 'neutral'">{{ c.bloqueioAoExceder ? 'Ativo' : 'Desativado' }}</acc-badge></p>
          </acc-card>
        </div>

        <h2 class="secao">Por produto e ambiente</h2>
        <div class="tabela-wrap">
          <table>
            <thead><tr><th>Produto</th><th>Ambiente</th><th>Período</th><th>Total</th><th>Sucesso</th><th>4xx</th><th>5xx</th><th>Timeout</th><th>Indeterminado</th></tr></thead>
            <tbody>
              @for (a of c.agregados; track a.produto + a.ambiente + a.periodo + a.referenciaPeriodo) {
                <tr>
                  <td>{{ a.produto }}</td>
                  <td><acc-badge [status]="a.ambiente === 'TRIAL' ? 'warning' : 'success'">{{ a.ambiente }}</acc-badge></td>
                  <td>{{ a.periodo }} · {{ a.referenciaPeriodo | date: 'dd/MM/yyyy' }}</td>
                  <td>{{ a.totalChamadas }}</td>
                  <td>{{ a.chamadasSucesso }}</td>
                  <td>{{ a.chamadas4xx }}</td>
                  <td>{{ a.chamadas5xx }}</td>
                  <td>{{ a.chamadasTimeout }}</td>
                  <td>{{ a.chamadasIndeterminado }}</td>
                </tr>
              } @empty {
                <tr><td colspan="9">Nenhuma chamada no período.</td></tr>
              }
            </tbody>
          </table>
        </div>

        <h2 class="secao">Histórico recente</h2>
        <div class="tabela-wrap">
          <table>
            <thead><tr><th>Quando</th><th>Produto</th><th>Serviço</th><th>Ambiente</th><th>Resultado</th><th>Status</th><th>Duração</th></tr></thead>
            <tbody>
              @for (ch of c.chamadasRecentes; track ch.id) {
                <tr>
                  <td>{{ ch.criadoEm | date: 'dd/MM/yyyy HH:mm' }}</td>
                  <td>{{ ch.produto }}</td>
                  <td>{{ ch.servico ?? ch.verbo }}</td>
                  <td>{{ ch.ambiente }}</td>
                  <td>{{ ch.resultado }}</td>
                  <td>{{ ch.statusHttp ?? '—' }}</td>
                  <td>{{ ch.duracaoMs }} ms</td>
                </tr>
              } @empty {
                <tr><td colspan="7">Nenhuma chamada registrada.</td></tr>
              }
            </tbody>
          </table>
        </div>

        <h2 class="secao">LGPD — retenção de dados</h2>
        <acc-card>
          <p class="legenda">Prazos em dias aplicados ao escritório. Alterações exigem perfil com escrita.</p>
          <form class="form" (ngSubmit)="salvarRetencao()">
            <label>Consultas fiscais <input type="number" min="1" max="3650" [(ngModel)]="retencao.consultasDias" name="consultas" /></label>
            <label>Relatórios <input type="number" min="1" max="3650" [(ngModel)]="retencao.relatoriosDias" name="relatorios" /></label>
            <label>Auditoria <input type="number" min="1" max="3650" [(ngModel)]="retencao.auditoriaDias" name="auditoria" /></label>
            <button acc-button type="submit" variant="primary" [disabled]="salvandoRetencao()">Salvar retenção</button>
          </form>
          @if (avisoRetencao()) { <p class="legenda" role="status">{{ avisoRetencao() }}</p> }
        </acc-card>
      }
    </div>
  `,
  styles: `
    .page-container { padding: 1rem; display: grid; gap: 1rem; }
    .cards { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); }
    .numero { font-size: 1.5rem; font-weight: 700; margin: .25rem 0; }
    .barra { height: 10px; background: #e5e7eb; border-radius: 999px; overflow: hidden; }
    .barra span { display: block; height: 100%; background: #2563eb; }
    .legenda { color: #4b5563; font-size: .875rem; }
    .aviso { background: #fff7e6; border: 1px solid #f0b429; padding: .75rem; border-radius: 8px; color: #6b4b00; }
    .aviso--critico { background: #fef2f2; border-color: #ef4444; color: #991b1b; }
    .secao { margin-top: .5rem; }
    .tabela-wrap { overflow-x: auto; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: .5rem; border-bottom: 1px solid #e5e7eb; text-align: left; white-space: nowrap; }
    .form { display: flex; flex-wrap: wrap; gap: .75rem; align-items: end; }
    .form label { display: grid; gap: .25rem; }
    .form input { width: 8rem; padding: .35rem; }
  `,
})
export class ConsumoPage implements OnInit {
  private readonly servico = inject(ConsumoService);

  protected readonly consumo = this.servico.consumo;
  protected readonly carregando = this.servico.carregando;
  protected readonly erro = this.servico.erro;
  protected readonly salvandoRetencao = signal(false);
  protected readonly avisoRetencao = signal<string | null>(null);

  protected retencao: RetencaoLgpdDto = {
    consultasDias: 90,
    relatoriosDias: 30,
    auditoriaDias: 365,
  };

  async ngOnInit(): Promise<void> {
    await this.recarregar();
    try {
      this.retencao = await this.servico.obterRetencao();
    } catch {
      // mantém os padrões exibidos
    }
  }

  protected async recarregar(): Promise<void> {
    await this.servico.carregar();
  }

  protected min(valor: number, maximo: number): number {
    return Math.min(valor, maximo);
  }

  protected async salvarRetencao(): Promise<void> {
    this.salvandoRetencao.set(true);
    this.avisoRetencao.set(null);
    try {
      this.retencao = await this.servico.salvarRetencao(this.retencao);
      this.avisoRetencao.set('Política de retenção salva.');
    } catch (erro) {
      this.avisoRetencao.set(mensagemDeErro(erro, 'Não foi possível salvar a retenção.'));
    } finally {
      this.salvandoRetencao.set(false);
    }
  }
}
