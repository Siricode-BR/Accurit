import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { SessionStore } from './session.store';

export const authGuard: CanActivateFn = (_route, state) => {
  const session = inject(SessionStore);
  const router = inject(Router);

  if (session.isAuthenticated()) return true;

  return router.createUrlTree(['/login'], {
    queryParams: { returnUrl: state.url },
  });
};

export const publicGuard: CanActivateFn = () => {
  const session = inject(SessionStore);
  const router = inject(Router);

  if (!session.isAuthenticated()) return true;

  return router.createUrlTree(['/dashboard']);
};

