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
  EstadoProcuracaoClienteDto,
} from '@accurit/core';
import { sanitizeDocument } from '@accurit/shared';
import { ProcuracaoService } from '../../data/procuracao.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
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
  AccBackButtonComponent,
  AccPageHeaderComponent,
  AccTab,
} from '@accurit/ui';
import { AbaId, EstadoCliente, LeituraEstado } from './cliente-detalhes.types';

/** Abas por `?aba=`; "Unidades" é placeholder da fase seguinte (FASE 2). */
const ABAS: readonly AccTab[] = [
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
    AccBackButtonComponent,
    AccPageHeaderComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './cliente-detalhes.page.html',
  styleUrl: './cliente-detalhes.page.css',
})
export class ClienteDetalhesPage implements OnInit {
  protected readonly consultas = inject(ConsultasFiscaisService);
  protected readonly clienteService = inject(ClienteService);
  protected readonly sessao = inject(SessionStore);
  private readonly procuracaoService = inject(ProcuracaoService);
  protected readonly procuracaoValidando = signal(false);
  protected readonly procuracaoCarregando = signal(false);
  protected readonly erroProcuracao = signal<string | null>(null);

  protected readonly dadosProcuracao = computed<EstadoProcuracaoClienteDto | null>(() => {
    const id = this.cliente()?.id;
    return id ? this.procuracaoService.estadoDe(id) : null;
  });

  protected readonly statusProcuracao = computed(() => {
    return this.dadosProcuracao()?.status ?? 'PENDENTE';
  });

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
    // Dados do cliente prontos: histórico, situação fiscal e procurações persistentes
    await this.consultas.carregar(id);
    this.carregarProcuracao(id);
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

  protected abrirIntegraContador(): void {
    const cliente = this.cliente();
    if (!cliente) return;
    this.router.navigate(['/integra-contador'], { queryParams: { clienteId: cliente.id } });
  }

  protected recarregarCliente(): void {
    if (this.idCliente) {
      void this.carregarCliente(this.idCliente);
      this.carregarProcuracao(this.idCliente);
    }
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

  protected carregarProcuracao(clienteId: string): void {
    this.procuracaoCarregando.set(true);
    this.procuracaoService.obterPorCliente(clienteId).subscribe({
      next: () => this.procuracaoCarregando.set(false),
      error: () => this.procuracaoCarregando.set(false),
    });
  }

  protected validarProcuracao(clienteId: string): void {
    this.erroProcuracao.set(null);
    this.procuracaoValidando.set(true);
    this.procuracaoService.validarProcuracao(clienteId).subscribe({
      next: () => this.procuracaoValidando.set(false),
      error: (erro: unknown) => {
        this.procuracaoValidando.set(false);
        this.erroProcuracao.set(mensagemDeErro(erro, 'Falha ao validar a procuração e-CAC.'));
      },
    });
  }

  protected revalidarProcuracao(clienteId: string): void {
    this.erroProcuracao.set(null);
    this.procuracaoValidando.set(true);
    this.procuracaoService.revalidarProcuracao(clienteId).subscribe({
      next: () => this.procuracaoValidando.set(false),
      error: (erro: unknown) => {
        this.procuracaoValidando.set(false);
        this.erroProcuracao.set(mensagemDeErro(erro, 'Falha ao revalidar a procuração e-CAC.'));
      },
    });
  }

  protected rotuloStatusProcuracao(status: string | null | undefined): string {
    switch (status) {
      case 'AUTORIZADA':
        return 'Autorizada';
      case 'NAO_AUTORIZADA':
        return 'Não autorizada';
      case 'INDETERMINADA':
        return 'Indeterminada';
      default:
        return 'Pendente';
    }
  }

  protected badgeStatusProcuracao(status: string | null | undefined): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'AUTORIZADA':
        return 'success';
      case 'NAO_AUTORIZADA':
        return 'danger';
      case 'INDETERMINADA':
      case 'PENDENTE':
      default:
        return 'warning';
    }
  }

  protected rotuloEstadoServico(estado: string): string {
    switch (estado) {
      case 'NAO_EXIGIDA':
        return 'Não exigida';
      case 'AUTORIZADA':
        return 'Autorizada';
      case 'CONSULTADA':
        return 'Consultada';
      case 'PENDENTE':
        return 'Pendente';
      default:
        return estado;
    }
  }

  protected badgeEstadoServico(estado: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (estado) {
      case 'AUTORIZADA':
        return 'success';
      case 'NAO_EXIGIDA':
      case 'CONSULTADA':
        return 'info';
      case 'PENDENTE':
      default:
        return 'warning';
    }
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
      return digitos.replace(/(\\d{2})(\\d{4})(\\d{4})/, '($1) $2-$3');
    }
    if (digitos.length === 11) {
      return digitos.replace(/(\\d{2})(\\d{5})(\\d{4})/, '($1) $2-$3');
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
