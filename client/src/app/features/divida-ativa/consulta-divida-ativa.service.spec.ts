import { HttpErrorResponse, provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import {
  ConsultaDividaAtivaService,
  mensagemErroConsulta,
} from './consulta-divida-ativa.service';

describe('ConsultaDividaAtivaService', () => {
  let service: ConsultaDividaAtivaService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(ConsultaDividaAtivaService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('normaliza CPF/CNPJ e usa a rota autenticada de consulta', () => {
    service.consultarDevedor('529.982.247-25').subscribe();

    const request = http.expectOne('/api/v1/divida-ativa/devedores/52998224725');
    expect(request.request.method).toBe('GET');
    request.flush({
      ambiente: 'TRIAL',
      estado: 'SEM_PENDENCIAS',
      fonteConsulta: 'CPF_CNPJ',
      chaveMascarada: '***.***.247-25',
      horarioConsulta: '2026-11-02T12:00:00Z',
      tentativas: 1,
      aviso: 'Consulta informativa',
      inscricoes: [],
    });
  });

  it('normaliza o número da inscrição antes de consultar', () => {
    service.consultarInscricao('90 6 17 000964-19').subscribe();

    const request = http.expectOne('/api/v1/divida-ativa/inscricoes/9061700096419');
    expect(request.request.method).toBe('GET');
    request.flush({});
  });

  it('distingue produção não configurada e indisponibilidade externa', () => {
    expect(
      mensagemErroConsulta(
        new HttpErrorResponse({
          status: 503,
          error: { error: { code: 'SERPRO_PROD_NOT_CONFIGURED' } },
        }),
      ),
    ).toContain('produção ainda não está habilitada');

    expect(
      mensagemErroConsulta(
        new HttpErrorResponse({
          status: 503,
          error: { error: { code: 'SERPRO_UNAVAILABLE' } },
        }),
      ),
    ).toContain('serviço da SERPRO está indisponível');
  });
});
