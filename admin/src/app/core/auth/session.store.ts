import { Injectable, computed, signal } from '@angular/core';

import { SessaoResposta, Usuario } from './auth.models';

@Injectable({ providedIn: 'root' })
export class SessionStore {
  private readonly usuarioInterno = signal<Usuario | null>(null);
  private readonly tokenInterno = signal<string | null>(null);

  readonly usuario = this.usuarioInterno.asReadonly();
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

  limpar(): void {
    this.tokenInterno.set(null);
    this.usuarioInterno.set(null);
  }
}
