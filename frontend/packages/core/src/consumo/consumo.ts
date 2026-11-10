export interface AgregadoConsumoDto {
  produto: string;
  ambiente: string;
  periodo: string;
  referenciaPeriodo: string;
  totalChamadas: number;
  chamadasSucesso: number;
  chamadas4xx: number;
  chamadas5xx: number;
  chamadasTimeout: number;
  chamadasIndeterminado: number;
  duracaoTotalMs: number;
}

export interface ChamadaRecenteDto {
  id: string;
  produto: string;
  verbo: string;
  servico: string | null;
  ambiente: string;
  clienteId: string | null;
  resultado: string;
  statusHttp: number | null;
  duracaoMs: number;
  criadoEm: string;
}

export interface ConsumoClienteDto {
  escritorioId: string;
  planoId: string;
  chamadasMesLimite: number;
  clientesLimite: number;
  usuariosLimite: number;
  bloqueioAoExceder: boolean;
  chamadasUsadasMes: number;
  percentualUtilizacao: number;
  avisoConsumo: string | null;
  agregados: AgregadoConsumoDto[];
  chamadasRecentes: ChamadaRecenteDto[];
}

export interface ConsumoAdminEscritorioDto {
  escritorioId: string;
  nome: string;
  cnpj: string;
  planoNome: string | null;
  chamadasLimite: number | null;
  chamadasUsadas: number;
  chamadasSucesso: number;
  chamadasErro: number;
  taxaErro: number;
  duracaoMediaMs: number;
}

export interface RankingConsumoDto {
  escritorioId: string;
  nome: string;
  chamadas: number;
}

export interface SerieConsumoDto {
  dia: string;
  chamadas: number;
  erros: number;
  duracaoMediaMs: number;
}

export interface ConsumoAdminDto {
  inicio: string;
  fim: string;
  totalChamadas: number;
  totalErros: number;
  taxaErro: number;
  duracaoMediaMs: number;
  escritorios: ConsumoAdminEscritorioDto[];
  ranking: RankingConsumoDto[];
  serie: SerieConsumoDto[];
}

export interface PlanoEscritorioDto {
  id: string;
  nome: string;
  descricao: string | null;
  chamadasMes: number;
  clientesLimite: number;
  usuariosLimite: number;
  precoMensalCents: number;
  ativo: boolean;
  criadoEm: string;
  atualizadoEm: string;
}

export interface CriarPlanoRequestDto {
  nome: string;
  descricao: string | null;
  chamadasMes: number;
  clientesLimite: number;
  usuariosLimite: number;
  precoMensalCents: number;
}

export interface AtribuirPlanoRequestDto {
  planoId: string;
  chamadasMesLimite: number;
  bloqueioAoExceder: boolean;
}

export interface AtribuirPlanoResponseDto {
  id: string;
  escritorioId: string;
  planoId: string;
  chamadasMesLimite: number;
  clientesLimite: number;
  usuariosLimite: number;
  bloqueioAoExceder: boolean;
  ativo: boolean;
  atualizadoEm: string;
}
