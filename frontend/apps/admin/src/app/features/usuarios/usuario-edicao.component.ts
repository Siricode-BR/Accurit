import { ChangeDetectionStrategy, Component, OnInit, inject, input, output, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { UsuarioDto } from '@accurit/core';
import { AccButtonComponent } from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { UsuarioService } from '../../data/usuario.service';
import { USUARIO_FORM_STYLES } from './usuario-form.styles';

/** Edição de nome e papel (conteúdo do modal "Editar usuário"). */
@Component({
  selector: 'app-usuario-edicao',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="salvar()" class="form" novalidate>
      @if (erro(); as mensagem) {
        <div class="alert-error" role="alert">{{ mensagem }}</div>
      }
      <div class="field field--full">
        <label for="e-nome" class="label">Nome completo</label>
        <input id="e-nome" type="text" formControlName="nome" class="input"
          [class.input--error]="form.controls.nome.invalid && form.controls.nome.touched" />
        @if (form.controls.nome.invalid && form.controls.nome.touched) {
          <span class="field-error">Informe o nome.</span>
        }
      </div>
      <div class="field">
        <label class="label">E-mail</label>
        <input type="text" class="input" [value]="usuario().email" disabled />
      </div>
      <div class="field">
        <label for="e-papel" class="label">Papel</label>
        <select id="e-papel" formControlName="papel" class="input">
          <option value="OPERADOR">Operador</option>
          <option value="CLIENTE">Titular</option>
        </select>
      </div>
      <div class="actions">
        <button acc-button type="button" variant="outline" size="md" (click)="cancelado.emit()">
          Cancelar
        </button>
        <button acc-button type="submit" variant="primary" size="md" [loading]="salvando()"
          [disabled]="salvando()">
          Salvar alterações
        </button>
      </div>
    </form>
  `,
  styles: USUARIO_FORM_STYLES,
})
export class UsuarioEdicaoComponent implements OnInit {
  private readonly usuarios = inject(UsuarioService);

  readonly usuario = input.required<UsuarioDto>();
  readonly salvo = output<UsuarioDto>();
  readonly cancelado = output<void>();

  protected readonly salvando = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).nonNullable.group({
    nome: ['', [Validators.required, Validators.maxLength(120)]],
    papel: ['OPERADOR' as 'OPERADOR' | 'CLIENTE', Validators.required],
  });

  ngOnInit(): void {
    const u = this.usuario();
    this.form.setValue({ nome: u.nome, papel: u.papel });
  }

  protected async salvar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const v = this.form.getRawValue();
    const u = this.usuario();
    this.salvando.set(true);
    this.erro.set(null);
    try {
      const atualizado = await this.usuarios.atualizar(u.id, {
        ...(v.nome.trim() !== u.nome ? { nome: v.nome.trim() } : {}),
        ...(v.papel !== u.papel ? { papel: v.papel } : {}),
      });
      this.salvo.emit(atualizado);
    } catch (e) {
      this.erro.set(mensagemDeErro(e, 'Não foi possível salvar as alterações.'));
    } finally {
      this.salvando.set(false);
    }
  }
}
