import { CommonModule } from '@angular/common';
import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';

import {
  ConsultaDividaAtivaService,
  ConsultaInscricao,
  mensagemErroConsulta,
} from './consulta-divida-ativa.service';

@Component({
  selector: 'app-inscricao-detalhe',
  standalone: true,
  imports: [CommonModule, RouterLink],
  template: `
    <main class="pagina">
      <header class="cabecalho">
        <a routerLink="/clientes" class="marca">Accurit</a>
        <a routerLink="/divida-ativa" class="voltar">Voltar à consulta</a>
      </header>

      <section class="conteudo">
        <p class="sobretitulo">Consulta exclusivamente informativa</p>
        <h1>Detalhe da inscrição</h1>

        @if (carregando()) {
          <p role="status">Consultando inscrição...</p>
        } @else if (erro()) {
          <p class="erro" role="alert">{{ erro() }}</p>
        } @else if (detalhe(); as consulta) {
          <section class="cartao" aria-live="polite">
            <div class="metadados">
              <span class="selo" [class.selo--trial]="consulta.ambiente === 'TRIAL'">
                {{ consulta.ambiente === 'TRIAL' ? 'Demonstração (Trial)' : 'Produção' }}
              </span>
              <span class="estado">{{ consulta.estado === 'SEM_PENDENCIAS' ? 'Sem pendências' : 'Inscrição retornada' }}</span>
              <span>Consultado em {{ formatarData(consulta.horarioConsulta) }}</span>
            </div>

            @if (consulta.ambiente === 'TRIAL') {
              <p class="aviso-trial">
                Resultado de demonstração. Os dados podem não representar a situação atual.
              </p>
            }

            <dl>
              <div><dt>Número da inscrição</dt><dd>{{ consulta.numeroInscricao }}</dd></div>
              <div><dt>Situação</dt><dd>{{ consulta.situacao || 'Não informada' }}</dd></div>
              <div><dt>Processo administrativo</dt><dd>{{ consulta.processoAdministrativo || 'Não informado' }}</dd></div>
              <div><dt>Data da inscrição</dt><dd>{{ consulta.dataInscricao || 'Não informada' }}</dd></div>
              <div><dt>Receita principal</dt><dd>{{ consulta.nomeReceita || consulta.codigoNaturezaReceita || 'Não informada' }}</dd></div>
              <div><dt>Órgão responsável</dt><dd>{{ consulta.nomeUnidade || consulta.orgaoOrigem || 'Não informado' }}</dd></div>
              <div><dt>Prescrição</dt><dd>{{ rotuloPrescricao(consulta.indicadorPrescricao) }}</dd></div>
              <div><dt>Valor inscrito</dt><dd>{{ formatarValor(consulta.valorTotalInscrito) }}</dd></div>
              <div><dt>Valor consolidado</dt><dd>{{ formatarValor(consulta.valorTotalConsolidado) }}</dd></div>
              <div><dt>Valor remanescente</dt><dd>{{ formatarValor(consulta.valorRemanescente) }}</dd></div>
              @if (consulta.fonteConsulta) {
                <div><dt>Fonte da consulta</dt><dd>{{ rotuloFonte(consulta.fonteConsulta) }}</dd></div>
              }
            </dl>
            <p class="aviso">{{ consulta.aviso }}</p>
          </section>
        }
      </section>
    </main>
  `,
  styles: `
    :host { display: block; min-height: 100dvh; background: #f8fafc; color: #0f172a; }
    .pagina { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
    .cabecalho, .metadados { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }
    .marca { color: inherit; font-size: 1.125rem; font-weight: 700; text-decoration: none; }
    .voltar { color: #1d4ed8; font-weight: 600; }
    .conteudo { margin-top: 3rem; }
    .sobretitulo { color: #64748b; font-size: .75rem; font-weight: 700; text-transform: uppercase; }
    h1 { margin: .5rem 0 1.5rem; font-size: clamp(2rem, 5vw, 3rem); }
    .cartao { border: 1px solid #e2e8f0; border-radius: .75rem; background: white; padding: 1.25rem; }
    .metadados { justify-content: flex-start; flex-wrap: wrap; color: #475569; font-size: .875rem; }
    .selo, .estado { border-radius: 999px; background: #dcfce7; padding: .25rem .625rem; color: #166534; font-weight: 700; }
    .selo--trial { background: #fef3c7; color: #92400e; }
    .aviso-trial { border-radius: .5rem; background: #fffbeb; padding: .75rem; color: #92400e; }
    dl { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin-top: 1.5rem; }
    dl div { border-bottom: 1px solid #e2e8f0; padding-bottom: .75rem; }
    dt { color: #64748b; font-size: .875rem; }
    dd { margin: .25rem 0 0; font-weight: 600; }
    .aviso { margin: 1.5rem 0 0; color: #64748b; font-size: .875rem; }
    .erro { border-left: 4px solid #dc2626; padding: .75rem; color: #991b1b; }
  `,
})
export class InscricaoDetalheComponent implements OnInit {
  private readonly route = inject(ActivatedRoute);
  private readonly service = inject(ConsultaDividaAtivaService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly detalhe = signal<ConsultaInscricao | null>(null);
  protected readonly carregando = signal(true);
  protected readonly erro = signal<string | null>(null);

  ngOnInit(): void {
    this.route.paramMap.pipe(takeUntilDestroyed(this.destroyRef)).subscribe((params) => {
      const numero = params.get('numero');
      if (!numero) {
        this.erro.set('Número de inscrição não informado.');
        this.carregando.set(false);
        return;
      }

      this.carregando.set(true);
      this.erro.set(null);
      this.service
        .consultarInscricao(numero)
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe({
          next: (detalhe) => {
            this.detalhe.set(detalhe);
            this.carregando.set(false);
          },
          error: (error: unknown) => {
            this.erro.set(mensagemErroConsulta(error));
            this.carregando.set(false);
          },
        });
    });
  }

  protected formatarData(valor: string): string {
    return new Date(valor).toLocaleString('pt-BR');
  }

  protected formatarValor(valor: ConsultaInscricao['valorTotalInscrito']): string {
    return valor ? `${valor.valor} ${valor.remarca}` : 'Não informado';
  }

  protected rotuloPrescricao(valor: boolean | null | undefined): string {
    if (valor === true) return 'Sim';
    if (valor === false) return 'Não';
    return 'Não informado';
  }

  protected rotuloFonte(fonte: NonNullable<ConsultaInscricao['fonteConsulta']>): string {
    return fonte === 'CODIGO_CONTRIBUINTE' ? 'Código de Contribuinte' : 'CPF/CNPJ';
  }
}
