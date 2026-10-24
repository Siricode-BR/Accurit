import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Observable, of, throwError } from 'rxjs';
import { AuthService } from '../../core/auth/auth.service';
import { SaudeService } from '../../data/saude.service';
import { AmbienteSerproInfo, SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { Inicio } from './inicio';

describe('Inicio', () => {
  function leitura(ambiente: 'TRIAL' | 'PRODUCAO'): AmbienteSerproInfo {
    return {
      ambienteEfetivo: ambiente,
      atualizadoEm: '2026-10-24T09:00:00Z',
    };
  }

  async function montar(ambiente: () => Observable<AmbienteSerproInfo>) {
    await TestBed.configureTestingModule({
      imports: [Inicio],
      providers: [
        provideRouter([]),
        {
          provide: SaudeService,
          useValue: {
            verificar: () => of({ status: 'ok', service: 'accurit-backend', db: 'ok' }),
          },
        },
        {
          provide: AuthService,
          useValue: { logout: () => undefined },
        },
        {
          provide: SerproAmbienteService,
          useValue: { obter: ambiente },
        },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Inicio);
    await fixture.whenStable();
    return fixture;
  }

  it('monta a pagina com o ambiente do portal', async () => {
    const fixture = await montar(() => of(leitura('TRIAL')));
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('mostra o estado da plataforma', async () => {
    const fixture = await montar(() => of(leitura('TRIAL')));
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('Plataforma disponível');
  });

  it('renderiza a acao de sair', async () => {
    const fixture = await montar(() => of(leitura('TRIAL')));
    const texto = ((fixture.nativeElement as HTMLElement).querySelector('.sair')?.textContent) ?? '';
    expect(texto).toContain('Sair');
  });

  it('exibe o selo do ambiente trial', async () => {
    const fixture = await montar(() => of(leitura('TRIAL')));
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('SERPRO: Trial');
  });

  it('exibe o selo do ambiente de producao', async () => {
    const fixture = await montar(() => of(leitura('PRODUCAO')));
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('SERPRO: Produção');
  });

  it('omite o selo quando a leitura do ambiente falha', async () => {
    const fixture = await montar(() => throwError(() => new Error('503')));
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).not.toContain('SERPRO');
  });
});
