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

