export interface RetencaoLgpdDto {
  consultasDias: number;
  relatoriosDias: number;
  auditoriaDias: number;
}

export interface ConsentimentoLgpdDto {
  id: string;
  tipo: 'tratamento_dados' | 'compartilhamento_serpro' | 'comunicacoes' | string;
  versao: string;
  aceitoEm: string;
  revogadoEm: string | null;
}

export interface RegistrarConsentimentoRequestDto {
  tipo: string;
  versao?: string;
}
