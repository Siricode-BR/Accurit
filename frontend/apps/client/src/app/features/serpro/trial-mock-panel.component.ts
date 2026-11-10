import { ChangeDetectionStrategy, Component, OnInit, inject, input, output, signal } from '@angular/core';
import { CenarioTrialMockDto, ServicoTrialMock } from '@accurit/core';
import { TrialMockService } from '../../data/trial-mock.service';

/**
 * Banner "Ambiente Trial" + painel de cenários do simulador.
 * O botão "Usar documento" emite o documento de exemplo para a tela preencher.
 */
@Component({
  selector: 'app-trial-mock-panel',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (ativo()) {
      <div class="trial">
        <p class="trial__banner">
          <strong>Ambiente Trial</strong> — respostas simuladas; não use para decisões fiscais.
        </p>
        <button type="button" class="trial__toggle" (click)="aberto.set(!aberto())">
          {{ aberto() ? 'Fechar cenários de teste' : 'Cenários de teste' }}
        </button>
        @if (aberto()) {
          <div class="trial__painel">
            @for (cenario of cenarios(); track cenario.id) {
              <article class="trial__cenario">
                <div class="trial__texto">
                  <strong>{{ cenario.titulo }}</strong>
                  <span>{{ cenario.descricao }}</span>
                  <code>{{ cenario.resultadoEsperado }}</code>
                </div>
                <button type="button" (click)="usar(cenario)">Usar documento</button>
              </article>
            } @empty {
              <p class="trial__vazio">Nenhum cenário disponível.</p>
            }
          </div>
        }
      </div>
    }
  `,
  styles: `
    .trial { margin: 0 0 1rem; }
    .trial__banner { background: #fff7e6; border: 1px solid #f0b429; border-radius: 8px; padding: .65rem .8rem; margin: 0 0 .5rem; color: #6b4b00; }
    .trial__toggle { background: none; border: 1px solid #f0b429; border-radius: 6px; padding: .35rem .7rem; cursor: pointer; color: #6b4b00; }
    .trial__painel { margin-top: .6rem; display: grid; gap: .5rem; }
    .trial__cenario { display: flex; gap: .75rem; justify-content: space-between; align-items: center; border: 1px solid #e5e7eb; border-radius: 8px; padding: .6rem .75rem; background: #fff; }
    .trial__texto { display: flex; flex-direction: column; gap: .2rem; font-size: .875rem; }
    .trial__texto span { color: #4b5563; }
    .trial__texto code { color: #92400e; font-size: .75rem; }
    .trial__cenario button { white-space: nowrap; border: 0; border-radius: 6px; padding: .35rem .65rem; background: #1d4ed8; color: #fff; cursor: pointer; }
    .trial__vazio { color: #4b5563; font-size: .875rem; }
  `,
})
export class TrialMockPanelComponent implements OnInit {
  private readonly service = inject(TrialMockService);

  readonly servicos = input<readonly ServicoTrialMock[]>([]);
  readonly documentoSelecionado = output<string>();

  protected readonly aberto = signal(false);
  protected readonly ativo = this.service.ativo;
  protected readonly cenarios = () => this.service.porServico(...this.servicos());

  ngOnInit(): void {
    void this.service.carregar();
  }

  protected usar(cenario: CenarioTrialMockDto): void {
    this.documentoSelecionado.emit(cenario.documentoExemplo);
  }
}
