import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Router } from '@angular/router';
import { LoginRequestDto, UsuarioSessaoDto, UserRole } from '@accurit/core';
import { SessionStore } from './session.store';
import { mensagemDeErro } from '../http/mensagem-erro';

type AuthStatus = 'idle' | 'loading' | 'error';

/// Formato bruto retornado por `POST /api/v1/auth/admin/login` (ADR-0010).
/// O backoffice emite `accessToken` (não `token`) e `usuario` com `papel`/`escritorioId`.
type LoginAdminResposta = {
  accessToken: string;
  expiresIn: number;
  usuario: {
    id: string;
    email: string;
    papel: string;
    escritorioId: string | null;
  };
};

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

    this.http.post<LoginAdminResposta>('/api/v1/auth/admin/login', credentials).subscribe({
      next: (res) => {
        this.#status.set('idle');
        const usuario: UsuarioSessaoDto = {
          id: res.usuario.id,
          nome: res.usuario.email, // backoffice não envia `nome`; usa o e-mail como fallback
          email: res.usuario.email,
          role: (res.usuario.papel ?? 'ADMIN') as UserRole,
          tenantId: res.usuario.escritorioId ?? undefined,
        };
        this.session.setSession(usuario, res.accessToken);
        this.router.navigate(['/dashboard']);
      },
      error: (err: { error?: { message?: string }; message?: string }) => {
        this.#status.set('error');
        this.#errorMessage.set(
          mensagemDeErro(err, 'Credenciais de administrador inválidas.')
        );
      },
    });
  }

  logout(): void {
    this.session.clearSession();
    this.router.navigate(['/login']);
  }
}

