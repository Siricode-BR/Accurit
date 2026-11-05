import {
  Component,
  OnInit,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TenantService } from '../../data/tenant.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { formatPtBrDateTime } from '@accurit/shared';
import { TenantDto } from '@accurit/core';

@Component({
  selector: 'app-tenant-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="tenants-container">
      <header class="tenants-header">
        <div>
          <h1 class="page-title">Gestão de Escritórios (Tenants)</h1>
          <p class="page-subtitle">
            Administração SaaS: {{ tenantService.total() }} escritórios cadastrados
          </p>
        </div>
        <button acc-button variant="primary" size="md">
          + Novo Escritório
        </button>
      </header>

      <!-- Métricas SaaS -->
      <section class="metrics-grid">
        <acc-card>
          <div class="metric-card">
            <span class="metric-label">Escritórios em Produção</span>
            <span class="metric-value">{{ tenantService.totalProducao() }}</span>
          </div>
        </acc-card>
        <acc-card>
          <div class="metric-card">
            <span class="metric-label">Escritórios em Modo Trial</span>
            <span class="metric-value text-amber">{{ tenantService.totalTrial() }}</span>
          </div>
        </acc-card>
      </section>

      <!-- Lista / Tabela -->
      @if (tenantService.carregando() && tenantService.tenants().length === 0) {
        <div class="loading-state">
          <acc-spinner size="lg"></acc-spinner>
          <p>Carregando escritórios...</p>
        </div>
      } @else if (tenantService.erro()) {
        <div class="alert-error">
          <p><strong>Erro:</strong> {{ tenantService.erro() }}</p>
          <button acc-button variant="outline" size="sm" (click)="recarregar()">Tentar novamente</button>
        </div>
      } @else {
        <acc-card>
          <div class="table-responsive">
            <table class="acc-table">
              <thead>
                <tr>
                  <th>Nome do Escritório</th>
                  <th>Slug / Subdomínio</th>
                  <th>Ambiente SERPRO</th>
                  <th>Última Atualização</th>
                  <th>Ações</th>
                </tr>
              </thead>
              <tbody>
                @for (tenant of tenantService.tenants(); track tenant.id) {
                  <tr>
                    <td>
                      <strong>{{ tenant.nome }}</strong>
                    </td>
                    <td class="font-mono text-muted">
                      {{ tenant.slug }}
                    </td>
                    <td>
                      <acc-badge [status]="tenant.modoTrialSerpro ? 'warning' : 'success'">
                        {{ tenant.modoTrialSerpro ? 'Trial SERPRO' : 'Produção' }}
                      </acc-badge>
                    </td>
                    <td>
                      {{ formatarData(tenant.atualizadoEm) }}
                    </td>
                    <td>
                      <div class="action-buttons">
                        <button
                          acc-button
                          variant="outline"
                          size="sm"
                          (click)="alternarAmbiente(tenant)"
                        >
                          {{ tenant.modoTrialSerpro ? 'Ativar Produção' : 'Mudar para Trial' }}
                        </button>
                      </div>
                    </td>
                  </tr>
                } @empty {
                  <tr>
                    <td colspan="5" class="empty-message">
                      Nenhum escritório cadastrado.
                    </td>
                  </tr>
                }
              </tbody>
            </table>
          </div>
        </acc-card>
      }
    </div>
  `,
  styles: `
    .tenants-container {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
      padding: 1.5rem;
      max-width: 1200px;
      margin: 0 auto;
    }

    .tenants-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .page-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 0.25rem 0;
    }

    .page-subtitle {
      color: var(--acc-color-text-muted);
      margin: 0;
      font-size: 0.875rem;
    }

    .metrics-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 1rem;
    }

    .metric-card {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .metric-label {
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted);
    }

    .metric-value {
      font-size: 1.75rem;
      font-weight: 700;
      color: var(--acc-color-success);
    }

    .metric-value.text-amber {
      color: var(--acc-color-warning);
    }

    .table-responsive {
      overflow-x: auto;
    }

    .acc-table {
      width: 100%;
      border-collapse: collapse;
      text-align: left;
      font-size: 0.875rem;
    }

    .acc-table th {
      padding: 0.75rem 1rem;
      background-color: var(--acc-color-bg);
      border-bottom: 1px solid var(--acc-color-border);
      color: var(--acc-color-text-muted);
      font-weight: 600;
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .acc-table td {
      padding: 1rem;
      border-bottom: 1px solid var(--acc-color-border);
      vertical-align: middle;
    }

    .action-buttons {
      display: flex;
      gap: 0.5rem;
    }

    .font-mono {
      font-family: monospace;
    }

    .text-muted {
      color: var(--acc-color-text-muted);
    }

    .loading-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 3rem;
      gap: 1rem;
      color: var(--acc-color-text-muted);
    }

    .alert-error {
      padding: 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .empty-message {
      text-align: center;
      padding: 3rem 1rem !important;
      color: var(--acc-color-text-muted);
    }
  `,
})
export class TenantListComponent implements OnInit {
  protected readonly tenantService = inject(TenantService);

  ngOnInit(): void {
    this.tenantService.listar();
  }

  recarregar(): void {
    this.tenantService.listar();
  }

  formatarData(iso: string): string {
    return formatPtBrDateTime(iso);
  }

  async alternarAmbiente(tenant: TenantDto): Promise<void> {
    const novoModo = !tenant.modoTrialSerpro;
    await this.tenantService.alternarModoSerpro(tenant.id, novoModo, tenant.versao);
  }
}

