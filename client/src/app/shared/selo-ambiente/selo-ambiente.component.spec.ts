import { provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { API_BASE_URL } from '../../core/api.config';
import { SeloAmbienteComponent } from './selo-ambiente.component';

describe('SeloAmbienteComponent', () => {
  let http: HttpTestingController;

  async function montar() {
    await TestBed.configureTestingModule({
      imports: [SeloAmbienteComponent],
      providers: [
        provideHttpClient(withInterceptorsFromDi()),
        provideHttpClientTesting(),
        { provide: API_BASE_URL, useValue: '/api/v1' },
      ],
    }).compileComponents();

    http = TestBed.inject(HttpTestingController);
    const fixture = TestBed.createComponent(SeloAmbienteComponent);
    await fixture.whenStable();
    return fixture;
  }

  function responder(ambiente: string, instante: string): void {
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush(
        { ambienteEfetivo: ambiente, atualizadoEm: instante },
        { status: 200, statusText: 'OK' },
      );
  }

  it('exibe o selo TRIAL com classe do ambiente', async () => {
    const fixture = await montar();
    responder('TRIAL', '2026-10-24T09:00:00Z');
    await fixture.whenStable();

    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('SERPRO: Trial');
    const selo = (fixture.nativeElement as HTMLElement).querySelector('.selo');
    expect(selo?.className).toContain('selo--trial');
  });

  it('exibe o selo PRODUCAO quando ambiente efetivo for de producao', async () => {
    const fixture = await montar();
    responder('PRODUCAO', '2026-10-24T10:00:00Z');
    await fixture.whenStable();

    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('SERPRO: Produção');
    const selo = (fixture.nativeElement as HTMLElement).querySelector('.selo');
    expect(selo?.className).toContain('selo--producao');
  });

  it('omite o selo quando a leitura do ambiente falha', async () => {
    const fixture = await montar();
    http
      .expectOne('/api/v1/serpro/ambiente')
      .flush({ error: { message: 'boom' } }, { status: 503, statusText: 'Unavailable' });
    await fixture.whenStable();

    expect((fixture.nativeElement as HTMLElement).querySelector('.selo')).toBeNull();
  });
});
