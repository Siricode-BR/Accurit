import { provideHttpClient } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { API_BASE_URL } from '../../../core/api.config';
import { Login } from './login';

describe('Login', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Login],
      providers: [
        provideHttpClient(),
        provideRouter([]),
        { provide: API_BASE_URL, useValue: '/api/v1' },
      ],
    }).compileComponents();
  });

  it('should create the login page', () => {
    const fixture = TestBed.createComponent(Login);
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('should render the office login form', async () => {
    const fixture = TestBed.createComponent(Login);
    await fixture.whenStable();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('form')).not.toBeNull();
    expect(compiled.textContent).toContain('Painel do escritório');
    expect(compiled.querySelector('a')?.getAttribute('href')).toBe('/app/login');
  });
});
