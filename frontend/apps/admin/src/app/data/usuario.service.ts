import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import {
  UsuarioDto,
  AtualizarUsuarioRequestDto,
  ListagemUsuariosDto,
} from '@accurit/core';

@Injectable({ providedIn: 'root' })
export class UsuarioService {
  private readonly http = inject(HttpClient);
  // Rota removida do admin (Etapa 0.2) — usuários de escritório são gerenciados apenas no app client
  // baseUrl privada, métodos públicos lançcam erro se chamados
  private readonly baseUrl = '/api/v1/admin/usuarios';

  #state = signal<{ itens: UsuarioDto[]; total: number; pagina: number; tamanho: number; carregando: boolean; erro: string | null }>({ itens: [], total: 0, pagina: 1, tamanho: 20, carregando: false, erro: null });

  readonly itens = computed(() => this.#state().itens);
  readonly total = computed(() => this.#state().total);
  readonly carregando = computed(() => this.#state().carregando);
  readonly erro = computed(() => this.#state().erro);

  listar(): void {
    // Esta rota foi removida do admin (Etapa 0.2)
    this.#state.update((s) => ({
      ...s,
      erro: 'Rota de listagem de usuários de escritório removida do admin. Use o app client para gerenciar usuários de escritório.',
    }));
  }

  criar(): Promise<UsuarioDto> {
    return Promise.reject(new Error('Rota de criação de usuários de escritório removida do admin. Use o app client.'));
  }

  atualizar(): Promise<UsuarioDto> {
    return Promise.reject(new Error('Rota de atualização de usuários de escritório removida do admin. Use o app client.'));
  }

  async redefinirSenha(id: string, senha: string): Promise<UsuarioDto> {
    // Parâmetros mantidos para compatibilidade de chamadas antigas.
    void id;
    void senha;
    return Promise.reject(new Error('Rota de redefinição de senha de usuários de escritório removida do admin. Use o app client.'));
  }
}
