import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Router } from '@angular/router';
import { LoginRequestDto, UsuarioSessaoDto, UserRole } from '@accurit/core';
import { SessionStore } from './session.store';

type AuthStatus = 'idle' | 'loading' | 'error';

/// Formato bruto retornado por `POST /api/v1/auth/client/login`.
/// O backend emite `accessToken` (não `token`) e `usuario` com `papel`/`escritorioId`.
type LoginClientResposta = {
  accessToken: string;
  expiresIn: number;
  usuario: {
    id: string;
    email: string;
    papel: string;
    escritorio_id: string;
  };
};

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);
  private readonly session = inject(SessionStore);

  private readonly baseUrl = '/api/v1/auth';

  #status = signal<AuthStatus>('idle');
  #errorMessage = signal<string | null>(null);

  readonly isLoading = computed(() => this.#status() === 'loading');
  readonly errorMessage = computed(() => this.#errorMessage());

  login(credentials: LoginRequestDto): void {
    this.#status.set('loading');
    this.#errorMessage.set(null);

    // O escritório é identificado pelo e-mail (único na plataforma); não há slug no login.
    const { email, senha } = credentials;

    this.http
      .post<LoginClientResposta>(`${this.baseUrl}/client/login`, { email, senha })
      .subscribe({
        next: (res) => {
          this.#status.set('idle');
          const usuario: UsuarioSessaoDto = {
            id: res.usuario.id,
            nome: res.usuario.email,
            email: res.usuario.email,
            role: (res.usuario.papel ?? 'CONTADOR') as UserRole,
            tenantId: res.usuario.escritorio_id ?? undefined,
          };
          this.session.setSession(usuario, res.accessToken);
          void this.router.navigateByUrl(this.destinoAposLogin());
        },
        error: (err: { error?: { message?: string }; message?: string }) => {
          this.#status.set('error');
          this.#errorMessage.set(
            err.error?.message ?? err.message ?? 'Credenciais inválidas. Tente novamente.'
          );
        },
      });
  }

  /** Destino pós-login: `returnUrl` interno e válido; caso contrário, o dashboard. */
  private destinoAposLogin(): string {
    const retorno = this.router.parseUrl(this.router.url).queryParams['returnUrl'];
    const valido =
      typeof retorno === 'string' &&
      retorno.startsWith('/') &&
      !retorno.startsWith('//') &&
      !retorno.startsWith('/login');
    return valido ? retorno : '/dashboard';
  }

  logout(): void {
    this.session.clearSession();
    this.router.navigate(['/login']);
  }
}

