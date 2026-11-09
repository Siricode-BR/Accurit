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
  representanteNome?: string;
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
  representanteNome?: string;
  email?: string;
  telefone?: string;
}

export interface UpdateClienteRequestDto {
  tipo?: TipoCliente;
  porteRegime?: PorteRegime | null;
  cpfCnpj?: string;
  nomeRazaoSocial?: string;
  nomeFantasia?: string | null;
  representanteNome?: string | null;
  email?: string | null;
  telefone?: string | null;
}

// ===== Fase E — Gestão de carteira =====

export type PapelSocioContato = 'SOCIO' | 'CONTATO';

export interface SocioContatoDto {
  id: string;
  clienteId: string;
  papel: PapelSocioContato;
  nome: string;
  documento?: string | null;
  email?: string | null;
  telefone?: string | null;
  representantePrincipal: boolean;
  criadoEm: string;
  atualizadoEm: string;
}

export interface CreateSocioContatoRequestDto {
  papel: PapelSocioContato;
  nome: string;
  documento?: string | null;
  email?: string | null;
  telefone?: string | null;
  representantePrincipal: boolean;
}

export interface FilialDto {
  id: string;
  clienteId: string;
  cnpj: string;
  nomeUnidade?: string | null;
  endereco?: string | null;
  telefone?: string | null;
  criadoEm: string;
  atualizadoEm: string;
}

export interface CreateFilialRequestDto {
  cnpj: string;
  nomeUnidade?: string | null;
  endereco?: string | null;
  telefone?: string | null;
}

export interface ClienteTagDto {
  id: string;
  nome: string;
  cor?: string | null;
  criadoEm: string;
}

export interface CreateTagRequestDto {
  nome: string;
  cor?: string | null;
}

export interface VincularTagRequestDto {
  tagId: string;
}

export interface CarteiraClienteDto {
  clienteId: string;
  responsavelUsuarioId?: string | null;
  observacoes?: string | null;
  criadoEm: string;
  atualizadoEm: string;
}

export interface AtualizarCarteiraRequestDto {
  responsavelUsuarioId?: string | null;
  limparResponsavel?: boolean;
  observacoes?: string | null;
  limparObservacoes?: boolean;
}

export type FormatoImportacao = 'CSV' | 'XLSX';

export interface LinhaClienteImportacaoDto {
  linha: number;
  tipo: string;
  cpfCnpj: string;
  nomeRazaoSocial: string;
  nomeFantasia?: string | null;
  representanteNome?: string | null;
  email?: string | null;
  telefone?: string | null;
  porteRegime?: string | null;
}

export interface ErroLinhaImportacaoDto {
  linha: number;
  campo?: string | null;
  mensagem: string;
}

export interface AnaliseImportacaoDto {
  formato: FormatoImportacao;
  totalLinhas: number;
  validas: LinhaClienteImportacaoDto[];
  erros: ErroLinhaImportacaoDto[];
}

export interface RelatorioImportacaoDto {
  criados: number;
  ignoradosDuplicados: number;
  erros: ErroLinhaImportacaoDto[];
}

export interface ImportacaoRequestDto {
  formato: FormatoImportacao;
  conteudoBase64: string;
}
