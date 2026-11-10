import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { DatePipe } from '@angular/common';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccModalComponent,
} from '@accurit/ui';
import { AdministradorDto } from '@accurit/core';
import { AdministradorService } from '../../data/administrador.service';
import { mensagemDeErro } from '../../core/http/mensagem-erro';

type ModoModal = 'criar' | 'editar' | 'senha';

@Component({
  selector: 'app-administradores-list',
  standalone: true,
  imports: [ReactiveFormsModule, DatePipe, AccButtonComponent, AccBadgeComponent, AccModalComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-header">
      <h1>Administradores</h1>
      <p class="text-secondary">Pessoas autorizadas a operar o backoffice SaaS Accurit.</p>
    </div>

    @if (erro()) {
      <div class="alert alert-danger" role="alert">{{ erro() }}</div>
    }

    <div class="filtros">
      <input
        class="input"
        type="search"
        placeholder="Buscar por nome ou e-mail"
        [value]="busca()"
        (input)="busca.set($any($event.target).value)"
        (keyup.enter)="carregar(1)"
      />
      <button acc-button variant="outline" type="button" (click)="carregar(1)">Buscar</button>
      <button acc-button variant="primary" type="button" (click)="abrirCriar()">Novo administrador</button>
    </div>

    <div class="table-responsive">
      <table class="table">
        <thead>
          <tr>
            <th>Nome</th>
            <th>E-mail</th>
            <th>CPF</th>
            <th>Status</th>
            <th>Último login</th>
            <th>Ações</th>
          </tr>
        </thead>
        <tbody>
          @for (a of administradores(); track a.id) {
            <tr>
              <td>{{ a.nome }}</td>
              <td>{{ a.email }}</td>
              <td>{{ a.cpf }}</td>
              <td>
                <acc-badge [status]="a.ativo ? 'success' : 'danger'">
                  {{ a.ativo ? 'Ativo' : 'Inativo' }}
                </acc-badge>
              </td>
              <td>{{ a.ultimoLoginEm ? (a.ultimoLoginEm | date: 'dd/MM/yyyy HH:mm') : '—' }}</td>
              <td class="acoes">
                <button acc-button size="sm" variant="outline" (click)="abrirEditar(a)">Editar</button>
                <button acc-button size="sm" variant="outline" (click)="abrirSenha(a)">Senha</button>
                <button acc-button size="sm" variant="danger" (click)="alternarAtivo(a)">
                  {{ a.ativo ? 'Desativar' : 'Ativar' }}
                </button>
              </td>
            </tr>
          } @empty {
            <tr><td colspan="6">Nenhum administrador encontrado.</td></tr>
          }
        </tbody>
      </table>
    </div>

    <div class="paginacao">
      <button acc-button size="sm" variant="outline" [disabled]="pagina() <= 1" (click)="carregar(pagina() - 1)">
        Anterior
      </button>
      <span>Página {{ pagina() }} de {{ totalPaginas() }}</span>
      <button
        acc-button
        size="sm"
        variant="outline"
        [disabled]="pagina() >= totalPaginas()"
        (click)="carregar(pagina() + 1)"
      >
        Próxima
      </button>
    </div>

    @if (modo()) {
      <acc-modal [title]="tituloModal()" (fechar)="fechar()">
        <form [formGroup]="form" (ngSubmit)="salvar()" class="form" novalidate>
          @if (modo() !== 'senha') {
            <label>Nome <input class="input" formControlName="nome" /></label>
            <label>E-mail <input class="input" type="email" formControlName="email" /></label>
          }
          @if (modo() === 'criar') {
            <label>CPF <input class="input" formControlName="cpf" placeholder="000.000.000-00" /></label>
          }
          @if (modo() !== 'editar') {
            <label>
              Senha
              <input class="input" type="password" formControlName="senha" autocomplete="new-password" />
            </label>
          }
          @if (erroForm()) { <p class="field-error">{{ erroForm() }}</p> }
          <div class="acoes">
            <button acc-button type="button" variant="outline" (click)="fechar()">Cancelar</button>
            <button acc-button type="submit" variant="primary" [loading]="salvando()">Salvar</button>
          </div>
        </form>
      </acc-modal>
    }
  `,
  styles: `
    .page-header { margin-bottom: 1rem; }
    .filtros { display: flex; gap: .5rem; margin-bottom: 1rem; align-items: center; }
    .filtros .input { flex: 1; }
    .table { width: 100%; border-collapse: collapse; }
    .table th, .table td { text-align: left; padding: .5rem; border-bottom: 1px solid #e5e7eb; }
    .acoes { display: flex; gap: .35rem; flex-wrap: wrap; }
    .paginacao { display: flex; gap: .75rem; align-items: center; margin-top: 1rem; }
    .form { display: grid; gap: .75rem; }
    .form label { display: grid; gap: .25rem; }
    .field-error { color: #b91c1c; }
  `,
})
export class AdministradoresListComponent implements OnInit {
  private readonly servico = inject(AdministradorService);
  private readonly fb = inject(FormBuilder).nonNullable;

  readonly administradores = signal<AdministradorDto[]>([]);
  readonly carregando = signal(false);
  readonly salvando = signal(false);
  readonly erro = signal<string | null>(null);
  readonly erroForm = signal<string | null>(null);
  readonly pagina = signal(1);
  readonly total = signal(0);
  readonly busca = signal('');
  readonly modo = signal<ModoModal | null>(null);
  readonly selecionado = signal<AdministradorDto | null>(null);

  readonly form = this.fb.group({
    nome: [''],
    email: [''],
    cpf: [''],
    senha: [''],
  });

  readonly totalPaginas = () => Math.max(1, Math.ceil(this.total() / 20));

  ngOnInit(): void {
    void this.carregar(1);
  }

  async carregar(pagina: number): Promise<void> {
    this.carregando.set(true);
    this.erro.set(null);
    try {
      const resposta = await this.servico.listar(Math.max(1, pagina), 20, this.busca());
      this.administradores.set(resposta.itens);
      this.total.set(resposta.total);
      this.pagina.set(resposta.page);
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, 'Falha ao carregar administradores.'));
    } finally {
      this.carregando.set(false);
    }
  }

  abrirCriar(): void {
    this.selecionado.set(null);
    this.form.reset({ nome: '', email: '', cpf: '', senha: '' });
    this.form.controls.nome.setValidators([Validators.required]);
    this.form.controls.email.setValidators([Validators.required, Validators.email]);
    this.form.controls.cpf.setValidators([Validators.required]);
    this.form.controls.senha.setValidators([Validators.required, Validators.minLength(8)]);
    this.form.updateValueAndValidity();
    this.modo.set('criar');
  }

  abrirEditar(administrador: AdministradorDto): void {
    this.selecionado.set(administrador);
    this.form.reset({ nome: administrador.nome, email: administrador.email, cpf: '', senha: '' });
    this.form.controls.nome.setValidators([Validators.required]);
    this.form.controls.email.setValidators([Validators.required, Validators.email]);
    this.form.controls.cpf.clearValidators();
    this.form.controls.senha.clearValidators();
    this.form.updateValueAndValidity();
    this.modo.set('editar');
  }

  abrirSenha(administrador: AdministradorDto): void {
    this.selecionado.set(administrador);
    this.form.reset({ nome: '', email: '', cpf: '', senha: '' });
    this.form.controls.nome.clearValidators();
    this.form.controls.email.clearValidators();
    this.form.controls.cpf.clearValidators();
    this.form.controls.senha.setValidators([Validators.required, Validators.minLength(8)]);
    this.form.updateValueAndValidity();
    this.modo.set('senha');
  }

  fechar(): void {
    this.modo.set(null);
    this.erroForm.set(null);
  }

  async salvar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const v = this.form.getRawValue();
    // A validação do backend é a autoridade final; campos opcionais ficam vazios aqui.
    this.salvando.set(true);
    this.erroForm.set(null);
    try {
      if (this.modo() === 'criar') {
        await this.servico.criar({ nome: v.nome, email: v.email, cpf: v.cpf, senha: v.senha });
      } else if (this.modo() === 'editar' && this.selecionado()) {
        await this.servico.atualizar(this.selecionado()!.id, { nome: v.nome, email: v.email });
      } else if (this.modo() === 'senha' && this.selecionado()) {
        await this.servico.alterarSenha(this.selecionado()!.id, { senha: v.senha });
      }
      this.fechar();
      await this.carregar(this.pagina());
    } catch (erro) {
      this.erroForm.set(mensagemDeErro(erro, 'Não foi possível salvar o administrador.'));
    } finally {
      this.salvando.set(false);
    }
  }

  async alternarAtivo(administrador: AdministradorDto): Promise<void> {
    const acao = administrador.ativo ? 'desativar' : 'ativar';
    if (!confirm(`Confirma ${acao} ${administrador.nome}?`)) return;
    try {
      await this.servico.atualizar(administrador.id, { ativo: !administrador.ativo });
      await this.carregar(this.pagina());
    } catch (erro) {
      this.erro.set(mensagemDeErro(erro, `Não foi possível ${acao} o administrador.`));
    }
  }

  protected tituloModal(): string {
    return this.modo() === 'criar'
      ? 'Novo administrador'
      : this.modo() === 'editar'
        ? 'Editar administrador'
        : 'Redefinir senha';
  }
}
