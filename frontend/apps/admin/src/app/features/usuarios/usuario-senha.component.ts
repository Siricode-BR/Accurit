import { ChangeDetectionStrategy, Component, inject, input, output, signal } from '@angular/core';
import { AbstractControl, FormBuilder, ReactiveFormsModule, ValidationErrors, Validators } from '@angular/forms';
import { UsuarioDto } from '@accurit/core';
import { AccButtonComponent } from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { UsuarioService } from '../../data/usuario.service';
import { USUARIO_FORM_STYLES } from './usuario-form.styles';

function senhasIguais(g: AbstractControl): ValidationErrors | null {
  return g.get('senha')?.value === g.get('confirmacao')?.value ? null : { senhasDiferentes: true };
}

/** Redefinição de senha por quem administra (conteúdo do modal "Redefinir senha"). */
@Component({
  selector: 'app-usuario-senha',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="salvar()" class="form" novalidate>
      @if (erro(); as mensagem) {
        <div class="alert-error" role="alert">{{ mensagem }}</div>
      }
      <p class="hint field--full">
        Defina uma nova senha para <strong>{{ usuario().nome }}</strong> ({{ usuario().email }}).
        As sessões ativas do usuário podem ser encerradas.
      </p>
      <div class="field">
        <label for="s-senha" class="label">Nova senha</label>
        <input id="s-senha" type="password" formControlName="senha" autocomplete="new-password"
          class="input" [class.input--error]="form.controls.senha.invalid && form.controls.senha.touched"
          placeholder="mínimo 8 caracteres" />
        @if (form.controls.senha.invalid && form.controls.senha.touched) {
          <span class="field-error">Mínimo de 6 caracteres.</span>
        }
      </div>
      <div class="field">
        <label for="s-conf" class="label">Confirmar senha</label>
        <input id="s-conf" type="password" formControlName="confirmacao" autocomplete="new-password"
          class="input" [class.input--error]="form.touched && form.hasError('senhasDiferentes')" />
        @if (form.touched && form.hasError('senhasDiferentes')) {
          <span class="field-error">As senhas não conferem.</span>
        }
      </div>
      <div class="actions">
        <button acc-button type="button" variant="outline" size="md" (click)="cancelado.emit()">
          Cancelar
        </button>
        <button acc-button type="submit" variant="primary" size="md" [loading]="salvando()"
          [disabled]="salvando()">
          Redefinir senha
        </button>
      </div>
    </form>
  `,
  styles: USUARIO_FORM_STYLES,
})
export class UsuarioSenhaComponent {
  private readonly usuarios = inject(UsuarioService);

  readonly usuario = input.required<UsuarioDto>();
  readonly concluido = output<void>();
  readonly cancelado = output<void>();

  protected readonly salvando = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).nonNullable.group(
    { senha: ['', [Validators.required, Validators.minLength(8)]], confirmacao: ['', Validators.required] },
    { validators: senhasIguais },
  );

  protected async salvar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    this.salvando.set(true);
    this.erro.set(null);
    try {
      await this.usuarios.redefinirSenha(this.usuario().id, this.form.getRawValue().senha);
      this.concluido.emit();
    } catch (e) {
      this.erro.set(mensagemDeErro(e, 'Não foi possível redefinir a senha.'));
    } finally {
      this.salvando.set(false);
    }
  }
}
