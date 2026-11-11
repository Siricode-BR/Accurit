import { TestBed } from '@angular/core/testing';
import { AdministradoresListComponent } from './administradores-list.component';
import { AdministradorService } from '../../data/administrador.service';

const ADMIN: any = {
  id: 'ad1',
  nome: 'Gestor Um',
  email: 'gestor@accurit.test',
  cpf: '11144477735',
  ativo: true,
  ultimoLoginEm: null,
  criadoEm: '2026-01-01T00:00:00Z',
};

function servicoFalso() {
  return {
    listar: vi.fn().mockResolvedValue({ itens: [ADMIN], total: 1, page: 1, pageSize: 20 }),
    criar: vi.fn().mockResolvedValue(ADMIN),
    atualizar: vi.fn().mockResolvedValue(ADMIN),
    alterarSenha: vi.fn().mockResolvedValue(ADMIN),
  };
}

describe('AdministradoresListComponent', () => {
  let servico: ReturnType<typeof servicoFalso>;

  beforeEach(async () => {
    servico = servicoFalso();
    await TestBed.configureTestingModule({
      imports: [AdministradoresListComponent],
      providers: [{ provide: AdministradorService, useValue: servico }],
    }).compileComponents();
  });

  it('lista administradores na carga inicial e permite buscar', async () => {
    const fixture = TestBed.createComponent(AdministradoresListComponent);
    fixture.detectChanges();
    await fixture.whenStable();
    await new Promise((resolve) => setTimeout(resolve, 0));
    fixture.detectChanges();
    const texto = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(servico.listar).toHaveBeenCalledWith(1, 20, '');
    expect(texto).toContain('Gestor Um');
    expect(texto).toContain('gestor@accurit.test');

    const componente = fixture.componentInstance as any;
    componente.busca.set('gestor');
    await componente.carregar(1);
    expect(servico.listar).toHaveBeenLastCalledWith(1, 20, 'gestor');
  });

  it('valida senha mínima de 8 caracteres no cadastro', async () => {
    const fixture = TestBed.createComponent(AdministradoresListComponent);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    componente.abrirCriar();
    componente.form.setValue({
      nome: 'Novo Gestor',
      email: 'novo@accurit.test',
      cpf: '11144477735',
      senha: '123',
    });
    expect(componente.form.invalid).toBe(true);

    componente.form.controls.senha.setValue('senha-forte-123');
    expect(componente.form.valid).toBe(true);
  });

  it('cadastra administrador com dados válidos', async () => {
    const fixture = TestBed.createComponent(AdministradoresListComponent);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    componente.abrirCriar();
    componente.form.setValue({
      nome: 'Novo Gestor',
      email: 'novo@accurit.test',
      cpf: '11144477735',
      senha: 'senha-forte-123',
    });
    await componente.salvar();

    expect(servico.criar).toHaveBeenCalledWith({
      nome: 'Novo Gestor',
      email: 'novo@accurit.test',
      cpf: '11144477735',
      senha: 'senha-forte-123',
    });
    expect(componente.modo()).toBeNull();
  });

  it('desativa somente após confirmação no diálogo', async () => {
    const fixture = TestBed.createComponent(AdministradoresListComponent);
    fixture.detectChanges();
    await fixture.whenStable();

    const componente = fixture.componentInstance as any;
    await componente.alternarAtivo(ADMIN);
    expect(componente.confirmacao()).toEqual(ADMIN);
    expect(servico.atualizar).not.toHaveBeenCalled();

    await componente.confirmarAlternancia();
    expect(servico.atualizar).toHaveBeenCalledWith('ad1', { ativo: false });
    expect(componente.confirmacao()).toBeNull();
  });
});
