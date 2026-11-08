import { inject } from '@angular/core';
import { Router, Routes } from '@angular/router';
import { authGuard, publicGuard } from './core/auth/auth.guards';

export const routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    redirectTo: 'dashboard',
  },
  {
    path: 'login',
    canActivate: [publicGuard],
    loadComponent: () =>
      import('./features/auth/login.component').then((m) => m.AdminLoginComponent),
  },
  {
    path: 'dashboard',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/dashboard/dashboard.page').then((m) => m.DashboardPage),
  },
  {
    path: 'tenants',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/tenants/tenant-list.component').then((m) => m.TenantListComponent),
  },
  {
    // Cadastro agora é um modal em /tenants; mantém links antigos funcionando.
    path: 'tenants/novo',
    redirectTo: () => inject(Router).parseUrl('/tenants?novo=1'),
  },
  {
    path: 'tenants/:id',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/tenants/tenant-detalhes.page').then((m) => m.TenantDetalhesPage),
  },
  {
    path: 'usuarios',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/usuarios/usuario-list.component').then((m) => m.UsuarioListComponent),
  },
  { path: '**', redirectTo: 'dashboard' },
];
