import { Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { AccTableComponent } from './acc-table.component';
import { AccColunaTabela } from './acc-table.types';

interface LinhaTeste {
  id: string;
  nome: string;
  total: number;
}

@Component({
  standalone: true,
  imports: [AccTableComponent],
  template: `
    <acc-table
      [colunas]="colunas"
      [linhas]="linhas"
      [carregando]="carregando"
      [erro]="erro"
      [linhaClicavel]="true"
      vazio="Nada aqui"
      (linhaSelecionada)="selecionadas.push($event)"
      (tentarNovamente)="retentativas = retentativas + 1"
    />
  `,
})
class Hospedeiro {
  readonly colunas: AccColunaTabela<LinhaTeste>[] = [
    { campo: 'nome', titulo: 'Nome', ordenavel: true },
    { campo: 'total', titulo: 'Total', alinhamento: 'right' },
  ];
  linhas: LinhaTeste[] = [
    { id: '1', nome: 'Beta', total: 2 },
    { id: '2', nome: 'Alfa', total: 1 },
  ];
  carregando = false;
  erro: string | null = null;
  selecionadas: LinhaTeste[] = [];
  retentativas = 0;
}

describe('AccTableComponent', () => {
  let fixture: ComponentFixture<Hospedeiro>;
  let hospedeiro: Hospedeiro;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [Hospedeiro] }).compileComponents();
    fixture = TestBed.createComponent(Hospedeiro);
    hospedeiro = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('renderiza cabeçalhos e linhas', () => {
    const cabecalhos = fixture.debugElement.queryAll(By.css('th'));
    expect(cabecalhos.map((th) => th.nativeElement.textContent.trim())).toEqual([
      'Nome ↕',
      'Total',
    ]);
    expect(fixture.debugElement.queryAll(By.css('tbody tr')).length).toBe(2);
  });

  it('ordena ao clicar no cabeçalho', () => {
    const botao = fixture.debugElement.query(By.css('.acc-table__ordenar')).nativeElement as HTMLButtonElement;
    botao.click();
    fixture.detectChanges();

    const nomes = fixture.debugElement
      .queryAll(By.css('tbody tr td:first-child'))
      .map((td) => td.nativeElement.textContent.trim());
    expect(nomes).toEqual(['Alfa', 'Beta']);
  });

  it('mostra estado vazio', () => {
    const vazio = TestBed.createComponent(Hospedeiro);
    vazio.componentInstance.linhas = [];
    vazio.detectChanges();
    expect(vazio.nativeElement.textContent).toContain('Nada aqui');
  });

  it('mostra erro com tentar novamente', () => {
    const comErro = TestBed.createComponent(Hospedeiro);
    comErro.componentInstance.erro = 'Falha ao carregar';
    comErro.detectChanges();
    expect(comErro.nativeElement.textContent).toContain('Falha ao carregar');

    (comErro.nativeElement.querySelector('.acc-table__retry') as HTMLButtonElement).click();
    expect(comErro.componentInstance.retentativas).toBe(1);
  });

  it('emite linha selecionada em clique e teclado', () => {
    const primeira = fixture.debugElement.query(By.css('tbody tr')).nativeElement as HTMLElement;
    primeira.click();
    primeira.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    expect(hospedeiro.selecionadas.length).toBe(2);
    expect(hospedeiro.selecionadas[0].nome).toBe('Beta');
  });
});
