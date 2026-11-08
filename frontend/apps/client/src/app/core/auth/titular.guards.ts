import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { SessionStore } from './session.store';

/** Papel `CLIENTE` (titular do escritório) na sessão — único que gerencia usuários. */
export function ehTitular(role: string | undefined): boolean {
  return role === 'CLIENTE';
}

/** Redireciona não-titulares (ex.: OPERADOR) para `/dashboard`. */
export const titularGuard: CanActivateFn = (_route, state) => {
  const session = inject(SessionStore);
  const router = inject(Router);

  if (!session.isAuthenticated()) {
    return router.createUrlTree(['/login'], { queryParams: { returnUrl: state.url } });
  }
  return ehTitular(session.user()?.role) ? true : router.createUrlTree(['/dashboard']);
};
