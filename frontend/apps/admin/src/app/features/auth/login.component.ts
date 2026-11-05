import {
  Component,
  inject,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { AuthService } from '../../core/auth/auth.service';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';

@Component({
  selector: 'app-admin-login',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent, AccCardComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="login-layout">
      <div class="login-brand">
        <span class="brand-name">Accurit</span>
        <p class="brand-tagline">Backoffice Administrativo</p>
      </div>

      <acc-card>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="login-form">
          <h2 class="login-title">Acesso Restrito</h2>

          @if (auth.errorMessage()) {
            <div class="alert-error" role="alert">{{ auth.errorMessage() }}</div>
          }

          <div class="field">
            <label for="email" class="label">E-mail do administrador</label>
            <input
              id="email"
              type="email"
              formControlName="email"
              autocomplete="username"
              class="input"
              [class.input--error]="isInvalid('email')"
            />
          </div>

          <div class="field">
            <label for="senha" class="label">Senha</label>
            <input
              id="senha"
              type="password"
              formControlName="senha"
              autocomplete="current-password"
              class="input"
              [class.input--error]="isInvalid('senha')"
            />
          </div>

          <button
            acc-button
            type="submit"
            variant="secondary"
            size="lg"
            [loading]="auth.isLoading()"
            [disabled]="form.invalid"
            class="login-submit"
          >
            Entrar como Administrador
          </button>
        </form>
      </acc-card>
    </div>
  `,
  styles: `
    .login-layout {
      min-height: 100dvh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 2rem;
      padding: 1.5rem;
      background-color: var(--acc-color-secondary);
    }

    .login-brand {
      text-align: center;
    }

    .brand-name {
      font-size: 2rem;
      font-weight: 800;
      color: #ffffff;
      letter-spacing: -0.04em;
    }

    .brand-tagline {
      margin: 0.25rem 0 0;
      color: rgba(255, 255, 255, 0.5);
      font-size: 0.875rem;
    }

    .login-form {
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
      min-width: 340px;
      max-width: 400px;
      width: 100%;
      padding: 2rem;
    }

    .login-title {
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
      border-color: var(--acc-color-secondary);
      box-shadow: 0 0 0 3px rgba(15, 23, 42, 0.1);
    }

    .input--error {
      border-color: var(--acc-color-danger);
    }

    .login-submit {
      width: 100%;
      margin-top: 0.5rem;
    }

    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
    }
  `,
})
export class AdminLoginComponent {
  protected readonly auth = inject(AuthService);

  protected readonly form = inject(FormBuilder).group({
    email: ['', [Validators.required, Validators.email]],
    senha: ['', [Validators.required, Validators.minLength(6)]],
  });

  protected isInvalid(field: 'email' | 'senha'): boolean {
    const c = this.form.get(field);
    return !!(c?.invalid && c.touched);
  }

  protected onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const { email, senha } = this.form.getRawValue();
    this.auth.login({ email: email!, senha: senha! });
  }
}

