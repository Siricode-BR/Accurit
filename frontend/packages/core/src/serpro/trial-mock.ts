export type ServicoTrialMock = 'DIVIDA_ATIVA' | 'INTEGRA_CONTADOR' | 'PROCURACAO';

export interface CenarioTrialMockDto {
  id: string;
  servico: ServicoTrialMock;
  titulo: string;
  descricao: string;
  documentoExemplo: string;
  resultadoEsperado: string;
}

export interface CenariosTrialMockDto {
  ativo: boolean;
  cenarios: CenarioTrialMockDto[];
}
