import { CommonModule } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';

import {
  ConsultaDevedor,
  ConsultaDividaAtivaService,
  InscricaoResumo,
  mensagemErroConsulta,
} from './consulta-divida-ativa.service';

@Component({
  selector: 'app-consulta-divida-ativa',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  template: `
    <main class="pagina">
      <header class="cabecalho">
        <a routerLink="/clientes" class="marca">Accurit</a>
        <a routerLink="/clientes" class="voltar">Voltar aos clientes</a>
      </header>

      <section class="conteudo">
        <p class="sobretitulo">Consulta exclusivamente informativa</p>
        <h1>Dívida Ativa</h1>
        <p class="descricao">
          Consulte inscrições e valores por CPF ou CNPJ. Esta tela não oferece negociação,
          parcelamento nem emissão de DARF.
        </p>

        <form class="busca" (ngSubmit)="consultar()">
          <label for="documento">CPF ou CNPJ</label>
          <div class="linha-busca">
            <input
              id="documento"
              name="documento"
              type="text"
              inputmode="numeric"
              autocomplete="off"
              maxlength="18"
              placeholder="Digite CPF ou CNPJ"
              [ngModel]="documento()"
              (ngModelChange)="documento.set($event)"
              [disabled]="carregando()"
              required
            />
            <button type="submit" [disabled]="carregando()">
              {{ carregando() ? 'Consultando...' : 'Consultar' }}
            </button>
          </div>
        </form>

        @if (erro()) {
          <p class="mensagem mensagem--erro" role="alert">{{ erro() }}</p>
        }

        @if (carregando()) {
          <p class="mensagem" role="status">Consultando a Dívida Ativa...</p>
        }

        @if (resultado(); as consulta) {
          <section class="resultado" aria-live="polite">
            <div class="metadados">
              <span class="selo" [class.selo--trial]="consulta.ambiente === 'TRIAL'">
                {{ consulta.ambiente === 'TRIAL' ? 'Demonstração (Trial)' : 'Produção' }}
              </span>
              <span class="estado">{{ rotuloEstado(consulta.estado) }}</span>
              <span>Chave consultada: {{ consulta.chaveMascarada }}</span>
              <span>Fonte: {{ rotuloFonte(consulta.fonteConsulta) }}</span>
              <span>Consultado em {{ formatarData(consulta.horarioConsulta) }}</span>
            </div>

            @if (consulta.ambiente === 'TRIAL') {
              <p class="aviso-trial">
                Resultado de demonstração. Os dados podem não representar a situação atual.
              </p>
            }

            @if (consulta.inscricoes.length === 0) {
              <div class="vazio">
                <h2>Nenhuma pendência retornada</h2>
                <p>A SERPRO respondeu sem inscrições para a chave consultada.</p>
              </div>
            } @else {
              <h2>Inscrições encontradas</h2>
              <div class="lista">
                @for (inscricao of consulta.inscricoes; track inscricao.numeroInscricao) {
                  <article class="inscricao">
                    <div>
                      <h3>{{ inscricao.situacaoDescricao || 'Situação não informada' }}</h3>
                      <p>Inscrição {{ inscricao.numeroInscricao }}</p>
                      @if (inscricao.numeroProcesso) {
                        <p>Processo {{ inscricao.numeroProcesso }}</p>
                      }
                    </div>
                    <div class="valor">
                      <strong>{{ inscricao.valorTotalConsolidado.valor }} {{ inscricao.valorTotalConsolidado.remarca }}</strong>
                      <a [routerLink]="['/divida-ativa/inscricoes', inscricao.numeroInscricao]">
                        Ver detalhes
                      </a>
                    </div>
                  </article>
                }
              </div>
            }

            <p class="aviso">{{ consulta.aviso }}</p>
          </section>
        }
      </section>
    </main>
  `,
  styles: `
    :host { display: block; min-height: 100dvh; background: #f8fafc; color: #0f172a; }
    .pagina { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
    .cabecalho, .linha-busca, .metadados, .inscricao, .valor {
      display: flex; align-items: center; justify-content: space-between; gap: 1rem;
    }
    .marca { color: inherit; font-size: 1.125rem; font-weight: 700; text-decoration: none; }
    .voltar, .inscricao a { color: #1d4ed8; font-weight: 600; }
    .conteudo { margin-top: 3rem; }
    .sobretitulo { color: #64748b; font-size: .75rem; font-weight: 700; text-transform: uppercase; }
    h1 { margin: .5rem 0; font-size: clamp(2rem, 5vw, 3rem); }
    .descricao { max-width: 42rem; color: #475569; }
    .busca, .resultado { margin-top: 2rem; border: 1px solid #e2e8f0; border-radius: .75rem; background: white; padding: 1.25rem; }
    .busca label { display: block; margin-bottom: .5rem; font-weight: 600; }
    input { min-width: 0; flex: 1; border: 1px solid #cbd5e1; border-radius: .5rem; padding: .75rem; }
    button { border: 0; border-radius: .5rem; background: #1d4ed8; padding: .75rem 1rem; color: white; font-weight: 700; cursor: pointer; }
    button:disabled { cursor: wait; opacity: .65; }
    .mensagem { margin-top: 1rem; color: #475569; }
    .mensagem--erro { border-left: 4px solid #dc2626; padding: .75rem; color: #991b1b; }
    .metadados { justify-content: flex-start; flex-wrap: wrap; color: #475569; font-size: .875rem; }
    .selo, .estado { border-radius: 999px; background: #dcfce7; padding: .25rem .625rem; color: #166534; font-weight: 700; }
    .selo--trial { background: #fef3c7; color: #92400e; }
    .aviso-trial { border-radius: .5rem; background: #fffbeb; padding: .75rem; color: #92400e; }
    .lista { display: grid; gap: .75rem; }
    .inscricao { align-items: flex-start; border: 1px solid #e2e8f0; border-radius: .5rem; padding: 1rem; }
    .inscricao h3, .inscricao p { margin: 0 0 .4rem; }
    .inscricao p { color: #475569; }
    .valor { flex-direction: column; align-items: flex-end; white-space: nowrap; }
    .vazio { border-radius: .5rem; background: #f1f5f9; padding: 1rem; }
    .aviso { margin: 1.5rem 0 0; color: #64748b; font-size: .875rem; }
    @media (max-width: 600px) {
      .linha-busca, .inscricao { align-items: stretch; flex-direction: column; }
      .valor { align-items: flex-start; }
    }
  `,
})
export class ConsultaDividaAtivaComponent {
  private readonly service = inject(ConsultaDividaAtivaService);

