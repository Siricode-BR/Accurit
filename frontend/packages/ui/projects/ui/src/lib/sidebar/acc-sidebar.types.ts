import { AccIconName } from '../icon/acc-icon.types';

export interface AccNavItem {
  id: string;
  label: string;
  icon: AccIconName;
  /** Rota do Angular Router (ex.: `/dashboard`). */
  route: string;
  /** Ativa apenas quando a URL é exatamente igual (use no dashboard). */
  exact?: boolean;
  /** Exibe o item desabilitado com selo "Em breve". */
  emBreve?: boolean;
}

export interface AccNavGroup {
  id: string;
  label?: string;
  items: readonly AccNavItem[];
}
