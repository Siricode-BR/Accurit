import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccCheckboxComponent } from './acc-checkbox.component';

describe('AccCheckboxComponent', () => {
  let fixture: ComponentFixture<AccCheckboxComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccCheckboxComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccCheckboxComponent);
    fixture.componentRef.setInput('label', 'Li e aceito');
    fixture.detectChanges();
  });

  it('associa label e funciona como CVA booleano', () => {
    const valores: boolean[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    fixture.componentInstance.writeValue(true);
    fixture.detectChanges();

    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    const label = fixture.nativeElement.querySelector('label') as HTMLLabelElement;
    expect(label.htmlFor).toBe(input.id);
    expect(input.checked).toBeTrue();

    input.checked = false;
    input.dispatchEvent(new Event('change'));
    expect(valores).toEqual([false]);
  });

  it('expõe hint e erro', () => {
    fixture.componentRef.setInput('hint', 'Você pode alterar depois');
    fixture.componentRef.setInput('erro', 'Campo obrigatório');
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('Campo obrigatório');
    expect(fixture.nativeElement.textContent).not.toContain('Você pode alterar depois');
  });
});
