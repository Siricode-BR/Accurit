import {
  Component,
  inject,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { CriarTenantRequestDto } from '@accurit/core';
import { TenantService } from '../../data/tenant.service';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';

@Component({
  selector: 'app-admin-tenant-form',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent, AccCardComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="form-layout">
      <div class="form-brand">
        <span class="brand-name">Accurit</span>
        <p class="brand-tagline">Cadastro de Escritório (Tenant)</p>
      </div>

      <acc-card>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="tenant-form">
          <h2 class="form-title">Novo Escritório</h2>

          @if (tenantService.erro()) {
            <div class="alert-error" role="alert">{{ tenantService.erro() }}</div>
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

          <div class="field">
            <label for="senhaTitular" class="label">Senha do Titular</label>
            <input
              id="senhaTitular"
              type="password"
              formControlName="senhaTitular"
              autocomplete="new-password"
              class="input"
              [class.input--error]="isInvalid('senhaTitular')"
              placeholder="mínimo 6 caracteres"
            />
          </div>

          <button
            acc-button
            type="submit"
            variant="primary"
            size="lg"
            [loading]="tenantService.carregando()"
            [disabled]="form.invalid || tenantService.carregando()"
            class="tenant-submit"
          >
            Criar Escritório
          </button>

          <button
            acc-button
            type="button"
            variant="outline"
            size="lg"
            (click)="router.navigate(['/tenants'])"
            class="tenant-cancel"
          >
            Cancelar
          </button>
        </form>
      </acc-card>
    </div>
  `,
  styles: `
    .form-layout {
      min-height: calc(100dvh - 4rem);
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

    .tenant-form {
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
      min-width: 340px;
      max-width: 460px;
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

    .hint {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
      margin-top: 0.125rem;
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
      gap: 0.75rem;
      margin-top: 0.5rem;
    }

    .tenant-submit {
      width: 100%;
    }

    .tenant-cancel {
      width: 100%;
    }

    @media (min-width: 480px) {
      .actions {
        flex-direction: row;
      }
      .tenant-submit,
      .tenant-cancel {
        width: auto;
        flex: 1;
      }
    }
  `,
})
export class TenantFormComponent {
  protected readonly tenantService = inject(TenantService);
  protected readonly router = inject(Router);

  protected readonly form = inject(FormBuilder).group({
    slug: ['', [Validators.required, Validators.pattern(/^[a-z0-9]+(?:-[a-z0-9]+)*$/)]],
    nome: ['', [Validators.required, Validators.minLength(3)]],
    emailTitular: ['', [Validators.required, Validators.email]],
    cpfTitular: ['', [Validators.required]],
    senhaTitular: ['', [Validators.required, Validators.minLength(6)]],
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
    this.tenantService.criar(payload).then(
      () => {
        this.router.navigate(['/tenants']);
      },
      () => {
        // erro já é exibido pelo tenantService.erro() no template
        this.form.markAllAsTouched();
      },
    );
  }
}
