export type UserRole = 'ADMIN' | 'CONTADOR' | 'OPERADOR' | 'LEITURA';

export interface LoginRequestDto {
  email: string;
  senha: string;
  /// Slug do escritório (obrigatório para login de cliente/escritório).
  escritorio?: string;
  tenantId?: string;
}

export interface LoginResponseDto {
  token: string;
  refreshToken?: string;
  expiresIn: number;
  usuario: UsuarioSessaoDto;
}

export interface UsuarioSessaoDto {
  id: string;
  nome: string;
  email: string;
  role: UserRole;
  tenantId?: string;
  tenantNome?: string;
}

export interface AuthSessionState {
  isAuthenticated: boolean;
  user: UsuarioSessaoDto | null;
  token: string | null;
}

