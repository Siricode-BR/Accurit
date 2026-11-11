import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccInputComponent } from './acc-input.component';

describe('AccInputComponent', () => {
  let fixture: ComponentFixture<AccInputComponent>;
  let componente: AccInputComponent;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccInputComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccInputComponent);
    componente = fixture.componentInstance;
    fixture.componentRef.setInput('label', 'E-mail');
    fixture.detectChanges();
  });

  it('associa label e hint ao input', () => {
    fixture.componentRef.setInput('hint', 'Use o e-mail corporativo');
    fixture.detectChanges();
    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    const label = fixture.nativeElement.querySelector('label') as HTMLLabelElement;
    expect(label.textContent).toContain('E-mail');
    expect(label.htmlFor).toBe(input.id);
    expect(input.getAttribute('aria-describedby')).toBe(`${input.id}-hint`);
  });

  it('funciona como ControlValueAccessor', () => {
    const mudancas: string[] = [];
    componente.registerOnChange((valor) => mudancas.push(valor));
    componente.writeValue('inicial');
    fixture.detectChanges();

    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    expect(input.value).toBe('inicial');

    input.value = 'novo@teste.test';
    input.dispatchEvent(new Event('input'));
    expect(mudancas).toEqual(['novo@teste.test']);
  });

  it('exibe erro com aria-invalid e descrição', () => {
    fixture.componentRef.setInput('erro', 'E-mail inválido');
    fixture.detectChanges();
    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(input.getAttribute('aria-describedby')).toBe(`${input.id}-erro`);
    expect(fixture.nativeElement.querySelector('.acc-input__erro').textContent).toContain(
      'E-mail inválido',
    );
  });

  it('alterna visibilidade da senha', () => {
    fixture.componentRef.setInput('tipo', 'password');
    fixture.detectChanges();
    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    (fixture.nativeElement.querySelector('.acc-input__toggle') as HTMLButtonElement).click();
    fixture.detectChanges();
    expect(input.type).toBe('text');
  });

  it('respeita disabled do formulário', () => {
    componente.setDisabledState(true);
    fixture.detectChanges();
    expect(
      (fixture.nativeElement.querySelector('input') as HTMLInputElement).disabled,
    ).toBeTrue();
  });
});
