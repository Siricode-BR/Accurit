import '@angular/compiler';
import { Injector, runInInjectionContext } from '@angular/core';
import { Router } from '@angular/router';
import { operadorGuard } from './operador.guards';
import { SessionStore } from './session.store';
import { vi } from 'vitest';

class RouterStub {
  createUrlTree = vi.fn().mockReturnValue({} as any);
  navigate = vi.fn().mockResolvedValue(true);
}

function criarGuard(sessionRole: 'OPERADOR' | 'CLIENTE' | null): {
  guard: typeof operadorGuard;
  router: RouterStub;
  injector: Injector;
} {
  const router = new RouterStub();
  const injector = Injector.create({
    providers: [
      SessionStore,
      { provide: Router, useValue: router },
    ],
  });

  const session = injector.get(SessionStore);
  if (sessionRole) {
    session.setSession(
      { role: sessionRole, tenantNome: 'Escritório' } as any,
      'token',
    );
  } else {
    session.clearSession();
  }

  return { guard: operadorGuard, router, injector };
}

describe('operadorGuard', () => {
  it('deve permitir acesso quando o usuário tem papel OPERADOR', () => {
    const { guard, router, injector } = criarGuard('OPERADOR');

    const resultado = runInInjectionContext(injector, () =>
      guard({} as any, { url: '/clientes/1/editar' } as any),
    );

    expect(resultado).toBe(true);
    expect(router.createUrlTree).not.toHaveBeenCalled();
  });

  it('deve redirecionar para /clientes quando o usuário tem papel CLIENTE', () => {
    const { guard, router, injector } = criarGuard('CLIENTE');

    const resultado = runInInjectionContext(injector, () =>
      guard({} as any, { url: '/clientes/1/editar' } as any),
    ) as any;

    expect(resultado).not.toBe(true);
    expect(router.createUrlTree).toHaveBeenCalledWith(['/clientes']);
  });

  it('deve redirecionar para /login quando não autenticado', () => {
    const { guard, router, injector } = criarGuard(null);

    const resultado = runInInjectionContext(injector, () =>
      guard({} as any, { url: '/clientes/1/editar' } as any),
    ) as any;

    expect(resultado).not.toBe(true);
    expect(router.createUrlTree).toHaveBeenCalledWith(
      ['/login'],
      expect.objectContaining({ queryParams: { returnUrl: '/clientes/1/editar' } }),
    );
  });
});
