/** Estado de carregamento da ficha do cliente. */
export type EstadoCliente = 'carregando' | 'pronto' | 'naoEncontrado' | 'erro';

/** Abas aceitas em `?aba=`. */
export type AbaId =
  | 'visao-geral'
  | 'consultas'
  | 'contatos'
  | 'socios-contatos'
  | 'filiais';

export interface LeituraEstado {
  /** rótulo curto do cartão/badge em pt-BR */
  readonly rotulo: string;
  /** cor do badge: success = sem pendências; warning = atenção; danger = falha */
  readonly status: 'success' | 'warning' | 'danger' | 'info' | 'neutral';
  /** texto explicativo sob o estado */
  readonly descricao: string;
}
