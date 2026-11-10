import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { App } from './app';
import { SessionStore } from './core/auth/session.store';

describe('App', () => {
  beforeEach(async () => {
    localStorage.clear();
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideRouter([]), provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
  });

  it('should create the app', () => {
    const fixture = TestBed.createComponent(App);
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('sem sessão renderiza só o router outlet, sem sidebar', async () => {
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    const el = fixture.nativeElement as HTMLElement;
    expect(el.querySelector('router-outlet')).toBeTruthy();
    expect(el.querySelector('acc-app-shell')).toBeNull();
  });

  it('com sessão renderiza o shell com a sidebar de navegação', async () => {
    TestBed.inject(SessionStore).setSession(
      { id: '1', nome: 'Ana', email: 'ana@escritorio.com', role: 'OPERADOR' },
      'token',
    );
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    const el = fixture.nativeElement as HTMLElement;
    expect(el.querySelector('acc-app-shell')).toBeTruthy();
    const links = [...el.querySelectorAll('acc-sidebar a')].map((a) => a.getAttribute('href'));
    expect(links).toEqual([
      '/dashboard',
      '/clientes',
      '/consultas/integra-contador',
      '/consultas/divida-ativa',
      '/trabalhos',
      '/consultas-lote',
      '/relatorios',
      '/consumo',
    ]);
  });
});
