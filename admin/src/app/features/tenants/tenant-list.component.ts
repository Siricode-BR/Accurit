import { CommonModule } from '@angular/common';
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { MatTableModule } from '@angular/material/table';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatTooltipModule } from '@angular/material/tooltip';

import { TenantService, Tenant } from './tenant.service';

@Component({
  selector: 'app-tenant-list',
  standalone: true,
  imports: [
    CommonModule,
    RouterLink,
    MatTableModule,
    MatButtonModule,
    MatIconModule,
    MatCardModule,
    MatProgressSpinnerModule,
    MatTooltipModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>Escritórios (Tenants)</mat-card-title>
        <mat-card-subtitle>Gerencie os escritórios cadastrados na plataforma</mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <div class="header-actions">
          <button mat-raised-button color="primary" routerLink="/admin/tenants/novo">
            <mat-icon>add</mat-icon>
            Novo Escritório
          </button>
        </div>

        @if (carregando()) {
          <div class="loading-container">
            <mat-spinner diameter="40"></mat-spinner>
            <p>Carregando escritórios...</p>
          </div>
        } @else if (tenants().length === 0) {
          <div class="empty-state">
            <mat-icon>business</mat-icon>
            <h3>Nenhum escritório cadastrado</h3>
            <p>Clique em "Novo Escritório" para criar o primeiro.</p>
          </div>
        } @else {
          <table mat-table [dataSource]="tenants()" class="tenants-table">
            <ng-container matColumnDef="slug">
              <th mat-header-cell *matHeaderCellDef>Slug</th>
              <td mat-cell *matCellDef="let tenant">{{ tenant.slug }}</td>
            </ng-container>

            <ng-container matColumnDef="nome">
              <th mat-header-cell *matHeaderCellDef>Nome</th>
              <td mat-cell *matCellDef="let tenant">{{ tenant.nome }}</td>
            </ng-container>

            <ng-container matColumnDef="cnpj">
              <th mat-header-cell *matHeaderCellDef>CNPJ</th>
              <td mat-cell *matCellDef="let tenant">{{ tenant.cnpj || '-' }}</td>
            </ng-container>

            <ng-container matColumnDef="modoTrialSerpro">
              <th mat-header-cell *matHeaderCellDef>Ambiente SERPRO</th>
              <td mat-cell *matCellDef="let tenant">
                <span class="badge" [class.trial]="tenant.modoTrialSerpro" [class.producao]="!tenant.modoTrialSerpro">
                  {{ tenant.modoTrialSerpro ? 'TRIAL' : 'PRODUÇÃO' }}
                </span>
              </td>
            </ng-container>

            <ng-container matColumnDef="criadoEm">
              <th mat-header-cell *matHeaderCellDef>Criado em</th>
              <td mat-cell *matCellDef="let tenant">{{ formatDate(tenant.criadoEm) }}</td>
            </ng-container>

            <ng-container matColumnDef="acoes">
              <th mat-header-cell *matHeaderCellDef>Ações</th>
              <td mat-cell *matCellDef="let tenant">
                <button mat-icon-button [routerLink]="['/serpro', tenant.id]" matTooltip="Configurar SERPRO">
                  <mat-icon>settings</mat-icon>
                </button>
              </td>
            </ng-container>

            <tr mat-header-row *matHeaderRowDef="displayedColumns"></tr>
            <tr mat-row *matRowDef="let row; columns: displayedColumns"></tr>
          </table>
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

    .mat-column-acoes {
      width: 80px;
      text-align: center;
    }
  `,
})
export class TenantListComponent implements OnInit {
  private readonly tenantService = inject(TenantService);

  protected readonly tenants = signal<Tenant[]>([]);
  protected readonly carregando = signal(true);
  protected readonly displayedColumns = ['slug', 'nome', 'cnpj', 'modoTrialSerpro', 'criadoEm', 'acoes'];

  ngOnInit(): void {
    this.carregar();
  }

  private carregar(): void {
    this.carregando.set(true);
    this.tenantService.listar().subscribe({
      next: (tenants) => {
        this.tenants.set(tenants);
        this.carregando.set(false);
      },
      error: () => {
        this.carregando.set(false);
      },
    });
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
}