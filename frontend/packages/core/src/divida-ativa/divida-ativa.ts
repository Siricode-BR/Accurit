export type AmbienteConsulta = 'TRIAL' | 'PRODUCAO';
export type EstadoConsulta = 'COM_PENDENCIAS' | 'SEM_PENDENCIAS';
export type FonteConsulta = 'CPF_CNPJ' | 'CODIGO_CONTRIBUINTE';

export interface ValorConsultaDto {
  valor: string;
  remarca: string;
}

export interface InscricaoResumoDto {
  numeroInscricao: string;
  numeroProcesso?: string | null;
  situacaoDescricao: string;
  nomeDevedor?: string | null;
  tipoDevedor: string;
  valorTotalConsolidado: ValorConsultaDto;
}

export interface ConsultaDevedorDto {
  ambiente: AmbienteConsulta;
  estado: EstadoConsulta;
  fonteConsulta: FonteConsulta;
  chaveMascarada: string;
  horarioConsulta: string;
  tentativas: number;
  aviso: string;
  inscricoes: InscricaoResumoDto[];
}

export interface ConsultaInscricaoDto {
  ambiente: AmbienteConsulta;
  estado: EstadoConsulta;
  fonteConsulta: FonteConsulta | null;
  chaveMascarada: string;
  horarioConsulta: string;
  tentativas: number;
  aviso: string;
  numeroInscricao: string;
  processoAdministrativo?: string | null;
  situacao: string;
  dataInscricao?: string | null;
  orgaoOrigem?: string | null;
  codigoNaturezaReceita?: string | null;
  nomeReceita?: string | null;
  valorTotalInscrito?: ValorConsultaDto | null;
  valorTotalConsolidado?: ValorConsultaDto | null;
}

