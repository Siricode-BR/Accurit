import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { AuthService } from '../../core/auth/auth.service';
import { SaudeService } from '../../data/saude.service';
import { Dashboard } from './dashboard';

describe('Dashboard', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Dashboard],
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

  it('should create the dashboard', () => {
    const fixture = TestBed.createComponent(Dashboard);
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('should show the operational status', async () => {
    const fixture = TestBed.createComponent(Dashboard);
    await fixture.whenStable();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.textContent).toContain('Serviços operacionais');
  });

  it('should render the logout action', async () => {
    const fixture = TestBed.createComponent(Dashboard);
    await fixture.whenStable();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.sair')?.textContent).toContain('Sair');
  });
});
