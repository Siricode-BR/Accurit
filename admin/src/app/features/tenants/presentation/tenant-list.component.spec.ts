import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { MatDialog, MatDialogRef } from '@angular/material/dialog';
import { MatSlideToggleChange } from '@angular/material/slide-toggle';
import { MatSnackBar } from '@angular/material/snack-bar';
import { of } from 'rxjs';
import { Mock, vi } from 'vitest';

import { Tenant } from '../domain/tenant';
import type { TenantAlterado } from '../domain/tenant';
import { TenantsStore } from '../application/tenants.store';
import { TenantListComponent } from './tenant-list.component';
import type { ConfirmaProducaoDialog } from './confirma-producao.dialog';
import type { ConfirmacaoProducaoDados } from './confirma-producao.dialog';

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

function pagina(itens: Tenant[]) {
  return { itens, page: 1, pageSize: 20, total: itens.length };
}

function respostaTroca(versao: number): TenantAlterado {
  return {
    modoTrialSerpro: false,
    ambienteEfetivo: 'PRODUCAO',
    versaoAtual: versao,
    atualizadoEm: '2026-10-24T10:00:00Z',
  };
}

function changeFalso(tenant: Tenant): MatSlideToggleChange {
  return {
    checked: !tenant.modoTrialSerpro,
    source: { checked: false } as unknown as MatSlideToggleChange['source'],
  } as MatSlideToggleChange;
}

describe('TenantListComponent', () => {
  let http: HttpTestingController;
  let snackBar: { open: Mock };
  let dialogos: { open: Mock };

  async function montar(julgamento: boolean | 'sem-dialogo' = 'sem-dialogo') {
    await TestBed.configureTestingModule({
      imports: [TenantListComponent],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        provideRouter([]),
        { provide: MatSnackBar, useValue: { open: vi.fn() } },
        {
          provide: MatDialog,
          useValue:
            julgamento === 'sem-dialogo'
              ? { open: vi.fn() }
              : {
                  open: vi.fn(
                    (
                      _componente: typeof ConfirmaProducaoDialog,
                      _config: { data?: ConfirmacaoProducaoDados },
                    ): Partial<MatDialogRef<ConfirmaProducaoDialog, boolean>> => ({
                      afterClosed: () => of(julgamento),
                    }),
                  ),
                },
        },
      ],
    }).compileComponents();

    http = TestBed.inject(HttpTestingController);
    snackBar = TestBed.inject(MatSnackBar) as unknown as { open: Mock };
    dialogos = TestBed.inject(MatDialog) as unknown as { open: Mock };
    const fixture = TestBed.createComponent(TenantListComponent);
    await fixture.whenStable();
    return { fixture };
  }

  function listar(): { flush(c: unknown): void } {
    return http.expectOne('/api/v1/admin/escritorios?page=1&pageSize=20') as unknown as {
      flush(c: unknown): void;
    };
  }

  const componenteDe = async (fixture: ComponentFixtureWrapper) => {
    return fixture.componentInstance as unknown as {
      store: TenantsStore;
      trocar(t: Tenant, e: MatSlideToggleChange): Promise<void>;
      recarregar(): Promise<void>;
    };
  };

  it('lista escritorios com badge, versao e data formatada', async () => {
    const { fixture } = await montar();
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 3), resumo(ID_B, 'beta', false, 4)]));
    await bombear();
    await fixture.whenStable();

    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('alfa');
    expect(texto).toContain('beta');
    expect(texto).toContain('TRIAL');
    expect(texto).toContain('PRODUÇÃO');
    expect(texto).toContain('v3');
    expect(texto).toContain('v4');
  });

  it('abre o modal antes de migrar e bitola o toggle sem enviar PATCH ao recusar', async () => {
    const { fixture } = await montar(false);
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 5)]));
    await fixture.whenStable();

    const componente = await componenteDe(fixture);
    const entidade = componente.store.tenants()[0];
    await componente.trocar(entidade, changeFalso(entidade));
    await fixture.whenStable();

    expect(dialogos.open).toHaveBeenCalledTimes(1);
    http.verify();
    expect(snackBar.open).not.toHaveBeenCalled();
    // Sem troca: a linha segue em trial com a mesma versão.
    expect(componente.store.tenants()[0].modoTrialSerpro).toBe(true);
    expect(componente.store.tenants()[0].versao).toBe(5);
  });

  it('migra para producao com confirmacao e atualiza badge e versao', async () => {
    const { fixture } = await montar(true);
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 5)]));
    await fixture.whenStable();

    const componente = await componenteDe(fixture);
    const entidade = componente.store.tenants()[0];
    const troca = componente.trocar(entidade, changeFalso(entidade));
    // Zoneless: a resolução do diálogo e a emissão do PATCH são microtasks.
    await bombear();
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    expect(patch.request.method).toBe('PATCH');
    expect(patch.request.body).toEqual({ modoTrialSerpro: false, versao: 5 });
    patch.flush(respostaTroca(6));
    await troca;
    await fixture.whenStable();

    expect(componente.store.tenants()[0].modoTrialSerpro).toBe(false);
    expect(componente.store.tenants()[0].versao).toBe(6);
    expect(snackBar.open).toHaveBeenCalledTimes(1);
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('PRODUÇÃO');
  });

  it('recarrega a lista ao receber 409 e avisa o gestor', async () => {
    const { fixture } = await montar(true);
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 5)]));
    await fixture.whenStable();

    const componente = await componenteDe(fixture);
    const entidade = componente.store.tenants()[0];
    const troca = componente.trocar(entidade, changeFalso(entidade));
    await bombear();
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    patch.flush({ error: { message: 'divergiu' } }, { status: 409, statusText: 'Conflict' });
    await bombear();
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 9)]));
    await troca;
    await fixture.whenStable();

    expect(componente.store.tenants()[0].versao).toBe(9);
    expect(componente.store.tenants()[0].modoTrialSerpro).toBe(true);
    const mensagem = snackBar.open.mock.calls[0]?.[0] ?? '';
    expect(String(mensagem)).toContain('recarregamos');
  });

  it('exibe a mensagem do backend ao ser bloqueado (400) revertendo o toggle', async () => {
    const { fixture } = await montar(true);
    listar().flush(pagina([resumo(ID_A, 'alfa', true, 5)]));
    await fixture.whenStable();

    const componente = await componenteDe(fixture);
    const entidade = componente.store.tenants()[0];
    const troca = componente.trocar(entidade, changeFalso(entidade));
    await bombear();
    const patch = http.expectOne(`/api/v1/admin/escritorios/${ID_A}/serpro/trial`);
    patch.flush(
      { error: { message: 'credencial INTEGRA_CONTADOR faltando' } },
      { status: 400, statusText: 'Bad Request' },
    );
    await troca;
    await fixture.whenStable();

    const mensagem = snackBar.open.mock.calls[0]?.[0] ?? '';
    expect(String(mensagem)).toContain('INTEGRA_CONTADOR');
    expect(componente.store.tenants()[0].modoTrialSerpro).toBe(true);
  });
});

type ComponentFixtureWrapper = {
  componentInstance: unknown;
  nativeElement: HTMLElement;
  whenStable(): Promise<boolean>;
};

/** Drena microtasks pendentes (zoneless: resolução do diálogo, requisições,
 * atualização de signals) antes das próximas interações de teste. */
async function bombear(): Promise<void> {
  for (let i = 0; i < 25; i += 1) {
    await Promise.resolve();
  }
}
