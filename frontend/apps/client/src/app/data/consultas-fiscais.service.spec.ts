import '@angular/compiler';
import { Injector } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ConsultasFiscaisService } from './consultas-fiscais.service';
import {
  ListagemConsultasDto,
  SituacaoFiscalDto,
} from '@accurit/core';
import { vi } from 'vitest';

const clienteId = 'cliente-123';

const listagemMock: ListagemConsultasDto = {
  itens: [
    {
      id: 'c1',
      produto: 'DIVIDA_ATIVA',
      servico: 'DEVEDOR',
      verbo: 'GET',
      ambiente: 'TRIAL',
      estado: 'SUCESSO',
      fonteConsulta: 'CPF_CNPJ',
      chaveConsultaMascarada: '123.456.789-00',
      statusCode: 200,
      criadoEm: '2026-01-15T10:30:00Z',
    },
  ],
  total: 1,
  pagina: 1,
  tamanho: 20,
};

const situacaoMock: SituacaoFiscalDto = {
  aviso: 'AVISO_CONSULTIVO: dados somente leitura.',
  servicos: [
    {
      servico: 'DEVEDOR',
      estado: 'SUCESSO',
      ambiente: 'TRIAL',
      fonteConsulta: 'CPF_CNPJ',
      chaveConsultaMascarada: '123.456.789-00',
      statusCode: 200,
      criadoEm: '2026-01-15T10:30:00Z',
    },
  ],
};

function criarService(mockGet: ReturnType<typeof vi.fn>): ConsultasFiscaisService {
  const injector = Injector.create({
    providers: [
      { provide: HttpClient, useValue: { get: mockGet } },
      ConsultasFiscaisService,
    ],
  });
  return injector.get(ConsultasFiscaisService);
}

describe('ConsultasFiscaisService', () => {
  it('deve ser criado', () => {
    const service = criarService(vi.fn().mockResolvedValue(listagemMock));
    expect(service).toBeTruthy();
  });

  it('deve iniciar com estado vazio', () => {
    const service = criarService(vi.fn());
    expect(service.itens()).toEqual([]);
    expect(service.total()).toBe(0);
    expect(service.pagina()).toBe(1);
    expect(service.tamanho()).toBe(20);
    expect(service.carregando()).toBe(false);
    expect(service.erroCarregamento()).toBeNull();
    expect(service.situacao()).toBeNull();
    expect(service.avisoConsultivo()).toBeNull();
    expect(service.erroSituacao()).toBeNull();
  });

  it('deve limpar o estado', () => {
    const service = criarService(vi.fn());
    service.limpar();
    expect(service.itens()).toEqual([]);
    expect(service.situacao()).toBeNull();
    expect(service.erroCarregamento()).toBeNull();
    expect(service.erroSituacao()).toBeNull();
  });
});
