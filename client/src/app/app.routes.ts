import { Routes } from '@angular/router';

import { autenticadoGuard, naoAutenticadoGuard, operadorGuard } from './core/auth/auth.guards';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'clientes' },
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
    path: 'clientes',
    title: 'Clientes | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-list.component').then((m) => m.ClienteListComponent),
  },
  {
    path: 'clientes/novo',
    title: 'Novo Cliente | Accurit Cliente',
    canActivate: [operadorGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-form.component').then((m) => m.ClienteFormComponent),
  },
  {
    path: 'clientes/:id',
    title: 'Detalhes do Cliente | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-form.component').then((m) => m.ClienteFormComponent),
  },
  {
    path: 'clientes/:id/editar',
    title: 'Editar Cliente | Accurit Cliente',
    canActivate: [operadorGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-form.component').then((m) => m.ClienteFormComponent),
  },
  { path: '**', redirectTo: 'clientes' },
];
