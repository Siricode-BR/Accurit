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
    :host { display: block; min-height: 100dvh; background: #f8fafc; color: #0f172a; }
    .pagina { max-width: 860px; margin: 0 auto; padding: 2rem 1.25rem 4rem; }
    header { margin-bottom: 1.5rem; }
    h1 { margin: .25rem 0; font-size: clamp(2rem, 5vw, 3rem); }
    h2 { margin: 0; font-size: 1.125rem; }
    p { line-height: 1.5; }
    .sobretitulo { margin: 0; color: #64748b; font-size: .75rem; font-weight: 700; text-transform: uppercase; }
    .descricao { max-width: 44rem; color: #475569; }
    .cartao { margin-top: 1rem; border: 1px solid #e2e8f0; border-radius: .75rem; background: white; padding: 1.25rem; }
    .linha { display: flex; align-items: center; justify-content: space-between; gap: .75rem; }
    .ambiente, .rotulo { border-radius: 999px; padding: .25rem .625rem; font-size: .75rem; font-weight: 700; }
    .ambiente { background: #dcfce7; color: #166534; }
    .ambiente--trial { background: #fef3c7; color: #92400e; }
    .aviso { border-left: 4px solid #d97706; background: #fffbeb; padding: .75rem; color: #78350f; }
    .servicos { display: grid; gap: .5rem; margin: 1rem 0 0; padding: 0; list-style: none; }
    .servicos li { display: flex; align-items: center; justify-content: space-between; gap: 1rem; border-top: 1px solid #e2e8f0; padding: .75rem 0; }
    .servicos p { margin: .25rem 0 0; color: #64748b; font-size: .875rem; }
    .rotulo--ok { background: #dcfce7; color: #166534; }
    .rotulo--pendente { background: #f1f5f9; color: #475569; }
    label { display: block; margin: 1rem 0 .4rem; font-weight: 600; }
    input { min-width: 0; flex: 1; border: 1px solid #cbd5e1; border-radius: .5rem; padding: .7rem; }
    button { border: 0; border-radius: .5rem; background: #1d4ed8; padding: .7rem 1rem; color: white; font-weight: 700; cursor: pointer; }
    button:disabled { cursor: wait; opacity: .6; }
    .mensagem { margin: 1rem 0; color: #475569; }
    .mensagem--erro { border-left: 4px solid #dc2626; background: #fef2f2; padding: .75rem; color: #991b1b; }
    .resultado { margin-top: 1rem; border-radius: .5rem; background: #f8fafc; padding: .9rem; }
    .resultado p { margin-bottom: 0; color: #475569; }
    .vazio { color: #64748b; }
    @media (max-width: 540px) {
      .linha { align-items: stretch; flex-direction: column; }
      .servicos li { align-items: flex-start; flex-direction: column; }
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
