export type Papel = 'ADMIN' | 'OPERADOR' | 'CLIENTE';

export interface Usuario {
  id: string;
  email: string;
  papel: Papel;
  escritorioId: string;
}

export interface Credenciais {
  escritorio: string;
  email: string;
  senha: string;
}

export interface SessaoResposta {
  accessToken: string;
  expiresIn: number;
  usuario: Usuario;
}

export interface UsuarioAtualResposta {
  usuario: Usuario;
}
