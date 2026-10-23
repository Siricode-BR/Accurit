import { provideRouter } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { WritableSignal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ActivatedRoute, convertToParamMap } from '@angular/router';
import { of, throwError } from 'rxjs';
import { Mock, vi } from 'vitest';

import { SerproConfiguracao, SerproService } from './serpro.service';
import { SerproPageComponent } from './serpro-page.component';

const ESCRITORIO = '00000000-0000-0000-0000-0000000000ab';

interface SerproHarness {
  config: () => SerproConfiguracao | null;
  confirmaProducao: WritableSignal<boolean>;
  alternar(modoTrial: boolean): void;
}

function configuracao(versao: number, modoTrial: boolean): SerproConfiguracao {
  return {
    escritorioId: ESCRITORIO,
    modoTrialSerpro: modoTrial,
    ambienteEfetivo: modoTrial ? 'TRIAL' : 'PRODUCAO',
    versao,
    atualizadoEm: '2026-10-20T12:00:00Z',
  };
}

function erroHttp(
  status: number,
  mensagem: string,
): HttpErrorResponse {
  return new HttpErrorResponse({ status, error: { error: { message: mensagem } } });
}

async function montar(servico: {
  obterConfiguracao: Mock;
  alterarModoTrial: Mock;
}) {
  await TestBed.configureTestingModule({
    imports: [SerproPageComponent],
    providers: [
      provideRouter([]),
      {
        provide: SerproService,
        useValue: {
          obterConfiguracao: servico.obterConfiguracao,
          alterarModoTrial: servico.alterarModoTrial,
        },
      },
      {
        provide: ActivatedRoute,
        useValue: { snapshot: { paramMap: convertToParamMap({ escritorioId: ESCRITORIO }) } },
      },
      { provide: MatSnackBar, useValue: { open: vi.fn() } },
    ],
  }).compileComponents();

  const fixture = TestBed.createComponent(SerproPageComponent);
  const snackBar = (TestBed.inject(MatSnackBar) as unknown as { open: Mock }).open;

  return {
    fixture,
    snackBar,
    pagina: fixture.componentInstance as unknown as SerproHarness,
  };
}

describe('SerproPageComponent', () => {
  it('carrega e exibe o ambiente TRIAL do escritorio', async () => {
    const obterConfiguracao = vi.fn(() => of(configuracao(1, true)));
    const alterarModoTrial = vi.fn(() => of({}));
    const { fixture, pagina } = await montar({ obterConfiguracao, alterarModoTrial });
    await fixture.whenStable();

    expect(obterConfiguracao).toHaveBeenCalledWith(ESCRITORIO);
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('TRIAL');
    expect(pagina.config()?.versao).toBe(1);
  });

  it('pede confirmacao antes de migrar para producao', async () => {
    const obterConfiguracao = vi.fn(() => of(configuracao(1, true)));
    const alterarModoTrial = vi.fn(() => of({}));
    const { fixture, pagina } = await montar({ obterConfiguracao, alterarModoTrial });
    await fixture.whenStable();

    pagina.alternar(false);
    await fixture.whenStable();

    expect(pagina.confirmaProducao()).toBe(true);
    expect(alterarModoTrial).not.toHaveBeenCalled();
  });

  it('migra para producao com versao otimista e atualiza o estado', async () => {
    const obterConfiguracao = vi.fn(() => of(configuracao(1, true)));
    const alterarModoTrial = vi.fn(() =>
      of({
        modoTrialSerpro: false,
        ambienteEfetivo: 'PRODUCAO',
        versaoAtual: 2,
        atualizadoEm: '2026-10-20T12:00:01Z',
      }),
    );
    const { fixture, snackBar, pagina } = await montar({ obterConfiguracao, alterarModoTrial });
    await fixture.whenStable();

    pagina.confirmaProducao.set(true);
    pagina.alternar(false);
    await fixture.whenStable();

    expect(alterarModoTrial).toHaveBeenCalledWith({
      escritorioId: ESCRITORIO,
      modoTrialSerpro: false,
      versao: 1,
    });
    expect(pagina.config()?.modoTrialSerpro).toBe(false);
    expect(pagina.config()?.versao).toBe(2);
    expect(snackBar).toHaveBeenCalled();
  });

  it('recarrega configuracao quando a versao divergiu (409)', async () => {
    const respostas = [of(configuracao(3, true)), of(configuracao(5, true))];
    const obterConfiguracao = vi.fn(() => respostas.shift() ?? of(configuracao(3, true)));
    const alterarModoTrial = vi.fn(() =>
      throwError(
        () => erroHttp(409, 'versão do escritório divergiu; recarregue e tente novamente'),
      ),
    );
    const { fixture, pagina } = await montar({ obterConfiguracao, alterarModoTrial });
    await fixture.whenStable();

    pagina.confirmaProducao.set(true);
    pagina.alternar(false);
    await fixture.whenStable();

    expect(alterarModoTrial).toHaveBeenCalledWith({
      escritorioId: ESCRITORIO,
      modoTrialSerpro: false,
      versao: 3,
    });
    expect(obterConfiguracao).toHaveBeenCalledTimes(2);
    expect(pagina.config()?.versao).toBe(5);
  });

  it('mostra mensagem do backend quando a migracao e bloqueada (400)', async () => {
    const obterConfiguracao = vi.fn(() => of(configuracao(1, true)));
    const alterarModoTrial = vi.fn(() =>
      throwError(
        () =>
          erroHttp(
            400,
            'credencial INTEGRA_CONTADOR de produção não configurada; cadastre-a antes de desativar o modo trial',
          ),
      ),
    );
    const { fixture, snackBar, pagina } = await montar({ obterConfiguracao, alterarModoTrial });
    await fixture.whenStable();

    pagina.confirmaProducao.set(true);
    pagina.alternar(false);
    await fixture.whenStable();

    expect(alterarModoTrial).toHaveBeenCalledWith({
      escritorioId: ESCRITORIO,
      modoTrialSerpro: false,
      versao: 1,
    });
    expect(obterConfiguracao).toHaveBeenCalledTimes(1);
    expect(snackBar.mock.calls[0]?.[0]).toContain('INTEGRA_CONTADOR');
  });
});
