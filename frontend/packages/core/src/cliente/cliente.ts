export type TipoCliente = 'PF' | 'PJ';

export type PorteRegime = 'MEI' | 'SIMPLES_NACIONAL' | 'LUCRO_PRESUMIDO' | 'LUCRO_REAL';

export type FiltroClientes =
  | 'todos'
  | 'cpf'
  | 'cnpj_mei'
  | 'cnpj_simples'
  | 'cnpj_presumido'
  | 'cnpj_real';

export interface ClienteDto {
  id: string;
  tipo: TipoCliente;
  porteRegime?: PorteRegime;
  cpfCnpj: string;
  nomeRazaoSocial: string;
  nomeFantasia?: string;
  email?: string;
  telefone?: string;
  criadoEm: string;
  atualizadoEm: string;
}

export interface ListagemClientesDto {
  itens: ClienteDto[];
  total: number;
  pagina: number;
  tamanho: number;
}

export interface CreateClienteRequestDto {
  tipo: TipoCliente;
  porteRegime?: PorteRegime;
  cpfCnpj: string;
  nomeRazaoSocial: string;
  nomeFantasia?: string;
  email?: string;
  telefone?: string;
}

export interface UpdateClienteRequestDto {
  tipo?: TipoCliente;
  porteRegime?: PorteRegime | null;
  cpfCnpj?: string;
  nomeRazaoSocial?: string;
  nomeFantasia?: string | null;
  email?: string | null;
  telefone?: string | null;
}

