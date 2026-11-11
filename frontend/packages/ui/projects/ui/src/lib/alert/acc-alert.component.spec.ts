import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccAlertComponent } from './acc-alert.component';

describe('AccAlertComponent', () => {
  let fixture: ComponentFixture<AccAlertComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccAlertComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccAlertComponent);
  });

  it('usa role alert para erro e status para sucesso', () => {
    fixture.componentRef.setInput('variante', 'erro');
    fixture.detectChanges();
    expect(fixture.nativeElement.querySelector('[role=alert]')).toBeTruthy();

    fixture.componentRef.setInput('variante', 'sucesso');
    fixture.detectChanges();
    expect(fixture.nativeElement.querySelector('[role=status]')).toBeTruthy();
  });

  it('emite fechado quando descartável', () => {
    const fechados: number[] = [];
    fixture.componentInstance.fechado.subscribe(() => fechados.push(1));
    fixture.componentRef.setInput('descartavel', true);
    fixture.detectChanges();

    (fixture.nativeElement.querySelector('.acc-alert__fechar') as HTMLButtonElement).click();
    expect(fechados.length).toBe(1);
  });
});
