import { Injectable, signal, computed } from '@angular/core';
import { UsuarioSessaoDto, AuthSessionState } from '@accurit/core';

const STORAGE_KEY = 'accurit.client.session';

function restoreSession(): AuthSessionState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { isAuthenticated: false, user: null, token: null };
    const parsed = JSON.parse(raw) as Omit<AuthSessionState, 'isAuthenticated'>;
    if (!parsed.token || !parsed.user) {
      return { isAuthenticated: false, user: null, token: null };
    }
    return { ...parsed, isAuthenticated: true };
  } catch {
    return { isAuthenticated: false, user: null, token: null };
  }
}

@Injectable({ providedIn: 'root' })
export class SessionStore {
  #state = signal<AuthSessionState>(restoreSession());

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
    this.persist();
  }

  clearSession(): void {
    this.#state.set({
      isAuthenticated: false,
      user: null,
      token: null,
    });
    this.persist();
  }

  private persist(): void {
    try {
      const s = this.#state();
      if (s.isAuthenticated && s.token && s.user) {
        localStorage.setItem(STORAGE_KEY, JSON.stringify({ user: s.user, token: s.token }));
      } else {
        localStorage.removeItem(STORAGE_KEY);
      }
    } catch {
      // storage indisponível — sessão permanece apenas em memória
    }
  }
}

