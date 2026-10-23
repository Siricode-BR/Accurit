import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';

import { AuthService } from './auth.service';

export const autenticadoGuard: CanActivateFn = (_rota, estado) => {
  const auth = inject(AuthService);
  if (auth.autenticado()) {
    return true;
  }

  return inject(Router).createUrlTree(['/login'], {
    queryParams: { returnUrl: estado.url },
  });
};

export const naoAutenticadoGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  return auth.autenticado() ? inject(Router).parseUrl('/inicio') : true;
};

export const operadorGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  if (!auth.autenticado()) {
    return router.createUrlTree(['/login'], { queryParams: { returnUrl: '/inicio' } });
  }

  const usuario = auth.usuario();
  if (usuario?.papel === 'OPERADOR') {
    return true;
  }

  return router.createUrlTree(['/acesso-negado']);
};
