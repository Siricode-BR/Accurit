import {
  Component,
  ChangeDetectionStrategy,
  OnInit,
  inject,
  signal,
  computed,
} from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  ClienteDto,
  EstadoConsultaFiscal,
  SituacaoFiscalServicoDto,
  ConsultaFiscalDto,
} from '@accurit/core';
import { sanitizeDocument } from '@accurit/shared';
import { ClienteService } from '../../data/cliente.service';
import { ConsultasFiscaisService } from '../../data/consultas-fiscais.service';
import { SessionStore } from '../../core/auth/session.store';
import {
  AccButtonComponent,
  AccCardComponent,
  AccTabsComponent,
  AccEmptyStateComponent,
  AccSkeletonComponent,
  AccCopyButtonComponent,
  AccBadgeComponent,
  AccTab,
} from '@accurit/ui';

type EstadoCliente = 'carregando' | 'pronto' | 'naoEncontrado' | 'erro';
type AbaId = 'visao-geral' | 'consultas' | 'contatos' | 'unidades';

/** Abas por `?aba=`; "Unidades" é placeholder da fase seguinte (FASE 2). */
const ABAS:readonly AccTab[] = [
  { id: 'visao-geral', label: 'Visão geral' },
  { id: 'consultas', label: 'Consultas' },
  { id: 'contatos', label: 'Contatos' },
  { id: 'unidades', label: 'Unidades', emBreve: true },
];

const ABAS_VALIDAS: readonly string[] = ABAS.map((t) => t.id);

/** Estados aceitos no filtro `?estado=` — espelham o backend (SPEC §4.5). */
const ESTADOS_FILTRO: readonly EstadoConsultaFiscal[] = [
  'SUCESSO',
  'SEM_PENDENCIAS',
  'NAO_ENCONTRADO',
  'NAO_AUTORIZADO',
  'LIMITE_EXCEDIDO',
  'INDISPONIVEL',
  'RESPOSTA_INVALIDA',
  'INDETERMINADO',
  'ERRO_INTERNO',
];

interface LeituraEstado {
  /** rótulo curto do cartão/badge em pt-BR */
  readonly rotulo: string;
  /** cor do badge: success = sem pendências; warning = atenção; danger = falha */
  readonly status: 'success' | 'warning' | 'danger' | 'info' | 'neutral';
  /** texto explicativo sob o estado */
  readonly descricao: string;
}

/** Mapa único estado → leitura em português (nunca expõe código ao usuário). */
const LEITURAS_ESTADO: Record<EstadoConsultaFiscal | 'NUNCA', LeituraEstado> = {
  SUCESSO: {
    rotulo: 'Dados recuperados',
    status: 'info',
    descricao: 'Última consulta retornou informações da Dachser.',
  } as unknown as LeituraEstado,
  SEM_PENDENCIAS: {
    rotulo: 'Sem pendências',
    status: 'success',
    descricao: 'A última consulta não encontrou inscrições para esta chave.',
  },
  NAO_ENCONTRADO: {
    rotulo: 'Não encontrado',
    status: 'neutral',
    descricao: 'A chave não é de um contribuinte devedor ativo.',
  },
  NAO_AUTORIZADO: {
    rotulo: 'Falha de autorização',
    status: 'danger',
    descricao: 'As credenciais de consulta foram recusadas. Tente novamente.',
  },
  LIMITE_EXCEDIDO: {
    rotulo: 'Limite excedido',
    status: 'warning',
    descricao: 'Limite de consultas atingido. Aguarde alguns instantes.',
  },
  INDISPONIVEL: {
    rotulo: 'Serviço indisponível',
    status: 'danger',
    descricao: 'O serviço de consulta não respondeu. Tente novamente.',
  },
  RESPOSTA_INVALIDA: {
    rotulo: 'Resposta inválida',
    status: 'danger',
    descricao: 'A resposta veio fora do formato esperado.',
  },
  INDETERMINADO: {
    rotulo: 'Estado indeterminado',
    status: 'warning',
    descricao: 'A consulta não foi confirmada. Não repita imediatamente.',
  },
  ERRO_INTERNO: {
    rotulo: 'Erro interno',
    status: 'danger',
    descricao: 'Falha inesperada durante a consulta.',
  },
  NUNCA: {
    rotulo: 'Nunca consultado',
    status: 'neutral',
    descricao: 'Ainda não há consulta registrada para este cliente.',
  },
};

