import { CommonModule } from '@angular/common';
import { Component, inject, OnInit, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { MatTableModule } from '@angular/material/table';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSelectModule } from '@angular/material/select';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatPaginatorModule, PageEvent } from '@angular/material/paginator';
import { MatTooltipModule } from '@angular/material/tooltip';
import { FormsModule } from '@angular/forms';

import { ClienteService, ListagemClientes, FiltroClientes, TipoCliente } from './cliente.service';

@Component({
  selector: 'app-cliente-list',
  standalone: true,
  imports: [
    CommonModule,
    RouterLink,
    MatTableModule,
    MatButtonModule,
    MatIconModule,
    MatCardModule,
    MatProgressSpinnerModule,
    MatSelectModule,
    MatFormFieldModule,
    MatPaginatorModule,
    MatTooltipModule,
    FormsModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>Clientes</mat-card-title>
        <mat-card-subtitle>Gerencie os clientes do escritório</mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <div class="header-actions">
          <mat-form-field appearance="outline" class="filtro-select">
            <mat-label>Filtrar</mat-label>
            <mat-select [(ngModel)]="filtroSelecionado" (selectionChange)="onFiltroChange()">
              @for (opcao of opcoesFiltro; track opcao.value) {
                <mat-option [value]="opcao.value">{{ opcao.label }}</mat-option>
              }
            </mat-select>
          </mat-form-field>

          <button mat-raised-button color="primary" routerLink="/clientes/novo">
            <mat-icon>person_add</mat-icon>
            Novo Cliente
          </button>
        </div>

        @if (carregando()) {
          <div class="loading-container">
            <mat-spinner diameter="40"></mat-spinner>
            <p>Carregando clientes...</p>
          </div>
        } @else if (listagem().itens.length === 0) {
          <div class="empty-state">
            <mat-icon>people</mat-icon>
            <h3>Nenhum cliente cadastrado</h3>
            <p>Clique em "Novo Cliente" para cadastrar o primeiro.</p>
          </div>
        } @else {
          <table mat-table [dataSource]="listagem().itens" class="clientes-table">
            <ng-container matColumnDef="tipo">
              <th mat-header-cell *matHeaderCellDef>Tipo</th>
              <td mat-cell *matCellDef="let cliente">
                <span class="badge" [class.pf]="cliente.tipo === 'PF'" [class.pj]="cliente.tipo === 'PJ'">
                  {{ cliente.tipo }}
                </span>
              </td>
            </ng-container>

            <ng-container matColumnDef="porteRegime">
              <th mat-header-cell *matHeaderCellDef>Porte/Regime</th>
              <td mat-cell *matCellDef="let cliente">{{ cliente.porteRegime || '-' }}</td>
            </ng-container>

            <ng-container matColumnDef="cpfCnpj">
              <th mat-header-cell *matHeaderCellDef>CPF/CNPJ</th>
              <td mat-cell *matCellDef="let cliente">{{ formatCpfCnpj(cliente.cpfCnpj, cliente.tipo) }}</td>
            </ng-container>

            <ng-container matColumnDef="nomeRazaoSocial">
              <th mat-header-cell *matHeaderCellDef>Nome / Razão Social</th>
              <td mat-cell *matCellDef="let cliente">{{ cliente.nomeRazaoSocial }}</td>
            </ng-container>

            <ng-container matColumnDef="nomeFantasia">
              <th mat-header-cell *matHeaderCellDef>Nome Fantasia</th>
              <td mat-cell *matCellDef="let cliente">{{ cliente.nomeFantasia || '-' }}</td>
            </ng-container>

            <ng-container matColumnDef="criadoEm">
              <th mat-header-cell *matHeaderCellDef>Criado em</th>
              <td mat-cell *matCellDef="let cliente">{{ formatDate(cliente.criadoEm) }}</td>
            </ng-container>

            <ng-container matColumnDef="acoes">
              <th mat-header-cell *matHeaderCellDef>Ações</th>
              <td mat-cell *matCellDef="let cliente">
                <button mat-icon-button [routerLink]="['/clientes', cliente.id]" matTooltip="Ver detalhes">
                  <mat-icon>visibility</mat-icon>
                </button>
                <button mat-icon-button [routerLink]="['/clientes', cliente.id, 'editar']" matTooltip="Editar">
                  <mat-icon>edit</mat-icon>
                </button>
              </td>
            </ng-container>

            <tr mat-header-row *matHeaderRowDef="displayedColumns"></tr>
            <tr mat-row *matRowDef="let row; columns: displayedColumns"></tr>
          </table>

          <mat-paginator
            [length]="listagem().total"
            [pageSize]="tamanhoPagina()"
            [pageSizeOptions]="[10, 20, 50]"
            [pageIndex]="paginaAtual() - 1"
            (page)="onPageChange($event)"
            showFirstLastButtons
            aria-label="Paginação de clientes">
          </mat-paginator>
        }
      </mat-card-content>
    </mat-card>
  `,
  styles: `
    .header-actions {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      flex-wrap: wrap;
      gap: 16px;

      .filtro-select {
        min-width: 200px;
      }
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

    .clientes-table {
      width: 100%;
    }

    .badge {
      display: inline-block;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 500;
      text-transform: uppercase;
    }

    .badge.pf {
      background-color: #e3f2fd;
      color: #1565c0;
    }

    .badge.pj {
      background-color: #f3e5f5;
      color: #7b1fa2;
    }

    .mat-column-acoes {
      width: 100px;
      text-align: center;
    }

    .mat-column-tipo {
      width: 80px;
    }

    .mat-column-porteRegime {
      width: 180px;
    }
  `,
})
export class ClienteListComponent implements OnInit {
  private readonly clienteService = inject(ClienteService);

  protected readonly listagem = signal<ListagemClientes>({ itens: [], total: 0, pagina: 1, tamanho: 20 });
  protected readonly carregando = signal(true);
  protected readonly filtroSelecionado = signal<FiltroClientes>('todos');
  protected readonly paginaAtual = signal(1);
  protected readonly tamanhoPagina = signal(20);

  protected readonly opcoesFiltro = [
    { value: 'todos', label: 'Todos' },
    { value: 'cpf', label: 'CPF' },
    { value: 'cnpj_mei', label: 'CNPJ - MEI' },
    { value: 'cnpj_simples', label: 'CNPJ - Simples Nacional' },
    { value: 'cnpj_presumido', label: 'CNPJ - Lucro Presumido' },
    { value: 'cnpj_real', label: 'CNPJ - Lucro Real' },
  ];

  protected readonly displayedColumns = ['tipo', 'porteRegime', 'cpfCnpj', 'nomeRazaoSocial', 'nomeFantasia', 'criadoEm', 'acoes'];

  ngOnInit(): void {
    this.carregar();
  }

  private carregar(): void {
    this.carregando.set(true);
    this.clienteService.listar(this.filtroSelecionado(), this.paginaAtual(), this.tamanhoPagina()).subscribe({
      next: (listagem) => {
        this.listagem.set(listagem);
        this.carregando.set(false);
      },
      error: () => {
        this.carregando.set(false);
      },
    });
  }

  protected onFiltroChange(): void {
    this.paginaAtual.set(1);
    this.carregar();
  }

  protected onPageChange(event: PageEvent): void {
    this.paginaAtual.set(event.pageIndex + 1);
    this.tamanhoPagina.set(event.pageSize);
    this.carregar();
  }

  protected formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  protected formatCpfCnpj(cpfCnpj: string, tipo: TipoCliente): string {
    if (tipo === 'PF') {
      return cpfCnpj.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, '$1.$2.$3-$4');
    }
    return cpfCnpj.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, '$1.$2.$3/$4-$5');
  }
}