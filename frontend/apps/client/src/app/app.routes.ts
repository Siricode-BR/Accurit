import { Routes } from '@angular/router';
import { authGuard, publicGuard } from './core/auth/auth.guards';
import { operadorGuard } from './core/auth/operador.guards';

export const routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    redirectTo: 'clientes',
  },
  {
    path: 'login',
    canActivate: [publicGuard],
    loadComponent: () =>
      import('./features/auth/login.component').then((m) => m.LoginComponent),
  },
  {
    path: 'clientes',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-list.component').then((m) => m.ClienteListComponent),
  },
  {
    path: 'clientes/novo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-form.component').then((m) => m.ClienteFormComponent),
  },
  {
    path: 'clientes/:id',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-detalhes.page').then((m) => m.ClienteDetalhesPage),
  },
  {
    path: 'clientes/:id/editar',
    canActivate: [authGuard, operadorGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-editar.page').then((m) => m.ClienteEditarPage),
  },
  {
    path: 'divida-ativa',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/divida-ativa/consulta.page').then((m) => m.DividaAtivaConsultaPage),
  },
];
