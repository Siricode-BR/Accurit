import {
  Component,
  inject,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { CreateClienteRequestDto } from '@accurit/core';
import {
  isValidDocument,
  isValidEmail,
  sanitizeDocument,
} from '@accurit/shared';
import { ClienteService } from '../../data/cliente.service';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';
import { AccDocumentMaskDirective } from '@accurit/ui';

function apenasDigitos(value: string | null | undefined): string {
  if (!value) return '';
  return value.replace(/\D/g, '');
}

@Component({
  selector: 'app-client-form',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    AccButtonComponent,
    AccCardComponent,
    AccDocumentMaskDirective,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="form-layout">
      <div class="form-brand">
        <span class="brand-name">Accurit</span>
        <p class="brand-tagline">Cadastro de Cliente</p>
      </div>

      <acc-card>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="cliente-form">
          <h2 class="form-title">Novo Cliente</h2>

          @if (clienteService.erro()) {
            <div class="alert-error" role="alert">{{ clienteService.erro() }}</div>
          }

          <div class="field">
            <label for="tipo" class="label">Tipo de Pessoa</label>
            <select id="tipo" formControlName="tipo" class="input input--select">
              <option value="PF">Pessoa Física (CPF)</option>
              <option value="PJ">Pessoa Jurídica (CNPJ)</option>
            </select>
          </div>

          <div class="field">
            <label for="cpfCnpj" class="label">CPF / CNPJ</label>
            <input
              id="cpfCnpj"
              type="text"
              accDocMask
              formControlName="cpfCnpj"
              autocomplete="off"
              class="input"
              [class.input--error]="isInvalid('cpfCnpj')"
              placeholder="000.000.000-00 ou 00.000.000/0000-00"
            />
            @if (isInvalid('cpfCnpj')) {
              <span class="field-error">
                {{ cpfCnpjErro() ?? 'Informe um CPF/CNPJ válido.' }}
              </span>
            }
          </div>

          <div class="field">
            <label for="nomeRazaoSocial" class="label">Nome / Razão Social</label>
            <input
              id="nomeRazaoSocial"
              type="text"
              formControlName="nomeRazaoSocial"
              autocomplete="off"
              class="input"
              [class.input--error]="isInvalid('nomeRazaoSocial')"
              placeholder="ex: João Silva ou Silva Contabilidade Ltda."
            />
          </div>

          <div class="field">
            <label for="nomeFantasia" class="label">Nome Fantasia</label>
            <input
              id="nomeFantasia"
              type="text"
              formControlName="nomeFantasia"
              autocomplete="off"
              class="input"
              placeholder="ex: Silva & Associados (opcional)"
            />
          </div>

          <div class="field">
            <label for="porteRegime" class="label">Porte / Regime Tributário</label>
            <select id="porteRegime" formControlName="porteRegime" class="input input--select">
              <option value="">Selecione (opcional)</option>
              <option value="MEI">MEI</option>
              <option value="SIMPLES_NACIONAL">Simples Nacional</option>
              <option value="LUCRO_PRESUMIDO">Lucro Presumido</option>
              <option value="LUCRO_REAL">Lucro Real</option>
            </select>
          </div>

          <div class="field">
            <label for="email" class="label">E-mail</label>
            <input
              id="email"
              type="email"
              formControlName="email"
              autocomplete="email"
              class="input"
              [class.input--error]="isInvalid('email')"
              placeholder="cliente@email.com.br"
            />
            @if (isInvalid('email')) {
              <span class="field-error">{{ emailErro() }}</span>
            }
          </div>

          <div class="field">
            <label for="telefone" class="label">Telefone</label>
            <input
              id="telefone"
              type="tel"
              formControlName="telefone"
              autocomplete="tel"
              class="input"
              [class.input--error]="isInvalid('telefone')"
              placeholder="(11) 99999-9999"
            />
            @if (isInvalid('telefone')) {
              <span class="field-error">{{ telefoneErro() }}</span>
            }
          </div>

          <div class="actions">
            <button
              acc-button
              type="submit"
              variant="primary"
              size="lg"
              [loading]="clienteService.carregando()"
              [disabled]="form.invalid || clienteService.carregando()"
              class="cliente-submit"
            >
              Cadastrar Cliente
            </button>
            <button
              acc-button
              type="button"
              variant="outline"
              size="lg"
              (click)="router.navigate(['/clientes'])"
              class="cliente-cancel"
            >
              Cancelar
            </button>
          </div>
        </form>
      </acc-card>
    </div>
  `,
  styles: `
    .form-layout {
      min-height: 100dvh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 2rem;
      padding: 1.5rem;
      background-color: var(--acc-color-bg);
    }

    .form-brand {
      text-align: center;
    }

    .brand-name {
      font-size: 2rem;
      font-weight: 800;
      color: var(--acc-color-primary);
      letter-spacing: -0.04em;
    }

    .brand-tagline {
      margin: 0.25rem 0 0;
      color: var(--acc-color-text-muted);
      font-size: 0.875rem;
    }

    .cliente-form {
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
      min-width: 340px;
      max-width: 520px;
      width: 100%;
      padding: 2rem;
    }

    .form-title {
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 0.5rem;
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
    }

    .label {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--acc-color-text-main);
    }

    .input {
      padding: 0.625rem 0.875rem;
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-size: 0.875rem;
      outline: none;
      font-family: inherit;
      background-color: var(--acc-color-surface);
    }

    .input:focus {
      border-color: var(--acc-color-primary);
      box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.1);
    }

    .input--error {
      border-color: var(--acc-color-danger);
    }

    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
    }

    .actions {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      margin-top: 0.5rem;
    }

    .cliente-submit {
      width: 100%;
    }

    .cliente-cancel {
      width: 100%;
    }

    @media (min-width: 480px) {
      .actions {
        flex-direction: row;
      }
      .cliente-submit,
      .cliente-cancel {
        width: auto;
        flex: 1;
      }
    }
  `,
})
export class ClienteFormComponent {
  protected readonly clienteService = inject(ClienteService);
  protected readonly router = inject(Router);

  protected readonly form = inject(FormBuilder).group({
    tipo: ['', [Validators.required]],
    cpfCnpj: ['', [Validators.required, cpfCnpjValido]],
    nomeRazaoSocial: ['', [Validators.required, Validators.minLength(3)]],
    nomeFantasia: [''],
    porteRegime: [''],
    email: ['', [Validators.email]],
    telefone: ['', [telefoneValido]],
  });

  protected isInvalid(field: string): boolean {
    const c = this.form.get(field);
    return !!(c?.invalid && c.touched);
  }

  protected cpfCnpjErro(): string | null {
    const c = this.form.get('cpfCnpj');
    if (c?.touched && c?.errors?.['invalido']) {
      return 'CPF/CNPJ inválido. Verifique os dígitos.';
    }
    return null;
  }

  protected emailErro(): string | null {
    const c = this.form.get('email');
    if (c?.touched && c?.invalid) {
      return 'E-mail inválido.';
    }
    return null;
  }

  protected telefoneErro(): string | null {
    const c = this.form.get('telefone');
    if (c?.touched && c?.errors?.['invalido']) {
      return 'Telefone inválido. Informe apenas números (ex: 11999999999).';
    }
    return null;
  }

  protected onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    // Sanitiza todos os campos antes de enviar, exatamente como o backend espera:
    // - CPF/CNPJ: apenas dígitos (máscara removida)
    // - Telefone: apenas dígitos
    // - E-mail: trim + lowercase
    // - Nomes: trim
    const cpfCnpj = sanitizeDocument(this.form.get('cpfCnpj')?.value ?? '');
    const telefone = apenasDigitos(this.form.get('telefone')?.value ?? '');
    const email = (this.form.get('email')?.value ?? '').trim().toLowerCase();
    const nomeRazaoSocial = (this.form.get('nomeRazaoSocial')?.value ?? '').trim();
    const nomeFantasia = (this.form.get('nomeFantasia')?.value ?? '').trim() || undefined;

    // Revalida o documento já sanitizado
    if (!isValidDocument(cpfCnpj)) {
      const c = this.form.get('cpfCnpj');
      c?.markAsTouched();
      c?.setErrors({ ...(c?.errors ?? {}), invalido: true });
      return;
    }

    // Revalida email e telefone já sanitizados
    if (email && !isValidEmail(email)) {
      const c = this.form.get('email');
      c?.markAsTouched();
      c?.setErrors({ ...(c?.errors ?? {}), invalido: true });
      return;
    }

    if (telefone && !/^\d{10,11}$/.test(telefone)) {
      const c = this.form.get('telefone');
      c?.markAsTouched();
      c?.setErrors({ ...(c?.errors ?? {}), invalido: true });
      return;
    }

    const payload = {
      tipo: this.form.get('tipo')?.value === 'PJ' ? 'PJ' : 'PF',
      porteRegime: this.form.get('porteRegime')?.value || undefined,
      cpfCnpj,
      nomeRazaoSocial,
      nomeFantasia,
      email: email || undefined,
      telefone: telefone || undefined,
    } as unknown as CreateClienteRequestDto;

    this.clienteService.criar(payload).then(
      () => {
        this.router.navigate(['/clientes']);
      },
      (erro: any) => {
        // erro já é exibido pelo clienteService.erro() no template
        this.form.markAllAsTouched();
      },
    );
  }
}

function cpfCnpjValido(control: {
  value: string | null | undefined;
}): { [key: string]: boolean } | null {
  if (!control.value) return null;
  const clean = sanitizeDocument(control.value);
  if (!isValidDocument(clean)) {
    return { invalido: true };
  }
  return null;
}

function telefoneValido(control: {
  value: string | null | undefined;
}): { [key: string]: boolean } | null {
  if (!control.value) return null;
  const digitos = apenasDigitos(control.value);
  if (!/^\d{10,11}$/.test(digitos)) {
    return { invalido: true };
  }
  return null;
}
