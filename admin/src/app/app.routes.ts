import { Routes } from '@angular/router';

import { autenticadoGuard, naoAutenticadoGuard } from './core/auth/auth.guards';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'dashboard' },
  {
    path: 'login',
    title: 'Entrar | Accurit Admin',
    canActivate: [naoAutenticadoGuard],
    loadComponent: () => import('./features/auth/login/login').then((m) => m.Login),
  },
  {
    path: 'acesso-negado',
    title: 'Acesso negado | Accurit Admin',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/acesso-negado/acesso-negado').then((m) => m.AcessoNegado),
  },
  {
    path: 'dashboard',
    title: 'Dashboard | Accurit Admin',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/dashboard/dashboard').then((m) => m.Dashboard),
  },
  { path: '**', redirectTo: 'dashboard' },
];
