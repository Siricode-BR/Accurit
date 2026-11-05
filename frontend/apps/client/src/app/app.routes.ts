import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    redirectTo: 'clientes',
  },
  {
    path: 'clientes',
    loadComponent: () =>
      import('./features/clientes/cliente-list.component').then((m) => m.ClienteListComponent),
  },
];
