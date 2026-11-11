import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccFormFieldComponent } from './acc-form-field.component';

describe('AccFormFieldComponent', () => {
  let fixture: ComponentFixture<AccFormFieldComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccFormFieldComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccFormFieldComponent);
    fixture.componentRef.setInput('label', 'Período');
    fixture.detectChanges();
  });

  it('renderiza label obrigatório e hint', () => {
    fixture.componentRef.setInput('obrigatorio', true);
    fixture.componentRef.setInput('hint', 'Formato mensal');
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('Período');
    expect(fixture.nativeElement.textContent).toContain('*');
    expect(fixture.nativeElement.textContent).toContain('Formato mensal');
  });

  it('prioriza o erro sobre o hint', () => {
    fixture.componentRef.setInput('hint', 'Formato mensal');
    fixture.componentRef.setInput('erro', 'Competência inválida');
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('Competência inválida');
    expect(fixture.nativeElement.textContent).not.toContain('Formato mensal');
  });
});
