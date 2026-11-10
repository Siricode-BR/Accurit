/** Administrador do backoffice (papel ADMIN da plataforma, rota
 * `/api/v1/administradores`). Contrato espelha `AdministradorResponse` do
 * backend com `rename_all = "camelCase"`. */
export interface AdministradorDto {
  id: string;
  nome: string;
  email: string;
  cpf: string;
  ativo: boolean;
  criadoEm: string;
  atualizadoEm: string;
  ultimoLoginEm: string | null;
}

export interface ListagemAdministradoresDto {
  itens: AdministradorDto[];
  page: number;
  pageSize: number;
  total: number;
}

export interface CriarAdministradorRequestDto {
  nome: string;
  email: string;
  cpf: string;
  senha: string;
}

/** `PATCH /administradores/{id}` — alterações parciais (a senha tem fluxo próprio). */
export interface AtualizarAdministradorRequestDto {
  nome?: string;
  email?: string;
  ativo?: boolean;
}

/** `PUT /administradores/{id}` — alteração de senha. */
export interface AlterarSenhaAdministradorRequestDto {
  senha: string;
}
