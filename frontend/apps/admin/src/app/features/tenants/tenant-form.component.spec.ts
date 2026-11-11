import { TestBed } from '@angular/core/testing';
import { TenantFormComponent } from './tenant-form.component';
import { TenantService } from '../../data/tenant.service';

const CNPJ_VALIDO = '11.222.333/0001-81';
const CNPJ_INVALIDO = '11.222.333/0001-00';

function servicoFalso() {
  return {
    criar: vi.fn().mockResolvedValue({ escritorioId: 'esc1', nome: 'Escritório' }),
  };
}

describe('TenantFormComponent', () => {
  let servico: ReturnType<typeof servicoFalso>;

  beforeEach(async () => {
    servico = servicoFalso();
    await TestBed.configureTestingModule({
      imports: [TenantFormComponent],
      providers: [{ provide: TenantService, useValue: servico }],
    }).compileComponents();
  });

  it('rejeita CNPJ com dígitos verificadores inválidos', () => {
    const fixture = TestBed.createComponent(TenantFormComponent);
    const componente = fixture.componentInstance as any;

    componente.form.controls.cnpj.setValue(CNPJ_VALIDO);
    expect(componente.form.controls.cnpj.valid).toBe(true);

    componente.form.controls.cnpj.setValue(CNPJ_INVALIDO);
    expect(componente.form.controls.cnpj.hasError('cnpj')).toBe(true);
  });

  it('exige senhas iguais e senha mínima de 8 caracteres', () => {
    const fixture = TestBed.createComponent(TenantFormComponent);
    const componente = fixture.componentInstance as any;

    componente.form.controls.cnpj.setValue(CNPJ_VALIDO);
    componente.form.controls.nome.setValue('Escritório Teste');
    componente.form.controls.emailTitular.setValue('titular@teste.test');
    componente.form.controls.cpfTitular.setValue('111.444.777-35');
    componente.form.controls.senhaTitular.setValue('curta');
    componente.form.controls.senhaConfirmada.setValue('curta');
    expect(componente.form.controls.senhaTitular.valid).toBe(false);

    componente.form.controls.senhaTitular.setValue('senha-forte-123');
    componente.form.controls.senhaConfirmada.setValue('senha-diferente-1');
    expect(componente.form.hasError('senhasIguais')).toBe(true);

    componente.form.controls.senhaConfirmada.setValue('senha-forte-123');
    expect(componente.form.valid).toBe(true);
  });

  it('cria o escritório saneando CPF e normaliza e-mail', async () => {
    const fixture = TestBed.createComponent(TenantFormComponent);
    const componente = fixture.componentInstance as any;

    componente.form.setValue({
      cnpj: CNPJ_VALIDO,
      nome: 'Escritório Teste',
      emailTitular: 'Titular@Teste.Test',
      cpfTitular: '111.444.777-35',
      senhaTitular: 'senha-forte-123',
      senhaConfirmada: 'senha-forte-123',
    });
    await componente.enviar();

    expect(servico.criar).toHaveBeenCalledWith({
      cnpj: CNPJ_VALIDO,
      nome: 'Escritório Teste',
      emailTitular: 'titular@teste.test',
      cpfTitular: '11144477735',
      senhaTitular: 'senha-forte-123',
    });
  });

  it('não envia formulário inválido', async () => {
    const fixture = TestBed.createComponent(TenantFormComponent);
    const componente = fixture.componentInstance as any;

    componente.form.controls.cnpj.setValue(CNPJ_INVALIDO);
    await componente.enviar();

    expect(servico.criar).not.toHaveBeenCalled();
  });
});
