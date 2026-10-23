import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Observable, of, throwError } from 'rxjs';
import { AuthService } from '../../core/auth/auth.service';
import { SaudeService } from '../../data/saude.service';
import { SerproConfiguracao, SerproConfiguracaoService } from '../../data/serpro.configuracao.service';
import { Inicio } from './inicio';

type Ambiente = 'TRIAL' | 'PRODUCAO';

describe('Inicio', () => {
  const configuracao = (ambiente: Ambiente): SerproConfiguracao => ({
    escritorioId: 'esc',
    modoTrialSerpro: ambiente === 'TRIAL',
    ambienteEfetivo: ambiente,
    versao: 1,
    atualizadoEm: '2026-10-20T12:00:00Z',
  });

  async function montar(
    ambiente: () => Observable<SerproConfiguracao>,
  ) {
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
          provide: SerproConfiguracaoService,
          useValue: { obterConfiguracao: ambiente },
        },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Inicio);
    await fixture.whenStable();
    return fixture;
  }

  it('should create the inicio page', async () => {
    const fixture = await montar(() => of(configuracao('TRIAL')));
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('should show the available status', async () => {
    const fixture = await montar(() => of(configuracao('TRIAL')));
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.textContent).toContain('Plataforma disponível');
  });

  it('should render the logout action', async () => {
    const fixture = await montar(() => of(configuracao('TRIAL')));
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.sair')?.textContent).toContain('Sair');
  });

  it('exibe o selo do ambiente trial', async () => {
    const fixture = await montar(() => of(configuracao('TRIAL')));
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.selo-serpro')?.textContent).toContain('SERPRO: Trial');
  });

  it('exibe o selo do ambiente de producao', async () => {
    const fixture = await montar(() => of(configuracao('PRODUCAO')));
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.selo-serpro')?.textContent).toContain('SERPRO: Produção');
  });

  it('omite o selo quando a configuracao nao pode ser obtida', async () => {
    const fixture = await montar(() => throwError(() => new Error('500')));
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.selo-serpro')).toBeNull();
  });
});

