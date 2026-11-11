import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccDateFieldComponent } from './acc-date-field.component';

describe('AccDateFieldComponent', () => {
  let fixture: ComponentFixture<AccDateFieldComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccDateFieldComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccDateFieldComponent);
    fixture.componentRef.setInput('label', 'Vencimento');
    fixture.detectChanges();
  });

  it('usa input type=date e funciona como CVA', () => {
    const valores: string[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    fixture.componentInstance.writeValue('2026-11-30');
    fixture.detectChanges();

    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    expect(input.type).toBe('date');
    expect(input.value).toBe('2026-11-30');

    input.value = '2026-12-01';
    input.dispatchEvent(new Event('input'));
    expect(valores).toEqual(['2026-12-01']);
  });
});
