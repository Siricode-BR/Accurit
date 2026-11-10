import { AmbienteSerpro } from '../serpro/serpro';

export interface TenantDto {
  id: string;
  cnpj: string;
  nome: string;
  modoTrialSerpro: boolean;
  versao: number;
  atualizadoEm: string;
}

export interface TenantAlteradoDto {
  modoTrialSerpro: boolean;
  ambienteEfetivo: AmbienteSerpro;
  versaoAtual: number;
  atualizadoEm: string;
}

export interface CriarTenantRequestDto {
  cnpj: string;
  nome: string;
  emailTitular: string;
  cpfTitular: string;
  senhaTitular: string;
}

export interface CriarTenantResponseDto {
  escritorioId: string;
  cnpj: string;
  titularId: string;
  emailTitular: string;
}