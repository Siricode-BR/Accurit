import type { VerboIntegraContador } from '../../data/integra-contador.service';

export type { VerboIntegraContador };

/** Entrada do catálogo de referência (verbo → sistema → serviço → versão). */
export interface ServicoCatalogo {
  idSistema: string;
  idServico: string;
  versaoSistema: string | null;
  verbo: VerboIntegraContador;
}

export interface VerboOpcao {
  valor: VerboIntegraContador;
  label: string;
}

/** Estado do carregamento do cliente na consulta vinculada. */
export type EstadoClienteConsulta = 'carregando' | 'pronto' | 'naoEncontrado';
