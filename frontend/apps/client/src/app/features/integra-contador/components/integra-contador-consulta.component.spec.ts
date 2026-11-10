import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { IntegraContadorConsultaComponent } from './integra-contador-consulta.component';

describe('IntegraContadorConsultaComponent', () => {
  let fixture: ComponentFixture<IntegraContadorConsultaComponent>;
  let el: HTMLElement;

  const selects = () => [...el.querySelectorAll('select')] as HTMLSelectElement[];
  const valores = (s: HTMLSelectElement) => [...s.options].map((o) => o.value);

  async function escolher(select: HTMLSelectElement, valor: string): Promise<void> {
    select.value = valor;
    select.dispatchEvent(new Event('change'));
    await fixture.whenStable();
  }

  beforeEach(async () => {
    localStorage.clear();
    await TestBed.configureTestingModule({
      imports: [IntegraContadorConsultaComponent],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    fixture = TestBed.createComponent(IntegraContadorConsultaComponent);
    el = fixture.nativeElement;
    await fixture.whenStable();
  });

  it('exibe "Consultar" como verbo padrão', () => {
    const [verbo] = selects();
    expect(verbo.value).toBe('Consultar');
    expect(verbo.options[verbo.selectedIndex].textContent?.trim()).toBe('Consultar');
  });

  it('permite trocar o verbo e repopula o sistema', async () => {
    const [verbo, sistema] = selects();
    expect(valores(sistema)).toEqual(['', 'PROCURACOES', 'PGDASD', 'CAIXAPOSTAL', 'PARCSN']);

    await escolher(verbo, 'Emitir');

    expect(selects()[0].value).toBe('Emitir');
    expect(valores(selects()[1])).toEqual(['', 'SITFIS', 'PGDASD', 'PGMEI', 'DCTFWEB']);
  });

  it('preenche serviço e versão conforme o sistema escolhido', async () => {
    await escolher(selects()[1], 'PROCURACOES');

    const [, , servico, versao] = selects();
    expect(servico.value).toBe('OBTERPROCURACAO41');
    expect(versao.value).toBe('1');
  });

  it('mantém versão desabilitada quando o serviço não usa versão', async () => {
    await escolher(selects()[1], 'PGDASD');

    const [, , servico, versao] = selects();
    expect(servico.value).toBe('CONSDECLARACAO13');
    expect(versao.disabled).toBe(true);
  });

  it('limpa sistema, serviço e versão ao trocar o verbo', async () => {
    await escolher(selects()[1], 'PROCURACOES');
    await escolher(selects()[0], 'Declarar');

    const [, sistema, servico, versao] = selects();
    expect(valores(sistema)).toEqual(['']);
    expect(sistema.value).toBe('');
    expect(servico.value).toBe('');
    expect(versao.disabled).toBe(true);
  });

  it('documento bloqueado não pode ser alterado', async () => {
    fixture.componentRef.setInput('documentoInicial', '12345678000195');
    fixture.componentRef.setInput('documentoBloqueado', true);
    await fixture.whenStable();

    const input = el.querySelector('acc-document-field input') as HTMLInputElement;
    expect(input.readOnly).toBe(true);
    expect(input.value).toBe('12.345.678/0001-95');

    input.value = '99999999999';
    input.dispatchEvent(new Event('input'));
    await fixture.whenStable();

    expect(input.value).toBe('12.345.678/0001-95');
  });

  it('documento livre é editável e recebe máscara', async () => {
    const input = el.querySelector('acc-document-field input') as HTMLInputElement;
    expect(input.readOnly).toBe(false);

    input.value = '12345678909';
    input.dispatchEvent(new Event('input'));
    await fixture.whenStable();

    expect(input.value).toBe('123.456.789-09');
  });
});
