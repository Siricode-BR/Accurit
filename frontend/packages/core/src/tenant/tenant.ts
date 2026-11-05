import { AmbienteSerpro } from '../serpro/serpro';

export interface TenantDto {
  id: string;
  slug: string;
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
  slug: string;
  nome: string;
  emailTitular: string;
  cpfTitular: string;
  senhaTitular: string;
}

export interface CriarTenantResponseDto {
  escritorioId: string;
  slug: string;
  titularId: string;
  emailTitular: string;
}
