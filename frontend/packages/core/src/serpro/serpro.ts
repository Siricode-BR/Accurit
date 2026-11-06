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

