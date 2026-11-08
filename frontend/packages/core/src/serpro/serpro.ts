export type AmbienteSerpro = 'TRIAL' | 'PRODUCAO';
export type EstadoAmbiente = 'indefinido' | 'carregando' | 'pronto' | 'indisponivel';

export interface AmbienteSerproInfoDto {
  ambienteEfetivo: AmbienteSerpro;
  atualizadoEm: string;
}

export interface CredencialSerproDto {
  consumerKey: string;
  consumerSecret: string;
  configurado: boolean;
  atualizadoEm?: string;
}

export interface CertificadoA1Dto {
  nomeArquivo: string;
  validoAte: string;
  emissor: string;
  status: 'VALIDO' | 'EXPIRADO' | 'PENDENTE';
}

// DTO para a requisição de Autenticação de Procurador (AUTENTICAPROCURADOR /
// ENVIOXMLASSINADO81). O escritório (Procurador) e o ambiente são resolvidos
// pelo backend a partir do contexto autenticado — o frontend informa apenas
// o cliente (contribuinte) que receberá a procuração e-CAC validada.
export interface AutenticarProcuradorRequestDto {
  clienteId: string;
}

// DTO para a resposta da Autenticação de Procurador. Por decisão de
// segurança (ADR-0007 / REGRA SEG), o `jwt_token` nunca é exposto ao
// frontend: ele fica cifrado em `tokens_serpro` e é usado apenas pelo
// backend nas chamadas subsequentes à API do SERPRO.
export interface AutenticarProcuradorResponseDto {
  clienteId: string;
  status: 'AUTORIZADA';
  expiraEm: string | null;
}

export type StatusProcuracaoGeral = 'AUTORIZADA' | 'NAO_AUTORIZADA' | 'INDETERMINADA' | 'PENDENTE';
export type EstadoProcuracaoServicoTipo = 'NAO_EXIGIDA' | 'AUTORIZADA' | 'CONSULTADA' | 'PENDENTE';

export interface EstadoProcuracaoServicoDto {
  idSistema: string;
  idServico: string;
  verbo: string;
  habilitado: boolean;
  estado: EstadoProcuracaoServicoTipo | string;
}

export interface EstadoProcuracaoClienteDto {
  contribuinteCpfCnpj: string;
  procuradorCpfCnpj: string;
  status: StatusProcuracaoGeral | null;
  verificadoEm: string | null;
  validoDe: string | null;
  validoAte: string | null;
  servicos: EstadoProcuracaoServicoDto[];
}

export interface ValidarProcuracaoRequestDto {
  clienteId: string;
}

export interface ProcuracaoRegistroDto {
  id: string;
  clienteId: string;
  ambiente: string;
  contribuinteCpfCnpj: string;
  procuradorCpfCnpj: string;
  codigoServico: string;
  status: string;
  validoDe: string | null;
  validoAte: string | null;
  verificadoEm: string;
  origem: string;
}

export interface ResultadoValidarProcuracaoResponseDto {
  fonte: 'CACHE' | 'SERPRO';
  registro: ProcuracaoRegistroDto;
}

export interface LinhaCatalogoServicoDto {
  id: string;
  idSistema: string;
  idServico: string;
  verbo: string;
  versaoSistema: string | null;
  versaoConfirmada: boolean;
  habilitado: boolean;
  apenasConsulta: boolean;
  requerProcuracao: boolean;
  codigosProcuracao: string[];
  descricao: string;
  atualizadoEm: string;
}

export interface RequisicaoHabilitarServicoDto {
  habilitado: boolean;
}
