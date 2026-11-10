import { ErrorHandler, Injectable, isDevMode } from '@angular/core';

/**
 * ErrorHandler global: em produção nunca imprime HttpErrorResponse, URL,
 * headers ou stack no console. O detalhe fica apenas no backend/observabilidade.
 */
@Injectable()
export class QuietErrorHandler implements ErrorHandler {
  handleError(error: unknown): void {
    if (isDevMode()) {
      console.error(error);
      return;
    }
    // Mensagem intencionalmente genérica e sem dados da requisição.
    console.error('Ocorreu um erro inesperado. Consulte o suporte se persistir.');
  }
}
