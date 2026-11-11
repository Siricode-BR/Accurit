import { Injectable, signal } from '@angular/core';

export type VarianteToast = 'sucesso' | 'erro' | 'aviso' | 'info';

export interface Toast {
  readonly id: number;
  readonly variante: VarianteToast;
  readonly mensagem: string;
  readonly duracaoMs: number;
}

/**
 * Serviço de notificações efêmeras (toasts). O componente `acc-toast-host`
 * deve ser renderizado uma vez na casca da aplicação.
 *
 * ```ts
 * this.toasts.sucesso('Cliente salvo com sucesso.');
 * this.toasts.erro(mensagemDeErro(err, 'Falha ao salvar.'));
 * ```
 */
@Injectable({ providedIn: 'root' })
export class AccToastService {
  private proximoId = 0;
  private readonly _toasts = signal<readonly Toast[]>([]);
  readonly toasts = this._toasts.asReadonly();

  sucesso(mensagem: string, duracaoMs = 4000): number {
    return this.exibir('sucesso', mensagem, duracaoMs);
  }

  erro(mensagem: string, duracaoMs = 6000): number {
    return this.exibir('erro', mensagem, duracaoMs);
  }

  aviso(mensagem: string, duracaoMs = 5000): number {
    return this.exibir('aviso', mensagem, duracaoMs);
  }

  info(mensagem: string, duracaoMs = 4000): number {
    return this.exibir('info', mensagem, duracaoMs);
  }

  fechar(id: number): void {
    this._toasts.update((atuais) => atuais.filter((toast) => toast.id !== id));
  }

  limpar(): void {
    this._toasts.set([]);
  }

  private exibir(variante: VarianteToast, mensagem: string, duracaoMs: number): number {
    const id = ++this.proximoId;
    this._toasts.update((atuais) => [...atuais, { id, variante, mensagem, duracaoMs }]);
    if (typeof setTimeout === 'function') {
      setTimeout(() => this.fechar(id), duracaoMs);
    }
    return id;
  }
}
