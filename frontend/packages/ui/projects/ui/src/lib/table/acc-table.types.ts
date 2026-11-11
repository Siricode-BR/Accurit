import { TemplateRef } from '@angular/core';

/** Coluna declarativa de `acc-table`. */
export interface AccColunaTabela<T = Record<string, unknown>> {
  /** Campo da linha usado por padrão como valor e para ordenação. */
  readonly campo: string;
  readonly titulo: string;
  readonly ordenavel?: boolean;
  readonly alinhamento?: 'left' | 'center' | 'right';
  readonly largura?: string;
  /** Célula customizada; contexto: `{ $implicit: linha, linha, valor }`. */
  readonly template?: TemplateRef<{ $implicit: T; linha: T; valor: unknown }>;
}

export interface AccOrdenacaoTabela {
  readonly campo: string;
  readonly direcao: 'asc' | 'desc';
}
