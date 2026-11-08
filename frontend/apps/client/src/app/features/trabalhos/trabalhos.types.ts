//! Tipos locais da feature de trabalhos agendados e consultas em lote (FASE B).

export type AbaTrabalhos = 'trabalhos' | 'execucoes' | 'lotes';

export interface TrabalhoResumo {
  id: string;
  nome: string;
  tipo: string;
  ambiente: string;
  habilitado: boolean;
  proximaExecucao: string | null;
}
