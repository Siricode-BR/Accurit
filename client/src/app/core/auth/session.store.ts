import { Injectable, computed, signal } from '@angular/core';

import { SessaoResposta, Usuario } from './auth.models';

const CHAVE_SLUG = 'accurit.client.slug';

@Injectable({ providedIn: 'root' })
export class SessionStore {
  private readonly usuarioInterno = signal<Usuario | null>(null);
  private readonly tokenInterno = signal<string | null>(null);
  private readonly slugInterno = signal<string | null>(lerSlugArmazenado());

  readonly usuario = this.usuarioInterno.asReadonly();
  readonly slug = this.slugInterno.asReadonly();
  readonly autenticado = computed(() => this.usuarioInterno() !== null);

  accessToken(): string | null {
    return this.tokenInterno();
  }

  definirSessao(resposta: SessaoResposta): void {
    this.tokenInterno.set(resposta.accessToken);
    this.usuarioInterno.set(resposta.usuario);
  }

  definirUsuario(usuario: Usuario): void {
    this.usuarioInterno.set(usuario);
  }

  definirSlug(slug: string): void {
    this.slugInterno.set(slug);
    try {
      localStorage.setItem(CHAVE_SLUG, slug);
    } catch {
      // armazenamento indisponível (modo privado); a sessão segue em memória
    }
  }

  limpar(): void {
    this.tokenInterno.set(null);
    this.usuarioInterno.set(null);
  }
}

function lerSlugArmazenado(): string | null {
  try {
    return localStorage.getItem(CHAVE_SLUG);
  } catch {
    return null;
  }
}
