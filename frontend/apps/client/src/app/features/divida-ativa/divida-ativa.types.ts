/** Forma de consulta na Dívida Ativa. */
export type ConsultaModo = 'devedor' | 'inscricao';

export interface ModoOpcao {
  id: ConsultaModo;
  label: string;
}

/** Estado do carregamento do cliente na consulta vinculada. */
export type EstadoClienteConsulta = 'carregando' | 'pronto' | 'naoEncontrado';