function leituraEstado(estado: EstadoConsultaFiscal | 'NUNCA'): LeituraEstado {
  return LEITURAS_ESTADO[estado] ?? LEITURAS_ESTADO.ERRO_INTERNO;
}

function formatarData(value: string | null | undefined): string {
  if (!value) return '—';
  const data = new Date(value);
  if (Number.isNaN(data.getTime())) return value;
  return data.toLocaleString('pt-BR');
}

@Component({
  selector: 'app-cliente-detalhes-page',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccCardComponent,
    AccTabsComponent,
    AccEmptyStateComponent,
    AccSkeletonComponent,
    AccCopyButtonComponent,
    AccBadgeComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-container">
      <header class="page-header">
        <button
          acc-button
          variant="ghost"
          size="sm"
          (click)="voltarParaLista()"
        >
          ← Voltar à lista
        </button>
        <h1 class="page-title">Detalhes do cliente</h1>
        @if (cliente(); as c) {
          <span class="type-badge">{{ c.tipo }}</span>
        }
        <div class="page-acoes">
          @if (podeEditar()) {
            <button
              acc-button
              variant="outline"
              size="sm"
              (click)="abrirEdicao()"
              [disabled]="estadoCliente() !== 'pronto'"
            >
              Editar
            </button>
          }
          <button
            acc-button
            variant="primary"
            size="sm"
            (click)="abrirDividaAtiva()"
            [disabled]="estadoCliente() !== 'pronto'"
          >
            Consultar Dívida Ativa
          </button>
        </div>
      </header>

      @if (estadoCliente() === 'carregando') {
        <div class="carregando-cliente" aria-hidden="true">
          @for (_ of [1, 2, 3]; track $index) {
            <acc-skeleton variante="linha" altura="1.5rem" largura="100%" />
          }
        </div>
      }

      <!-- 404 alheio/inexistente: mesmo texto, sem vazar existência -->
      @if (estadoCliente() === 'naoEncontrado') {
        <acc-empty-state
          title="Cliente não encontrado"
          descricao="Verifique o endereço ou volte à lista para escolher um cliente."
        >
          <button acc-button variant="outline" size="md" (click)="voltarParaLista()">
            Voltar à lista
          </button>
        </acc-empty-state>
      }

      @if (estadoCliente() === 'erro') {
        <div class="alert-error" role="alert" aria-live="polite">{{ erroCliente() }}</div>
        <button acc-button variant="outline" size="md" (click)="recarregarCliente()">
          Tentar novamente
        </button>
      }

      @if (cliente(); as c) {
        <acc-tabs
          [tabs]="abas"
          [active]="abaSincronizada()"
          (activeChange)="trocarAba($event)"
        />

        <!-- ===== Aba: Visão geral ===== -->
        @if (abaSincronizada() === 'visao-geral') {
          <section class="section">
            <div class="dado-card-grid">
              <acc-card class="dado-card">
                <h2 class="section-title">Dados cadastrais</h2>
                <dl class="dado-lista">
                  <div class="dado-item">
                    <dt>Tipo</dt>
                    <dd>{{ c.tipo === 'PF' ? 'Pessoa física' : 'Pessoa jurídica' }}</dd>
                  </div>
                  <div class="dado-item">
                    <dt>{{ c.tipo === 'PF' ? 'CPF' : 'CNPJ' }}</dt>
                    <dd>
                      <span class="dado-valor">{{ aplicarMascara(c.cpfCnpj) }}</span>
                      <acc-copy-button [valor]="c.cpfCnpj" />
                    </dd>
                  </div>
                  <div class="dado-item">
                    <dt>Nome / Razão social</dt>
                    <dd>{{ c.nomeRazaoSocial }}</dd>
                  </div>
                  @if (c.nomeFantasia) {
                    <div class="dado-item">
                      <dt>Nome fantasia</dt>
                      <dd>{{ c.nomeFantasia }}</dd>
                    </div>
                  }
                  @if (c.porteRegime) {
                    <div class="dado-item">
                      <dt>Porte / Regime</dt>
                      <dd>{{ rotuloPorte(c.porteRegime) }}</dd>
                    </div>
                  }
                  <div class="dado-item">
                    <dt>Cadastrado em</dt>
                    <dd>{{ formatarData(c.criadoEm) }}</dd>
                  </div>
                  <div class="dado-item">
                    <dt>Atualizado em</dt>
                    <dd>{{ formatarData(c.atualizadoEm) }}</dd>
                  </div>
                </dl>
              </acc-card>
            </div>
          </section>

          <section class="section">
            <div class="section-header">
              <h2 class="section-title">Situação fiscal</h2>
            </div>
            @if (consultas.avisoConsultivo(); as aviso) {
              <p class="aviso-consultivo" role="note">{{ aviso }}</p>
            }

            @if (consultas.situacaoCarregando()) {
              <div class="cartoes-grid" aria-hidden="true">
                @for (_ of [1, 2]; track $index) {
                  <acc-skeleton variante="card" altura="10rem" largura="100%" />
                }
              </div>
            } @else if (consultas.erroSituacao(); as erro) {
              <div class="alert-error" role="alert" aria-live="polite">{{ erro }}</div>
              <button acc-button variant="outline" size="sm" (click)="recarregarSituacao()">
                Tentar novamente
              </button>
            } @else {
              <div class="cartoes-grid">
                @for (cartao of cartoesFiscais(); track cartao.servico) {
                  <acc-card class="cartao-fiscal" [attr.data-estado]="cartao.estado.leitura.status">
                    <h3 class="cartao-titulo">{{ cartao.rotuloServico }}</h3>
                    <acc-badge [status]="cartao.estado.leitura.status">
                      {{ cartao.estado.leitura.rotulo }}
                    </acc-badge>
                    <p class="cartao-descricao">{{ cartao.estado.leitura.descricao }}</p>
                    <ul class="cartao-meta">
                      <li>
                        <span>Cadastrado em</span>
                        <strong>{{ formatarData(cartao.criadoEm) }}</strong>
                      </li>
                      <li>
                        <span>Ambiente</span>
                        <strong>{{ rotuloAmbiente(cartao.ambiente) }}</strong>
                      </li>
                      @if (cartao.chaveMascarada) {
                        <li>
                          <span>Chave consultada</span>
                          <strong>{{ cartao.chaveMascarada }}</strong>
                        </li>
                      }
                    </ul>
                  </acc-card>
                } @empty {
                  <p class="aviso-consultivo">Nenhuma consulta registrada para este cliente ainda.</p>
                }
              </div>
            }
          </section>
        }

        <!-- ===== Aba: Consultas ===== -->
        @if (abaSincronizada() === 'consultas') {
          <section class="section">
            <div class="filtro-linha">
              <label class="label" for="estado-consulta">Estado</label>
              <select
                id="estado-consulta"
                class="input input--select filtro-estado"
                [value]="consultas.filtroEstado() ?? ''"
                (change)="trocarFiltroEstado($event)"
              >
                <option value="">Todos os estados</option>
                @for (estado of estadosFiltro; track estado) {
                  <option [value]="estado">{{ leitura(estado).rotulo }}</option>
                }
              </select>
            </div>

            @if (consultas.avisoConsultivo(); as aviso) {
              <p class="aviso-consultivo" role="note">{{ aviso }}</p>
            }

            @if (consultas.carregando()) {
              <div class="lista-consultas" aria-hidden="true">
                @for (_ of [1, 2, 3]; track $index) {
                  <acc-skeleton variante="linha" altura="3rem" largura="100%" />
                }
              </div>
            } @else if (consultas.erroCarregamento(); as erro) {
              <div class="alert-error" role="alert" aria-live="polite">{{ erro }}</div>
              <button acc-button variant="outline" size="md" (click)="recarregarConsultas()">
                Tentar novamente
              </button>
            } @else if (consultas.itens().length === 0) {
              <acc-empty-state
                title="Nenhuma consulta registrada"
                descricao="As consultas de Dívida Ativa deste cliente aparecerão aqui."
              >
                <button acc-button variant="primary" size="md" (click)="abrirDividaAtiva()">
                  Consultar agora
                </button>
              </acc-empty-state>
            } @else {
              <ul class="lista-consultas" role="list">
                @for (consulta of consultas.itens(); track consulta.id) {
                  <li>
                    <button
                      type="button"
                      class="consulta-row"
                      (click)="abrirDividaAtiva()"
                      [attr.aria-label]="
                        'Consulta de ' + rotuloServico(consulta.servico) + ' de ' + formatarData(consulta.criadoEm)
                      "
                    >
                      <span class="consulta-col consulta-col--servico">
                        <strong>{{ rotuloServico(consulta.servico) }}</strong>
                        <span class="consulta-sub">{{ rotuloAmbiente(consulta.ambiente) }}</span>
                      </span>
                      <acc-badge [status]="leitura(consulta.estado).status">
                        {{ leitura(consulta.estado).rotulo }}
                      </acc-badge>
                      <span class="consulta-col consulta-col--quando">
                        <span>{{ formatarData(consulta.criadoEm) }}</span>
                        @if (consulta.chaveConsultaMascarada) {
                          <span class="consulta-sub">{{ consulta.chaveConsultaMascarada }}</span>
                        }
                      </span>
                    </button>
                  </li>
                }
              </ul>

              <div class="paginacao" aria-label="Paginação do histórico">
                <button
                  acc-button
                  variant="ghost"
                  size="sm"
                  [disabled]="!temPaginaAnterior()"
                  (click)="trocarPagina(-1)"
                >
                  ← Anterior
                </button>
                <span class="paginacao-info">
                  Página {{ consultas.pagina() }} de {{ totalDePaginas() }}
                </span>
                <button
                  acc-button
                  variant="ghost"
                  size="sm"
                  [disabled]="!temProximaPagina()"
                  (click)="trocarPagina(1)"
                >
                  Próxima →
                </button>
              </div>
            }
          </section>
        }

        <!-- ===== Aba: Contatos ===== -->
        @if (abaSincronizada() === 'contatos') {
          <section class="section">
            <div class="dado-card-grid">
              <acc-card class="dado-card">
                <h2 class="section-title">Contatos</h2>
                @if (c.email || c.telefone) {
                  <dl class="dado-lista">
                    @if (c.email) {
                      <div class="dado-item">
                        <dt>E-mail</dt>
                        <dd>
                          <span class="dado-valor">{{ c.email }}</span>
                          <acc-copy-button [valor]="c.email" />
                        </dd>
                      </div>
                    }
                    @if (c.telefone) {
                      <div class="dado-item">
                        <dt>Telefone</dt>
                        <dd>
                          <span class="dado-valor">{{ formatarTelefone(c.telefone) }}</span>
                          <acc-copy-button [valor]="c.telefone" />
                        </dd>
                      </div>
                    }
                  </dl>
                } @else {
                  <acc-empty-state
                    title="Sem contatos cadastrados"
                    descricao="Use a aba de edição para atualizar o e-mail e o telefone."
                  >
                    @if (podeEditar()) {
                      <button acc-button variant="outline" size="md" (click)="abrirEdicao()">
                        Editar contatos
                      </button>
                    }
                  </acc-empty-state>
                }
              </acc-card>
            </div>
          </section>
        }

        <!-- ===== Aba: Unidades (fase seguinte) ===== -->
        @if (abaSincronizada() === 'unidades') {
          <section class="section">
            <acc-empty-state
              title="Gestão de unidades em breve"
              descricao="Os endereços deste cliente serão organizados aqui. Configure agora os contatos da matriz."
            />
          </section>
        }
      }
    </div>
  `,
  styles: `
    .page-container {
      max-width: 1000px;
      margin: 0 auto;
      padding: 1.5rem;
    }

    .page-header {
      display: flex;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1.5rem;
      flex-wrap: wrap;
    }

    .page-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .type-badge {
      padding: 0.25rem 0.75rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-bg);
      color: var(--acc-color-text-muted);
      font-size: 0.875rem;
      font-weight: 600;
    }

    .page-acoes {
      margin-left: auto;
      display: flex;
      gap: 0.5rem;
    }

    .carregando-cliente {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      margin-bottom: 1.5rem;
    }

    .section {
      margin-bottom: 2rem;
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
    }

    .section-title {
      font-size: 1.125rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 0.75rem;
    }

    .dado-card-grid {
      display: grid;
      gap: 1rem;
    }

    .dado-lista {
      margin: 0;
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 1rem;
    }

    .dado-item {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .dado-item dt {
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--acc-color-text-muted);
    }

    .dado-item dd {
      margin: 0;
      display: flex;
      align-items: center;
      gap: 0.375rem;
      font-size: 0.9375rem;
      color: var(--acc-color-text-main);
      word-break: break-word;
    }

    .cartoes-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 1rem;
    }

    .cartao-fiscal {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .cartao-titulo {
      margin: 0;
      font-size: 1rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
    }

    .cartao-descricao {
      margin: 0;
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted);
    }

    .cartao-meta {
      margin: 0.25rem 0 0;
      padding: 0;
      list-style: none;
      display: grid;
      gap: 0.375rem;
      border-top: 1px solid var(--acc-color-border);
      padding-top: 0.75rem;
    }

    .cartao-meta li {
      display: flex;
      justify-content: space-between;
      gap: 0.75rem;
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted);
    }

    .cartao-meta strong {
      color: var(--acc-color-text-main);
      font-weight: 600;
      text-align: right;
    }

    .aviso-consultivo {
      margin: 0 0 1rem;
      padding: 0.625rem 0.875rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-warning-bg, #fffbeb);
      color: var(--acc-color-warning, #b45309);
      font-size: 0.8125rem;
    }

    .filtro-linha {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      margin-bottom: 1rem;
      flex-wrap: wrap;
    }

    .filtro-estado {
      max-width: 16rem;
    }

    .lista-consultas {
      margin: 0;
      padding: 0;
      list-style: none;
      display: grid;
      gap: 0.5rem;
    }

    .consulta-row {
      display: grid;
      grid-template-columns: minmax(160px, 1fr) auto minmax(180px, max-content);
      align-items: center;
      gap: 1rem;
      width: 100%;
      padding: 0.75rem 1rem;
      text-align: left;
      background: var(--acc-color-surface);
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-family: inherit;
      cursor: pointer;
      transition: border-color 0.15s ease, box-shadow 0.15s ease;
    }

    .consulta-row:hover {
      border-color: var(--acc-color-primary);
      box-shadow: var(--acc-shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
    }

    .consulta-row:focus-visible {
      outline: 2px solid var(--acc-color-primary);
      outline-offset: 1px;
    }

    .consulta-col {
      display: flex;
      flex-direction: column;
      gap: 0.125rem;
      font-size: 0.875rem;
      color: var(--acc-color-text-main);
    }

    .consulta-col--quando {
      text-align: right;
      align-items: flex-end;
    }

    .consulta-sub {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
    }

    .paginacao {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-top: 1rem;
    }

    .paginacao-info {
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted);
    }

    .input {
      padding: 0.625rem 0.875rem;
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-size: 0.875rem;
      outline: none;
      font-family: inherit;
      background-color: var(--acc-color-input-bg, var(--acc-color-surface));
    }

    .input:focus {
      border-color: var(--acc-color-primary);
      box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.1);
    }

    .input--select {
      appearance: none;
      background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'/%3e%3c/svg%3e");
      background-repeat: no-repeat;
      background-position: right 0.75rem center;
      background-size: 1rem;
      padding-right: 2.5rem;
    }

    .label {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--acc-color-text-main);
    }

    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
      margin-bottom: 1rem;
    }

    @media (max-width: 640px) {
      .page-container {
        padding: 1rem;
      }

      .page-acoes {
        width: 100%;
        margin-left: 0;
      }

      .consulta-row {
        grid-template-columns: 1fr auto;
      }

      .consulta-col--quando {
        grid-column: 1 / -1;
        text-align: left;
        align-items: flex-start;
      }
    }
  `,
})
export class ClienteDetalhesPage implements OnInit {
  protected readonly consultas = inject(ConsultasFiscaisService);
  protected readonly clienteService = inject(ClienteService);
  protected readonly sessao = inject(SessionStore);
  private readonly route = inject(ActivatedRoute);
  protected readonly router = inject(Router);

  protected readonly cliente = signal<ClienteDto | null>(null);
  protected readonly estadoCliente = signal<EstadoCliente>('carregando');
  protected readonly erroCliente = signal<string | null>(null);

  protected readonly aba = signal<AbaId>('visao-geral');
  /** ID/aba em sincronia com a URL (`?aba=`) — fonte única do switch de abas. */
  protected readonly abaSincronizada = this.aba.asReadonly();
  protected readonly abas = ABAS;
  protected readonly estadosFiltro = ESTADOS_FILTRO;

  protected readonly podeEditar = computed(() => this.sessao.user()?.role === 'OPERADOR');

  protected readonly temPaginaAnterior = computed(() => this.consultas.pagina() > 1);
  protected readonly temProximaPagina = computed(
    () => this.consultas.pagina() * this.consultas.tamanho() < this.consultas.total(),
  );
  protected readonly totalDePaginas = computed(() => {
    const total = this.consultas.total();
    if (total === 0) return 1;
    return Math.max(1, Math.ceil(total / this.consultas.tamanho()));
  });

  private idCliente: string | null = null;

  async ngOnInit(): Promise<void> {
    const aba = this.route.snapshot.queryParamMap.get('aba');
    if (aba && ABAS_VALIDAS.includes(aba) && this.ehAbaHabilitada(aba)) {
      this.aba.set(aba as AbaId);
    }

    const id = this.route.snapshot.paramMap.get('id');
    this.idCliente = id;
    if (!id) {
      this.estadoCliente.set('naoEncontrado');
      return;
    }

    await this.carregarCliente(id);
    // Dados do cliente prontos: histórico e cartões em paralelo.
    await this.consultas.carregar(id);
  }

  /** Cartões genéricos por serviço federal (Situação fiscal da visão geral). */
  protected readonly cartoesFiscais = computed<{
    servico: string;
    rotuloServico: string;
    estado: { leitura: LeituraEstado };
    ambiente: string;
    chaveMascarada: string | null;
    criadoEm: string | null;
  }[]>(() => {
    const situacao = this.consultas.situacao();
    if (!situacao) return [];

    return situacao.servicos.map((servico) => ({
      servico: servico.servico,
      rotuloServico: this.rotuloServico(servico.servico),
      estado: { leitura: leituraEstado(servico.estado) },
      ambiente: this.rotuloAmbiente(servico.ambiente),
      chaveMascarada: servico.chaveConsultaMascarada,
      criadoEm: servico.criadoEm,
    }));
  });

  protected ehAbaHabilitada(id: string): boolean {
    const tab = this.abas.find((t) => t.id === id);
    return !!tab && !tab.disabled;
  }

  protected trocarAba(nova: string): void {
    if (!this.ehAbaHabilitada(nova)) return;
    this.aba.set(nova as AbaId);
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { aba: nova },
      queryParamsHandling: 'merge',
    });
  }

  protected voltarParaLista(): void {
    this.router.navigate(['/clientes']);
  }

  protected abrirEdicao(): void {
    const id = this.idCliente ?? this.cliente()?.id;
    if (id) {
      void this.router.navigate(['/clientes', id, 'editar']);
    }
  }

  protected abrirDividaAtiva(): void {
    const cliente = this.cliente();
    if (!cliente) return;
    this.router.navigate(['/divida-ativa'], { queryParams: { clienteId: cliente.id } });
  }

  protected recarregarCliente(): void {
    if (this.idCliente) void this.carregarCliente(this.idCliente);
  }

  protected recarregarConsultas(): void {
    const id = this.idCliente;
    if (id) void this.consultas.carregar(id, { pagina: this.consultas.pagina() });
  }

  protected recarregarSituacao(): void {
    const id = this.idCliente;
    if (id) void this.consultas.carregarSituacao(id);
  }

  protected trocarFiltroEstado(evento: Event): void {
    const valor = (evento.target as HTMLSelectElement).value;
    const id = this.idCliente;
    if (!id) return;
    void this.consultas.carregar(
      id,
      valor === '' ? { pagina: 1 } : { pagina: 1, estado: valor as EstadoConsultaFiscal },
    );
  }

  protected trocarPagina(direcao: -1 | 1): void {
    const id = this.idCliente;
    if (!id) return;
    void this.consultas.carregar(id, { pagina: this.consultas.pagina() + direcao });
  }

  protected leitura(estado: EstadoConsultaFiscal): LeituraEstado {
    return leituraEstado(estado);
  }

  protected rotuloServico(servico: string): string {
    switch (servico) {
      case 'DEVEDOR':
        return 'Devedor (Dívida Ativa)';
      case 'INSCRICAO':
        return 'Inscrição (Dívida Ativa)';
      default:
        return servico;
    }
  }

  protected rotuloAmbiente(ambiente: string): string {
    return ambiente === 'TRIAL' ? 'Ambiente de avaliação' : 'Produção';
  }

  protected rotuloPorte(porte: string): string {
    const rotulos: Record<string, string> = {
      MEI: 'MEI',
      SIMPLES_NACIONAL: 'Simples Nacional',
      LUCRO_PRESUMIDO: 'Lucro presumido',
      LUCRO_REAL: 'Lucro real',
    };
    return rotulos[porte] ?? porte;
  }

  protected formatarData = formatarData;

  protected aplicarMascara(documento: string): string {
    if (documento.length === 11) {
      return documento.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, '$1.$2.$3-$4');
    }
    if (documento.length === 14) {
      return documento.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, '$1.$2.$3/$4-$5');
    }
    return documento;
  }

  protected formatarTelefone(telefone: string): string {
    const digitos = sanitizeDocument(telefone) ?? telefone;
    if (digitos.length === 10) {
      return digitos.replace(/(\d{2})(\d{4})(\d{4})/, '($1) $2-$3');
    }
    if (digitos.length === 11) {
      return digitos.replace(/(\d{2})(\d{5})(\d{4})/, '($1) $2-$3');
    }
    return telefone;
  }

  private async carregarCliente(id: string): Promise<void> {
    this.estadoCliente.set('carregando');
    this.erroCliente.set(null);
    try {
      // 404 de cliente alheio/inexistente chega como `NAO_ENCONTRADO`.
      const cliente = await this.clienteService.obter(id);
      this.cliente.set(cliente);
      this.estadoCliente.set('pronto');
    } catch (erro) {
      const codigo = (erro as { error?: { code?: string } })?.error?.code ?? null;
      if (codigo === 'NAO_ENCONTRADO') {
        this.estadoCliente.set('naoEncontrado');
      } else {
        this.estadoCliente.set('erro');
        this.erroCliente.set(
          (erro as { error?: { message?: string } })?.error?.message ??
            'Falha ao carregar os dados do cliente.',
        );
      }
    }
  }
}
