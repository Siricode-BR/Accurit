import {
  Component,
  OnInit,
  inject,
  signal,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { CatalogoIntegraService } from '../../data/catalogo-integra.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { LinhaCatalogoServicoDto } from '@accurit/core';

@Component({
  selector: 'app-catalogo-integra-page',
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
    <div class="catalogo-container">
      <header class="catalogo-header">
        <div>
          <h1 class="page-title">Catálogo de Serviços Integra Contador</h1>
          <p class="page-subtitle">
            Habilitação granular de serviços do Integra Contador (SERPRO). Serviços sem versão confirmada permanecem desabilitados com segurança.
          </p>
        </div>
        <button
          acc-button
          variant="outline"
          size="sm"
          [disabled]="catalogoService.carregando()"
          (click)="recarregar()"
        >
          Atualizar
        </button>
      </header>

      @if (mensagemAviso()) {
        <div class="alert-info" role="status">
          {{ mensagemAviso() }}
        </div>
      }

      @if (catalogoService.erro(); as erro) {
        <div class="alert-error" role="alert">
          {{ erro }}
        </div>
      }

      @if (catalogoService.carregando() && catalogoService.servicos().length === 0) {
        <div class="loading-state">
          <acc-spinner size="lg" />
          <p>Carregando catálogo de serviços...</p>
        </div>
      } @else {
        <acc-card>
          <div class="table-responsive">
            <table class="acc-table">
              <thead>
                <tr>
                  <th>Sistema</th>
                  <th>Serviço</th>
                  <th>Verbo</th>
                  <th>Versão</th>
                  <th>Versão Confirmada</th>
                  <th>Requer Procuração</th>
                  <th>Status</th>
                  <th>Ação</th>
                </tr>
              </thead>
              <tbody>
                @for (servico of catalogoService.servicos(); track servico.id) {
                  <tr>
                    <td><strong>{{ servico.idSistema }}</strong></td>
                    <td>
                      <div>
                        <strong>{{ servico.idServico }}</strong>
                        <div class="servico-descricao">{{ servico.descricao }}</div>
                      </div>
                    </td>
                    <td><span class="font-mono">{{ servico.verbo }}</span></td>
                    <td><span class="font-mono">{{ servico.versaoSistema ?? '—' }}</span></td>
                    <td>
                      <acc-badge [status]="servico.versaoConfirmada ? 'success' : 'warning'">
                        {{ servico.versaoConfirmada ? 'Confirmada' : 'Pendente' }}
                      </acc-badge>
                    </td>
                    <td>
                      <acc-badge [status]="servico.requerProcuracao ? 'info' : 'neutral'">
                        {{ servico.requerProcuracao ? 'Sim' : 'Não' }}
                      </acc-badge>
                    </td>
                    <td>
                      <acc-badge [status]="servico.habilitado ? 'success' : 'neutral'">
                        {{ servico.habilitado ? 'Habilitado' : 'Desabilitado' }}
                      </acc-badge>
                    </td>
                    <td>
                      <button
                        acc-button
                        [variant]="servico.habilitado ? 'outline' : 'primary'"
                        size="sm"
                        [disabled]="atualizando() === servico.id"
                        (click)="alternar(servico)"
                      >
                        {{ servico.habilitado ? 'Desabilitar' : 'Habilitar' }}
                      </button>
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
    .catalogo-container {
      padding: 1.5rem;
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
    }
    .catalogo-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 1rem;
    }
    .page-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }
    .page-subtitle {
      font-size: 0.875rem;
      color: var(--acc-color-text-muted);
      margin: 0.25rem 0 0;
    }
    .servico-descricao {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
      margin-top: 0.125rem;
    }
    .table-responsive {
      overflow-x: auto;
    }
    .acc-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.875rem;
    }
    .acc-table th {
      text-align: left;
      padding: 0.75rem;
      border-bottom: 2px solid var(--acc-color-border);
      color: var(--acc-color-text-muted);
      font-weight: 600;
    }
    .acc-table td {
      padding: 0.75rem;
      border-bottom: 1px solid var(--acc-color-border);
      vertical-align: middle;
    }
    .font-mono {
      font-family: monospace;
    }
    .loading-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.75rem;
      padding: 3rem;
      color: var(--acc-color-text-muted);
    }
    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
    }
    .alert-info {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-info-bg, #eff6ff);
      color: var(--acc-color-info, #1d4ed8);
      font-size: 0.875rem;
    }
  `,
})
export class CatalogoIntegraPage implements OnInit {
  protected readonly catalogoService = inject(CatalogoIntegraService);
  protected readonly atualizando = signal<string | null>(null);
  protected readonly mensagemAviso = signal<string | null>(null);

  ngOnInit(): void {
    this.recarregar();
  }

  recarregar(): void {
    this.catalogoService.listar().subscribe();
  }

  alternar(servico: LinhaCatalogoServicoDto): void {
    this.mensagemAviso.set(null);
    if (!servico.habilitado && !servico.versaoConfirmada) {
      this.mensagemAviso.set(
        `O serviço ${servico.idSistema}/${servico.idServico} não pode ser habilitado porque sua versão na SERPRO ainda não foi confirmada (fail-closed conforme ADR-0013 / FASE A).`,
      );
      return;
    }

    this.atualizando.set(servico.id);
    this.catalogoService
      .definirHabilitado(servico.idSistema, servico.idServico, !servico.habilitado)
      .subscribe({
        next: () => this.atualizando.set(null),
        error: (err: { error?: { message?: string } }) => {
          this.atualizando.set(null);
          this.mensagemAviso.set(
            err?.error?.message ?? 'Falha ao atualizar o catálogo de serviços.',
          );
        },
      });
  }
}
