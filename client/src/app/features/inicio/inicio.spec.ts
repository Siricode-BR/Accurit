import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { AuthService } from '../../core/auth/auth.service';
import { SaudeService } from '../../data/saude.service';
import { Inicio } from './inicio';

describe('Inicio', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Inicio],
      providers: [
        provideRouter([]),
        {
          provide: SaudeService,
          useValue: {
            verificar: () =>
              of({ status: 'ok', service: 'accurit-backend', db: 'ok' }),
          },
        },
        {
          provide: AuthService,
          useValue: { logout: () => undefined },
        },
      ],
    }).compileComponents();
  });

  it('should create the inicio page', () => {
    const fixture = TestBed.createComponent(Inicio);
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('should show the available status', async () => {
    const fixture = TestBed.createComponent(Inicio);
    await fixture.whenStable();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.textContent).toContain('Plataforma disponível');
  });

  it('should render the logout action', async () => {
    const fixture = TestBed.createComponent(Inicio);
    await fixture.whenStable();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.sair')?.textContent).toContain('Sair');
  });
});
