//! DTOs de emissão de guias (FASE D).
//! Espelham o backend `presentation/guias/dto.rs`.

export type TipoGuia = 'DAS' | 'DARF';
export type StatusGuia = 'solicitada' | 'emitida' | 'baixada' | 'falhou';

export interface GuiaEmissaoDto {
  id: string;
  usuarioId: string;
  escritorioId: string;
  clienteId: string;
  tipoGuia: TipoGuia;
  competencia: string; // 'YYYY-MM'
  valorTotal: number;
  pdfUrl: string | null;
  status: StatusGuia;
  idempotenciaChave: string;
  criadoEm: string;
  atualizadoEm: string;
}

export interface RequisicaoEmitirGuia {
  clienteId: string;
  tipoGuia: TipoGuia;
  competencia: string; // 'YYYY-MM'
  valorTotal: number;
  idempotenciaChave: string;
}
