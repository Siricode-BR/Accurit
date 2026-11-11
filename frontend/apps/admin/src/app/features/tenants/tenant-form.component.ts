import { ChangeDetectionStrategy, Component, OnInit, inject, input, output, signal } from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { CriarTenantResponseDto } from '@accurit/core';
import { isValidCNPJ, isValidCPF, sanitizeDocument } from '@accurit/shared';
import {
  AccAlertComponent,
  AccButtonComponent,
  AccDocumentFieldComponent,
  AccInputComponent,
} from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { TenantService } from '../../data/tenant.service';
import { USUARIO_FORM_STYLES } from '../usuarios/usuario-form.styles';

@Component({
  selector: 'app-tenant-form',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    AccAlertComponent,
    AccButtonComponent,
    AccDocumentFieldComponent,
    AccInputComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="enviar()" class="form" novalidate>
      @if (erro(); as mensagem) {
        <acc-alert variante="erro" [descartavel]="true" (fechado)="erro.set(null)">
          {{ mensagem }}
        </acc-alert>
      }

      <acc-document-field
        label="CNPJ"
        formControlName="cnpj"
        placeholder="00.000.000/0000-00"
        hint="Somente dígitos verificadores válidos."
        [validarDigitos]="true"
      />

      <acc-input
        label="Nome do escritório"
        formControlName="nome"
        placeholder="Nome do escritório"
        [erro]="invalido('nome') ? 'Informe o nome.' : null"
      />

      <acc-input
        label="E-mail do titular"
        type="email"
        formControlName="emailTitular"
        autocomplete="off"
        placeholder="titular@escritorio.com.br"
        [erro]="invalido('emailTitular') ? 'Informe e-mail válido.' : null"
      />

      <acc-document-field
        label="CPF do titular"
        formControlName="cpfTitular"
        placeholder="000.000.000-00"
        [validarDigitos]="true"
      />

      <acc-input
        label="Senha do titular"
        type="password"
        formControlName="senhaTitular"
        autocomplete="new-password"
        placeholder="Senha inicial do titular"
        [erro]="invalido('senhaTitular') ? 'Senha deve ter ao menos 8 caracteres.' : null"
      />

      <acc-input
        label="Confirmar senha"
        type="password"
        formControlName="senhaConfirmada"
        autocomplete="new-password"
        placeholder="Repita a senha"
        [erro]="form.hasError('senhasIguais') ? 'As senhas não coincidem.' : null"
      />

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
      cnpj: ['', [Validators.required, (c: AbstractControl) => (isValidCNPJ(sanitizeDocument(c.value ?? '')) ? null : { cnpj: true })]],
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