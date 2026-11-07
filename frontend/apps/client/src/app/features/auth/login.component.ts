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
  templateUrl: './login.component.html',
  styleUrl: './login.component.css',
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

