import {
  Component,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { AuthService } from '../../core/auth/auth.service';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [ReactiveFormsModule, AccButtonComponent, AccCardComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="login-layout">
      <div class="login-brand">
        <span class="brand-name">Accurit</span>
        <p class="brand-tagline">Portal do Escritório Contábil</p>
      </div>

      <acc-card>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="login-form">
          <h2 class="login-title">Entrar na sua conta</h2>

          @if (auth.errorMessage()) {
            <div class="alert-error" role="alert">
              {{ auth.errorMessage() }}
            </div>
          }

          <div class="field">
            <label for="escritorio" class="label">Escritório (slug)</label>
            <input
              id="escritorio"
              type="text"
              formControlName="escritorio"
              autocomplete="off"
              class="input"
              [class.input--error]="isFieldInvalid('escritorio')"
              placeholder="ex: escritorio-silva"
            />
            @if (isFieldInvalid('escritorio')) {
              <span class="field-error">Informe o slug do escritório.</span>
            }
          </div>

          <div class="field">
            <label for="email" class="label">E-mail</label>
            <input
              id="email"
              type="email"
              formControlName="email"
              autocomplete="username"
              class="input"
              [class.input--error]="isFieldInvalid('email')"
              placeholder="seu@email.com.br"
            />
            @if (isFieldInvalid('email')) {
              <span class="field-error">Informe um e-mail válido.</span>
            }
          </div>

          <div class="field">
            <label for="senha" class="label">Senha</label>
            <input
              id="senha"
              type="password"
              formControlName="senha"
              autocomplete="current-password"
              class="input"
              [class.input--error]="isFieldInvalid('senha')"
              placeholder="••••••••"
            />
            @if (isFieldInvalid('senha')) {
              <span class="field-error">A senha é obrigatória.</span>
            }
          </div>

          <button
            acc-button
            type="submit"
            variant="primary"
            size="lg"
            [loading]="auth.isLoading()"
            [disabled]="form.invalid"
            class="login-submit"
          >
            Entrar
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
      background-color: var(--acc-color-bg);
    }

    .login-brand {
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
      transition: border-color 0.15s ease, box-shadow 0.15s ease;
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

    .field-error {
      font-size: 0.75rem;
      color: var(--acc-color-danger);
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
      border: 1px solid currentColor;
    }
  `,
})
export class LoginComponent {
  protected readonly auth = inject(AuthService);

  protected readonly form = inject(FormBuilder).group({
    escritorio: ['', [Validators.required]],
    email: ['', [Validators.required, Validators.email]],
    senha: ['', [Validators.required, Validators.minLength(6)]],
  });

  protected isFieldInvalid(field: 'escritorio' | 'email' | 'senha'): boolean {
    const control = this.form.get(field);
    return !!(control?.invalid && control.touched);
  }

  protected onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const { escritorio, email, senha } = this.form.getRawValue();
    this.auth.login({ escritorio: escritorio!, email: email!, senha: senha! });
  }
}

