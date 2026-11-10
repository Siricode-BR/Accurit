/** Papéis de usuário da plataforma do escritório (ADMIN é exclusivo do backoffice). */
export type PapelUsuario = 'CLIENTE' | 'OPERADOR';

/** Função granular na carteira do escritório (Fase E). */
export type FuncaoUsuario = 'GESTOR' | 'ANALISTA_FISCAL' | 'OPERADOR_LEITURA';

export interface UsuarioDto {
  id: string;
  escritorioId: string;
  escritorioNome: string;
  escritorioCnpj: string;
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
  pagina: number;
  tamanho: number;
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

// ===== Fase E — função e carteira do usuário =====

export interface AcessoUsuarioDto {
  funcao: FuncaoUsuario;
  podeAdministrarUsuarios: boolean;
  clientesPermitidos?: string[] | null;
}

export interface DefinirAcessoRequestDto {
  funcao: FuncaoUsuario;
}

export interface DefinirCarteiraRequestDto {
  clientes: string[];
}
