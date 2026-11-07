import { inject } from '@angular/core';
import { Router, Routes } from '@angular/router';
import { authGuard, publicGuard } from './core/auth/auth.guards';
import { operadorGuard } from './core/auth/operador.guards';

export const routes: Routes = [
  {
    // Usuário logado cai direto no dashboard; sem sessão o authGuard envia ao login.
    path: '',
    pathMatch: 'full',
    redirectTo: 'dashboard',
  },
  {
    path: 'login',
    canActivate: [publicGuard],
    loadComponent: () =>
      import('./features/auth/login.component').then((m) => m.LoginComponent),
  },
  {
    path: 'dashboard',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/dashboard/dashboard.page').then((m) => m.DashboardPage),
  },
  {
    path: 'clientes',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/clientes/cliente-list.component').then((m) => m.ClienteListComponent),
  },
  {
    // Cadastro agora é um modal em /clientes; mantém links antigos funcionando.
    path: 'clientes/novo',
    redirectTo: () => inject(Router).parseUrl('/clientes?novo=1'),
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
  // Consultas vinculadas a um cliente cadastrado (documento bloqueado). Exigem ?clienteId=.
  {
    path: 'divida-ativa',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/divida-ativa/consulta.page').then((m) => m.DividaAtivaConsultaPage),
  },
  {
    path: 'integra-contador',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/integra-contador/consulta.page').then((m) => m.IntegraContadorConsultaPage),
  },
  // Consultas avulsas para contribuintes não cadastrados (documento livre).
  {
    path: 'consultas/divida-ativa',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/divida-ativa/consulta-avulsa.page').then((m) => m.DividaAtivaAvulsaPage),
  },
  {
    path: 'consultas/integra-contador',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/integra-contador/consulta-avulsa.page').then(
        (m) => m.IntegraContadorAvulsaPage,
      ),
  },
  { path: '**', redirectTo: 'dashboard' },
];
