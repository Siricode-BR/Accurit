import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { UsuarioDto, ClienteDto } from '@accurit/core';
import { formatDocument, formatPtBrDateTime } from '@accurit/shared';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccCtaButtonComponent,
  AccFilterChipsComponent,
  AccIconComponent,
  AccModalComponent,
  AccSearchFieldComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import type { AccFilterOption } from '@accurit/ui';
import { SessionStore } from '../../core/auth/session.store';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { UsuarioService } from '../../data/usuario.service';
import { ClienteService } from '../../data/cliente.service';
import { UsuarioEdicaoComponent } from './usuario-edicao.component';
import { UsuarioFormComponent } from './usuario-form.component';
import { UsuarioSenhaComponent } from './usuario-senha.component';

type FiltroUsuarios = 'todos' | 'ativos' | 'inativos';

interface AcessoEdicao {
  usuarioId: string;
  funcao: 'GESTOR' | 'ANALISTA_FISCAL' | 'OPERADOR_LEITURA';
  clientesPermitidos: Set<string>;
}

@Component({
  selector: 'app-usuario-list',
  standalone: true,
  imports: [
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccCtaButtonComponent,
    AccFilterChipsComponent,
    AccIconComponent,
    AccModalComponent,
    AccSearchFieldComponent,
    AccSpinnerComponent,
    UsuarioFormComponent,
    UsuarioEdicaoComponent,
    UsuarioSenhaComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './usuario-list.component.html',
  styleUrl: './usuario-list.component.css',
})
export class UsuarioListComponent implements OnInit {
  protected readonly usuarioService = inject(UsuarioService);
  protected readonly clienteService = inject(ClienteService);
  private readonly sessao = inject(SessionStore);

  protected readonly opcoesFiltro: readonly AccFilterOption<FiltroUsuarios>[] = [
    { value: 'todos', label: 'Todos' },
    { value: 'ativos', label: 'Ativos' },
    { value: 'inativos', label: 'Inativos' },
  ];

  protected readonly termoBusca = signal('');
  protected readonly filtroAtivo = signal<FiltroUsuarios>('todos');
  protected readonly modalNovoAberto = signal(false);
  protected readonly editando = signal<UsuarioDto | null>(null);
  protected readonly redefinindoSenha = signal<UsuarioDto | null>(null);
  protected readonly alterando = signal<ReadonlySet<string>>(new Set());
  protected readonly erroAcao = signal<string | null>(null);
  protected readonly aviso = signal<string | null>(null);

  /** Gestão de função/carteira (Fase E) — somente gestores do escritório. */
  protected readonly podeGerenciarAcesso = computed(() => {
    const role = this.sessao.user()?.role;
    return role === 'ADMIN' || role === 'CONTADOR';
  });
  protected readonly acessoEdicao = signal<AcessoEdicao | null>(null);

  ngOnInit(): void {
    void this.listarPagina(1);
  }

  protected async listarPagina(pagina: number): Promise<void> {
    const ativo =
      this.filtroAtivo() === 'todos' ? null : this.filtroAtivo() === 'ativos';
    this.usuarioService.listarPaginado({
      ativo,
      busca: this.termoBusca().trim() || undefined,
      pagina,
      tamanho: 20,
    });
  }

  protected aplicarFiltro(filtro: FiltroUsuarios): void {
    this.filtroAtivo.set(filtro);
    void this.listarPagina(1);
  }

  protected buscar(): void {
    void this.listarPagina(1);
  }

  protected anterior(): void {
    const pagina = Math.max(1, this.usuarioService.pagina() - 1);
    void this.listarPagina(pagina);
  }

  protected proxima(): void {
    const pagina = this.usuarioService.pagina() + 1;
    if (pagina > this.usuarioService.totalPaginas()) return;
    void this.listarPagina(pagina);
  }

  protected recarregar(): void {
    void this.listarPagina(this.usuarioService.pagina());
  }

  protected ehVoce(u: UsuarioDto): boolean {
    return u.id === this.sessao.user()?.id;
  }

  protected rotuloPapel(papel: string): string {
    return papel === 'CLIENTE' ? 'Titular' : 'Operador';
  }

  protected rotuloFuncao(funcao: string | null | undefined): string {
    return {
      GESTOR: 'Gestor',
      ANALISTA_FISCAL: 'Analista fiscal',
      OPERADOR_LEITURA: 'Operador (leitura)',
    }[funcao ?? ''] ?? '—';
  }

  protected formatarDocCliente(doc: string): string {
    return formatDocument(doc);
  }

  protected formatarCpf(cpf: string): string {
    return formatDocument(cpf);
  }

  protected formatarData(iso: string | null): string {
    return iso ? formatPtBrDateTime(iso) : 'Nunca acessou';
  }

  protected textoDo(evento: Event): string {
    return ((evento.target as HTMLInputElement)?.value) ?? '';
  }

  protected marcadoDo(evento: Event): boolean {
    return ((evento.target as HTMLInputElement)?.checked) ?? false;
  }

  protected novo(): void {
    this.modalNovoAberto.set(true);
  }

  protected aoCriar(u: UsuarioDto): void {
    this.modalNovoAberto.set(false);
    this.aviso.set(`Usuário ${u.nome} criado. Ele já pode entrar com ${u.email}.`);
  }

  protected editar(u: UsuarioDto): void {
    this.erroAcao.set(null);
    this.editando.set(u);
  }

  protected senha(u: UsuarioDto): void {
    this.erroAcao.set(null);
    this.redefinindoSenha.set(u);
  }

  protected aoRedefinirSenha(): void {
    const u = this.redefinindoSenha();
    this.redefinindoSenha.set(null);
    if (u) this.aviso.set(`Senha de ${u.nome} redefinida.`);
  }

  protected async alternarAtivo(u: UsuarioDto): Promise<void> {
    this.erroAcao.set(null);
    this.alterando.update((s) => new Set(s).add(u.id));
    try {
      await this.usuarioService.atualizar(u.id, { ativo: !u.ativo });
    } catch (e) {
      this.erroAcao.set(mensagemDeErro(e, 'Não foi possível alterar o status do usuário.'));
    } finally {
      this.alterando.update((s) => {
        const p = new Set(s);
        p.delete(u.id);
        return p;
      });
    }
  }

  // ===== Fase E — acesso do usuário (função e carteira) =====

  protected async abrirModalAcesso(u: UsuarioDto): Promise<void> {
    if (!this.podeGerenciarAcesso()) return;
    let acesso;
    try {
      acesso = await this.usuarioService.obterAcesso(u.id);
    } catch (e) {
      this.erroAcao.set(mensagemDeErro(e, 'Não foi possível carregar o acesso do usuário.'));
      return;
    }
    this.acessoEdicao.set({
      usuarioId: u.id,
      funcao: (acesso.funcao as AcessoEdicao['funcao']) ?? 'OPERADOR_LEITURA',
      clientesPermitidos: new Set(acesso.clientesPermitidos ?? []),
    });
  }

  protected fecharModalAcesso(): void {
    this.acessoEdicao.set(null);
  }

  protected clienteSelecionado(clienteId: string): boolean {
    return this.acessoEdicao()?.clientesPermitidos.has(clienteId) ?? false;
  }

  protected selecionarFuncao(valor: string): void {
    const acesso = this.acessoEdicao();
    if (!acesso) return;
    this.acessoEdicao.set({ ...acesso, funcao: valor as AcessoEdicao['funcao'] });
  }

  protected alternarCarteira(clienteId: string): void {
    const acesso = this.acessoEdicao();
    if (!acesso) return;
    const clientes = new Set(acesso.clientesPermitidos);
    if (clientes.has(clienteId)) clientes.delete(clienteId);
    else clientes.add(clienteId);
    this.acessoEdicao.set({ ...acesso, clientesPermitidos: clientes });
  }

  protected rotuloCliente(clienteId: string): string {
    return clienteId; // exibido como ID; ideal seria nome, mas a carteira opera por IDs
  }

  protected async salvarAcesso(): Promise<void> {
    const acesso = this.acessoEdicao();
    if (!acesso) return;
    this.erroAcao.set(null);
    try {
      await this.usuarioService.definirAcesso(acesso.usuarioId, {
        funcao: acesso.funcao,
      });
      await this.usuarioService.definirCarteira(acesso.usuarioId, {
        clientes: [...acesso.clientesPermitidos],
      });
      this.aviso.set('Acesso do usuário atualizado.');
      this.fecharModalAcesso();
      void this.listarPagina(this.usuarioService.pagina());
    } catch (e) {
      this.erroAcao.set(mensagemDeErro(e, 'Não foi possível atualizar o acesso.'));
    }
  }
}
