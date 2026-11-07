import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { SessionStore } from './session.store';

/**
 * Guard que exige papel `OPERADOR` na sessão.
 * Redireciona para `/clientes` quando o usuário autenticado não tem o papel exigido.
 * Usado pela rota de edição de clientes (`clientes/:id/editar`).
 */
export const operadorGuard: CanActivateFn = (_route, state) => {
  const session = inject(SessionStore);
  const router = inject(Router);

  if (!session.isAuthenticated()) {
    return router.createUrlTree(['/login'], {
      queryParams: { returnUrl: state.url },
    });
  }

  if (session.user()?.role === 'OPERADOR') {
    return true;
  }

  return router.createUrlTree(['/clientes']);
};
