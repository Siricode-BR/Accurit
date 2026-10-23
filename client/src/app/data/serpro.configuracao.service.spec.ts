import { provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { SerproConfiguracaoService } from './serpro.configuracao.service';

describe('SerproConfiguracaoService', () => {
  let servico: SerproConfiguracaoService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(withInterceptorsFromDi()), provideHttpClientTesting()],
    });
    servico = TestBed.inject(SerproConfiguracaoService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('consulta /serpro/configuracao', () => {
    const esperado = {
      escritorioId: 'esc',
      modoTrialSerpro: true,
      ambienteEfetivo: 'TRIAL',
      versao: 1,
      atualizadoEm: '2026-10-20T12:00:00Z',
    };

    servico.obterConfiguracao().subscribe((r) => expect(r).toEqual(esperado));

    http
      .expectOne('/api/v1/serpro/configuracao')
      .flush(esperado, { status: 200, statusText: 'OK' });
  });
});
