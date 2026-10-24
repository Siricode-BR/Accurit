import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { Tenant } from '../domain/tenant';
import { TenantsStore } from './tenants.store';

const ID_A = '00000000-0000-0000-0000-000000000001';
const ID_B = '00000000-0000-0000-0000-000000000002';

function resumo(id: string, slug: string, modoTrial: boolean, versao = 1): Tenant {
  return {
    id,
    slug,
    nome: `Escritorio ${slug}`,
    modoTrialSerpro: modoTrial,
    versao,
    atualizadoEm: '2026-10-24T09:00:00Z',
  };
}

function pagina(itens: Tenant[], tamPagina = 20) {
  return { itens, page: 1, pageSize: tamPagina, total: itens.length };
}

/** Drena microtasks pendentes (zoneless) antes de próximas interações
 * com o HttpTestingController. */
async function bombear(): Promise<void> {
  for (let i = 0; i < 25; i += 1) {
    await Promise.resolve();
  }
}

describe('TenantsStore', () => {
  let store: TenantsStore;
  let http: HttpTestingController;

  function listarEsperado(num = 1) {
    return http.expectOne(`/api/v1/admin/escritorios?page=${num}&pageSize=20`);
  }

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    store = TestBed.inject(TenantsStore);
    http = TestBed.inject(HttpTestingController);
  });

  it('carrega uma pagina com estado pronta', async () => {
    const carregar = store.carregar();
    listarEsperado(1).flush(pagina([resumo(ID_A, 'alfa', true)]));
    await carregar;

    expect(store.carregando()).toBe(false);
    expect(store.indisponivel()).toBe(false);
    expect(store.tenants().map((t) => t.slug)).toEqual(['alfa']);
    expect(store.total()).toBe(1);
    expect(store.paginaAtual()).toBe(1);
    expect(store.porPagina()).toBe(20);
  });

  it('marca indisponivel quando a listagem falha', async () => {
    const carregar = store.carregar();
    listarEsperado(1).error(new ProgressEvent('erro'));
    await carregar;

    expect(store.carregando()).toBe(false);
    expect(store.indisponivel()).toBe(true);
    expect(store.tenants()).toEqual([]);
  });

  it('atualiza a linha alternada pessimisticamente com versao e timestamp', async () => {
    const carregar = store.carregar();
    listarEsperado(1).flush(
      pagina([resumo(ID_A, 'alfa', true, 3), resumo(ID_B, 'beta', true, 1)]),
    );
    await carregar;

    const troca = store.alternarTrial(store.tenants()[0], false);
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    expect(patch.request.method).toBe('PATCH');
    expect(patch.request.body).toEqual({ modoTrialSerpro: false, versao: 3 });
    patch.flush({
      modoTrialSerpro: false,
      ambienteEfetivo: 'PRODUCAO',
      versaoAtual: 4,
      atualizadoEm: '2026-10-24T10:00:00Z',
    });
    expect(await troca).toEqual({ tipo: 'ok' });

    const atualizada = store.tenants().find((t) => t.id === ID_A);
    expect(atualizada?.modoTrialSerpro).toBe(false);
    expect(atualizada?.versao).toBe(4);
    expect(atualizada?.atualizadoEm).toBe('2026-10-24T10:00:00Z');
    // A outra linha permanece intacta.
    expect(store.tenants().find((t) => t.id === ID_B)?.modoTrialSerpro).toBe(true);
    expect(store.listarTrocas().size).toBe(0);
  });

  it('em 409 recarrega a pagina corrente e reporta conflito', async () => {
    const carregar = store.carregar();
    listarEsperado(1).flush(pagina([resumo(ID_A, 'alfa', true, 3)]));
    await carregar;

    const troca = store.alternarTrial(store.tenants()[0], false);
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    patch.flush({ error: { message: 'divergiu' } }, { status: 409, statusText: 'Conflict' });
    await bombear();
    listarEsperado(1).flush(pagina([resumo(ID_A, 'alfa', true, 7)]));
    expect(await troca).toEqual({ tipo: 'conflito' });

    expect(store.tenants()[0].versao).toBe(7);
    expect(store.listarTrocas().size).toBe(0);
  });

  it('mapeia bloqueio 400 com a mensagem do backend', async () => {
    const carregar = store.carregar();
    listarEsperado(1).flush(pagina([resumo(ID_A, 'alfa', true, 1)]));
    await carregar;

    const troca = store.alternarTrial(store.tenants()[0], false);
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    patch.flush(
      { error: { message: 'credencial INTEGRA_CONTADOR faltando' } },
      { status: 400, statusText: 'Bad Request' },
    );
    const resultado = await troca;

    expect(resultado).toEqual({
      tipo: 'bloqueado',
      mensagem: 'credencial INTEGRA_CONTADOR faltando',
    });
    expect(store.tenants()[0].modoTrialSerpro).toBe(true);
  });

  it('ignora trocas simultaneas para o mesmo escritorio', async () => {
    const carregar = store.carregar();
    listarEsperado(1).flush(pagina([resumo(ID_A, 'alfa', true, 1)]));
    await carregar;

    const a = store.alternarTrial(store.tenants()[0], false);
    const b = store.alternarTrial(store.tenants()[0], false);
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    patch.flush({
      modoTrialSerpro: false,
      ambienteEfetivo: 'PRODUCAO',
      versaoAtual: 2,
      atualizadoEm: '2026-10-24T10:00:00Z',
    });

    expect((await a).tipo).toBe('ok');
    expect((await b).tipo).toBe('ok');
    http.verify();
  });
});
