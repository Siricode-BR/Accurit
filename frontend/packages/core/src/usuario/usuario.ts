/** Papéis de usuário da plataforma do escritório (ADMIN é exclusivo do backoffice). */
export type PapelUsuario = 'CLIENTE' | 'OPERADOR';

export interface UsuarioDto {
  id: string;
  escritorioId: string;
  escritorioNome: string;
  escritorioSlug: string;
  nome: string;
  email: string;
  cpf: string;
  papel: PapelUsuario;
  ativo: boolean;
  ultimoLoginEm: string | null;
  criadoEm: string;
}

export interface ListagemUsuariosDto {
  itens: UsuarioDto[];
  total: number;
}

export interface CriarUsuarioRequestDto {
  nome: string;
  email: string;
  cpf: string;
  papel: PapelUsuario;
  senha: string;
}

/** No backoffice o escritório é informado explicitamente. */
export interface CriarUsuarioAdminRequestDto extends CriarUsuarioRequestDto {
  escritorioId: string;
}

export interface AtualizarUsuarioRequestDto {
  nome?: string;
  papel?: PapelUsuario;
  ativo?: boolean;
}

export interface RedefinirSenhaUsuarioRequestDto {
  senha: string;
}
