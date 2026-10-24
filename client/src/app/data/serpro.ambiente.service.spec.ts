import { provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { firstValueFrom } from 'rxjs';

import { API_BASE_URL } from '../core/api.config';
import { Usuario } from '../core/auth/auth.models';
import { SessionStore } from '../core/auth/session.store';
import { SerproAmbienteService } from './serpro.ambiente.service';

const usuarioUm: Usuario = {
  id: '1',
  email: 'um@accurit.test',
  papel: 'CLIENTE',
  escritorioId: 'esc-1',
};

describe('SerproAmbienteService', () => {
  let servico: SerproAmbienteService;
  let http: HttpTestingController;
  let sessao: SessionStore;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [
        provideHttpClient(withInterceptorsFromDi()),
        provideHttpClientTesting(),
        { provide: API_BASE_URL, useValue: '/api/v1' },
      ],
    }).compileComponents();

    servico = TestBed.inject(SerproAmbienteService);
    sessao = TestBed.inject(SessionStore);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('consulta /serpro/ambiente e devolve ambiente e instante', async () => {
    const esperado = { ambienteEfetivo: 'TRIAL', atualizadoEm: '2026-10-24T09:00:00Z' };

    const leitura = firstValueFrom(servico.obter());
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush(esperado, { status: 200, statusText: 'OK' });

    expect(await leitura).toEqual(esperado);
    expect(servico.leitura()).toEqual({ estado: 'pronto', ambiente: esperado });
  });

  it('reaproveita a leitura em curso (single-flight)', async () => {
    const primeira = firstValueFrom(servico.obter());
    const segunda = servico.obter();

    const resposta = { ambienteEfetivo: 'PRODUCAO', atualizadoEm: '2026-10-24T10:00:00Z' };
    http.expectOne('/api/v1/serpro/ambiente').flush(resposta, { status: 200, statusText: 'OK' });
    expect(await primeira).toEqual(resposta);
    expect(await firstValueFrom(segunda)).toEqual(resposta);

    expect(servico.leitura().estado).toBe('pronto');
    // Uma única requisição foi emitida mesmo com dois observadores.
    http.verify();
  });

  it('reaproveita o cache da sessão em leituras seguintes', async () => {
    const inicial = { ambienteEfetivo: 'TRIAL', atualizadoEm: '2026-10-24T09:00:00Z' };
    const primeira = firstValueFrom(servico.obter());
    http.expectOne('/api/v1/serpro/ambiente').flush(inicial, { status: 200, statusText: 'OK' });
    expect(await primeira).toEqual(inicial);

    const cacheada = servico.obter();
    let lida: unknown = null;
    cacheada.subscribe((r) => (lida = r));
    expect(lida).toEqual(inicial);
  });

  it('invalida o cache quando a sessão muda de usuário', async () => {
    sessao.definirUsuario(usuarioUm);
    const primeira = firstValueFrom(servico.obter());
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush(
        { ambienteEfetivo: 'TRIAL', atualizadoEm: '2026-10-24T09:00:00Z' },
        { status: 200, statusText: 'OK' },
      );
    expect(await primeira).toEqual({
      ambienteEfetivo: 'TRIAL',
      atualizadoEm: '2026-10-24T09:00:00Z',
    });

    sessao.definirUsuario({ ...usuarioUm, id: '2', email: 'outro@accurit.test' });
    const segunda = firstValueFrom(servico.obter());
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush(
        { ambienteEfetivo: 'PRODUCAO', atualizadoEm: '2026-10-24T11:00:00Z' },
        { status: 200, statusText: 'OK' },
      );
    expect(servico.leitura().ambiente).toEqual({
      ambienteEfetivo: 'PRODUCAO',
      atualizadoEm: '2026-10-24T11:00:00Z',
    });
    await segunda;
  });

  it('marca indisponivel quando o backend falha', async () => {
    const leitura = firstValueFrom(servico.obter());
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush({ error: { message: 'boom' } }, { status: 503, statusText: 'Unavailable' });

    await expect(leitura).rejects.toThrow();
    expect(servico.leitura().estado).toBe('indisponivel');
    expect(servico.leitura().ambiente).toBeNull();
  });
});
