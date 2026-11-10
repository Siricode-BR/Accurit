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
  {
    id: 'automacao',
    label: 'Automação (Fase B)',
    items: [
      { id: 'trabalhos', label: 'Trabalhos agendados', icon: 'users', route: '/trabalhos' },
      { id: 'consultas-lote', label: 'Consultas em lote', icon: 'search', route: '/consultas-lote' },
    ],
  },
  {
    id: 'relatorios',
    label: 'Relatórios (Fase F)',
    items: [{ id: 'relatorios', label: 'Relatórios', icon: 'beaker', route: '/relatorios' },
      { id: 'consumo', label: 'Consumo SERPRO', icon: 'dashboard', route: '/consumo' }],
  },
];
