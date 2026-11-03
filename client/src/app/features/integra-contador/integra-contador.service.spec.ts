import { HttpErrorResponse, provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import {
  IntegraContadorService,
  categoriaErroIntegraContador,
  mensagemErroIntegraContador,
} from './integra-contador.service';

describe('IntegraContadorService', () => {
  let service: IntegraContadorService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(IntegraContadorService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('obtém do backend o catálogo e o ambiente efetivo', () => {
    service.consultarStatus().subscribe();
    const request = http.expectOne('/api/v1/serpro/integra-contador/status');
    expect(request.request.method).toBe('GET');
    request.flush({
      ambienteEfetivo: 'TRIAL',
      producaoHabilitada: false,
      assinaturaHabilitada: false,
      motivoBloqueioProducao: 'pendente',
      motivoBloqueioAssinatura: 'pendente',
      servicos: [],
    });
  });

  it('consulta procurações sem permitir que o cliente determine o autor', () => {
    service.consultarProcuracoes('529.982.247-25').subscribe();
    const request = http.expectOne('/api/v1/serpro/integra-contador/procuracoes/consultar');
    expect(request.request.method).toBe('POST');
    expect(request.request.body).toEqual({ documentoContribuinte: '529.982.247-25' });
    request.flush({});
  });

  it('classifica separadamente autorização, transporte e rejeição de negócio', () => {
    expect(
      categoriaErroIntegraContador(
        new HttpErrorResponse({ status: 403, error: { error: { code: 'ACESSO_NEGADO' } } }),
      ),
    ).toBe('autorizacao');
    expect(categoriaErroIntegraContador(new HttpErrorResponse({ status: 504 }))).toBe('transporte');
    expect(
      categoriaErroIntegraContador(
        new HttpErrorResponse({
          status: 409,
          error: { error: { code: 'VERSAO_NAO_CONFIRMADA' } },
        }),
      ),
    ).toBe('negocio');
  });

  it('não orienta repetição automática quando o resultado é indeterminado', () => {
    expect(
      mensagemErroIntegraContador(
        new HttpErrorResponse({
          status: 504,
          error: { error: { code: 'ESTADO_INDETERMINADO' } },
        }),
      ),
    ).toContain('antes de qualquer repetição');
  });
});
