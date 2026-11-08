import {
  Component,
  inject,
  output,
  signal,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { ClienteDto, CreateClienteRequestDto } from '@accurit/core';
import {
  isValidDocument,
  isValidEmail,
  sanitizeDocument,
} from '@accurit/shared';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { ClienteService } from '../../data/cliente.service';
import { AccButtonComponent, AccDocumentMaskDirective } from '@accurit/ui';

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
    AccDocumentMaskDirective,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './cliente-form.component.html',
  styleUrl: './cliente-form.component.css',
})
export class ClienteFormComponent {
  protected readonly clienteService = inject(ClienteService);

  /** Emitido com o cliente criado; quem hospeda decide para onde navegar. */
  readonly cadastrado = output<ClienteDto>();
  readonly cancelado = output<void>();

  /** Erro do cadastro fica local para não derrubar a listagem que está atrás do modal. */
  protected readonly erroCadastro = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).group({
    tipo: ['', [Validators.required]],
    cpfCnpj: ['', [Validators.required, cpfCnpjValido]],
    nomeRazaoSocial: ['', [Validators.required, Validators.minLength(3)]],
    nomeFantasia: [''],
    representanteNome: [''],
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
    const tipoSel = this.form.get('tipo')?.value === 'PJ' ? 'PJ' : 'PF';
    const representanteNome =
      tipoSel === 'PJ' ? (this.form.get('representanteNome')?.value ?? '').trim() || undefined : undefined;

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
      representanteNome,
      email: email || undefined,
      telefone: telefone || undefined,
    } as unknown as CreateClienteRequestDto;

    this.erroCadastro.set(null);
    this.clienteService.criar(payload).then(
      (novo) => this.cadastrado.emit(novo),
      (erro: { error?: { message?: string }; message?: string }) => {
        this.erroCadastro.set(mensagemDeErro(erro, 'Falha ao cadastrar cliente.'));
        this.clienteService.definirErro('');
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
