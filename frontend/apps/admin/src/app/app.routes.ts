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
    path: 'administradores',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/administradores/administradores-list.component').then((m) => m.AdministradoresListComponent),
  },
  {
    path: 'catalogo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/integra-contador/catalogo.page').then((m) => m.CatalogoIntegraPage),
  },
  {
    path: 'consumo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/consumo/consumo-admin.page').then((m) => m.ConsumoAdminPage),
  },
  { path: '**', redirectTo: 'dashboard' },
];