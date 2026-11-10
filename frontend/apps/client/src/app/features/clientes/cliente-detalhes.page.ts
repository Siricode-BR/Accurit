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
  SocioContatoDto,
  CreateSocioContatoRequestDto,
  PapelSocioContato,
  FilialDto,
  CreateFilialRequestDto,
  ClienteTagDto,
  CreateTagRequestDto,
  CarteiraClienteDto,
  AtualizarCarteiraRequestDto,
  AcessoUsuarioDto,
  FuncaoUsuario,
} from '@accurit/core';
import { sanitizeDocument, isValidDocument } from '@accurit/shared';
import { ProcuracaoService } from '../../data/procuracao.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { ClienteService } from '../../data/cliente.service';
import { ConsultasFiscaisService } from '../../data/consultas-fiscais.service';
import { LgpdService } from '../../data/lgpd.service';
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
  AccSelectComponent,
  AccModalComponent,
} from '@accurit/ui';
import { AbaId, EstadoCliente, LeituraEstado } from './cliente-detalhes.types';
import { TrialMockPanelComponent } from '../serpro/trial-mock-panel.component';

/** Abas por `?aba=`; a aba "unidades" foi substituída pelas abas reais da Fase E. */
const ABAS: readonly AccTab[] = [
  { id: 'visao-geral', label: 'Visão geral' },
  { id: 'consultas', label: 'Consultas' },
  { id: 'contatos', label: 'Contatos' },
  { id: 'socios-contatos', label: 'Sócios e contatos' },
  { id: 'filiais', label: 'Filiais' },
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
    AccSelectComponent,
    AccModalComponent,
    TrialMockPanelComponent,
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
  private readonly lgpdService = inject(LgpdService);
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

  /** Escrita em itens de carteira (sócios, filiais, tags, responsável/observações). */
  protected readonly podeGravarCarteira = computed(() => {
    const role = this.sessao.user()?.role;
    return role !== 'LEITURA';
  });

  /** Função efetiva do usuário logado na carteira (lida do backend por requisição). */
  protected readonly acesso = signal<AcessoUsuarioDto | null>(null);

  protected readonly sociosContatos = signal<SocioContatoDto[]>([]);
  protected readonly filiais = signal<FilialDto[]>([]);
  protected readonly tagsCliente = signal<ClienteTagDto[]>([]);
  protected readonly tagsEscritorio = signal<ClienteTagDto[]>([]);
  protected readonly carteira = signal<CarteiraClienteDto | null>(null);

  protected readonly carregandoSocios = signal(false);
  protected readonly carregandoFiliais = signal(false);
  protected readonly carregandoTags = signal(false);
  protected readonly carregandoCarteira = signal(false);

  protected readonly modalSocioAberto = signal(false);
  protected readonly socioForm = signal<{
    papel: string;
    nome: string;
    documento: string;
    email: string;
    telefone: string;
    representantePrincipal: boolean;
  }>({
    papel: 'SOCIO',
    nome: '',
    documento: '',
    email: '',
    telefone: '',
    representantePrincipal: false,
  });

  protected readonly modalFilialAberto = signal(false);
  protected readonly filialForm = signal<{
    cnpj: string;
    nomeUnidade: string;
    endereco: string;
    telefone: string;
  }>({ cnpj: '', nomeUnidade: '', endereco: '', telefone: '' });

  protected readonly modalTagsAberto = signal(false);
  protected readonly tagVinculada = signal<ReadonlySet<string>>(new Set());

  protected readonly modalCarteiraAberto = signal(false);
  protected readonly carteiraForm = signal<{
    responsavelUsuarioId: string;
    observacoes: string;
  }>({ responsavelUsuarioId: '', observacoes: '' });

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
    await this.carregarItensCarteira(id);
    // Dados do cliente prontos: histórico, situação fiscal e procurações persistentes
    await this.consultas.carregar(id);
    this.carregarProcuracao(id);
  }

  // ===== Fase E — carregamento de carteira =====

  private async carregarItensCarteira(id: string): Promise<void> {
    try {
      await Promise.all([
        this.carregarSociosContatos(id),
        this.carregarFiliais(id),
        this.carregarTags(id),
        this.carregarCarteira(id),
        this.carregarMeuAcesso(),
      ]);
    } catch {
      // Itens opcionais da carteira: não bloqueiam a ficha do cliente.
    }
  }

  private async carregarSociosContatos(id: string): Promise<void> {
    this.carregandoSocios.set(true);
    try {
      this.sociosContatos.set(await this.clienteService.listarSociosContatos(id));
    } finally {
      this.carregandoSocios.set(false);
    }
  }

  private async carregarFiliais(id: string): Promise<void> {
    this.carregandoFiliais.set(true);
    try {
      this.filiais.set(await this.clienteService.listarFiliais(id));
    } finally {
      this.carregandoFiliais.set(false);
    }
  }

  private async carregarTags(id: string): Promise<void> {
    this.carregandoTags.set(true);
    try {
      await Promise.all([
        this.clienteService.listarTagsDoCliente(id).then((t) => this.tagsCliente.set(t)),
        this.clienteService.listarTags().then((t) => this.tagsEscritorio.set(t)),
      ]);
    } finally {
      this.carregandoTags.set(false);
    }
  }

  private async carregarCarteira(id: string): Promise<void> {
    this.carregandoCarteira.set(true);
    try {
      this.carteira.set(await this.clienteService.obterCarteira(id));
    } catch {
      this.carteira.set(null);
    } finally {
      this.carregandoCarteira.set(false);
    }
  }

  private async carregarMeuAcesso(): Promise<void> {
    try {
      this.acesso.set(await this.clienteService.meuAcesso());
    } catch {
      this.acesso.set(null);
    }
  }

  // ===== Fase E — sócios e contatos =====

  protected rotuloPapelSocio(papel: PapelSocioContato): string {
    return papel === 'SOCIO' ? 'Sócio' : 'Contato';
  }

  protected abrirNovoSocio(): void {
    if (!this.podeGravarCarteira()) return;
    this.socioForm.set({
      papel: 'SOCIO',
      nome: '',
      documento: '',
      email: '',
      telefone: '',
      representantePrincipal: false,
    });
    this.modalSocioAberto.set(true);
  }

  protected fecharModalSocio(): void {
    this.modalSocioAberto.set(false);
  }

  protected async salvarSocio(): Promise<void> {
    const id = this.idCliente;
    if (!id) return;
    const doc = this.socioForm().documento.replace(/\D/g, '');
    if (doc && !isValidDocument(doc)) {
      // Validação prévia de CPF/CNPJ no front.
      return;
    }
    try {
      await this.clienteService.criarSocioContato(id, {
        papel: this.socioForm().papel as PapelSocioContato,
        nome: this.socioForm().nome.trim(),
        documento: this.socioForm().documento.trim() || undefined,
        email: this.socioForm().email.trim() || undefined,
        telefone: this.socioForm().telefone.trim() || undefined,
        representantePrincipal: this.socioForm().representantePrincipal,
      });
      this.fecharModalSocio();
      await this.carregarSociosContatos(id);
    } catch (e) {
      // Erro tratado pelo serviço (estado `erro` não exposto aqui); recarrega para consistência.
      await this.carregarSociosContatos(id);
    }
  }

  protected async removerSocio(socioId: string): Promise<void> {
    if (!this.podeGravarCarteira()) return;
    const id = this.idCliente;
    if (!id || !confirm('Remover este sócio/contato?')) return;
    try {
      await this.clienteService.removerSocioContato(id, socioId);
      await this.carregarSociosContatos(id);
    } catch {
      await this.carregarSociosContatos(id);
    }
  }

  // ===== Fase E — filiais =====

  protected abrirNovaFilial(): void {
    if (!this.podeGravarCarteira()) return;
    this.filialForm.set({ cnpj: '', nomeUnidade: '', endereco: '', telefone: '' });
    this.modalFilialAberto.set(true);
  }

  protected fecharModalFilial(): void {
    this.modalFilialAberto.set(false);
  }

  protected async salvarFilial(): Promise<void> {
    const id = this.idCliente;
    if (!id) return;
    const cnpj = this.filialForm().cnpj.replace(/\D/g, '');
    if (!cnpj || !isValidDocument(cnpj)) return;
    try {
      await this.clienteService.criarFilial(id, {
        cnpj,
        nomeUnidade: this.filialForm().nomeUnidade.trim() || undefined,
        endereco: this.filialForm().endereco.trim() || undefined,
        telefone: this.filialForm().telefone.trim() || undefined,
      });
      this.fecharModalFilial();
      await this.carregarFiliais(id);
    } catch {
      await this.carregarFiliais(id);
    }
  }

  protected async removerFilial(filialId: string): Promise<void> {
    if (!this.podeGravarCarteira()) return;
    const id = this.idCliente;
    if (!id || !confirm('Remover esta filial/unidade?')) return;
    try {
      await this.clienteService.removerFilial(id, filialId);
      await this.carregarFiliais(id);
    } catch {
      await this.carregarFiliais(id);
    }
  }

  // ===== Fase E — tags =====

  protected async abrirModalTags(): Promise<void> {
    if (!this.podeGravarCarteira()) return;
    const id = this.idCliente;
    if (!id) return;
    await this.carregarTags(id);
    const vinculados = new Set(this.tagsCliente().map((t) => t.id));
    this.tagVinculada.set(vinculados);
    this.modalTagsAberto.set(true);
  }

  protected fecharModalTags(): void {
    this.modalTagsAberto.set(false);
  }

  protected estaVinculada(tagId: string): boolean {
    return this.tagVinculada().has(tagId);
  }

  protected async alternarTag(tagId: string): Promise<void> {
    const id = this.idCliente;
    if (!id || !this.podeGravarCarteira()) return;
    const vinculada = this.estaVinculada(tagId);
    try {
      if (vinculada) {
        await this.clienteService.desvincularTag(id, tagId);
      } else {
        await this.clienteService.vincularTag(id, tagId);
      }
      this.tagVinculada.update((s) => {
        const p = new Set(s);
        vinculada ? p.delete(tagId) : p.add(tagId);
        return p;
      });
      await this.carregarTags(id);
    } catch {
      await this.carregarTags(id);
    }
  }

  // ===== Fase E — carteira (responsável e observações) =====

  protected abrirModalCarteira(): void {
    if (!this.podeGravarCarteira()) return;
    const c = this.carteira();
    this.carteiraForm.set({
      responsavelUsuarioId: c?.responsavelUsuarioId ?? '',
      observacoes: c?.observacoes ?? '',
    });
    this.modalCarteiraAberto.set(true);
  }

  protected fecharModalCarteira(): void {
    this.modalCarteiraAberto.set(false);
  }

  protected async salvarCarteira(): Promise<void> {
    const id = this.idCliente;
    if (!id) return;
    try {
      await this.clienteService.definirCarteira(id, {
        responsavelUsuarioId: this.carteiraForm().responsavelUsuarioId.trim() || null,
        observacoes: this.carteiraForm().observacoes.trim() || null,
      });
      this.fecharModalCarteira();
      await this.carregarCarteira(id);
    } catch {
      await this.carregarCarteira(id);
    }
  }

  protected documentoSocio(documento?: string | null): string {
    if (!documento) return '—';
    const d = documento.replace(/\D/g, '');
    if (d.length === 11) {
      return d.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, '$1.$2.$3-$4');
    }
    if (d.length === 14) {
      return d.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, '$1.$2.$3/$4-$5');
    }
    return documento;
  }

  protected telefoneSocio(telefone?: string | null): string {
    if (!telefone) return '—';
    const d = sanitizeDocument(telefone) ?? telefone;
    if (d.length === 10) {
      return d.replace(/(\d{2})(\d{4})(\d{4})/, '($1) $2-$3');
    }
    if (d.length === 11) {
      return d.replace(/(\d{2})(\d{5})(\d{4})/, '($1) $2-$3');
    }
    return telefone;
  }

  /** Valor de um campo de formulário disparador do evento (contorna tipagem de `EventTarget`). */
  protected textoDo(evento: Event): string {
    return ((evento.target as HTMLInputElement)?.value) ?? '';
  }

  /** Estado de um checkbox disparador do evento. */
  protected marcadoDo(evento: Event): boolean {
    return ((evento.target as HTMLInputElement)?.checked) ?? false;
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
          mensagemDeErro(erro, 'Falha ao carregar os dados do cliente.'),
        );
      }
    }
  }

  protected async exportarLgpd(): Promise<void> {
    const cliente = this.cliente();
    if (!cliente) return;
    try {
      const blob = await this.lgpdService.exportarCliente(cliente.id);
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `lgpd-cliente-${cliente.id}.json`;
      link.click();
      URL.revokeObjectURL(url);
    } catch (erro) {
      this.erroCliente.set(mensagemDeErro(erro, 'Não foi possível exportar os dados do cliente.'));
    }
  }

  protected async anonimizarLgpd(): Promise<void> {
    const cliente = this.cliente();
    if (!cliente) return;
    if (
      !confirm(
        'Anonimizar os dados pessoais deste cliente? A ação é irreversible e será auditada.',
      )
    ) {
      return;
    }
    try {
      await this.lgpdService.anonimizarCliente(cliente.id);
      await this.carregarCliente(cliente.id);
      this.erroCliente.set('Dados do cliente anonimizados e auditoria registrada.');
    } catch (erro) {
      this.erroCliente.set(mensagemDeErro(erro, 'Não foi possível anonimizar o cliente.'));
    }
  }
}
