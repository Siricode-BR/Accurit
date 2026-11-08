import '@angular/compiler';
import { Injector } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { TrabalhosService } from './trabalhos.service';
import {
  TrabalhoAgendadoDto,
  TipoTrabalho,
  ConsultaLoteDto,
  ExecucaoTrabalhoDto,
} from '@accurit/core';
import { vi } from 'vitest';
import { of, throwError } from 'rxjs';

const trabalhoMock: TrabalhoAgendadoDto = {
  id: 'trabalho-uuid-1',
  escritorioId: 'escritorio-uuid-1',
  nome: 'Consulta semanal DEVEDOR',
  tipo: 'consulta_divida_ativa',
  produto: 'DIVIDA_ATIVA',
  ambiente: 'TRIAL',
  cronExpression: '0 2 * * 1',
  habilitado: true,
  proximaExecucao: '2026-11-09T02:00:00Z',
  ultimaExecucao: null,
  criadoEm: '2026-11-08T10:00:00Z',
  atualizadoEm: '2026-11-08T10:00:00Z',
};

const loteMock: ConsultaLoteDto = {
  id: 'lote-uuid-1',
  escritorioId: 'escritorio-uuid-1',
  nome: 'Lote mensal DEVEDOR',
  produto: 'DIVIDA_ATIVA',
  servico: 'DEVEDOR',
  verbo: null,
  status: 'concluido',
  totalItens: 10,
  itensConcluidos: 9,
  itensComErro: 1,
  criadoEm: '2026-11-08T10:00:00Z',
  iniciadoEm: '2026-11-08T10:05:00Z',
  finalizadoEm: '2026-11-08T10:10:00Z',
};

const execucaoMock: ExecucaoTrabalhoDto = {
  id: 'execucao-uuid-1',
  trabalhoAgendadoId: 'trabalho-uuid-1',
  escritorioId: 'escritorio-uuid-1',
  status: 'concluido',
  iniciadoEm: '2026-11-09T02:00:00Z',
  finalizadoEm: '2026-11-09T02:05:00Z',
  erroMensagem: null,
  registrosProcessados: 5,
  registrosComErro: 0,
  criadoEm: '2026-11-09T02:00:00Z',
};

function criarService(mockHttp: {
  get?: ReturnType<typeof vi.fn>;
  post?: ReturnType<typeof vi.fn>;
  patch?: ReturnType<typeof vi.fn>;
}): TrabalhosService {
  const injector = Injector.create({
    providers: [{ provide: HttpClient, useValue: mockHttp }, TrabalhosService],
  });
  return injector.get(TrabalhosService);
}

describe('TrabalhosService', () => {
  it('deve listar trabalhos agendados', async () => {
    const mockGet = vi.fn().mockReturnValue(of([trabalhoMock]));
    const service = criarService({ get: mockGet });

    await service.listarTrabalhos();

    expect(mockGet).toHaveBeenCalledWith('/api/v1/trabalhos');
    expect(service.trabalhos()).toEqual([trabalhoMock]);
  });

  it('deve criar um trabalho agendado', async () => {
    const mockPost = vi.fn().mockReturnValue(of(trabalhoMock));
    const service = criarService({ post: mockPost });

    const resultado = await service.criarTrabalho({
      nome: 'Novo trabalho',
      tipo: 'consulta_divida_ativa',
      produto: 'DIVIDA_ATIVA',
      ambiente: 'TRIAL',
      cronExpression: '0 2 * * 1',
    });

    expect(mockPost).toHaveBeenCalledWith('/api/v1/trabalhos', {
      nome: 'Novo trabalho',
      tipo: 'consulta_divida_ativa',
      produto: 'DIVIDA_ATIVA',
      ambiente: 'TRIAL',
      cronExpression: '0 2 * * 1',
    });
    expect(resultado).toEqual(trabalhoMock);
    expect(service.trabalhos()).toContainEqual(trabalhoMock);
  });

  it('deve chamar PATCH para alternar habilitado de um trabalho', async () => {
    const mockPatch = vi.fn().mockReturnValue(of(null));
    const service = criarService({ patch: mockPatch });

    await service.alternarTrabalho('trabalho-uuid-1', { habilitado: false });

    expect(mockPatch).toHaveBeenCalledWith(
      '/api/v1/trabalhos/trabalho-uuid-1/alternar',
      { habilitado: false },
    );
  });

  it('deve listar consultas em lote', async () => {
    const mockGet = vi.fn().mockReturnValue(of([loteMock]));
    const service = criarService({ get: mockGet });

    await service.listarLotes();

    expect(mockGet).toHaveBeenCalledWith('/api/v1/consultas-lote');
    expect(service.lotes()).toEqual([loteMock]);
  });

  it('deve criar uma consulta em lote', async () => {
    const mockPost = vi.fn().mockReturnValue(of(loteMock));
    const service = criarService({ post: mockPost });

    const resultado = await service.criarConsultaLote({
      nome: 'Lote teste',
      produto: 'DIVIDA_ATIVA',
      servico: 'DEVEDOR',
      verbo: null,
      documentos: [
        { documento: '52998224725', tipo: 'PF' },
        { documento: '12345678000195', tipo: 'PJ' },
      ],
    });

    expect(mockPost).toHaveBeenCalledWith('/api/v1/consultas-lote', {
      nome: 'Lote teste',
      produto: 'DIVIDA_ATIVA',
      servico: 'DEVEDOR',
      verbo: null,
      documentos: [
        { documento: '52998224725', tipo: 'PF' },
        { documento: '12345678000195', tipo: 'PJ' },
      ],
    });
    expect(resultado).toEqual(loteMock);
  });

  it('deve listar execuções por trabalho', async () => {
    const mockGet = vi.fn().mockReturnValue(of([execucaoMock]));
    const service = criarService({ get: mockGet });

    await service.listarExecucoesPorTrabalho('trabalho-uuid-1');

    expect(mockGet).toHaveBeenCalledWith(
      '/api/v1/execucoes/trabalho/trabalho-uuid-1',
    );
    expect(service.execucoes()).toEqual([execucaoMock]);
  });

  it('propaga erro de listagem de trabalhos', async () => {
    const erroMock = new Error('Falha no backend');
    const mockGet = vi.fn().mockReturnValue(throwError(() => erroMock));
    const service = criarService({ get: mockGet });

    await service.listarTrabalhos().catch(() => {});

    expect(service.erroTrabalhos()).toBe('Falha no backend');
  });
});
