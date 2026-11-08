import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { UsuarioDto } from '@accurit/core';
import { formatDocument, formatPtBrDateTime } from '@accurit/shared';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccCtaButtonComponent,
  AccFilterChipsComponent,
  AccFilterOption,
  AccIconComponent,
  AccModalComponent,
  AccSearchFieldComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { SessionStore } from '../../core/auth/session.store';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { UsuarioService } from '../../data/usuario.service';
import { UsuarioEdicaoComponent } from './usuario-edicao.component';
import { UsuarioFormComponent } from './usuario-form.component';
import { UsuarioSenhaComponent } from './usuario-senha.component';

type FiltroUsuarios = 'todos' | 'ativos' | 'inativos';

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

  protected readonly usuariosExibidos = computed(() => {
    const termo = this.termoBusca().trim().toLowerCase();
    const digitos = termo.replace(/\D/g, '');
    return this.usuarioService.usuarios().filter((u) => {
      if (!termo) return true;
      return (
        u.nome.toLowerCase().includes(termo) ||
        u.email.toLowerCase().includes(termo) ||
        (digitos.length > 0 && u.cpf.includes(digitos))
      );
    });
  });

  ngOnInit(): void {
    this.usuarioService.listar(null);
  }

  protected aplicarFiltro(filtro: FiltroUsuarios): void {
    this.filtroAtivo.set(filtro);
    this.usuarioService.listar(filtro === 'todos' ? null : filtro === 'ativos');
  }

  protected recarregar(): void {
    this.aplicarFiltro(this.filtroAtivo());
  }

  protected ehVoce(u: UsuarioDto): boolean {
    return u.id === this.sessao.user()?.id;
  }

  protected rotuloPapel(papel: string): string {
    return papel === 'CLIENTE' ? 'Titular' : 'Operador';
  }

  protected formatarCpf(cpf: string): string {
    return formatDocument(cpf);
  }

  protected formatarData(iso: string | null): string {
    return iso ? formatPtBrDateTime(iso) : 'Nunca acessou';
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
}
