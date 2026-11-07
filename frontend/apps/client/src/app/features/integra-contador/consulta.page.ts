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
import { IntegraContadorService } from '../../data/integra-contador.service';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { SessionStore } from '../../core/auth/session.store';
import { AccButtonComponent, AccCardComponent, AccEmptyStateComponent } from '@accurit/ui';

export type VerboIntegraContador = 'Consultar' | 'Declarar' | 'Emitir' | 'Apoiar' | 'Monitorar';

const VERBOS: { valor: VerboIntegraContador; label: string }[] = [
  { valor: 'Consultar', label: 'Consultar' },
  { valor: 'Declarar', label: 'Declarar' },
  { valor: 'Emitir', label: 'Emitir' },
  { valor: 'Apoiar', label: 'Apoiar' },
  { valor: 'Monitorar', label: 'Monitorar' },
];

interface ConsultaState {
  carregando: boolean;
  erro: string | null;
  resultado: any | null;
}

@Component({
  selector: 'app-integra-contador-consulta-page',
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
        <h1 class="page-title">Integra Contador</h1>
      </header>

      @if (cliente(); as c) {
        <p class="cliente-info">
          Contribuinte: <strong>{{ c.nomeRazaoSocial }}</strong>
          <span class="documento">{{ aplicarMascara(c.cpfCnpj) }}</span>
        </p>
      }

      @if (ambienteEfetivo(); as env) {
        <p class="ambiente-sel">
          Ambiente: {{ env === 'TRIAL' ? 'Trial (avaliação)' : 'Produção' }}
        </p>
      }

      <section class="section">
        <form (ngSubmit)="onSubmit()" class="consulta-form">
          <div class="field">
            <label for="verbo" class="label">Verbo</label>
            <select id="verbo"
              [(ngModel)]="verboSelecionado"
              class="input input--select"
            >
              @for (v of verbos; track v.valor) {
                <option [value]="v.valor">{{ v.label }}</option>
              }
            </select>
          </div>

          <div class="field">
            <label for="documento" class="label">Documento do contribuinte</label>
            <input
              id="documento"
              type="text"
              [value]="documento()"
              (input)="atualizarDocumento($event)"
              class="input"
              placeholder="000.000.000-00 ou 00.000.000/0000-00"
            />
          </div>

          <div class="field">
            <label for="idSistema" class="label">idSistema</label>
            <input
              id="idSistema"
              type="text"
              [value]="idSistema()"
              (input)="atualizarIdSistema($event)"
              class="input"
              placeholder="ex: SICALC"
            />
          </div>

          <div class="field">
            <label for="idServico" class="label">idServico</label>
            <input
              id="idServico"
              type="text"
              [value]="idServico()"
              (input)="atualizarIdServico($event)"
              class="input"
              placeholder="ex: CONSDECLARACAO13"
            />
          </div>

          <div class="field">
            <label for="versaoSistema" class="label">versaoSistema</label>
            <input
              id="versaoSistema"
              type="text"
              [value]="versaoSistema()"
              (input)="atualizarVersaoSistema($event)"
              class="input"
              placeholder="ex: 1"
            />
          </div>

          <div class="field">
            <label for="dados" class="label">dados (JSON string)</label>
            <textarea
              id="dados"
              [value]="dados()"
              (input)="atualizarDados($event)"
              class="input textarea"
              placeholder='ex: {"cnpj": "00000000000000"}'
            ></textarea>
          </div>

          <div class="field field--checkbox">
            <label class="label">
              <input
                type="checkbox"
                [checked]="confirmacaoExplicita()"
                (change)="atualizarConfirmacaoExplicita($event)"
              />
              Confirmação explícita de execução
            </label>
          </div>

          <button
            acc-button
            type="submit"
            variant="primary"
            [disabled]="carregando() || !documento().trim() || !idSistema().trim() || !idServico().trim()"
          >
            Executar
          </button>
        </form>
      </section>

      @if (carregando()) {
        <div class="carregando" aria-hidden="true">
          <p>Executando no Integra Contador...</p>
        </div>
      }

      @if (erro()) {
        <div class="alert-error" role="alert" aria-live="polite">{{ erro() }}</div>
        <button acc-button variant="outline" size="md" (click)="limparErro()">
          Limpar
        </button>
      }

      @if (resultado(); as r) {
        <acc-card class="resultado-card">
          <h2 class="resultado-titulo">Resultado da execução</h2>
          <dl class="resultado-lista">
            <div class="resultado-item">
              <dt>Verbo</dt>
              <dd>{{ r.verbo }}</dd>
            </div>
            <div class="resultado-item">
              <dt>idSistema</dt>
              <dd>{{ r.idSistema }}</dd>
            </div>
            <div class="resultado-item">
              <dt>idServico</dt>
              <dd>{{ r.idServico }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Estado</dt>
              <dd>{{ r.estado }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Ambiente</dt>
              <dd>{{ r.ambiente }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Status SERPRO</dt>
              <dd>{{ r.statusSerpro }}</dd>
            </div>
            <div class="resultado-item">
              <dt>Mensagem</dt>
              <dd>{{ r.mensagem }}</dd>
            </div>
            @if (r.referenciaResposta) {
              <div class="resultado-item">
                <dt>Referência de resposta</dt>
                <dd>{{ r.referenciaResposta }}</dd>
              </div>
            }
            @if (r.tempoEsperaSegundos !== null && r.tempoEsperaSegundos !== undefined) {
              <div class="resultado-item">
                <dt>Tempo de espera (s)</dt>
                <dd>{{ r.tempoEsperaSegundos }}</dd>
              </div>
            }
            @if (r.etag) {
              <div class="resultado-item">
                <dt>ETag</dt>
                <dd>{{ r.etag }}</dd>
              </div>
            }
            @if (r.expires) {
              <div class="resultado-item">
                <dt>Expira em</dt>
                <dd>{{ r.expires }}</dd>
              </div>
            }
            @if (r.codigosMensagem?.length) {
              <div class="resultado-item">
                <dt>Códigos de mensagem</dt>
                <dd>{{ r.codigosMensagem.join(', ') }}</dd>
              </div>
            }
          </dl>
        </acc-card>
      }

      @if (!carregando() && !erro() && !resultado() && documento()) {
        <acc-empty-state
          title="Sem resultado"
          descricao="Execute uma consulta no Integra Contador para ver o resultado aqui."
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

    .section {
      margin-bottom: 2rem;
    }

    .consulta-form {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      flex-wrap: wrap;
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
      min-width: 200px;
    }

    .field--checkbox {
      flex-direction: row;
      align-items: center;
      gap: 0.5rem;
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

    .input--select {
      appearance: none;
      background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'/%3e%3c/svg%3e");
      background-repeat: no-repeat;
      background-position: right 0.75rem center;
      background-size: 1rem;
      padding-right: 2.5rem;
    }

    textarea.input {
      min-height: 6rem;
      resize: vertical;
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
      word-break: break-word;
    }
  `,
})
export class IntegraContadorConsultaPage implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly integraContador = inject(IntegraContadorService);
  protected readonly serproAmbiente = inject(SerproAmbienteService);
  protected readonly sessao = inject(SessionStore);
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  protected readonly cliente = signal<ClienteDto | null>(null);
  protected readonly verboSelecionado = signal<VerboIntegraContador>('Consultar');
  protected readonly documento = signal<string>('');
  protected readonly idSistema = signal<string>('');
  protected readonly idServico = signal<string>('');
  protected readonly versaoSistema = signal<string>('');
  protected readonly dados = signal<string>('');
  protected readonly confirmacaoExplicita = signal<boolean>(false);

  protected readonly carregando = this.integraContador.carregando;
  protected readonly erro = this.integraContador.erro;
  protected readonly resultado = this.integraContador.ultimoResultado;

  protected readonly verbos = VERBOS;
  protected readonly ambienteEfetivo = this.serproAmbiente.ambienteEfetivo;

  async ngOnInit(): Promise<void> {
    const clienteId = this.route.snapshot.queryParamMap.get('clienteId');
    if (clienteId) {
      try {
        const c = await this.clienteService.obter(clienteId);
        this.cliente.set(c);
        this.documento.set(c.cpfCnpj);
      } catch {
        // cliente não encontrado — segue sem pré-preenchimento
      }
    }
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

  protected atualizarDocumento(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.documento.set(target.value);
  }

  protected atualizarIdSistema(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.idSistema.set(target.value);
  }

  protected atualizarIdServico(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.idServico.set(target.value);
  }

  protected atualizarVersaoSistema(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.versaoSistema.set(target.value);
  }

  protected atualizarDados(event: Event): void {
    const target = event.target as HTMLTextAreaElement;
    this.dados.set(target.value);
  }

  protected atualizarConfirmacaoExplicita(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.confirmacaoExplicita.set(target.checked);
  }

  protected async onSubmit(): Promise<void> {
    const doc = sanitizeDocument(this.documento());
    if (!doc) return;

    this.integraContador.limparResultado();

    try {
      await this.integraContador.executar({
        documentoContribuinte: doc,
        verbo: this.verboSelecionado(),
        pedidoDados: {
          idSistema: this.idSistema(),
          idServico: this.idServico(),
          versaoSistema: this.versaoSistema(),
          dados: this.dados(),
        },
        confirmacaoExplicita: this.confirmacaoExplicita(),
      });
    } catch {
      // erro já definido pelo serviço
    }
  }

  protected limparErro(): void {
    this.integraContador.limparResultado();
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
