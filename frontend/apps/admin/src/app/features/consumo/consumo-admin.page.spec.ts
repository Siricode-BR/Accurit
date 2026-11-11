import { TestBed } from '@angular/core/testing';
import { signal } from '@angular/core';
import { ConsumoAdminPage } from './consumo-admin.page';
import { ConsumoAdminService } from '../../data/consumo-admin.service';
import { TenantService } from '../../data/tenant.service';

const CONSUMO: any = {
  inicio: '2026-11-01T00:00:00Z',
  fim: '2026-11-30T00:00:00Z',
  totalChamadas: 42,
  totalErros: 2,
  taxaErro: 0.0476,
  duracaoMediaMs: 120,
  escritorios: [
    {
      escritorioId: 'esc1',
      nome: 'Escritório A',
      cnpj: '11222333000181',
      planoNome: 'Essencial',
      chamadasLimite: 100,
      chamadasUsadas: 42,
      chamadasSucesso: 40,
      chamadasErro: 2,
      taxaErro: 0.0476,
      duracaoMediaMs: 120,
    },
  ],
  ranking: [{ escritorioId: 'esc1', nome: 'Escritório A', chamadas: 42 }],
  serie: [{ dia: '2026-11-10', chamadas: 42, erros: 2, duracaoMediaMs: 120 }],
};

function servicoFalso() {
  return {
    consumo: vi.fn().mockResolvedValue(CONSUMO),
    exportarCsv: vi.fn().mockResolvedValue(new Blob(['a,b'], { type: 'text/csv' })),
    planos: vi.fn().mockResolvedValue([]),
    criarPlano: vi.fn(),
    atribuirPlano: vi.fn(),
  };
}

function tenantFalso() {
  return {
    tenants: signal([]),
    listar: vi.fn(),
  };
}

describe('ConsumoAdminPage', () => {
  let servico: ReturnType<typeof servicoFalso>;
  let tenants: ReturnType<typeof tenantFalso>;

  beforeEach(async () => {
    servico = servicoFalso();
    tenants = tenantFalso();
    Object.defineProperty(URL, 'createObjectURL', {
      configurable: true,
      value: vi.fn().mockReturnValue('blob:consumo'),
    });
    Object.defineProperty(URL, 'revokeObjectURL', {
      configurable: true,
      value: vi.fn(),
    });

    await TestBed.configureTestingModule({
      imports: [ConsumoAdminPage],
      providers: [
        { provide: ConsumoAdminService, useValue: servico },
        { provide: TenantService, useValue: tenants },
      ],
    }).compileComponents();
  });

  it('carrega consumo e lista os escritórios na carga inicial', async () => {
    const fixture = TestBed.createComponent(ConsumoAdminPage);
    fixture.detectChanges();
    await fixture.whenStable();
    await new Promise((resolve) => setTimeout(resolve, 0));
    fixture.detectChanges();

    expect(servico.consumo).toHaveBeenCalledWith({});
    expect(servico.planos).toHaveBeenCalled();
    expect(tenants.listar).toHaveBeenCalledWith(1, 100);

    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(texto).toContain('Escritório A');
    expect(texto).toContain('11.222.333/0001-81');
    expect(texto).toContain('42 / 100');
  });

  it('aplica filtros ao recarregar o consumo', async () => {
    const fixture = TestBed.createComponent(ConsumoAdminPage);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    componente.filtros = { produto: 'DIVIDA_ATIVA', ambiente: 'TRIAL' };
    await componente.carregarConsumo();

    expect(servico.consumo).toHaveBeenLastCalledWith({
      produto: 'DIVIDA_ATIVA',
      ambiente: 'TRIAL',
    });
  });

  it('exporta CSV com os filtros ativos', async () => {
    const fixture = TestBed.createComponent(ConsumoAdminPage);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    componente.filtros = { escritorioId: 'esc1' };
    await componente.baixarCsv();

    expect(servico.exportarCsv).toHaveBeenCalledWith({ escritorioId: 'esc1' });
    expect(URL.createObjectURL).toHaveBeenCalled();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:consumo');
  });

  it('mostra erro quando a exportação falha', async () => {
    servico.exportarCsv.mockRejectedValueOnce(new Error('falhou'));
    const fixture = TestBed.createComponent(ConsumoAdminPage);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    await componente.baixarCsv();
    expect(componente.erro()).toBeTruthy();
  });
});
