import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccTextareaComponent } from './acc-textarea.component';

describe('AccTextareaComponent', () => {
  let fixture: ComponentFixture<AccTextareaComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccTextareaComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccTextareaComponent);
    fixture.componentRef.setInput('label', 'Observações');
    fixture.detectChanges();
  });

  it('associa label ao textarea e aplica rows', () => {
    const textarea = fixture.nativeElement.querySelector('textarea') as HTMLTextAreaElement;
    const label = fixture.nativeElement.querySelector('label') as HTMLLabelElement;
    expect(label.htmlFor).toBe(textarea.id);
    expect(textarea.rows).toBe(4);
  });

  it('funciona como ControlValueAccessor', () => {
    const valores: string[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    fixture.componentInstance.writeValue('texto inicial');
    fixture.detectChanges();

    const textarea = fixture.nativeElement.querySelector('textarea') as HTMLTextAreaElement;
    expect(textarea.value).toBe('texto inicial');
    textarea.value = 'linha 1\nlinha 2';
    textarea.dispatchEvent(new Event('input'));
    expect(valores).toEqual(['linha 1\nlinha 2']);
  });

  it('mostra erro acessível', () => {
    fixture.componentRef.setInput('erro', 'Obrigatório');
    fixture.detectChanges();
    const textarea = fixture.nativeElement.querySelector('textarea') as HTMLTextAreaElement;
    expect(textarea.getAttribute('aria-invalid')).toBe('true');
    expect(fixture.nativeElement.textContent).toContain('Obrigatório');
  });
});
