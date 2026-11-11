import '@angular/compiler';
import { Injector } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ProcuracaoService } from './procuracao.service';
import {
  EstadoProcuracaoClienteDto,
  ResultadoValidarProcuracaoResponseDto,
} from '@accurit/core';
import { vi } from 'vitest';
import { of, throwError } from 'rxjs';

const clienteId = 'cliente-uuid-123';

const estadoMock: EstadoProcuracaoClienteDto = {
  contribuinteCpfCnpj: '12345678000195',
  procuradorCpfCnpj: '11222333000181',
  status: 'AUTORIZADA',
  verificadoEm: '2026-11-08T10:00:00Z',
  validoDe: '2026-01-01T00:00:00Z',
  validoAte: '2027-01-01T00:00:00Z',
  servicos: [
    {
      idSistema: 'RECEITA',
      idServico: 'SITUACAO_FISCAL',
      verbo: 'Consultar',
      habilitado: true,
      estado: 'AUTORIZADA',
    },
  ],
};

const resultadoValidarMock: ResultadoValidarProcuracaoResponseDto = {
  fonte: 'SERPRO',
  registro: {
    id: 'proc-id-1',
    clienteId,
    ambiente: 'TRIAL',
    contribuinteCpfCnpj: '12345678000195',
    procuradorCpfCnpj: '11222333000181',
    codigoServico: 'SERPRO_TODOS',
    status: 'AUTORIZADA',
    validoDe: '2026-01-01T00:00:00Z',
    validoAte: '2027-01-01T00:00:00Z',
    verificadoEm: '2026-11-08T10:00:00Z',
    origem: 'SERPRO_ONLINE',
  },
};

function criarService(mockHttp: { get?: ReturnType<typeof vi.fn>; post?: ReturnType<typeof vi.fn> }): ProcuracaoService {
  const injector = Injector.create({
    providers: [
      { provide: HttpClient, useValue: mockHttp },
      ProcuracaoService,
    ],
  });
  return injector.get(ProcuracaoService);
}

describe('ProcuracaoService', () => {
  it('deve retornar PENDENTE por padrão para cliente desconhecido', () => {
    const service = criarService({});
    expect(service.statusDe('qualquer')).toBe('PENDENTE');
    expect(service.estadoDe('qualquer')).toBeNull();
  });

  it('deve obter e atualizar o estado do cliente a partir do backend', () => {
    const mockGet = vi.fn().mockReturnValue(of(estadoMock));
    const service = criarService({ get: mockGet });

    let resultado: EstadoProcuracaoClienteDto | undefined;
    service.obterPorCliente(clienteId).subscribe((res) => (resultado = res));

    expect(mockGet).toHaveBeenCalledWith('/api/v1/serpro/integra-contador/procuracoes', {
      params: { clienteId },
    });
    expect(resultado).toEqual(estadoMock);
    expect(service.statusDe(clienteId)).toBe('AUTORIZADA');
    expect(service.estadoDe(clienteId)).toEqual(estadoMock);
    expect(service.statusPorCliente()[clienteId]).toBe('AUTORIZADA');
  });

  it('deve validar e em seguida recarregar o estado do cliente', () => {
    const mockPost = vi.fn().mockReturnValue(of(resultadoValidarMock));
    const mockGet = vi.fn().mockReturnValue(of(estadoMock));
    const service = criarService({ post: mockPost, get: mockGet });

    let resultado: EstadoProcuracaoClienteDto | undefined;
    service.validarProcuracao(clienteId).subscribe((res) => (resultado = res));

    expect(mockPost).toHaveBeenCalledWith('/api/v1/serpro/integra-contador/procuracoes/validar', {
      clienteId,
    });
    expect(mockGet).toHaveBeenCalledWith('/api/v1/serpro/integra-contador/procuracoes', {
      params: { clienteId },
    });
    expect(resultado).toEqual(estadoMock);
  });

  it('deve revalidar forçadamente via endpoint /revalidar', () => {
    const mockPost = vi.fn().mockReturnValue(of(resultadoValidarMock));
    const mockGet = vi.fn().mockReturnValue(of(estadoMock));
    const service = criarService({ post: mockPost, get: mockGet });

    service.revalidarProcuracao(clienteId).subscribe();

    expect(mockPost).toHaveBeenCalledWith('/api/v1/serpro/integra-contador/procuracoes/revalidar', {
      clienteId,
    });
  });

  it('propaga erro de validação', () => {
    const erroMock = new Error('Falha no SERPRO');
    const mockPost = vi.fn().mockReturnValue(throwError(() => erroMock));
    const service = criarService({ post: mockPost });

    let capturado: unknown;
    service.validarProcuracao(clienteId).subscribe({
      error: (e) => (capturado = e),
    });

    expect(capturado).toBe(erroMock);
  });
});
