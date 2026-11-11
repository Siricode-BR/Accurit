import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccSwitchComponent } from './acc-switch.component';

describe('AccSwitchComponent', () => {
  let fixture: ComponentFixture<AccSwitchComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccSwitchComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccSwitchComponent);
    fixture.componentRef.setInput('label', 'Rotina habilitada');
    fixture.detectChanges();
  });

  it('usa role switch e CVA booleano', () => {
    const valores: boolean[] = [];
    fixture.componentInstance.registerOnChange((valor) => valores.push(valor));
    const input = fixture.nativeElement.querySelector('input') as HTMLInputElement;
    expect(input.getAttribute('role')).toBe('switch');

    input.checked = true;
    input.dispatchEvent(new Event('change'));
    fixture.detectChanges();
    expect(valores).toEqual([true]);
    expect(input.getAttribute('aria-checked')).toBe('true');
  });
});
