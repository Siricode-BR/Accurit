import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { AccFileFieldComponent } from './acc-file-field.component';

describe('AccFileFieldComponent', () => {
  let fixture: ComponentFixture<AccFileFieldComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccFileFieldComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccFileFieldComponent);
    fixture.componentRef.setInput('label', 'Planilha');
    fixture.detectChanges();
  });

  it('é o único wrapper permitido para input file', () => {
    const input = fixture.debugElement.query(By.css('input[type=file]'));
    expect(input).toBeTruthy();
  });

  it('aceita arquivo via writeValue e limpa', () => {
    const arquivo = new File(['conteudo'], 'clientes.csv', { type: 'text/csv' });
    fixture.componentInstance.writeValue(arquivo);
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('clientes.csv');

    (fixture.nativeElement.querySelector('.acc-file__limpar') as HTMLButtonElement).click();
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('Nenhum arquivo selecionado');
  });
});
