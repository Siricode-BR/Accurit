import { Routes } from '@angular/router';
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
    path: 'tenants/novo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/tenants/tenant-form.component').then((m) => m.TenantFormComponent),
  },
  { path: '**', redirectTo: 'dashboard' },
];
