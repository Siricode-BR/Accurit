import { ChangeDetectionStrategy, Component, OnInit, inject, input, output, signal } from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { TenantDto, UsuarioDto } from '@accurit/core';
import { isValidCPF, sanitizeDocument } from '@accurit/shared';
import { AccButtonComponent, AccDocumentMaskDirective } from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { UsuarioService } from '../../data/usuario.service';
import { USUARIO_FORM_STYLES } from './usuario-form.styles';

function senhasIguais(grupo: AbstractControl): ValidationErrors | null {
  const senha = grupo.get('senha')?.value;
  const confirmacao = grupo.get('confirmacao')?.value;
  return senha === confirmacao ? null : { senhasDiferentes: true };
}

/** Cadastro de usuário do escritório (conteúdo do modal "Novo usuário"). */
@Component({
  selector: 'app-usuario-form',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent, AccDocumentMaskDirective],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="enviar()" class="form" novalidate>
      @if (erro(); as mensagem) {
        <div class="alert-error" role="alert">{{ mensagem }}</div>
      }

      <div class="field field--full">
        <label for="u-escritorio" class="label">Escritório</label>
        <select id="u-escritorio" formControlName="escritorioId" class="input"
          [class.input--error]="invalido('escritorioId')">
          <option value="">Selecione o escritório</option>
          @for (t of escritorios(); track t.id) {
            <option [value]="t.id">{{ t.nome }} ({{ t.slug }})</option>
          }
        </select>
        @if (invalido('escritorioId')) { <span class="field-error">Selecione o escritório.</span> }
      </div>

      <div class="field field--full">
        <label for="u-nome" class="label">Nome completo</label>
        <input id="u-nome" type="text" formControlName="nome" autocomplete="off" class="input"
          [class.input--error]="invalido('nome')" placeholder="ex: Maria da Silva" />
        @if (invalido('nome')) { <span class="field-error">Informe o nome.</span> }
      </div>

      <div class="field">
        <label for="u-email" class="label">E-mail</label>
        <input id="u-email" type="email" formControlName="email" autocomplete="off" class="input"
          [class.input--error]="invalido('email')" placeholder="usuario@escritorio.com.br" />
        @if (invalido('email')) { <span class="field-error">Informe um e-mail válido.</span> }
        <small class="hint">O e-mail é o login e é único na plataforma.</small>
      </div>

      <div class="field">
        <label for="u-cpf" class="label">CPF</label>
        <input id="u-cpf" type="text" formControlName="cpf" accDocMask autocomplete="off" class="input"
          [class.input--error]="invalido('cpf')" placeholder="000.000.000-00" />
        @if (invalido('cpf')) { <span class="field-error">CPF inválido.</span> }
      </div>

      <div class="field field--full">
        <label for="u-papel" class="label">Papel</label>
        <select id="u-papel" formControlName="papel" class="input">
          <option value="OPERADOR">Operador — cadastra e consulta clientes</option>
          <option value="CLIENTE">Titular — também gerencia usuários</option>
        </select>
      </div>

      <div class="field">
        <label for="u-senha" class="label">Senha</label>
        <input id="u-senha" type="password" formControlName="senha" autocomplete="new-password"
          class="input" [class.input--error]="invalido('senha')" placeholder="mínimo 8 caracteres" />
        @if (invalido('senha')) { <span class="field-error">Mínimo de 6 caracteres.</span> }
      </div>

      <div class="field">
        <label for="u-confirmacao" class="label">Confirmar senha</label>
        <input id="u-confirmacao" type="password" formControlName="confirmacao"
          autocomplete="new-password" class="input"
          [class.input--error]="form.touched && form.hasError('senhasDiferentes')" />
        @if (form.touched && form.hasError('senhasDiferentes')) {
          <span class="field-error">As senhas não conferem.</span>
        }
      </div>

      <div class="actions">
        <button acc-button type="button" variant="outline" size="md" (click)="cancelado.emit()">
          Cancelar
        </button>
        <button acc-button type="submit" variant="primary" size="md" [loading]="enviando()"
          [disabled]="enviando()">
          Criar usuário
        </button>
      </div>
    </form>
  `,
  styles: USUARIO_FORM_STYLES,
})
export class UsuarioFormComponent implements OnInit {
  private readonly usuarios = inject(UsuarioService);

  readonly escritorios = input.required<readonly TenantDto[]>();
  /** Escritório pré-selecionado (ex.: o filtro ativo na lista). */
  readonly escritorioInicial = input<string>('');

  readonly criado = output<UsuarioDto>();
  readonly cancelado = output<void>();

  protected readonly enviando = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).nonNullable.group(
    {
      escritorioId: ['', Validators.required],
      nome: ['', [Validators.required, Validators.maxLength(120)]],
      email: ['', [Validators.required, Validators.email]],
      cpf: ['', [Validators.required, (c: AbstractControl) => (isValidCPF(sanitizeDocument(c.value ?? '')) ? null : { cpf: true })]],
      papel: ['OPERADOR' as 'OPERADOR' | 'CLIENTE', Validators.required],
      senha: ['', [Validators.required, Validators.minLength(8)]],
      confirmacao: ['', Validators.required],
    },
    { validators: senhasIguais },
  );

  ngOnInit(): void {
    this.form.controls.escritorioId.setValue(this.escritorioInicial());
  }

  protected invalido(campo: 'escritorioId' | 'nome' | 'email' | 'cpf' | 'senha'): boolean {
    const c = this.form.controls[campo];
    return c.invalid && c.touched;
  }

  protected async enviar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const v = this.form.getRawValue();
    this.enviando.set(true);
    this.erro.set(null);
    try {
      const novo = await this.usuarios.criar({
        escritorioId: v.escritorioId,
        nome: v.nome.trim(),
        email: v.email.trim().toLowerCase(),
        cpf: sanitizeDocument(v.cpf),
        papel: v.papel,
        senha: v.senha,
      });
      this.criado.emit(novo);
    } catch (e) {
      this.erro.set(mensagemDeErro(e, 'Não foi possível criar o usuário.'));
    } finally {
      this.enviando.set(false);
    }
  }
}
