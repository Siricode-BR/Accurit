import {
  Component,
  inject,
  output,
  signal,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { CriarTenantRequestDto, CriarTenantResponseDto } from '@accurit/core';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { TenantService } from '../../data/tenant.service';
import { AccButtonComponent } from '@accurit/ui';

@Component({
  selector: 'app-admin-tenant-form',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form [formGroup]="form" (ngSubmit)="onSubmit()" class="tenant-form" novalidate>
      @if (erroCadastro(); as mensagem) {
        <div class="alert-error" role="alert">{{ mensagem }}</div>
      }

      <div class="field">
        <label for="slug" class="label">Slug / Subdomínio</label>
        <input
          id="slug"
          type="text"
          formControlName="slug"
          autocomplete="off"
          class="input"
          [class.input--error]="isInvalid('slug')"
          placeholder="ex: escritorio-silva"
        />
        <small class="hint">Usado na URL do escritório (a-z, 0-9 e hífen).</small>
      </div>

      <div class="field">
        <label for="nome" class="label">Nome do Escritório</label>
        <input
          id="nome"
          type="text"
          formControlName="nome"
          autocomplete="off"
          class="input"
          [class.input--error]="isInvalid('nome')"
          placeholder="ex: Escritório Silva & Associados"
        />
      </div>

      <div class="field">
        <label for="emailTitular" class="label">E-mail do Titular</label>
        <input
          id="emailTitular"
          type="email"
          formControlName="emailTitular"
          autocomplete="username"
          class="input"
          [class.input--error]="isInvalid('emailTitular')"
        />
      </div>

      <div class="field">
        <label for="cpfTitular" class="label">CPF do Titular</label>
        <input
          id="cpfTitular"
          type="text"
          formControlName="cpfTitular"
          autocomplete="off"
          class="input"
          [class.input--error]="isInvalid('cpfTitular')"
          placeholder="000.000.000-00"
        />
      </div>

      <div class="field field--full">
        <label for="senhaTitular" class="label">Senha do Titular</label>
        <input
          id="senhaTitular"
          type="password"
          formControlName="senhaTitular"
          autocomplete="new-password"
          class="input"
          [class.input--error]="isInvalid('senhaTitular')"
          placeholder="mínimo 8 caracteres"
        />
      </div>

      <div class="actions">
        <button acc-button type="button" variant="outline" size="md" (click)="cancelado.emit()">
          Cancelar
        </button>
        <button
          acc-button
          type="submit"
          variant="primary"
          size="md"
          [loading]="enviando()"
          [disabled]="form.invalid || enviando()"
        >
          Criar Escritório
        </button>
      </div>
    </form>
  `,
  styles: `
    .tenant-form {
      display: grid;
      grid-template-columns: 1fr;
      gap: 1rem;
      width: 100%;
    }

    @media (min-width: 560px) {
      .tenant-form {
        grid-template-columns: 1fr 1fr;
      }
    }

    .field--full,
    .alert-error,
    .actions {
      grid-column: 1 / -1;
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

    .hint {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
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
      box-shadow: 0 0 0 3px rgb(30 64 175 / 0.14);
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
      justify-content: flex-end;
      gap: 0.75rem;
      margin-top: 0.5rem;
    }
  `,
})
export class TenantFormComponent {
  private readonly tenantService = inject(TenantService);

  /** Emitido com o escritório criado; quem hospeda decide para onde navegar. */
  readonly cadastrado = output<CriarTenantResponseDto>();
  readonly cancelado = output<void>();

  protected readonly enviando = signal(false);
  protected readonly erroCadastro = signal<string | null>(null);

  protected readonly form = inject(FormBuilder).group({
    slug: ['', [Validators.required, Validators.pattern(/^[a-z0-9]+(?:-[a-z0-9]+)*$/)]],
    nome: ['', [Validators.required, Validators.minLength(3)]],
    emailTitular: ['', [Validators.required, Validators.email]],
    cpfTitular: ['', [Validators.required]],
    senhaTitular: ['', [Validators.required, Validators.minLength(8)]],
  });

  protected isInvalid(field: string): boolean {
    const c = this.form.get(field);
    return !!(c?.invalid && c.touched);
  }

  protected onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const payload = this.form.getRawValue() as unknown as CriarTenantRequestDto;
    this.enviando.set(true);
    this.erroCadastro.set(null);
    this.tenantService.criar(payload).then(
      (novo) => {
        this.enviando.set(false);
        this.cadastrado.emit(novo);
      },
      (erro: { error?: { message?: string }; message?: string }) => {
        this.enviando.set(false);
        this.erroCadastro.set(mensagemDeErro(erro, 'Falha ao criar o escritório.'));
        this.form.markAllAsTouched();
      },
    );
  }
}
