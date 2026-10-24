import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { TenantService } from './tenant.service';

const ESCRITORIO = '00000000-0000-0000-0000-0000000000aa';

describe('TenantService', () => {
  let service: TenantService;
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    service = TestBed.inject(TenantService);
    http = TestBed.inject(HttpTestingController);
  });

  it('lista escritorios paginados com page e pageSize', () => {
    const resposta = {
      itens: [
        {
          id: ESCRITORIO,
          slug: 'contamais',
          nome: 'ContaMais',
          modoTrialSerpro: true,
          versao: 3,
          atualizadoEm: '2026-10-24T09:00:00Z',
        },
      ],
      page: 2,
      pageSize: 50,
      total: 7,
    };

    service.listar(2, 50).subscribe((pagina) => {
      expect(pagina).toEqual(resposta);
    });

    const requisicao = http.expectOne('/api/v1/admin/escritorios?page=2&pageSize=50');
    expect(requisicao.request.method).toBe('GET');
    requisicao.flush(resposta);

    http.verify();
  });

  it('envia PATCH por path com corpo de modo e versao', () => {
    const resposta = {
      modoTrialSerpro: false,
      ambienteEfetivo: 'PRODUCAO',
      versaoAtual: 4,
      atualizadoEm: '2026-10-24T10:00:00Z',
    };

    service.alterarTrial(ESCRITORIO, { modoTrialSerpro: false, versao: 3 }).subscribe((corpo) => {
      expect(corpo).toEqual(resposta);
    });

    const requisicao = http.expectOne(
      `/api/v1/admin/escritorios/${ESCRITORIO}/serpro/trial`,
    );
    expect(requisicao.request.method).toBe('PATCH');
    expect(requisicao.request.body).toEqual({ modoTrialSerpro: false, versao: 3 });
    requisicao.flush(resposta);

    http.verify();
  });

  it('provisiona tenant via POST na rota admin', () => {
    const pedido = {
      slug: 'contamais',
      nome: 'ContaMais',
      emailTitular: 'titular@contamais.test',
      cpfTitular: '52998224725',
      senhaTitular: 'senha-forte-123',
    };
    const resposta = {
      escritorioId: ESCRITORIO,
      slug: 'contamais',
      titularId: '00000000-0000-0000-0000-0000000000bb',
      emailTitular: 'titular@contamais.test',
    };

    service.criar(pedido).subscribe((corpo) => {
      expect(corpo).toEqual(resposta);
    });

    const requisicao = http.expectOne('/api/v1/admin/escritorios');
    expect(requisicao.request.method).toBe('POST');
    expect(requisicao.request.body).toEqual(pedido);
    requisicao.flush(resposta);

    http.verify();
  });
});
