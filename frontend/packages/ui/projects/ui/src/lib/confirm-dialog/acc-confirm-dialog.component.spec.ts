import { ComponentFixture, TestBed } from '@angular/core/testing';
import { AccConfirmDialogComponent } from './acc-confirm-dialog.component';

describe('AccConfirmDialogComponent', () => {
  let fixture: ComponentFixture<AccConfirmDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AccConfirmDialogComponent],
    }).compileComponents();
    fixture = TestBed.createComponent(AccConfirmDialogComponent);
    fixture.componentRef.setInput('titulo', 'Desativar usuário');
    fixture.componentRef.setInput('mensagem', 'As sessões serão revogadas.');
    fixture.detectChanges();
  });

  it('renderiza mensagem em um diálogo modal', () => {
    const dialogo = fixture.nativeElement.querySelector('[role=dialog]') as HTMLElement;
    expect(dialogo).toBeTruthy();
    expect(fixture.nativeElement.textContent).toContain('Desativar usuário');
    expect(fixture.nativeElement.textContent).toContain('As sessões serão revogadas.');
  });

  it('emite confirmar e cancelado', () => {
    const confirmacoes: number[] = [];
    const cancelamentos: number[] = [];
    fixture.componentInstance.confirmar.subscribe(() => confirmacoes.push(1));
    fixture.componentInstance.cancelado.subscribe(() => cancelamentos.push(1));

    const botoes = fixture.nativeElement.querySelectorAll('button') as NodeListOf<HTMLButtonElement>;
    const confirmar = Array.from(botoes).find((botao) =>
      botao.textContent?.includes('Confirmar'),
    ) as HTMLButtonElement;
    confirmar.click();

    const cancelar = Array.from(botoes).find((botao) =>
      botao.textContent?.includes('Cancelar'),
    ) as HTMLButtonElement;
    cancelar.click();

    expect(confirmacoes.length).toBe(1);
    expect(cancelamentos.length).toBe(1);
  });
});
