import {
  Component,
  inject,
  ChangeDetectionStrategy,
  OnInit,
  signal,
  computed,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  ClienteDto,
  UpdateClienteRequestDto,
  TipoCliente,
} from '@accurit/core';
import {
  isValidDocument,
  isValidEmail,
  sanitizeDocument,
} from '@accurit/shared';
import { ClienteService } from '../../data/cliente.service';
import { SessionStore } from '../../core/auth/session.store';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';
import { AccDocumentMaskDirective } from '@accurit/ui';

function apenasDigitos(value: string | null | undefined): string {
  if (!value) return '';
  return value.replace(/\D/g, '');
}

@Component({
  selector: 'app-cliente-editar-page',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    AccButtonComponent,
    AccCardComponent,
    AccDocumentMaskDirective,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './cliente-editar.page.html',
  styleUrl: './cliente-editar.page.css',
})
export class ClienteEditarPage implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly sessao = inject(SessionStore);
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly fb = inject(FormBuilder);

  protected readonly podeEditar = computed(() => this.sessao.user()?.role === 'OPERADOR');

  protected readonly form = this.fb.group({
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

  protected nomeRazaoSocialErro(): string | null {
    const c = this.form.get('nomeRazaoSocial');
    if (c?.touched && c?.invalid) {
      return 'Nome ou razão social é obrigatório.';
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

  async ngOnInit(): Promise<void> {
    const id = this.route.snapshot.paramMap.get('id');
    if (!id) {
      this.router.navigate(['/clientes']);
      return;
    }

    this.form.disable();
    try {
      const cliente = await this.clienteService.obter(id);
      this.preencherForm(cliente);
    } catch {
      this.router.navigate(['/clientes']);
      return;
    } finally {
      this.form.enable();
    }
  }

  private preencherForm(cliente: ClienteDto): void {
    this.form.patchValue({
      tipo: cliente.tipo as TipoCliente,
      cpfCnpj: cliente.cpfCnpj,
      nomeRazaoSocial: cliente.nomeRazaoSocial,
      nomeFantasia: cliente.nomeFantasia ?? '',
      representanteNome: cliente.representanteNome ?? '',
      porteRegime: cliente.porteRegime ?? '',
      email: cliente.email ?? '',
      telefone: cliente.telefone ?? '',
    });
  }

  protected onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const cpfCnpj = sanitizeDocument(this.form.get('cpfCnpj')?.value ?? '');
    const telefone = apenasDigitos(this.form.get('telefone')?.value ?? '');
    const email = (this.form.get('email')?.value ?? '').trim().toLowerCase() || undefined;
    const nomeRazaoSocial = (this.form.get('nomeRazaoSocial')?.value ?? '').trim();
    const nomeFantasia =
      (this.form.get('nomeFantasia')?.value ?? '').trim() || undefined;

    const representanteNome =
      this.form.get('tipo')?.value === 'PJ'
        ? (this.form.get('representanteNome')?.value ?? '').trim() || null
        : null;

    if (!isValidDocument(cpfCnpj)) {
      const c = this.form.get('cpfCnpj');
      c?.markAsTouched();
      c?.setErrors({ ...(c?.errors ?? {}), invalido: true });
      return;
    }

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

    const id = this.route.snapshot.paramMap.get('id');
    if (!id) return;

    const payload = {
      tipo: this.form.get('tipo')?.value === 'PJ' ? 'PJ' : 'PF',
      porteRegime: this.form.get('porteRegime')?.value || undefined,
      cpfCnpj,
      nomeRazaoSocial,
      nomeFantasia,
      representanteNome,
      email,
      telefone,
    } as UpdateClienteRequestDto;

    this.clienteService.atualizar(id, payload).then(
      () => {
        this.router.navigate(['/clientes', id]);
      },
      () => {
        this.form.markAllAsTouched();
      },
    );
  }

  protected voltar(): void {
    const id = this.route.snapshot.paramMap.get('id');
    void this.router.navigate(['/clientes', id ?? '']);
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
