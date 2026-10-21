import { TestBed } from '@angular/core/testing';
import { of } from 'rxjs';
import { SaudeService } from '../../data/saude.service';
import { Inicio } from './inicio';

describe('Inicio', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Inicio],
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
});
