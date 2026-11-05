import { Injectable, signal, computed } from '@angular/core';
import { UsuarioSessaoDto, AuthSessionState } from '@accurit/core';

@Injectable({ providedIn: 'root' })
export class SessionStore {
  #state = signal<AuthSessionState>({
    isAuthenticated: false,
    user: null,
    token: null,
  });

  readonly isAuthenticated = computed(() => this.#state().isAuthenticated);
  readonly user = computed(() => this.#state().user);
  readonly token = computed(() => this.#state().token);
  readonly tenantNome = computed(() => this.#state().user?.tenantNome ?? '');

  setSession(user: UsuarioSessaoDto, token: string): void {
    this.#state.set({
      isAuthenticated: true,
      user,
      token,
    });
  }

  clearSession(): void {
    this.#state.set({
      isAuthenticated: false,
      user: null,
      token: null,
    });
  }
}

