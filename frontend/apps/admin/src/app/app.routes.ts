import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    redirectTo: 'tenants',
  },
  {
    path: 'tenants',
    loadComponent: () =>
      import('./features/tenants/tenant-list.component').then((m) => m.TenantListComponent),
  },
];
