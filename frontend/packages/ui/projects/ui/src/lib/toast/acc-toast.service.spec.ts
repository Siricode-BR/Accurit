import { TestBed } from '@angular/core/testing';
import { AccToastService } from './acc-toast.service';

describe('AccToastService', () => {
  let servico: AccToastService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    servico = TestBed.inject(AccToastService);
  });

  it('adiciona e fecha toasts', () => {
    const id = servico.sucesso('Salvo');
    expect(servico.toasts().length).toBe(1);
    expect(servico.toasts()[0].variante).toBe('sucesso');

    servico.fechar(id);
    expect(servico.toasts().length).toBe(0);
  });

  it('mantém variante de erro e limpa tudo', () => {
    servico.erro('Falhou');
    servico.aviso('Atenção');
    servico.info('Info');
    expect(servico.toasts().map((toast) => toast.variante)).toEqual([
      'erro',
      'aviso',
      'info',
    ]);
    servico.limpar();
    expect(servico.toasts().length).toBe(0);
  });
});
