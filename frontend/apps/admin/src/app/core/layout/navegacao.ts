import { AccNavGroup } from '@accurit/ui';

/** Menu lateral do backoffice SaaS. */
export const NAVEGACAO_ADMIN: readonly AccNavGroup[] = [
  {
    id: 'principal',
    items: [
      { id: 'dashboard', label: 'Dashboard', icon: 'dashboard', route: '/dashboard', exact: true },
      { id: 'tenants', label: 'Escritórios', icon: 'building', route: '/tenants' },
      { id: 'consumo', label: 'Consumo e planos', icon: 'dashboard', route: '/consumo' },
      { id: 'administradores', label: 'Administradores', icon: 'users', route: '/administradores' },
      { id: 'catalogo', label: 'Catálogo Integra', icon: 'integra-contador', route: '/catalogo' },
    ],
  },
];