import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Router } from '@angular/router';
import { LoginRequestDto, LoginResponseDto } from '@accurit/core';
import { SessionStore } from './session.store';

type AuthStatus = 'idle' | 'loading' | 'error';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);
  private readonly session = inject(SessionStore);

  #status = signal<AuthStatus>('idle');
  #errorMessage = signal<string | null>(null);

  readonly isLoading = computed(() => this.#status() === 'loading');
  readonly errorMessage = computed(() => this.#errorMessage());

  login(credentials: LoginRequestDto): void {
    this.#status.set('loading');
    this.#errorMessage.set(null);

    this.http.post<LoginResponseDto>('/api/v1/auth/admin/login', credentials).subscribe({
      next: (res) => {
        this.#status.set('idle');
        this.session.setSession(res.usuario, res.token);
        this.router.navigate(['/tenants']);
      },
      error: (err: { error?: { message?: string }; message?: string }) => {
        this.#status.set('error');
        this.#errorMessage.set(
          err.error?.message ?? err.message ?? 'Credenciais de administrador inválidas.'
        );
      },
    });
  }

  logout(): void {
    this.session.clearSession();
    this.router.navigate(['/login']);
  }
}

