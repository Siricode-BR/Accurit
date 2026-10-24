import { Component, inject, OnInit } from '@angular/core';
import { RouterLink } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar } from '@angular/material/snack-bar';
import { MatSlideToggleChange, MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatTableModule } from '@angular/material/table';

import { Tenant } from '../domain/tenant';
import { TenantsStore } from '../application/tenants.store';
import { ConfirmaProducaoDialog, ConfirmacaoProducaoDados } from './confirma-producao.dialog';

@Component({
  selector: 'app-tenant-list',
  standalone: true,
  imports: [
    RouterLink,
    MatTableModule,
    MatButtonModule,
    MatIconModule,
    MatCardModule,
    MatProgressSpinnerModule,
    MatTooltipModule,
    MatSlideToggleModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>Escritórios (Tenants)</mat-card-title>
        <mat-card-subtitle>Gerencie os escritórios cadastrados na plataforma</mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <div class="header-actions">
          <button mat-raised-button color="primary" routerLink="/tenants/novo">
            <mat-icon>add</mat-icon>
            Novo Escritório
          </button>
        </div>

        @if (store.carregando()) {
          <div class="loading-container">
            <mat-spinner diameter="40"></mat-spinner>
            <p>Carregando escritórios...</p>
          </div>
        } @else if (store.indisponivel()) {
          <div class="empty-state">
            <mat-icon>cloud_off</mat-icon>
            <h3>Lista indisponível</h3>
            <p>Não foi possível carregar os escritórios. Tente novamente.</p>
            <button mat-raised-button (click)="recarregar()">Recarregar</button>
          </div>
        } @else if (store.tenants().length === 0) {
          <div class="empty-state">
            <mat-icon>business</mat-icon>
            <h3>Nenhum escritório cadastrado</h3>
            <p>Clique em "Novo Escritório" para criar o primeiro.</p>
          </div>
        } @else {
          <table mat-table [dataSource]="store.tenants()" class="tenants-table">
            <ng-container matColumnDef="nome">
              <th mat-header-cell *matHeaderCellDef>Nome (slug)</th>
              <td mat-cell *matCellDef="let tenant">{{ tenant.nome }} ({{ tenant.slug }})</td>
            </ng-container>

            <ng-container matColumnDef="serpro">
              <th mat-header-cell *matHeaderCellDef>Ambiente SERPRO</th>
              <td mat-cell *matCellDef="let tenant">
                <span class="badge" [class.trial]="tenant.modoTrialSerpro" [class.producao]="!tenant.modoTrialSerpro">
                  {{ tenant.modoTrialSerpro ? 'TRIAL' : 'PRODUÇÃO' }}
                </span>
                <mat-slide-toggle
                  [checked]="tenant.modoTrialSerpro"
                  [disabled]="store.trocaEmAndamento(tenant.id)"
                  [matTooltip]="tooltipDoToggle(tenant)"
                  (change)="trocar(tenant, $event)"
                >
                  <span class="sr-only">Alternar trial</span>
                </mat-slide-toggle>
              </td>
            </ng-container>

            <ng-container matColumnDef="versao">
              <th mat-header-cell *matHeaderCellDef>Versão</th>
              <td mat-cell *matCellDef="let tenant">v{{ tenant.versao }}</td>
            </ng-container>

            <ng-container matColumnDef="atualizadoEm">
              <th mat-header-cell *matHeaderCellDef>Atualizado em</th>
              <td mat-cell *matCellDef="let tenant">{{ formatar(tenant.atualizadoEm) }}</td>
            </ng-container>

            <tr mat-header-row *matHeaderRowDef="colunas"></tr>
            <tr mat-row *matRowDef="let row; columns: colunas"></tr>
          </table>

          <div class="paginacao">
            <button
              mat-icon-button
              (click)="paginaAnterior()"
              [disabled]="store.paginaAtual() <= 1 || store.carregando()"
              matTooltip="Página anterior"
            >
              <mat-icon>chevron_left</mat-icon>
            </button>
            <span>
              Página {{ store.paginaAtual() }} de {{ store.paginas() }} — {{ store.total() }}
              escritórios
            </span>
            <button
              mat-icon-button
              (click)="proximaPagina()"
              [disabled]="store.paginaAtual() >= store.paginas() || store.carregando()"
              matTooltip="Próxima página"
            >
              <mat-icon>chevron_right</mat-icon>
            </button>
          </div>
        }
      </mat-card-content>
    </mat-card>
  `,
  styles: `
    .header-actions {
      margin-bottom: 16px;
    }

    .loading-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 32px;
      gap: 16px;
    }

    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 48px;
      text-align: center;
      color: #666;
      gap: 16px;

      mat-icon {
        font-size: 64px;
        width: 64px;
        height: 64px;
      }
    }

    .tenants-table {
      width: 100%;
    }

    .badge {
      display: inline-block;
      margin-right: 12px;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 500;
      text-transform: uppercase;
    }

    .badge.trial {
      background-color: #fff3e0;
      color: #e65100;
    }

    .badge.producao {
      background-color: #e8f5e9;
      color: #2e7d32;
    }

    .sr-only {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
    }

    .paginacao {
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      padding-top: 16px;
    }

    .mat-column-serpro {
      text-align: center;
    }
  `,
})
export class TenantListComponent implements OnInit {
  protected readonly store = inject(TenantsStore);
  private readonly dialogos = inject(MatDialog);
  private readonly snack = inject(MatSnackBar);

  protected readonly colunas = ['nome', 'serpro', 'versao', 'atualizadoEm'];

  ngOnInit(): void {
    void this.recarregar();
  }

  protected async recarregar(): Promise<void> {
    await this.store.carregar(this.store.paginaAtual());
  }

  protected paginaAnterior(): void {
    void this.mudarPagina(this.store.paginaAtual() - 1);
  }

  protected proximaPagina(): void {
    void this.mudarPagina(this.store.paginaAtual() + 1);
  }

  private async mudarPagina(pagina: number): Promise<void> {
    if (pagina < 1 || pagina > this.store.paginas()) return;
    await this.store.carregar(pagina);
  }

  protected tooltipDoToggle(tenant: Tenant): string {
    return tenant.modoTrialSerpro ? 'Migrar para produção' : 'Voltar ao trial';
  }

  /** Troca de modo **pessimista**: nada muda na tela antes do PATCH concluir.
   * TRIAL->PRODUCAO passa pelo modal de confirmação; 409 recarrega a página e
   * qualquer caminho que falhe restaura o estado visual do toggle da linha
   * (`evento.source.checked`). */
  protected async trocar(tenant: Tenant, evento: MatSlideToggleChange): Promise<void> {
    const alvo = evento.checked;
    const controle = evento.source;
    if (alvo === tenant.modoTrialSerpro || this.store.trocaEmAndamento(tenant.id)) return;

    if (!alvo && tenant.modoTrialSerpro) {
      const confirmado = await firstValueFrom(
        this.dialogos
          .open<ConfirmaProducaoDialog, ConfirmacaoProducaoDados, boolean>(ConfirmaProducaoDialog, {
            data: { tenant },
          })
          .afterClosed(),
      );
      if (confirmado !== true) {
        controle.checked = tenant.modoTrialSerpro;
        return;
      }
    }

    const resultado = await this.store.alternarTrial(tenant, alvo);
    switch (resultado.tipo) {
      case 'ok':
        this.snack.open('Ambiente SERPRO atualizado.', 'Fechar', { duration: 3000 });
        break;
      case 'conflito':
        controle.checked = tenant.modoTrialSerpro;
        this.snack.open(
          'Versão divergiu: os dados mudaram em outro lugar, recarregamos a lista.',
          'Fechar',
          { duration: 5000 },
        );
        break;
      case 'bloqueado':
        controle.checked = tenant.modoTrialSerpro;
        this.snack.open(resultado.mensagem, 'Fechar', { duration: 5000 });
        break;
      case 'naoEncontrado':
        controle.checked = tenant.modoTrialSerpro;
        this.snack.open('Escritório não encontrado; recarregue a lista.', 'Fechar', {
          duration: 5000,
        });
        break;
      case 'indisponivel':
        controle.checked = tenant.modoTrialSerpro;
        this.snack.open('Serviço indisponível; tente novamente.', 'Fechar', { duration: 5000 });
        break;
    }
  }

  protected formatar(valor: string): string {
    return new Date(valor).toLocaleString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
}
