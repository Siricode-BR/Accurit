import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccPaginationComponent } from './acc-pagination.component';

describe('AccPaginationComponent', () => {
  let fixture: ComponentFixture<AccPaginationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [AccPaginationComponent] }).compileComponents();
    fixture = TestBed.createComponent(AccPaginationComponent);
    fixture.componentRef.setInput('pagina', 2);
    fixture.componentRef.setInput('tamanho', 20);
    fixture.componentRef.setInput('total', 45);
    fixture.detectChanges();
  });

  it('mostra página e total de páginas', () => {
    expect(fixture.nativeElement.textContent).toContain('Página 2 de 3');
    expect(fixture.nativeElement.textContent).toContain('45 registros');
  });

  it('emite paginaChange ao navegar', () => {
    const paginas: number[] = [];
    fixture.componentInstance.paginaChange.subscribe((pagina) => paginas.push(pagina));

    const botoes = fixture.nativeElement.querySelectorAll('button') as NodeListOf<HTMLButtonElement>;
    botoes[0].click();
    botoes[1].click();
    expect(paginas).toEqual([1, 3]);
  });

  it('desabilita anterior na primeira página', () => {
    fixture.componentRef.setInput('pagina', 1);
    fixture.detectChanges();
    const anterior = fixture.nativeElement.querySelector('button') as HTMLButtonElement;
    expect(anterior.disabled).toBeTrue();
  });
});
