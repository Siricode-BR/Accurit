import { AccNavGroup } from '@accurit/ui';

/** Menu lateral do portal do escritório. */
export const NAVEGACAO_CLIENT: readonly AccNavGroup[] = [
  {
    id: 'principal',
    items: [
      { id: 'dashboard', label: 'Dashboard', icon: 'dashboard', route: '/dashboard', exact: true },
      { id: 'clientes', label: 'Clientes', icon: 'clientes', route: '/clientes' },
    ],
  },
  {
    id: 'consultas',
    label: 'Consultas avulsas',
    items: [
      { id: 'integra-contador', label: 'Integra Contador', icon: 'integra-contador', route: '/consultas/integra-contador' },
      { id: 'divida-ativa', label: 'Dívida Ativa', icon: 'divida-ativa', route: '/consultas/divida-ativa' },
    ],
  },
  {
    id: 'administracao',
    label: 'Administração',
    items: [{ id: 'usuarios', label: 'Usuários', icon: 'users', route: '/usuarios' }],
  },
];
