import { TestBed } from '@angular/core/testing';
import { of } from 'rxjs';
import { SaudeService } from '../../data/saude.service';
import { Dashboard } from './dashboard';

describe('Dashboard', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Dashboard],
      providers: [
        {
          provide: SaudeService,
          useValue: {
            verificar: () =>
              of({ status: 'ok', service: 'accurit-backend', db: 'ok' }),
          },
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
});
