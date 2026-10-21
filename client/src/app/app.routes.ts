import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'inicio' },
  {
    path: 'inicio',
    title: 'Início | Accurit Cliente',
    loadComponent: () =>
      import('./features/inicio/inicio').then((m) => m.Inicio),
  },
  { path: '**', redirectTo: 'inicio' },
];
