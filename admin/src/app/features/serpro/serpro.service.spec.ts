import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { AlterarModoTrialPedido, SerproService } from './serpro.service';

const ESCRITORIO = '00000000-0000-0000-0000-0000000000aa';

describe('SerproService', () => {
  let service: SerproService;
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    service = TestBed.inject(SerproService);
    http = TestBed.inject(HttpTestingController);
  });

  it('busca configuracao do escritorio na rota admin', () => {
    const esperado = {
      escritorioId: ESCRITORIO,
      modoTrialSerpro: true,
      ambienteEfetivo: 'TRIAL',
      versao: 1,
      atualizadoEm: '2026-10-20T12:00:00Z',
    };

    service.obterConfiguracao(ESCRITORIO).subscribe((resposta) => {
      expect(resposta).toEqual(esperado);
    });

    const requisicao = http.expectOne('/api/v1/admin/serpro/configuracao?escritorioId=' + ESCRITORIO);
    expect(requisicao.request.method).toBe('GET');
    requisicao.flush(esperado);

    http.verify();
  });

  it('envia versao otimista no PATCH de troca de modo', () => {
    const pedido: AlterarModoTrialPedido = {
      escritorioId: ESCRITORIO,
      modoTrialSerpro: false,
      versao: 1,
    };
    const resposta = {
      modoTrialSerpro: false,
      ambienteEfetivo: 'PRODUCAO',
      versaoAtual: 2,
      atualizadoEm: '2026-10-20T12:00:01Z',
    };

    service.alterarModoTrial(pedido).subscribe((corpo) => {
      expect(corpo).toEqual(resposta);
    });

    const requisicao = http.expectOne('/api/v1/admin/serpro/trial');
    expect(requisicao.request.method).toBe('PATCH');
    expect(requisicao.request.body).toEqual(pedido);
    requisicao.flush(resposta);

    http.verify();
  });
});
