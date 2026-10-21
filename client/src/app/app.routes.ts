import { Routes } from '@angular/router';

import { autenticadoGuard, naoAutenticadoGuard } from './core/auth/auth.guards';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'inicio' },
  {
    path: 'login',
    title: 'Entrar | Accurit Cliente',
    canActivate: [naoAutenticadoGuard],
    loadComponent: () => import('./features/auth/login/login').then((m) => m.Login),
  },
  {
    path: 'acesso-negado',
    title: 'Acesso negado | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/acesso-negado/acesso-negado').then((m) => m.AcessoNegado),
  },
  {
    path: 'inicio',
    title: 'Início | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/inicio/inicio').then((m) => m.Inicio),
  },
  { path: '**', redirectTo: 'inicio' },
];