  protected readonly documento = signal('');
  protected readonly resultado = signal<ConsultaDevedor | null>(null);
  protected readonly carregando = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected consultar(): void {
    const digitos = this.documento().replace(/\D/g, '');
    if (digitos.length !== 11 && digitos.length !== 14) {
      this.resultado.set(null);
      this.erro.set('Informe um CPF com 11 dígitos ou um CNPJ com 14 dígitos.');
      return;
    }

    this.carregando.set(true);
    this.erro.set(null);
    this.resultado.set(null);
    this.service.consultarDevedor(digitos).subscribe({
      next: (resultado) => {
        this.resultado.set(resultado);
        this.carregando.set(false);
      },
      error: (error: unknown) => {
        this.erro.set(mensagemErroConsulta(error));
        this.carregando.set(false);
      },
    });
  }

  protected rotuloEstado(estado: ConsultaDevedor['estado']): string {
    return estado === 'SEM_PENDENCIAS' ? 'Sem pendências retornadas' : 'Pendências retornadas';
  }

  protected rotuloFonte(fonte: ConsultaDevedor['fonteConsulta']): string {
    return fonte === 'CODIGO_CONTRIBUINTE' ? 'Código de Contribuinte' : 'CPF/CNPJ';
  }

  protected formatarData(valor: string): string {
    return new Date(valor).toLocaleString('pt-BR');
  }
}
