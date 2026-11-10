import { ChangeDetectionStrategy, Component, OnInit, inject, input, output, signal } from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { CriarTenantResponseDto } from '@accurit/core';
import { isValidCPF, sanitizeDocument } from '@accurit/shared';
import { AccButtonComponent, AccDocumentMaskDirective } from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { TenantService } from '../../data/tenant.service';
import { USUARIO_FORM_STYLES } from '../usuarios/usuario-form.styles';

@Component({
  selector: 'app-tenant-form',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent, AccDocumentMaskDirective],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="enviar()" class="form" novalidate>
      @if (erro(); as mensagem) {
        <div class="alert-error" role="alert">{{ mensagem }}</div>
      }

      <div class="field field--full">
        <label for="t-cnpj" class="label">CNPJ</label>
        <input id="t-cnpj" type="text" formControlName="cnpj" accDocMask autocomplete="off" class="input"
          [class.input--error]="invalido('cnpj')" placeholder="00.000.000/0000-00" />
        @if (invalido('cnpj')) { <span class="field-error">CNPJ inválido.</span> }
      </div>

      <div class="field field--full">
        <label for="t-nome" class="label">Nome do escritório</label>
        <input id="t-nome" type="text" formControlName="nome" autocomplete="off" class="input"
          [class.input--error]="invalido('nome')" placeholder="Nome do escritório" />
        @if (invalido('nome')) { <span class="field-error">Informe o nome.</span> }
      </div>

      <div class="field">
        <label for="t-email" class="label">E-mail do titular</label>
        <input id="t-email" type="email" formControlName="emailTitular" autocomplete="off" class="input"
          [class.input--error]="invalido('emailTitular')" placeholder="titular@escritorio.com.br" />
        @if (invalido('emailTitular')) { <span class="field-error">Informe e-mail válido.</span> }
      </div>

      <div class="field">
        <label for="t-cpf" class="label">CPF do titular</label>
        <input id="t-cpf" type="text" formControlName="cpfTitular" accDocMask autocomplete="off" class="input"
          [class.input--error]="invalido('cpfTitular')" placeholder="000.000.000-00" />
        @if (invalido('cpfTitular')) { <span class="field-error">CPF inválido.</span> }
      </div>

      <div class="field">
        <label for="t-senha" class="label">Senha do titular</label>
        <input id="t-senha" type="password" formControlName="senhaTitular" autocomplete="new-password" class="input"
          [class.input--error]="invalido('senhaTitular')" placeholder="Senha inicial do titular" />
        @if (invalido('senhaTitular')) { <span class="field-error">Senha deve ter ao menos 8 caracteres.</span> }
      </div>

      <div class="field">
        <label for="t-senha-2" class="label">Confirmar senha</label>
        <input id="t-senha-2" type="password" formControlName="senhaConfirmada" autocomplete="new-password"
          class="input" [class.input--error]="form.hasError('senhasIguais')" placeholder="Repita a senha" />
        @if (form.hasError('senhasIguais')) { <span class="field-error">As senhas não coincidem.</span> }
      </div>

      <div class="actions">
        <button acc-button type="button" variant="outline" size="md" (click)="cancelado.emit()">
          Cancelar
        </button>
        <button acc-button type="submit" variant="primary" size="md" [loading]="enviando()"
          [disabled]="enviando()">
          Criar escritório
        </button>
      </div>
    </form>
  `,
  styles: USUARIO_FORM_STYLES,
})
export class TenantFormComponent implements OnInit {
  private readonly tenantService = inject(TenantService);

  readonly criado = output<CriarTenantResponseDto>();
  readonly cancelado = output<void>();

  protected readonly enviando = signal(false);
  protected readonly erro = signal<string | null>(null);
  protected readonly aviso = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).nonNullable.group(
    {
      cnpj: ['', [Validators.required, Validators.pattern(/^[0-9]{14}$/)]],
      nome: ['', [Validators.required, Validators.maxLength(200)]],
      emailTitular: ['', [Validators.required, Validators.email]],
      cpfTitular: ['', [Validators.required, (c: AbstractControl) => (isValidCPF(sanitizeDocument(c.value ?? '')) ? null : { cpf: true })]],
      senhaTitular: ['', [Validators.required, Validators.minLength(8)]],
      senhaConfirmada: ['', Validators.required],
    },
    { validators: senhasIguais },
  );

  ngOnInit(): void {
    // CNPJ já vem formatado ou vazio para preenchimento
  }

  protected invalido(campo: 'cnpj' | 'nome' | 'emailTitular' | 'cpfTitular' | 'senhaTitular' | 'senhaConfirmada'): boolean {
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
      const resposta = await this.tenantService.criar({
        cnpj: v.cnpj,
        nome: v.nome.trim(),
        emailTitular: v.emailTitular.trim().toLowerCase(),
        cpfTitular: sanitizeDocument(v.cpfTitular),
        senhaTitular: v.senhaTitular,
      });
      this.aviso.set(`Escritório criado com CNPJ ${v.cnpj}.`);
      this.criado.emit(resposta);
    } catch (e) {
      this.erro.set(mensagemDeErro(e, 'Não foi possível criar o escritório.'));
    } finally {
      this.enviando.set(false);
    }
  }
}

/** Validação de grupo: senha e confirmação precisam coincidir. */
function senhasIguais(grupo: AbstractControl): ValidationErrors | null {
  const senha = grupo.get('senhaTitular')?.value;
  const confirmacao = grupo.get('senhaConfirmada')?.value;
  return senha && confirmacao && senha !== confirmacao ? { senhasIguais: true } : null;
}