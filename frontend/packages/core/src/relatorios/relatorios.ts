//! Fase F — DTOs de relatórios e link seguro de compartilhamento.
//! Espelham o backend `presentation/relatorios/dto.rs`.

export type TipoRelatorio =
  | 'situacao_fiscal'
  | 'historico_consultas'
  | 'pendencias_carteira';

export type FormatoRelatorio = 'PDF' | 'XLSX';

export type StatusRelatorio = 'gerando' | 'concluido' | 'falhou';

export interface RelatorioDto {
  id: string;
  escritorioId: string;
  usuarioId: string;
  clienteId?: string | null;
  tipoRelatorio: TipoRelatorio;
  formato: FormatoRelatorio;
  status: StatusRelatorio;
  conteudoBase64?: string | null;
  nomeArquivo?: string | null;
  expiraEm: string;
  criadoEm: string;
  atualizadoEm: string;
}

export interface RelatorioResumoDto {
  id: string;
  tipoRelatorio: string;
  formato: string;
  status: string;
  nomeArquivo?: string | null;
  expiraEm: string;
  criadoEm: string;
}

export interface CriarRelatorioRequestDto {
  tipoRelatorio: TipoRelatorio;
  formato: FormatoRelatorio;
  clienteId?: string;
  competencia?: string; // 'YYYY-MM'
}

export interface LinkCompartilhamentoDto {
  id: string;
  relatorioId: string;
  token: string;
  expiraEm: string;
  acessadoEm?: string | null;
  criadoEm: string;
}

export interface CriarLinkCompartilhamentoRequestDto {
  expiraEm: string;
}
