import {
  Component,
  inject,
  ChangeDetectionStrategy,
  OnInit,
  signal,
  computed,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { ClienteDto } from '@accurit/core';
import { sanitizeDocument } from '@accurit/shared';
import { ClienteService } from '../../data/cliente.service';
import { DividaAtivaService } from '../../data/divida-ativa.service';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { SessionStore } from '../../core/auth/session.store';
import { AccButtonComponent, AccCardComponent, AccEmptyStateComponent } from '@accurit/ui';

type ConsultaModo = 'devedor' | 'inscricao';

@Component({
  selector: 'app-divida-ativa-consulta-page',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    AccButtonComponent,
    AccCardComponent,
    AccEmptyStateComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-container">
      <header class="page-header">
        <button acc-button variant="ghost" size="sm" (click)="voltar()">
          ← Voltar
        </button>
        <h1 class="page-title">Consulta Dívida Ativa</h1>
      </header>

      @if (cliente(); as c) {
        <p class="cliente-info">
          Consultando para: <strong>{{ c.nomeRazaoSocial }}</strong>
          <span class="documento">{{ aplicarMascara(c.cpfCnpj) }}</span>
        </p>
      }

      @if (ambienteEfetivo(); as env) {
        <p class="ambiente-sel">
          Ambiente: {{ env === 'TRIAL' ? 'Trial (avaliação)' : 'Produção' }}
        </p>
      }

      @if (avisoConsultivo(); as aviso) {
        <p class="aviso-consultivo" role="note">{{ aviso }}</p>
      }

      <section class="section">
        <div class="modos">
          <button
            acc-button
            [class.active]="modo() === 'devedor'"
            (click)="selecionarModo('devedor')"
          >
            Por devedor (CPF/CNPJ)
          </button>
          <button
            acc-button
            [class.active]="modo() === 'inscricao'"
            (click)="selecionarModo('inscricao')"
          >
            Por inscrição
          </button>
        </div>

        <form (ngSubmit)="onSubmit()" class="consulta-form">
          <div class="field">
            <label for="chave" class="label">
              {{ modo() === 'devedor' ? 'CPF/CNPJ do devedor' : 'Número da inscrição' }}
            </label>
            <input
              id="chave"
              type="text"
              [value]="chave()"
              (input)="atualizarChave($event)"
              class="input"
              placeholder="{{ modo() === 'devedor' ? '000.000.000-00' : 'ex: 123456789' }}"
            />
          </div>

          <button
            acc-button
            type="submit"
            variant="primary"
            [disabled]="carregando() || !chave().trim()"
          >
            Consultar
          </button>
        </form>
      </section>

      @if (carregando()) {
        <div class="carregando" aria-hidden="true">
          <p>Consultando a Dívida Ativa...</p>
        </div>
      }

      @if (erro()) {
        <div class="alert-error" role="alert" aria-live="polite">{{ erro() }}</div>
        <button acc-button variant="outline" size="md" (click)="limparErro()">
          Limpar
        </button>
      }

      @if (devedor(); as d) {
        <acc-card class="resultado-card">
          <h2 class="resultado-titulo">Devedor</h2>
          <dl class="resultado-lista">
            <div class="resultado-item">
              <dt>Estado</dt>
              <dd>{{ d.estado }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Ambiente</dt>
              <dd>{{ d.ambiente }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Chave consultada</dt>
              <dd>{{ d.chaveMascarada }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Horário da consulta</dt>
              <dd>{{ d.horarioConsulta }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Tentativas</dt>
              <dd>{{ d.tentativas }}</dd>
            </div>
            @if (d.aviso) {
              <div class="resultado-item">
                <dt>Aviso</dt>
                <dd class="aviso-inline">{{ d.aviso }}</dd>
              </div>
            }
          </dl>

          @if (d.inscricoes?.length) {
            <h3 class="inscricoes-titulo">Inscrições</h3>
            <ul class="inscricoes-lista">
              @for (inc of d.inscricoes; track inc.numeroInscricao) {
                <li class="inscricao-item">
                  <span class="inscricao-numero">{{ inc.numeroInscricao }}</span>
                  <span class="inscricao-situacao">{{ inc.situacaoDescricao }}</span>
                  <span class="inscricao-valor">{{ inc.valorTotalConsolidado.valor }}</span>
                </li>
              }
            </ul>
          } @else {
            <p class="sem-pendencias">Sem inscrições pendentes para este devedor.</p>
          }
        </acc-card>
      }

      @if (inscricao(); as i) {
        <acc-card class="resultado-card">
          <h2 class="resultado-titulo">Inscrição</h2>
          <dl class="resultado-lista">
            <div class="resultado-item">
              <dt>Número</dt>
              <dd>{{ i.numeroInscricao }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Situação</dt>
              <dd>{{ i.situacao }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Ambiente</dt>
              <dd>{{ i.ambiente }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Chave consultada</dt>
              <dd>{{ i.chaveMascarada }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Horário da consulta</dt>
              <dd>{{ i.horarioConsulta }}</dd>
            </div>
            @if (i.dataInscricao) {
              <div class="resultado-item">
                <dt>Data de inscrição</dt>
                <dd>{{ i.dataInscricao }}</dd>
              </div>
            }
            @if (i.valorTotalConsolidado) {
              <div class="resultado-item">
                <dt>Valor total consolidado</dt>
                <dd>{{ i.valorTotalConsolidado.valor }}</dd>
              </div>
            }
            @if (i.aviso) {
              <div class="resultado-item">
                <dt>Aviso</dt>
                <dd class="aviso-inline">{{ i.aviso }}</dd>
              </div>
            }
          </dl>
        </acc-card>
      }

      @if (!carregando() && !erro() && !devedor() && !inscricao() && chave()) {
        <acc-empty-state
          title="Sem resultado"
          descricao="Não foi encontrado nenhum resultado para a chave informada."
        />
      }
    </div>
  `,
  styles: `
    .page-container {
      max-width: 900px;
      margin: 0 auto;
      padding: 1.5rem;
    }

    .page-header {
      display: flex;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1.5rem;
      flex-wrap: wrap;
    }

    .page-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .cliente-info {
      margin: 0 0 1rem;
      font-size: 0.9375rem;
      color: var(--acc-color-text-main);
    }

    .documento {
      margin-left: 0.5rem;
      font-size: 0.875rem;
      color: var(--acc-color-text-muted);
    }

    .ambiente-sel {
      margin: 0 0 1rem;
      font-size: 0.875rem;
      color: var(--acc-color-text-muted);
    }

    .aviso-consultivo {
      margin: 0 0 1rem;
      padding: 0.625rem 0.875rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-warning-bg, #fffbeb);
      color: var(--acc-color-warning, #b45309);
      font-size: 0.8125rem;
    }

    .section {
      margin-bottom: 2rem;
    }

    .modos {
      display: flex;
      gap: 0.5rem;
      margin-bottom: 1rem;
      flex-wrap: wrap;
    }

    .consulta-form {
      display: flex;
      gap: 0.5rem;
      margin-bottom: 1.5rem;
      flex-wrap: wrap;
      align-items: flex-end;
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
      flex: 1;
      min-width: 200px;
    }

    .label {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--acc-color-text-main);
    }

    .input {
      padding: 0.625rem 0.875rem;
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-size: 0.875rem;
      outline: none;
      font-family: inherit;
      background-color: var(--acc-color-surface);
    }

    .input:focus {
      border-color: var(--acc-color-primary);
      box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.1);
    }

    .carregando {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.5rem;
      padding: 2rem;
      color: var(--acc-color-text-muted);
    }

    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
      margin-bottom: 1rem;
    }

    .resultado-card {
      margin-bottom: 1rem;
    }

    .resultado-titulo {
      font-size: 1.125rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 0.75rem;
    }

    .resultado-lista {
      margin: 0;
      display: grid;
      gap: 0.5rem;
    }

    .resultado-item {
      display: flex;
      justify-content: space-between;
      gap: 0.75rem;
      font-size: 0.875rem;
    }

    .resultado-item dt {
      margin: 0;
      color: var(--acc-color-text-muted);
    }

    .resultado-item dd {
      margin: 0;
      color: var(--acc-color-text-main);
      font-weight: 600;
    }

    .aviso-inline {
      color: var(--acc-color-warning, #b45309);
      font-size: 0.8125rem;
    }

    .inscricoes-titulo {
      font-size: 1rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 1rem 0 0.5rem;
    }

    .inscricoes-lista {
      margin: 0;
      padding: 0;
      list-style: none;
      display: grid;
      gap: 0.5rem;
    }

    .inscricao-item {
      display: flex;
      gap: 1rem;
      padding: 0.5rem 0.75rem;
      background: var(--acc-color-surface);
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      font-size: 0.875rem;
    }

    .inscricao-numero {
      font-weight: 600;
      color: var(--acc-color-text-main);
    }

    .inscricao-situacao {
      color: var(--acc-color-text-muted);
    }

    .inscricao-valor {
      margin-left: auto;
      font-weight: 600;
      color: var(--acc-color-text-main);
    }

    .sem-pendencias {
      color: var(--acc-color-text-muted);
      font-size: 0.875rem;
      margin-top: 1rem;
    }
  `,
})
export class DividaAtivaConsultaPage implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly dividaAtiva = inject(DividaAtivaService);
  protected readonly serproAmbiente = inject(SerproAmbienteService);
  protected readonly sessao = inject(SessionStore);
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  protected readonly cliente = signal<ClienteDto | null>(null);
  protected readonly modo = signal<ConsultaModo>('devedor');
  protected readonly chave = signal<string>('');
  protected readonly carregando = this.dividaAtiva.carregando;
  protected readonly erro = this.dividaAtiva.erro;
  protected readonly devedor = this.dividaAtiva.ultimoDevedor;
  protected readonly inscricao = this.dividaAtiva.ultimaInscricao;

  protected readonly ambienteEfetivo = this.serproAmbiente.ambienteEfetivo;
  /** Aviso consultivo permanente (ADR-0002): vem do resultado da consulta. */
  protected readonly avisoConsultivo = computed(() => {
    const d = this.dividaAtiva.ultimoDevedor();
    const i = this.dividaAtiva.ultimaInscricao();
    return d?.aviso ?? i?.aviso ?? null;
  });

  async ngOnInit(): Promise<void> {
    const clienteId = this.route.snapshot.queryParamMap.get('clienteId');
    if (clienteId) {
      try {
        const c = await this.clienteService.obter(clienteId);
        this.cliente.set(c);
        this.chave.set(c.cpfCnpj);
      } catch {
        // cliente não encontrado — segue sem pré-preenchimento
      }
    }
  }

  protected selecionarModo(m: ConsultaModo): void {
    this.modo.set(m);
    this.dividaAtiva.limpar();
  }

  protected atualizarChave(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.chave.set(target.value);
  }

  protected aplicarMascara(documento: string): string {
    if (documento.length === 11) {
      return documento.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, '$1.$2.$3-$4');
    }
    if (documento.length === 14) {
      return documento.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, '$1.$2.$3/$4-$5');
    }
    return documento;
  }

  protected async onSubmit(): Promise<void> {
    const valor = sanitizeDocument(this.chave());
    if (!valor) return;

    this.dividaAtiva.limpar();

    try {
      if (this.modo() === 'devedor') {
        await this.dividaAtiva.consultarDevedor(valor);
      } else {
        await this.dividaAtiva.consultarInscricao(valor);
      }
    } catch {
      // erro já definido pelo serviço
    }
  }

  protected limparErro(): void {
    this.dividaAtiva.limpar();
  }

  protected voltar(): void {
    const clienteId = this.cliente()?.id;
    if (clienteId) {
      void this.router.navigate(['/clientes', clienteId]);
    } else {
      void this.router.navigate(['/clientes']);
    }
  }
}
