import { inject } from '@angular/core';
import { Router, Routes } from '@angular/router';
import { authGuard, publicGuard } from './core/auth/auth.guards';
import { operadorGuard } from './core/auth/operador.guards';
import { titularGuard } from './core/auth/titular.guards';

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
  {
    // Gestão de usuários do escritório: somente o titular (papel CLIENTE).
    path: 'usuarios',
    canActivate: [authGuard, titularGuard],
    loadComponent: () =>
      import('./features/usuarios/usuario-list.component').then((m) => m.UsuarioListComponent),
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
  // Fase B — Trabalhos agendados e consultas em lote.
  {
    path: 'trabalhos',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/trabalhos-list.component').then(
        (m) => m.TrabalhosListComponent,
      ),
  },
  {
    path: 'trabalhos/novo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/trabalho-criar.component').then(
        (m) => m.TrabalhoCriarComponent,
      ),
  },
  {
    path: 'trabalhos/:id',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/trabalho-detalhes.component').then(
        (m) => m.TrabalhoDetalhesComponent,
      ),
  },
  {
    path: 'consultas-lote',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/consultas-lote-list.component').then(
        (m) => m.ConsultasLoteListComponent,
      ),
  },
  {
    path: 'consultas-lote/novo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/consulta-lote-criar.component').then(
        (m) => m.ConsultaLoteCriarComponent,
      ),
  },
  {
    path: 'consultas-lote/:id',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trabalhos/consulta-lote-detalhes.component').then(
        (m) => m.ConsultaLoteDetalhesComponent,
      ),
  },
  // Fase C — Notificações.
  {
    path: 'notificacoes',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/notificacoes/notificacoes-list.component').then(
        (m) => m.NotificacoesListComponent,
      ),
  },
  // Fase D — Emissão de guias.
  {
    path: 'guias',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/guias/guias-list.component').then(
        (m) => m.GuiasListComponent,
      ),
  },
  {
    path: 'guias/novo',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/guias/guia-emitir.component').then(
        (m) => m.GuiaEmitirComponent,
      ),
  },
  { path: '**', redirectTo: 'dashboard' },
];
