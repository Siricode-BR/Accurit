import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccRadioGroupComponent } from './acc-radio-group.component';

describe('AccRadioGroupComponent', () => {
  let fixture: ComponentFixture<AccRadioGroupComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccRadioGroupComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccRadioGroupComponent);
    fixture.componentRef.setInput('label', 'Tipo');
    fixture.componentRef.setInput('options', [
      { value: 'PF', label: 'Pessoa física' },
      { value: 'PJ', label: 'Pessoa jurídica' },
    ]);
    fixture.detectChanges();
  });

  it('renderiza as opções e emite o valor escolhido', () => {
    const valores: string[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    const radios = fixture.nativeElement.querySelectorAll('input[type=radio]') as NodeListOf<HTMLInputElement>;
    expect(radios.length).toBe(2);

    radios[1].checked = true;
    radios[1].dispatchEvent(new Event('change'));
    expect(valores).toEqual(['PJ']);
  });

  it('reflete o valor do formulário', () => {
    fixture.componentInstance.writeValue('PF');
    fixture.detectChanges();
    const radios = fixture.nativeElement.querySelectorAll('input[type=radio]') as NodeListOf<HTMLInputElement>;
    expect(radios[0].checked).toBeTrue();
    expect(radios[1].checked).toBeFalse();
  });
});
