//! DTOs de notificações (FASE C).
//! Espelham o backend `presentation/notificacoes/dto.rs`.

export type CanalNotificacao = 'in_app' | 'email' | 'whatsapp';
export type TipoNotificacao =
  | 'pendencia_divida_ativa'
  | 'mensagem_caixa_postal'
  | 'procuracao_vencendo'
  | 'certificado_a1_vencendo'
  | 'das_vencendo'
  | 'falha_rotina';

export interface NotificacaoDto {
  id: string;
  usuarioId: string;
  escritorioId: string;
  canal: CanalNotificacao;
  tipo: TipoNotificacao;
  titulo: string;
  corpo: string;
  lida: boolean;
  lidaEm: string | null;
  referenciaTipo: string | null;
  referenciaId: string | null;
  criadoEm: string;
}

export interface RequisicaoCriarNotificacao {
  usuarioId: string;
  canal: CanalNotificacao;
  tipo: TipoNotificacao;
  titulo: string;
  corpo: string;
  referenciaTipo?: string;
  referenciaId?: string;
}

export interface RequisicaoMarcarLida {
  id: string;
}

export interface RequisicaoPreferenciaNotificacao {
  canalEmail: boolean;
  canalWhatsapp: boolean;
  canalInApp: boolean;
  tiposAtivados: TipoNotificacao[];
}

export interface PreferenciaNotificacaoDto {
  id: string;
  usuarioId: string;
  canalEmail: boolean;
  canalWhatsapp: boolean;
  canalInApp: boolean;
  tiposAtivados: TipoNotificacao[];
  criadoEm: string;
  atualizadoEm: string;
}

export interface RespostaContagemNaoLidas {
  naoLidas: number;
}

export interface TemplateNotificacaoDto {
  id: string;
  tipo: TipoNotificacao;
  assunto: string;
  corpoTexto: string;
  corpoHtml: string | null;
  variaveis: string[];
  ativo: boolean;
  criadoEm: string;
  atualizadoEm: string;
}
