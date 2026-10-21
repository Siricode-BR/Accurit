import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { catchError, switchMap, throwError } from 'rxjs';

import { AuthService } from './auth.service';

const ROTAS_SEM_RENOVACAO = ['/auth/client/login', '/auth/client/refresh', '/auth/client/logout'];

function exigeRenovacao(url: string): boolean {
  return !ROTAS_SEM_RENOVACAO.some((rota) => url.includes(rota));
}

export const authInterceptor: HttpInterceptorFn = (requisicao, next) => {
  const auth = inject(AuthService);
  const router = inject(Router);
  const token = auth.accessToken();

  const comToken = token
    ? requisicao.clone({ setHeaders: { Authorization: `Bearer ${token}` } })
    : requisicao;

  return next(comToken).pipe(
    catchError((erro: unknown) => {
      if (!(erro instanceof HttpErrorResponse) || erro.status !== 401) {
        return throwError(() => erro);
      }
      if (!exigeRenovacao(requisicao.url)) {
        return throwError(() => erro);
      }

      return auth.renovar().pipe(
        switchMap(() => {
          const novoToken = auth.accessToken();
          const reenviada = novoToken
            ? requisicao.clone({ setHeaders: { Authorization: `Bearer ${novoToken}` } })
            : requisicao;
          return next(reenviada);
        }),
        catchError((falha: unknown) => {
          void router.navigate(['/login'], {
            queryParams: { returnUrl: router.url },
          });
          return throwError(() => falha);
        }),
      );
    }),
  );
};
