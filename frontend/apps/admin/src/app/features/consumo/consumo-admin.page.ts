import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ConsumoAdminDto, PlanoEscritorioDto } from '@accurit/core';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccPageHeaderComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { ConsumoAdminService, FiltrosConsumo } from '../../data/consumo-admin.service';
import { TenantService } from '../../data/tenant.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';

type Aba = 'consumo' | 'planos';

@Component({
  selector: 'app-consumo-admin-page',
  standalone: true,
  imports: [
    DecimalPipe,
    FormsModule,
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccPageHeaderComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-container">
      <acc-page-header
        title="Consumo e planos"
        subtitle="Chamadas SERPRO por escritório, produto e ambiente; planos e limites por tenant."
      />

      <nav class="abas" role="tablist">
        <button type="button" [class.ativa]="aba() === 'consumo'" (click)="aba.set('consumo')">Consumo</button>
        <button type="button" [class.ativa]="aba() === 'planos'" (click)="aba.set('planos')">Planos</button>
      </nav>

      @if (erro()) { <div class="erro" role="alert">{{ erro() }}</div> }

      @if (aba() === 'consumo') {
        <acc-card>
          <form class="filtros" (ngSubmit)="carregarConsumo()">
            <label>Início <input type="date" [(ngModel)]="filtros.inicio" name="inicio" /></label>
            <label>Fim <input type="date" [(ngModel)]="filtros.fim" name="fim" /></label>
            <label>Produto
              <select [(ngModel)]="filtros.produto" name="produto">
                <option value="">Todos</option>
                <option value="INTEGRA_CONTADOR">Integra Contador</option>
                <option value="DIVIDA_ATIVA">Dívida Ativa</option>
              </select>
            </label>
            <label>Ambiente
              <select [(ngModel)]="filtros.ambiente" name="ambiente">
                <option value="">Todos</option>
                <option value="TRIAL">Trial</option>
                <option value="PRODUCAO">Produção</option>
              </select>
            </label>
            <button acc-button type="submit" variant="primary">Aplicar</button>
            <button acc-button type="button" variant="outline" (click)="baixarCsv()">Exportar CSV</button>
          </form>
        </acc-card>

        @if (carregando()) {
          <acc-spinner />
        } @else if (consumo(); as c) {
          <div class="cards">
            <acc-card><h2>Chamadas</h2><p class="numero">{{ c.totalChamadas | number }}</p></acc-card>
            <acc-card><h2>Erros</h2><p class="numero">{{ c.totalErros | number }}</p><p class="legenda">Taxa {{ c.taxaErro * 100 | number: '1.0-1' }}%</p></acc-card>
            <acc-card><h2>Duração média</h2><p class="numero">{{ c.duracaoMediaMs | number }} ms</p></acc-card>
          </div>

          <h2 class="secao">Ranking por escritório</h2>
          <acc-card>
            @for (r of c.ranking; track r.escritorioId) {
              <div class="ranking">
                <span>{{ r.nome }}</span>
                <div class="ranking__barra"><span [style.width.%]="larguraRanking(r.chamadas)"></span></div>
                <strong>{{ r.chamadas }}</strong>
              </div>
            } @empty { <p class="legenda">Sem chamadas no período.</p> }
          </acc-card>

          <h2 class="secao">Série diária</h2>
          <div class="tabela-wrap">
            <table>
              <thead><tr><th>Dia</th><th>Chamadas</th><th>Erros</th><th>Duração média</th></tr></thead>
              <tbody>
                @for (s of c.serie; track s.dia) {
                  <tr><td>{{ s.dia }}</td><td>{{ s.chamadas }}</td><td>{{ s.erros }}</td><td>{{ s.duracaoMediaMs }} ms</td></tr>
                } @empty { <tr><td colspan="4">Sem dados.</td></tr> }
              </tbody>
            </table>
          </div>

          <h2 class="secao">Por escritório</h2>
          <div class="tabela-wrap">
            <table>
              <thead><tr><th>Escritório</th><th>CNPJ</th><th>Plano</th><th>Usado / limite</th><th>Erros</th><th>Taxa</th><th>Duração</th></tr></thead>
              <tbody>
                @for (e of c.escritorios; track e.escritorioId) {
                  <tr>
                    <td>{{ e.nome }}</td>
                    <td>{{ formatarCnpj(e.cnpj) }}</td>
                    <td>{{ e.planoNome ?? '—' }}</td>
                    <td>{{ e.chamadasUsadas }} / {{ e.chamadasLimite ?? '—' }}</td>
                    <td>{{ e.chamadasErro }}</td>
                    <td>{{ e.taxaErro * 100 | number: '1.0-1' }}%</td>
                    <td>{{ e.duracaoMediaMs }} ms</td>
                  </tr>
                } @empty { <tr><td colspan="7">Nenhum escritório.</td></tr> }
              </tbody>
            </table>
          </div>
        }
      } @else {
        <div class="cards">
          <acc-card>
            <h2>Novo plano</h2>
            <form class="form" (ngSubmit)="criarPlano()">
              <label>Nome <input [(ngModel)]="novoPlano.nome" name="nome" required /></label>
              <label>Descrição <input [(ngModel)]="novoPlano.descricao" name="descricao" /></label>
              <label>Chamadas/mês <input type="number" min="1" [(ngModel)]="novoPlano.chamadasMes" name="chamadas" /></label>
              <label>Clientes <input type="number" min="1" [(ngModel)]="novoPlano.clientesLimite" name="clientes" /></label>
              <label>Usuários <input type="number" min="1" [(ngModel)]="novoPlano.usuariosLimite" name="usuarios" /></label>
              <label>Preço (centavos) <input type="number" min="0" [(ngModel)]="novoPlano.precoMensalCents" name="preco" /></label>
              <button acc-button type="submit" variant="primary" [disabled]="salvandoPlano()">Criar plano</button>
            </form>
          </acc-card>
          <acc-card>
            <h2>Atribuir plano</h2>
            <form class="form" (ngSubmit)="atribuir()">
              <label>Escritório
                <select [(ngModel)]="atribuicao.escritorioId" name="escritorio">
                  <option value="">Selecione</option>
                  @for (t of tenants(); track t.id) { <option [value]="t.id">{{ t.nome }}</option> }
                </select>
              </label>
              <label>Plano
                <select [(ngModel)]="atribuicao.planoId" name="plano">
                  <option value="">Selecione</option>
                  @for (p of planos(); track p.id) { <option [value]="p.id">{{ p.nome }} ({{ p.chamadasMes }} chamadas)</option> }
                </select>
              </label>
              <label>Limite chamadas/mês <input type="number" min="1" [(ngModel)]="atribuicao.chamadasMesLimite" name="limite" /></label>
              <label class="check"><input type="checkbox" [(ngModel)]="atribuicao.bloqueioAoExceder" name="bloqueio" /> Bloquear ao exceder</label>
              <button acc-button type="submit" variant="primary" [disabled]="salvandoPlano()">Atribuir</button>
            </form>
          </acc-card>
        </div>

        <div class="tabela-wrap">
          <table>
            <thead><tr><th>Plano</th><th>Chamadas/mês</th><th>Clientes</th><th>Usuários</th><th>Preço</th><th>Status</th></tr></thead>
            <tbody>
              @for (p of planos(); track p.id) {
                <tr>
                  <td>{{ p.nome }}</td>
                  <td>{{ p.chamadasMes }}</td>
                  <td>{{ p.clientesLimite }}</td>
                  <td>{{ p.usuariosLimite }}</td>
                  <td>R$ {{ p.precoMensalCents / 100 | number: '1.2-2' }}</td>
                  <td><acc-badge [status]="p.ativo ? 'success' : 'neutral'">{{ p.ativo ? 'Ativo' : 'Inativo' }}</acc-badge></td>
                </tr>
              } @empty { <tr><td colspan="6">Nenhum plano cadastrado.</td></tr> }
            </tbody>
          </table>
        </div>
      }
    </div>
  `,
  styles: `
    .page-container { padding: 1rem; display: grid; gap: 1rem; }
    .abas { display: flex; gap: .5rem; }
    .abas button { padding: .45rem .9rem; border: 1px solid #d1d5db; background: #fff; border-radius: 6px; cursor: pointer; }
    .abas button.ativa { background: #1d4ed8; color: #fff; border-color: #1d4ed8; }
    .filtros, .form { display: flex; flex-wrap: wrap; gap: .75rem; align-items: end; }
    .filtros label, .form label { display: grid; gap: .25rem; }
    .cards { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); }
    .numero { font-size: 1.5rem; font-weight: 700; }
    .legenda { color: #4b5563; font-size: .875rem; }
    .ranking { display: grid; grid-template-columns: 12rem 1fr 4rem; gap: .5rem; align-items: center; padding: .25rem 0; }
    .ranking__barra { height: 8px; background: #e5e7eb; border-radius: 999px; overflow: hidden; }
    .ranking__barra span { display: block; height: 100%; background: #2563eb; }
    .secao { margin-top: .5rem; }
    .tabela-wrap { overflow-x: auto; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: .5rem; border-bottom: 1px solid #e5e7eb; text-align: left; white-space: nowrap; }
    .erro { background: #fef2f2; border: 1px solid #ef4444; color: #991b1b; padding: .75rem; border-radius: 8px; }
  `,
})
export class ConsumoAdminPage implements OnInit {
  private readonly servico = inject(ConsumoAdminService);
  private readonly tenantService = inject(TenantService);

  protected readonly aba = signal<Aba>('consumo');
  protected readonly consumo = signal<ConsumoAdminDto | null>(null);
  protected readonly planos = signal<PlanoEscritorioDto[]>([]);
  protected readonly tenants = this.tenantService.tenants;
  protected readonly carregando = signal(false);
  protected readonly salvandoPlano = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected filtros: FiltrosConsumo = {};
  protected novoPlano = {
    nome: '',
    descricao: '',
    chamadasMes: 1000,
    clientesLimite: 100,
    usuariosLimite: 10,
    precoMensalCents: 0,
  };
  protected atribuicao = {
    escritorioId: '',
    planoId: '',
    chamadasMesLimite: 1000,
    bloqueioAoExceder: true,
  };

  async ngOnInit(): Promise<void> {
    this.tenantService.listar(1, 100);
    await Promise.all([this.carregarConsumo(), this.carregarPlanos()]);
  }

  protected async carregarConsumo(): Promise<void> {
    this.carregando.set(true);
    this.erro.set(null);
    try {
      this.consumo.set(await this.servico.consumo(this.filtros));
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao carregar consumo.'));
    } finally {
      this.carregando.set(false);
    }
  }

  protected async carregarPlanos(): Promise<void> {
    try {
      this.planos.set(await this.servico.planos());
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao carregar planos.'));
    }
  }

  protected async baixarCsv(): Promise<void> {
    try {
      const blob = await this.servico.exportarCsv(this.filtros);
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'consumo-serpro.csv';
      link.click();
      URL.revokeObjectURL(url);
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao exportar CSV.'));
    }
  }

  protected async criarPlano(): Promise<void> {
    this.salvandoPlano.set(true);
    this.erro.set(null);
    try {
      await this.servico.criarPlano({
        nome: this.novoPlano.nome,
        descricao: this.novoPlano.descricao || null,
        chamadasMes: this.novoPlano.chamadasMes,
        clientesLimite: this.novoPlano.clientesLimite,
        usuariosLimite: this.novoPlano.usuariosLimite,
        precoMensalCents: this.novoPlano.precoMensalCents,
      });
      this.novoPlano.nome = '';
      await this.carregarPlanos();
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao criar plano.'));
    } finally {
      this.salvandoPlano.set(false);
    }
  }

  protected async atribuir(): Promise<void> {
    if (!this.atribuicao.escritorioId || !this.atribuicao.planoId) return;
    this.salvandoPlano.set(true);
    this.erro.set(null);
    try {
      await this.servico.atribuirPlano(this.atribuicao.escritorioId, {
        planoId: this.atribuicao.planoId,
        chamadasMesLimite: this.atribuicao.chamadasMesLimite,
        bloqueioAoExceder: this.atribuicao.bloqueioAoExceder,
      });
      await this.carregarConsumo();
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao atribuir plano.'));
    } finally {
      this.salvandoPlano.set(false);
    }
  }

  protected larguraRanking(chamadas: number): number {
    const maximo = Math.max(1, ...(this.consumo()?.ranking.map((r) => r.chamadas) ?? [1]));
    return (chamadas / maximo) * 100;
  }

  protected formatarCnpj(cnpj: string): string {
    if (cnpj.length !== 14) return cnpj;
    return `${cnpj.slice(0, 2)}.${cnpj.slice(2, 5)}.${cnpj.slice(5, 8)}/${cnpj.slice(8, 12)}-${cnpj.slice(12)}`;
  }
}
