import { CommonModule } from '@angular/common';
import { Component, OnInit, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import {
  CategoriaErroIntegraContador,
  IntegraContadorService,
  ResultadoIntegraContador,
  StatusIntegraContador,
  categoriaErroIntegraContador,
  mensagemErroIntegraContador,
} from './integra-contador.service';

@Component({
  selector: 'app-integra-contador',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <main class="pagina">
      <header>
        <p class="sobretitulo">Integração oficial SERPRO</p>
        <h1>Integra Contador</h1>
        <p class="descricao">
          Consultas são somente leitura. Serviços sem versão confirmada e operações mutáveis
          permanecem bloqueados.
        </p>
      </header>

      @if (carregandoStatus()) {
        <p class="mensagem" role="status">Carregando ambiente e catálogo...</p>
      }

      @if (erroStatus()) {
        <p class="mensagem mensagem--erro" role="alert">{{ erroStatus() }}</p>
      }

      @if (status(); as atual) {
        <section class="cartao" aria-labelledby="status-heading">
          <div class="linha">
            <h2 id="status-heading">Status</h2>
            <span class="ambiente" [class.ambiente--trial]="atual.ambienteEfetivo === 'TRIAL'">
              {{ atual.ambienteEfetivo === 'TRIAL' ? 'Demonstração (Trial)' : 'Produção' }}
            </span>
          </div>

          @if (!atual.producaoHabilitada) {
            <p class="aviso" role="status">{{ atual.motivoBloqueioProducao }}</p>
          }
          @if (!atual.assinaturaHabilitada) {
            <p class="aviso" role="status">
              Assinatura digital indisponível: {{ atual.motivoBloqueioAssinatura }}
            </p>
          }
        </section>

        <section class="cartao" aria-labelledby="catalogo-heading">
          <h2 id="catalogo-heading">Catálogo de serviços</h2>
          @if (atual.servicos.length === 0) {
            <p class="vazio">Nenhum serviço foi confirmado para este ambiente.</p>
          } @else {
            <ul class="servicos">
              @for (servico of atual.servicos; track servico.idSistema + servico.idServico) {
                <li>
                  <div>
                    <strong>{{ servico.idSistema }} / {{ servico.idServico }}</strong>
                    <p>
                      Verbo {{ servico.verbo }}
                      @if (servico.versaoSistema) {
                        · versão {{ servico.versaoSistema }}
                      } @else {
                        · versão pendente
                      }
                    </p>
                  </div>
                  @if (servico.habilitado) {
                    <span class="rotulo rotulo--ok">Habilitado</span>
                  } @else {
                    <span class="rotulo rotulo--pendente">Aguardando confirmação</span>
                  }
                </li>
              }
            </ul>
          }
        </section>

        @if (podeConsultarProcuracoes()) {
          <section class="cartao" aria-labelledby="procuracao-heading">
            <h2 id="procuracao-heading">Consultar procurações</h2>
            <p class="descricao">
              A consulta é enviada ao SERPRO com o contexto do escritório e do contribuinte
              validado no backend. Uma resposta de consulta, por si só, não concede autorização.
            </p>

            <form (ngSubmit)="consultarProcuracoes()">
              <label for="documento">CPF/CNPJ do contribuinte</label>
              <div class="linha">
                <input
                  id="documento"
                  name="documento"
                  type="text"
                  inputmode="numeric"
                  autocomplete="off"
                  maxlength="18"
                  placeholder="Digite CPF ou CNPJ"
                  [(ngModel)]="documentoContribuinte"
                  [disabled]="carregandoConsulta()"
                  required
                />
                <button type="submit" [disabled]="carregandoConsulta() || !documentoContribuinte.trim()">
                  {{ carregandoConsulta() ? 'Consultando...' : 'Consultar' }}
                </button>
              </div>
            </form>

            @if (carregandoConsulta()) {
              <p class="mensagem" role="status">Consultando procurações no SERPRO...</p>
            }
            @if (erroConsulta()) {
              <p class="mensagem mensagem--erro" role="alert">
                <strong>{{ rotuloErro(erroCategoria()) }}:</strong> {{ erroConsulta() }}
              </p>
            }
            @if (resultado(); as consulta) {
              <div class="resultado" aria-live="polite">
                <div class="linha">
                  <strong>{{ rotuloEstado(consulta.estado) }}</strong>
                  <span>HTTP SERPRO {{ consulta.statusSerpro }}</span>
                </div>
                <p>{{ consulta.mensagem }}</p>
                @if (consulta.codigosMensagem.length > 0) {
                  <p>Códigos: {{ consulta.codigosMensagem.join(', ') }}</p>
                }
                @if (consulta.tempoEsperaSegundos !== null || consulta.etag || consulta.expires) {
                  <p>
                    A resposta permite acompanhamento controlado. Nenhuma repetição ou consulta
                    automática foi iniciada.
                  </p>
                }
              </div>
            }
          </section>
        } @else {
          <section class="cartao">
            <h2>Autorização/procuração</h2>
            <p class="vazio">
              Não avaliada. Nenhuma operação protegida está habilitada neste catálogo.
            </p>
          </section>
        }

        @if (!temServicoHabilitado()) {
          <p class="vazio">Não há operações disponíveis até a confirmação do catálogo SERPRO.</p>
        }
      }
    </main>
  `,
  styles: `
    :host {
      display: block;
      min-height: 100dvh;
      background: var(--surface-canvas);
      color: var(--text-primary);
    }

    .pagina {
      max-width: 960px;
      margin: 0 auto;
      padding: 2.5rem clamp(1rem, 4vw, 2.5rem) 4rem;
    }

    header {
      margin-bottom: 2rem;
    }

    h1 {
      margin: 0.25rem 0;
      font-size: clamp(2rem, 5vw, 3rem);
      letter-spacing: -0.035em;
      line-height: 1.1;
    }

    h2 {
      margin: 0;
      font-size: 1.125rem;
      letter-spacing: -0.015em;
    }

    p {
      line-height: 1.5;
    }

    .sobretitulo {
      margin: 0;
      color: var(--text-secondary);
      font-size: 0.75rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    .descricao {
      max-width: 44rem;
      color: var(--text-secondary);
    }

    .cartao {
      margin-top: 1rem;
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-card);
      background: var(--surface-raised);
      padding: clamp(1rem, 3vw, 1.5rem);
    }

    .linha {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.75rem;
    }

    .ambiente,
    .rotulo {
      border-radius: 999px;
      padding: 0.25rem 0.625rem;
      font-size: 0.75rem;
      font-weight: 700;
    }

    .ambiente {
      background: var(--env-producao-surface);
      color: var(--env-producao);
    }

    .ambiente--trial {
      background: var(--env-trial-surface);
      color: var(--env-trial);
    }

    .aviso {
      border-left: 4px solid var(--state-warning);
      border-radius: 0 var(--radius-control) var(--radius-control) 0;
      background: var(--state-warning-surface);
      padding: 0.75rem;
      color: var(--state-warning);
    }

    .servicos {
      display: grid;
      gap: 0.5rem;
      margin: 1rem 0 0;
      padding: 0;
      list-style: none;
    }

    .servicos li {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      border-top: 1px solid var(--border-subtle);
      padding: 0.875rem 0;
    }

    .servicos p {
      margin: 0.25rem 0 0;
      color: var(--text-secondary);
      font-size: 0.875rem;
    }

    .rotulo--ok {
      background: var(--state-success-surface);
      color: var(--state-success);
    }

    .rotulo--pendente {
      background: var(--surface-subtle);
      color: var(--text-secondary);
    }

    label {
      display: block;
      margin: 1rem 0 0.4rem;
      font-weight: 600;
    }

    input {
      min-width: 0;
      flex: 1;
      border: 1px solid var(--border-default);
      border-radius: var(--radius-control);
      background: var(--surface-raised);
      padding: 0.75rem;
      color: var(--text-primary);
    }

    button {
      min-height: 2.75rem;
      border: 0;
      border-radius: var(--radius-control);
      background: var(--action-primary);
      padding: 0.7rem 1rem;
      color: var(--action-on-primary);
      font-weight: 700;
      cursor: pointer;
    }

    button:hover:not(:disabled) {
      background: var(--action-primary-hover);
    }

    button:disabled {
      cursor: wait;
      opacity: 0.6;
    }

    .mensagem {
      margin: 1rem 0;
      color: var(--text-secondary);
    }

    .mensagem--erro {
      border-left: 4px solid var(--state-danger);
      border-radius: 0 var(--radius-control) var(--radius-control) 0;
      background: var(--state-danger-surface);
      padding: 0.75rem;
      color: var(--state-danger);
    }

    .resultado {
      margin-top: 1rem;
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-control);
      background: var(--surface-subtle);
      padding: 1rem;
    }

    .resultado p {
      margin-bottom: 0;
      color: var(--text-secondary);
    }

    .vazio {
      color: var(--text-secondary);
    }

    @media (max-width: 540px) {
      .linha {
        align-items: stretch;
        flex-direction: column;
      }

      .servicos li {
        align-items: flex-start;
        flex-direction: column;
      }
    }
  `,
})
export class IntegraContadorComponent implements OnInit {
  private readonly service = inject(IntegraContadorService);

  readonly status = signal<StatusIntegraContador | null>(null);
  readonly carregandoStatus = signal(false);
  readonly erroStatus = signal<string | null>(null);
  documentoContribuinte = '';
  readonly carregandoConsulta = signal(false);
  readonly erroConsulta = signal<string | null>(null);
  readonly erroCategoria = signal<CategoriaErroIntegraContador>('negocio');
  readonly resultado = signal<ResultadoIntegraContador | null>(null);
  readonly podeConsultarProcuracoes = computed(
    () =>
      this.status()?.servicos.some(
        (servico) =>
          servico.habilitado &&
          servico.idSistema === 'PROCURACOES' &&
          servico.idServico === 'OBTERPROCURACAO41',
      ) ?? false,
  );
  readonly temServicoHabilitado = computed(
    () => this.status()?.servicos.some((servico) => servico.habilitado) ?? false,
  );

  ngOnInit(): void {
    this.carregarStatus();
  }

  carregarStatus(): void {
    this.carregandoStatus.set(true);
    this.erroStatus.set(null);
    this.service.consultarStatus().subscribe({
      next: (status) => this.status.set(status),
      error: (error: unknown) => {
        this.erroStatus.set(mensagemErroIntegraContador(error));
        this.carregandoStatus.set(false);
      },
      complete: () => this.carregandoStatus.set(false),
    });
  }

  consultarProcuracoes(): void {
    const documento = this.documentoContribuinte.trim();
    if (!documento || this.carregandoConsulta() || !this.podeConsultarProcuracoes()) {
      return;
    }

    this.carregandoConsulta.set(true);
    this.erroConsulta.set(null);
    this.resultado.set(null);
    this.service.consultarProcuracoes(documento).subscribe({
      next: (resultado) => this.resultado.set(resultado),
      error: (error: unknown) => {
        this.erroCategoria.set(categoriaErroIntegraContador(error));
        this.erroConsulta.set(mensagemErroIntegraContador(error));
        this.carregandoConsulta.set(false);
      },
      complete: () => this.carregandoConsulta.set(false),
    });
  }

  rotuloErro(categoria: CategoriaErroIntegraContador): string {
    switch (categoria) {
      case 'autorizacao':
        return 'Falha de autorização';
      case 'transporte':
        return 'Falha de transporte';
      case 'negocio':
        return 'Rejeição de negócio';
    }
  }

  rotuloEstado(estado: string): string {
    switch (estado) {
      case 'PROCESSANDO':
        return 'Processamento em andamento';
      case 'INDETERMINADO':
        return 'Estado indeterminado';
      case 'FALHA_AUTORIZACAO':
        return 'Falha de autorização';
      case 'REJEICAO_NEGOCIO':
        return 'Rejeição de negócio';
      case 'FALHA_TRANSPORTE':
        return 'Falha de transporte';
      case 'LIMITE_EXCEDIDO':
        return 'Limite excedido';
      default:
        return estado;
    }
  }
}
