import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccMonthFieldComponent } from './acc-month-field.component';

describe('AccMonthFieldComponent', () => {
  let fixture: ComponentFixture<AccMonthFieldComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccMonthFieldComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccMonthFieldComponent);
    fixture.componentRef.setInput('label', 'Competência');
    fixture.detectChanges();
  });

  it('usa input type=month e funciona como CVA', () => {
    const valores: string[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    fixture.componentInstance.writeValue('2026-11');
    fixture.detectChanges();

    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    expect(input.type).toBe('month');
    expect(input.value).toBe('2026-11');

    input.value = '2026-12';
    input.dispatchEvent(new Event('input'));
    expect(valores).toEqual(['2026-12']);
  });
});
