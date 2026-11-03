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
    path: 'inicio',
    title: 'Início | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () => import('./features/inicio/inicio').then((m) => m.Inicio),
  },
  {
    path: 'divida-ativa/inscricoes/:numero',
    title: 'Detalhe da inscrição | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/divida-ativa/inscricao-detalhe.component').then(
        (m) => m.InscricaoDetalheComponent,
      ),
  },
  {
    path: 'divida-ativa',
    title: 'Consulta Dívida Ativa | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/divida-ativa/consulta-divida-ativa.component').then(
        (m) => m.ConsultaDividaAtivaComponent,
      ),
  },
  {
    path: 'integra-contador',
    title: 'Integra Contador | Accurit Cliente',
    canActivate: [autenticadoGuard],
    loadComponent: () =>
      import('./features/integra-contador/integra-contador.component').then(
        (m) => m.IntegraContadorComponent,
      ),
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
