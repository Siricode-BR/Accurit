import { AccNavGroup } from '@accurit/ui';

/** Menu lateral do backoffice SaaS. */
export const NAVEGACAO_ADMIN: readonly AccNavGroup[] = [
  {
    id: 'principal',
    items: [
      { id: 'dashboard', label: 'Dashboard', icon: 'dashboard', route: '/dashboard', exact: true },
      { id: 'tenants', label: 'Escritórios', icon: 'building', route: '/tenants' },
      { id: 'usuarios', label: 'Usuários', icon: 'users', route: '/usuarios' },
      { id: 'catalogo', label: 'Catálogo Integra', icon: 'integra-contador', route: '/catalogo' },
    ],
  },
];