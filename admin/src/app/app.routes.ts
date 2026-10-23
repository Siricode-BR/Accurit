import { Routes } from '@angular/router';

import { autenticadoGuard, naoAutenticadoGuard } from './core/auth/auth.guards';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'tenants' },
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
    path: 'tenants',
    title: 'Escritórios | Accurit Admin',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/tenants/tenant-list.component').then((m) => m.TenantListComponent),
  },
  {
    path: 'tenants/novo',
    title: 'Novo Escritório | Accurit Admin',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/tenants/tenant-create.component').then((m) => m.TenantCreateComponent),
  },
  {
    path: 'serpro/:escritorioId',
    title: 'Ambiente SERPRO | Accurit Admin',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/serpro/serpro-page.component').then((m) => m.SerproPageComponent),
  },
  { path: '**', redirectTo: 'tenants' },
];
