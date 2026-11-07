import {
  Component,
  OnInit,
  DestroyRef,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { ClienteService } from '../../data/cliente.service';
import { ProcuracaoService } from '../../data/procuracao.service';
import { AuthService } from '../../core/auth/auth.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
  AccDocumentMaskDirective,
} from '@accurit/ui';
import { isValidDocument, formatDocument } from '@accurit/shared';
import { ClienteDto, FiltroClientes } from '@accurit/core';

@Component({
  selector: 'app-cliente-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
    AccDocumentMaskDirective,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="clientes-container">
      <header class="clientes-header">
        <div>
          <h1 class="page-title">Gestão de Clientes</h1>
          <p class="page-subtitle">
            Base cadastral do escritório contábil: {{ clienteService.total() }} clientes registrados
          </p>
        </div>
        <button acc-button variant="primary" size="md" (click)="abrirModalNovo()">
          + Novo Cliente
        </button>
        <button acc-button variant="outline" size="md" (click)="sair()" class="btn-sair">
          Sair
        </button>
      </header>

      <!-- Cards de Resumo -->
      <section class="metrics-grid">
        <acc-card>
          <div class="metric-card">
            <span class="metric-label">Pessoas Jurídicas (PJ)</span>
            <span class="metric-value">{{ clienteService.totalPJ() }}</span>
          </div>
        </acc-card>
        <acc-card>
          <div class="metric-card">
            <span class="metric-label">Pessoas Físicas (PF)</span>
            <span class="metric-value">{{ clienteService.totalPF() }}</span>
          </div>
        </acc-card>
      </section>

      <!-- Barra de Filtros e Busca -->
      <acc-card>
        <div class="filter-bar">
          <div class="search-input-wrapper">
            <input
              type="text"
              accDocMask
              class="search-input"
              placeholder="Buscar por Razão Social ou CPF/CNPJ..."
              [value]="termoBusca()"
              (input)="onSearchInput($event)"
            />
            @if (validacaoDocumentoAviso()) {
              <small class="doc-warning">{{ validacaoDocumentoAviso() }}</small>
            }
          </div>

          <div class="filter-actions">
            <button
              acc-button
              [variant]="filtroAtivo() === 'todos' ? 'primary' : 'outline'"
              size="sm"
              (click)="aplicarFiltro('todos')"
            >
              Todos
            </button>
            <button
              acc-button
              [variant]="filtroAtivo() === 'cpf' ? 'primary' : 'outline'"
              size="sm"
              (click)="aplicarFiltro('cpf')"
            >
              CPF
            </button>
            <button
              acc-button
              [variant]="filtroAtivo() === 'cnpj_simples' ? 'primary' : 'outline'"
              size="sm"
              (click)="aplicarFiltro('cnpj_simples')"
            >
              Simples Nacional
            </button>
            <button
              acc-button
              variant="outline"
              size="sm"
              [loading]="clienteService.carregando()"
              (click)="recarregar()"
            >
              Recarregar
            </button>
          </div>
        </div>
      </acc-card>

      <!-- Conteúdo / Tabela -->
      @if (clienteService.carregando() && clienteService.clientes().length === 0) {
        <div class="loading-state">
          <acc-spinner size="lg"></acc-spinner>
          <p>Carregando carteira de clientes...</p>
        </div>
      } @else if (clienteService.erro()) {
        <div class="alert-error">
          <p><strong>Erro:</strong> {{ clienteService.erro() }}</p>
          <button acc-button variant="outline" size="sm" (click)="recarregar()">Tentar novamente</button>
        </div>
      } @else {
        <acc-card>
          <div class="table-responsive">
            <table class="acc-table">
              <thead>
                <tr>
                  <th>Tipo</th>
                  <th>Razão Social / Nome</th>
                  <th>CPF / CNPJ</th>
                  <th>Regime</th>
                  <th>Contato</th>
                  <th>Ações</th>
                </tr>
              </thead>
              <tbody>
                @for (cliente of clientesExibidos(); track cliente.id) {
                  <tr>
                    <td>
                      <acc-badge [status]="cliente.tipo === 'PJ' ? 'info' : 'neutral'">
                        {{ cliente.tipo }}
                      </acc-badge>
                    </td>
                    <td>
                      <div class="client-name">
                        <strong>{{ cliente.nomeRazaoSocial }}</strong>
                        @if (cliente.nomeFantasia) {
                          <span class="client-fantasy">{{ cliente.nomeFantasia }}</span>
                        }
                      </div>
                    </td>
                    <td class="client-doc font-mono">
                      {{ formatarDoc(cliente.cpfCnpj) }}
                    </td>
                    <td>
                      {{ cliente.porteRegime ?? '—' }}
                    </td>
                    <td>
                      <div class="client-contact">
                        @if (cliente.email) {
                          <span>{{ cliente.email }}</span>
                        }
                        @if (cliente.telefone) {
                          <small>{{ cliente.telefone }}</small>
                        }
                      </div>
                    </td>
                    <td>
                      <div class="acoes-cliente">
                        <button
                          acc-button
                          variant="ghost"
                          size="sm"
                          (click)="abrirDetalhes(cliente)"
                        >
                          Detalhes
                        </button>
                        @if (procuracaoValidando().has(cliente.id)) {
                          <acc-spinner size="sm"></acc-spinner>
                        } @else {
                          <button
                            acc-button
                            variant="primary"
                            size="sm"
                            (click)="validarProcuracao(cliente)"
                          >
                            Validar Procuração e-CAC
                          </button>
                        }
                        <acc-badge
                          [status]="statusProcuracaoDe(cliente.id) === 'AUTORIZADA' ? 'success' : 'warning'"
                        >
                          {{ statusProcuracaoDe(cliente.id) === 'AUTORIZADA' ? 'Autorizada' : 'Pendente' }}
                        </acc-badge>
                      </div>
                    </td>
                  </tr>
                } @empty {
                  <tr>
                    <td colspan="6" class="empty-message">
                      Nenhum cliente encontrado para o filtro aplicado.
                    </td>
                  </tr>
                }
              </tbody>
            </table>
          </div>
        </acc-card>
      }

      <!-- Erro na validação de procuração -->
      @if (erroValidacao(); as erro) {
        <div class="alert-error" role="alert">
          <p><strong>Não foi possível validar a procuração.</strong> {{ erro }}</p>
          <button acc-button variant="outline" size="sm" (click)="erroValidacao.set(null)">
            Fechar
          </button>
        </div>
      }
    </div>
  `,
  styles: `
    .clientes-container {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
      padding: 1.5rem;
      max-width: 1200px;
      margin: 0 auto;
    }

    .clientes-header {
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
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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
      color: var(--acc-color-primary);
    }

    .filter-bar {
      display: flex;
      flex-wrap: wrap;
      gap: 1rem;
      align-items: center;
      justify-content: space-between;
      padding: 1rem;
    }

    .search-input-wrapper {
      flex: 1;
      min-width: 280px;
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .search-input {
      width: 100%;
      padding: 0.5rem 0.75rem;
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-size: 0.875rem;
      outline: none;
      transition: border-color 0.15s ease;
    }

    .search-input:focus {
      border-color: var(--acc-color-primary);
    }

    .doc-warning {
      color: var(--acc-color-warning);
      font-size: 0.75rem;
    }

    .filter-actions {
      display: flex;
      gap: 0.5rem;
      align-items: center;
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

    .acc-table tbody tr:hover {
      background-color: rgba(0, 0, 0, 0.015);
    }

    .client-name {
      display: flex;
      flex-direction: column;
      gap: 0.125rem;
    }

    .client-fantasy {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
    }

    .client-contact {
      display: flex;
      flex-direction: column;
      font-size: 0.8125rem;
    }

    .empty-message {
      text-align: center;
      padding: 3rem 1rem !important;
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

    .font-mono {
      font-family: monospace;
    }

    .acoes-cliente {
      display: flex;
      gap: 0.5rem;
      align-items: center;
      flex-wrap: wrap;
    }

    .btn-sair {
      color: var(--acc-color-danger);
      border-color: var(--acc-color-danger);
    }

    .btn-sair:hover {
      background-color: var(--acc-color-danger-bg);
    }
  `,
})
export class ClienteListComponent implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly procuracaoService = inject(ProcuracaoService);
  protected readonly router = inject(Router);
  protected readonly auth = inject(AuthService);
  private readonly destroyRef = inject(DestroyRef);

  readonly termoBusca = signal<string>('');
  readonly filtroAtivo = signal<FiltroClientes>('todos');

  readonly clientesExibidos = computed(() => {
    const termo = this.termoBusca().trim();
    if (!termo) {
      return this.clienteService.clientes();
    }
    return this.clienteService.filtrarPorTextoOuDocumento(termo);
  });

  readonly validacaoDocumentoAviso = computed(() => {
    const termo = this.termoBusca().trim();
    if (termo.length >= 11 && /\d/.test(termo)) {
      if (!isValidDocument(termo)) {
        return 'O número de documento informado parece incompleto ou com dígito verificador inválido.';
      }
    }
    return null;
  });

  // Estado da validação de procuração e-CAC por cliente (sessão).
  // O serviço guarda o status (Autorizada/Pendente); aqui ficam
  // os clientes com requisição em andamento e a última mensagem de erro.
  readonly procuracaoValidando = signal<ReadonlySet<string>>(new Set());
  readonly erroValidacao = signal<string | null>(null);

  ngOnInit(): void {
    this.clienteService.listar();
  }

  onSearchInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.termoBusca.set(target.value);
  }

  aplicarFiltro(filtro: FiltroClientes): void {
    this.filtroAtivo.set(filtro);
    this.clienteService.listar(filtro);
  }

  recarregar(): void {
    this.clienteService.listar(this.filtroAtivo());
  }

  formatarDoc(doc: string): string {
    return formatDocument(doc);
  }

  abrirModalNovo(): void {
    this.router.navigate(['/clientes/novo']);
  }

  abrirDetalhes(cliente: ClienteDto): void {
    this.router.navigate(['/clientes', cliente.id]);
  }

  protected sair(): void {
    this.auth.logout();
  }

  // Status exibido no acc-badge por cliente (ausência = Pendente).
  statusProcuracaoDe(clienteId: string): 'AUTORIZADA' | 'PENDENTE' {
    return this.procuracaoService.statusDe(clienteId);
  }

  private marcarValidando(clienteId: string, ativo: boolean): void {
    this.procuracaoValidando.update((atual) => {
      const proximo = new Set(atual);
      if (ativo) {
        proximo.add(clienteId);
      } else {
        proximo.delete(clienteId);
      }
      return proximo;
    });
  }

  // Valida a procuração e-CAC do cliente junto ao backend (AUTENTICAPROCURADOR).
  validarProcuracao(cliente: ClienteDto): void {
    this.erroValidacao.set(null);
    this.marcarValidando(cliente.id, true);

    this.procuracaoService
      .validarProcuracao(cliente.id)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: () => {
          this.marcarValidando(cliente.id, false);
        },
        error: (erro: { error?: { message?: string }; message?: string }) => {
          this.marcarValidando(cliente.id, false);
          this.erroValidacao.set(
            erro?.error?.message ?? erro?.message ?? 'Falha ao validar a procuração e-CAC.',
          );
        },
      });
  }
}
