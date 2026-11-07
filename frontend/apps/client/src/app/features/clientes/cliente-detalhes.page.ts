import {
  Component,
  inject,
  ChangeDetectionStrategy,
  OnInit,
  signal,
} from '@angular/core';
import {
  ReactiveFormsModule,
  FormBuilder,
  Validators,
} from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  UpdateClienteRequestDto,
  ClienteDto,
  TipoCliente,
  PorteRegime,
} from '@accurit/core';
import { sanitizeDocument } from '@accurit/shared';
import { ClienteService } from '../../data/cliente.service';
import { IntegraContadorService } from '../../data/integra-contador.service';
import { DividaAtivaService } from '../../data/divida-ativa.service';
import { AccButtonComponent, AccCardComponent } from '@accurit/ui';

export interface PendenciaDto {
  id: string;
  descricao: string;
  valor: number;
  vencimento: string;
  status: 'ABERTA' | 'PAGA' | 'VENCIDA';
  origem: string;
}

export interface ServicoIntegraContador {
  idSistema: string;
  idServico: string;
  versaoSistema?: string;
  verbo: string;
  habilitado: boolean;
}

const SERVICOS_INTEGRA_CONTADOR: ServicoIntegraContador[] = [
  { idSistema: 'SICALC', idServico: 'CONSULTAAPOIORECEITAS52', verbo: 'Apoiar', habilitado: false },
  { idSistema: 'PROCURACOES', idServico: 'OBTERPROCURACAO41', versaoSistema: '1', verbo: 'Consultar', habilitado: true },
  { idSistema: 'SITFIS', idServico: 'RELATORIOSITFIS92', verbo: 'Emitir', habilitado: false },
  { idSistema: 'PGDASD', idServico: 'CONSDECLARACAO13', verbo: 'Consultar', habilitado: false },
  { idSistema: 'PGDASD', idServico: 'TRANSDECLARACAO11', verbo: 'Declarar', habilitado: false },
  { idSistema: 'DCTFWEB', idServico: 'GERARGUIA31', verbo: 'Emitir', habilitado: false },
  { idSistema: 'CAIXAPOSTAL', idServico: 'INNOVAMSG63', verbo: 'Monitorar', habilitado: false },
  { idSistema: 'AUTENTICAPROCURADOR', idServico: 'ENVIOXMLASSINADO81', versaoSistema: '1.0', verbo: 'Apoiar', habilitado: false },
];

@Component({
  selector: 'app-cliente-detalhes-page',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    CommonModule,
    AccButtonComponent,
    AccCardComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-container">
      <header class="page-header">
        <button
          acc-button
          variant="ghost"
          size="sm"
          (click)="router.navigate(['/clientes'])"
        >
          ← Voltar à lista
        </button>
        <h1 class="page-title">Detalhes do Cliente</h1>
        @if (cliente()?.tipo) {
          <span class="type-badge">{{ cliente()?.tipo }}</span>
        }
      </header>

      @if (erroGeral()) {
        <div class="alert-error" role="alert">{{ erroGeral() }}</div>
      }

      <!-- Card de dados cadastrais -->
      <section class="section card-section">
        <acc-card class="card-cadastro">
          <h2 class="section-title">Dados cadastrais</h2>

          <form [formGroup]="form" (ngSubmit)="onSalvar()" class="cliente-form">
            <div class="field">
              <label for="tipo" class="label">Tipo de Pessoa</label>
              <select
                id="tipo"
                formControlName="tipo"
                class="input input--select"
                [disabled]="carregando()"
              >
                <option value="PF">Pessoa Física (CPF)</option>
                <option value="PJ">Pessoa Jurídica (CNPJ)</option>
              </select>
              @if (isInvalid('tipo')) {
                <span class="field-error">Tipo é obrigatório.</span>
              }
            </div>

            <div class="field">
              <label for="cpfCnpj" class="label">CPF / CNPJ</label>
              <input
                id="cpfCnpj"
                type="text"
                formControlName="cpfCnpj"
                autocomplete="off"
                class="input"
                [class.input--error]="isInvalid('cpfCnpj')"
                placeholder="000.000.000-00 ou 00.000.000/0000-00"
                [disabled]="carregando()"
              />
              @if (isInvalid('cpfCnpj')) {
                <span class="field-error">
                  {{ cpfCnpjErro() ?? 'Informe um CPF/CNPJ válido.' }}
                </span>
              }
            </div>

            <div class="field">
              <label for="nomeRazaoSocial" class="label">Nome / Razão Social</label>
              <input
                id="nomeRazaoSocial"
                type="text"
                formControlName="nomeRazaoSocial"
                autocomplete="off"
                class="input"
                [class.input--error]="isInvalid('nomeRazaoSocial')"
                placeholder="ex: João Silva ou Silva Contabilidade Ltda."
                [disabled]="carregando()"
              />
              @if (isInvalid('nomeRazaoSocial')) {
                <span class="field-error">Nome é obrigatório.</span>
              }
            </div>

            <div class="field">
              <label for="nomeFantasia" class="label">Nome Fantasia</label>
              <input
                id="nomeFantasia"
                type="text"
                formControlName="nomeFantasia"
                autocomplete="off"
                class="input"
                placeholder="ex: Silva & Associados (opcional)"
                [disabled]="carregando()"
              />
            </div>

            <div class="field">
              <label for="porteRegime" class="label">Porte / Regime Tributário</label>
              <select
                id="porteRegime"
                formControlName="porteRegime"
                class="input input--select"
                [disabled]="carregando()"
              >
                <option value="">Selecione (opcional)</option>
                <option value="MEI">MEI</option>
                <option value="SIMPLES_NACIONAL">Simples Nacional</option>
                <option value="LUCRO_PRESUMIDO">Lucro Presumido</option>
                <option value="LUCRO_REAL">Lucro Real</option>
              </select>
            </div>

            <div class="field">
              <label for="email" class="label">E-mail</label>
              <input
                id="email"
                type="email"
                formControlName="email"
                autocomplete="email"
                class="input"
                [class.input--error]="isInvalid('email')"
                placeholder="cliente@email.com.br"
                [disabled]="carregando()"
              />
              @if (isInvalid('email')) {
                <span class="field-error">E-mail inválido.</span>
              }
            </div>

            <div class="field">
              <label for="telefone" class="label">Telefone</label>
              <input
                id="telefone"
                type="tel"
                formControlName="telefone"
                autocomplete="tel"
                class="input"
                [class.input--error]="isInvalid('telefone')"
                placeholder="(11) 99999-9999"
                [disabled]="carregando()"
              />
              @if (isInvalid('telefone')) {
                <span class="field-error">
                  Telefone inválido. Informe 10 ou 11 dígitos.
                </span>
              }
            </div>

            <div class="form-actions">
              <button
                acc-button
                type="button"
                variant="danger"
                size="md"
                (click)="confirmarExclusao()"
                [disabled]="carregando()"
              >
                Excluir cliente
              </button>
              <button
                acc-button
                type="submit"
                variant="primary"
                size="md"
                [loading]="carregando()"
                [disabled]="form.invalid || carregando()"
              >
                Salvar alterações
              </button>
              <button
                acc-button
                type="button"
                variant="outline"
                size="md"
                (click)="onCancelar()"
                [disabled]="carregando()"
              >
                Cancelar
              </button>
            </div>
          </form>
        </acc-card>
      </section>

      <!-- Pendências -->
      <section class="section">
        <div class="section-header">
          <h2 class="section-title">Pendências</h2>
          <button
            acc-button
            variant="primary"
            size="sm"
            (click)="abrirNovaPendencia()"
            [disabled]="carregando()"
          >
            + Nova Pendência
          </button>
        </div>

        @if (pendencias().length === 0) {
          <div class="empty-state">
            <p>Nenhuma pendência cadastrada para este cliente.</p>
          </div>
        } @else {
          <table class="acc-table">
            <thead>
              <tr>
                <th>Descrição</th>
                <th>Valor</th>
                <th>Vencimento</th>
                <th>Status</th>
                <th>Origem</th>
                <th>Ações</th>
              </tr>
            </thead>
            <tbody>
              @for (p of pendencias(); track p.id) {
                <tr>
                  <td>{{ p.descricao }}</td>
                  <td>R$ {{ p.valor.toLocaleString('pt-BR', { minimumFractionDigits: 2 }) }}</td>
                  <td>{{ formatarData(p.vencimento) }}</td>
                  <td>
                    <span
                      class="status-badge"
                      [class.status-aberta]="p.status === 'ABERTA'"
                      [class.status-paga]="p.status === 'PAGA'"
                      [class.status-vencida]="p.status === 'VENCIDA'"
                    >
                      {{ p.status }}
                    </span>
                  </td>
                  <td>{{ p.origem }}</td>
                  <td>
                    <button
                      acc-button
                      variant="ghost"
                      size="sm"
                      (click)="excluirPendencia(p.id)"
                    >
                      Excluir
                    </button>
                  </td>
                </tr>
              } @empty {
                <tr>
                  <td colspan="6">Nenhuma pendência encontrada.</td>
                </tr>
              }
            </tbody>
          </table>
        }

        <!-- Modal nova pendência -->
        @if (novaPendenciaAberta()) {
          <div class="overlay" (click)="fecharNovaPendencia()"></div>
          <div class="modal">
            <div class="modal-header">
              <h3 class="modal-title">Nova Pendência</h3>
              <button acc-button variant="ghost" size="sm" (click)="fecharNovaPendencia()">×</button>
            </div>
            <form [formGroup]="formNovaPendencia" (ngSubmit)="onSalvarPendencia()" class="modal-form">
              <div class="field">
                <label for="pendDescricao" class="label">Descrição</label>
                <input
                  id="pendDescricao"
                  type="text"
                  formControlName="descricao"
                  class="input"
                  placeholder="ex: Débito de ICMS"
                  required
                />
              </div>
              <div class="field">
                <label for="pendValor" class="label">Valor (R$)</label>
                <input
                  id="pendValor"
                  type="number"
                  formControlName="valor"
                  class="input"
                  placeholder="0,00"
                  step="0.01"
                  min="0"
                  required
                />
              </div>
              <div class="field">
                <label for="pendVencimento" class="label">Vencimento</label>
                <input
                  id="pendVencimento"
                  type="date"
                  formControlName="vencimento"
                  class="input"
                  required
                />
              </div>
              <div class="field">
                <label for="pendStatus" class="label">Status</label>
                <select id="pendStatus" formControlName="status" class="input input--select">
                  <option value="ABERTA">Aberta</option>
                  <option value="PAGA">Paga</option>
                  <option value="VENCIDA">Vencida</option>
                </select>
              </div>
              <div class="field">
                <label for="pendOrigem" class="label">Origem</label>
                <input
                  id="pendOrigem"
                  type="text"
                  formControlName="origem"
                  class="input"
                  placeholder="ex: Dívida Ativa"
                  required
                />
              </div>
              <div class="modal-actions">
                <button
                  acc-button
                  type="submit"
                  variant="primary"
                  size="md"
                  [disabled]="formNovaPendencia.invalid"
                >
                  Salvar
                </button>
                <button
                  acc-button
                  type="button"
                  variant="outline"
                  size="md"
                  (click)="fecharNovaPendencia()"
                >
                  Cancelar
                </button>
              </div>
            </form>
          </div>
        }

        <!-- Modal confirmação de exclusão -->
        @if (confirmarExclusaoAberta()) {
          <div class="overlay" (click)="fecharConfirmarExclusao()"></div>
          <div class="modal">
            <div class="modal-header">
              <h3 class="modal-title">Excluir cliente</h3>
              <button acc-button variant="ghost" size="sm" (click)="fecharConfirmarExclusao()">×</button>
            </div>
            <div class="modal-body">
              <p>Tem certeza que deseja excluir o cliente <strong>{{ cliente()?.nomeRazaoSocial }}</strong>?</p>
              <p class="aviso-exclusao">Esta ação não pode ser desfeita.</p>
            </div>
            <div class="modal-actions">
              <button
                acc-button
                type="button"
                variant="outline"
                size="md"
                (click)="fecharConfirmarExclusao()"
              >
                Cancelar
              </button>
              <button
                acc-button
                type="button"
                variant="danger"
                size="md"
                (click)="excluirCliente()"
                [disabled]="carregando()"
              >
                Excluir
              </button>
            </div>
          </div>
        }
      </section>

      <!-- Integra Contador -->
      <section class="section section-integracao">
        <h2 class="section-title">Integra Contador (SERPRO)</h2>
        <p class="section-desc">
          Executa chamadas nos serviços do Integra Contador para este cliente.
        </p>

        @if (integra.erro()) {
          <div class="alert-error" role="alert">{{ integra.erro() }}</div>
        }

        <form [formGroup]="formIntegra" (ngSubmit)="onExecutarIntegra()" class="integra-form">
          <div class="field">
            <label for="docContribuinte" class="label">Documento do Contribuinte</label>
            <input
              id="docContribuinte"
              type="text"
              formControlName="documentoContribuinte"
              autocomplete="off"
              class="input"
              placeholder="CPF ou CNPJ do contribuinte"
            />
          </div>

          <div class="field">
            <label for="idSistema" class="label">ID do Sistema</label>
            <select id="idSistema" formControlName="idSistema" class="input input--select">
              <option value="">Selecione o sistema</option>
              @for (s of servicosIntegra(); track s.idSistema + s.idServico) {
                <option [value]="s.idSistema">{{ s.idSistema }}</option>
              }
            </select>
          </div>

          <div class="field">
            <label for="idServico" class="label">ID do Serviço</label>
            <select id="idServico" formControlName="idServico" class="input input--select">
              <option value="">Selecione o serviço</option>
              @for (s of servicosFiltrados(); track s.idServico) {
                <option [value]="s.idServico">{{ s.idServico }}</option>
              }
            </select>
          </div>

          <div class="field">
            <label for="verbo" class="label">Verbo</label>
            <select id="verbo" formControlName="verbo" class="input input--select">
              <option value="Consultar">Consultar</option>
              <option value="Declarar">Declarar</option>
              <option value="Emitir">Emitir</option>
              <option value="Apoiar">Apoiar</option>
              <option value="Monitorar">Monitorar</option>
            </select>
          </div>

          <div class="field">
            <label for="versaoSistema" class="label">Versão do Sistema</label>
            <input
              id="versaoSistema"
              type="text"
              formControlName="versaoSistema"
              class="input"
              placeholder="ex: 1"
            />
          </div>

          <div class="field">
            <label for="dados" class="label">Dados (JSON)</label>
            <textarea
              id="dados"
              formControlName="dados"
              class="input input--textarea"
              rows="4"
              placeholder='ex: {"outorgante": "00000000000", ...}'
            ></textarea>
          </div>

          <div class="field field-checkbox">
            <input
              id="confirmacaoExplicita"
              type="checkbox"
              formControlName="confirmacaoExplicita"
            />
            <label for="confirmacaoExplicita" class="label">
              Confirmação explícita
            </label>
          </div>

          <div class="field">
            <label for="chaveIdempotencia" class="label">Chave de Idempotência (opcional)</label>
            <input
              id="chaveIdempotencia"
              type="text"
              formControlName="chaveIdempotencia"
              class="input"
              placeholder="ex: req-001"
            />
          </div>

          <div class="form-actions">
            <button
              acc-button
              type="submit"
              variant="primary"
              size="md"
              [loading]="integra.carregando()"
              [disabled]="formIntegra.invalid || integra.carregando()"
            >
              Executar Integra Contador
            </button>
          </div>
        </form>

        @if (integra.ultimoResultado()) {
          <div class="resultado-card">
            <h3>Resultado da execução</h3>
            <div class="resultado-grid">
              <div class="resultado-item">
                <span class="resultado-label">Ambiente</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.ambiente }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Verbo</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.verbo }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Sistema</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.idSistema }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Serviço</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.idServico }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Estado</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.estado }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Status SERPRO</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.statusSerpro }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Mensagem</span>
                <span class="resultado-valor">{{ integra.ultimoResultado()?.mensagem }}</span>
              </div>
            </div>
            @if (integra.ultimoResultado()?.codigosMensagem?.length) {
              <div class="resultado-item resultado-codigos">
                <span class="resultado-label">Códigos</span>
                <span class="resultado-valor">
                  {{ integra.ultimoResultado()?.codigosMensagem.join(', ') }}
                </span>
              </div>
            }
            <button
              acc-button
              variant="outline"
              size="sm"
              (click)="integra.limparResultado()"
            >
              Limpar resultado
            </button>
          </div>
        }
      </section>

      <!-- Dívida Ativa -->
      <section class="section section-divida">
        <h2 class="section-title">Dívida Ativa (SERPRO)</h2>
        <p class="section-desc">
          Consulta pendências do cliente na Dívida Ativa da União.
        </p>

        @if (divida.erro()) {
          <div class="alert-error" role="alert">{{ divida.erro() }}</div>
        }

        <form [formGroup]="formDivida" (ngSubmit)="onConsultarDivida()" class="divida-form">
          <div class="field">
            <label for="dividaDoc" class="label">CPF / CNPJ para consulta</label>
            <input
              id="dividaDoc"
              type="text"
              formControlName="documento"
              autocomplete="off"
              class="input"
              placeholder="000.000.000-00 ou 00.000.000/0000-00"
            />
          </div>

          <div class="form-actions">
            <button
              acc-button
              type="submit"
              variant="primary"
              size="md"
              [loading]="divida.carregando()"
              [disabled]="formDivida.invalid || divida.carregando()"
            >
              Consultar devedor
            </button>
          </div>
        </form>

        @if (divida.ultimoDevedor()) {
          <div class="resultado-card">
            <h3>Resultado - Devedor</h3>
            <div class="resultado-grid">
              <div class="resultado-item">
                <span class="resultado-label">Ambiente</span>
                <span class="resultado-valor">{{ divida.ultimoDevedor()?.ambiente }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Estado</span>
                <span class="resultado-valor">{{ divida.ultimoDevedor()?.estado }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Fonte</span>
                <span class="resultado-valor">{{ divida.ultimoDevedor()?.fonteConsulta }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Chave</span>
                <span class="resultado-valor">{{ divida.ultimoDevedor()?.chaveMascarada }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Horário</span>
                <span class="resultado-valor">{{ divida.ultimoDevedor()?.horarioConsulta }}</span>
              </div>
            </div>
            <div class="inscricoes-list">
              <h4>Inscrições ({{ divida.ultimoDevedor()?.inscricoes.length }})</h4>
              <table class="acc-table">
                <thead>
                  <tr>
                    <th>Número</th>
                    <th>Situação</th>
                    <th>Nome do Devedor</th>
                    <th>Valor Consolidado</th>
                  </tr>
                </thead>
                <tbody>
                  @for (insc of divida.ultimoDevedor()?.inscricoes; track insc.numeroInscricao) {
                    <tr>
                      <td>{{ insc.numeroInscricao }}</td>
                      <td>{{ insc.situacaoDescricao }}</td>
                      <td>{{ insc.nomeDevedor ?? '—' }}</td>
                      <td>{{ insc.valorTotalConsolidado.valor }} {{ insc.valorTotalConsolidado.remarca }}</td>
                    </tr>
                  } @empty {
                    <tr>
                      <td colspan="4">Nenhuma inscrição encontrada.</td>
                    </tr>
                  }
                </tbody>
              </table>
            </div>
          </div>
        }

        <div class="divida-divider">
          <span>Ou consultar por inscrição</span>
        </div>

        <form [formGroup]="formInscricao" (ngSubmit)="onConsultarInscricao()" class="divida-form">
          <div class="field">
            <label for="dividaInscricao" class="label">Número da Inscrição</label>
            <input
              id="dividaInscricao"
              type="text"
              formControlName="numeroInscricao"
              autocomplete="off"
              class="input"
              placeholder="ex: 12345678901234"
            />
          </div>

          <div class="form-actions">
            <button
              acc-button
              type="submit"
              variant="primary"
              size="md"
              [loading]="divida.carregando()"
              [disabled]="formInscricao.invalid || divida.carregando()"
            >
              Consultar inscrição
            </button>
          </div>
        </form>

        @if (divida.ultimaInscricao()) {
          <div class="resultado-card">
            <h3>Resultado - Inscrição</h3>
            <div class="resultado-grid">
              <div class="resultado-item">
                <span class="resultado-label">Número</span>
                <span class="resultado-valor">{{ divida.ultimaInscricao()?.numeroInscricao }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Situação</span>
                <span class="resultado-valor">{{ divida.ultimaInscricao()?.situacao }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Processo</span>
                <span class="resultado-valor">{{ divida.ultimaInscricao()?.processoAdministrativo ?? '—' }}</span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Valor Inscrito</span>
                <span class="resultado-valor">
                  {{ divida.ultimaInscricao()?.valorTotalInscrito?.valor ?? '—' }} {{ divida.ultimaInscricao()?.valorTotalInscrito?.remarca ?? '' }}
                </span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Valor Consolidado</span>
                <span class="resultado-valor">
                  {{ divida.ultimaInscricao()?.valorTotalConsolidado?.valor ?? '—' }} {{ divida.ultimaInscricao()?.valorTotalConsolidado?.remarca ?? '' }}
                </span>
              </div>
              <div class="resultado-item">
                <span class="resultado-label">Data Inscrição</span>
                <span class="resultado-valor">{{ divida.ultimaInscricao()?.dataInscricao ?? '—' }}</span>
              </div>
            </div>
          </div>
        }
      </section>
    </div>
  `,
  styles: `
    .page-container {
      max-width: 1000px;
      margin: 0 auto;
      padding: 1.5rem;
    }

    .page-header {
      display: flex;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1.5rem;
    }

    .page-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .type-badge {
      margin-left: auto;
      padding: 0.25rem 0.75rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-bg);
      color: var(--acc-color-text-muted);
      font-size: 0.875rem;
      font-weight: 600;
    }

    .section {
      margin-bottom: 2rem;
    }

    .card-section .card-cadastro {
      margin-bottom: 1rem;
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
    }

    .section-title {
      font-size: 1.125rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .section-desc {
      font-size: 0.875rem;
      color: var(--acc-color-text-muted);
      margin: 0 0 1rem;
    }

    .cliente-form,
    .integra-form,
    .divida-form,
    .modal-form {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
    }

    .field-checkbox {
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
      background-color: var(--acc-color-input-bg, var(--acc-color-surface));
    }

    .input:focus {
      border-color: var(--acc-color-primary);
      box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.1);
    }

    .input--error {
      border-color: var(--acc-color-danger);
    }

    .input--select {
      appearance: none;
      background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'/%3e%3c/svg%3e");
      background-repeat: no-repeat;
      background-position: right 0.75rem center;
      background-size: 1rem;
      padding-right: 2.5rem;
    }

    .input--textarea {
      font-family: inherit;
      resize: vertical;
    }

    .field-error {
      font-size: 0.75rem;
      color: var(--acc-color-danger);
    }

    .alert-error {
      padding: 0.75rem 1rem;
      border-radius: var(--acc-radius-md);
      background-color: var(--acc-color-danger-bg);
      color: var(--acc-color-danger);
      font-size: 0.875rem;
      margin-bottom: 1rem;
    }

    .form-actions {
      display: flex;
      gap: 0.75rem;
      margin-top: 0.5rem;
    }

    .empty-state {
      padding: 2rem;
      text-align: center;
      color: var(--acc-color-text-muted);
    }

    .status-badge {
      display: inline-flex;
      align-items: center;
      padding: 0.25rem 0.625rem;
      border-radius: var(--acc-radius-md);
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
    }

    .status-aberta {
      background-color: #fef3c7;
      color: #92400e;
    }

    .status-paga {
      background-color: #d1fae5;
      color: #065f46;
    }

    .status-vencida {
      background-color: #fee2e2;
      color: #991b1b;
    }

    .resultado-card {
      margin-top: 1rem;
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-md);
      padding: 1rem;
      background-color: var(--acc-color-bg);
    }

    .resultado-card h3 {
      font-size: 1rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 1rem;
    }

    .resultado-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 0.75rem;
      margin-bottom: 1rem;
    }

    .resultado-item {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .resultado-label {
      font-size: 0.75rem;
      color: var(--acc-color-text-muted);
    }

    .resultado-valor {
      font-size: 0.875rem;
      color: var(--acc-color-text-main);
      word-break: break-word;
    }

    .resultado-codigos {
      grid-column: 1 / -1;
    }

    .inscricoes-list {
      margin-top: 1rem;
    }

    .inscricoes-list h4 {
      font-size: 0.9375rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0 0 0.5rem;
    }

    .acc-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.875rem;
    }

    .acc-table th {
      padding: 0.5rem 0.75rem;
      border-bottom: 1px solid var(--acc-color-border);
      color: var(--acc-color-text-muted);
      font-weight: 600;
      text-align: left;
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .acc-table td {
      padding: 0.5rem 0.75rem;
      border-bottom: 1px solid var(--acc-color-border);
    }

    .divida-divider {
      display: flex;
      align-items: center;
      text-align: center;
      margin: 1.5rem 0;
      color: var(--acc-color-text-muted);
      font-size: 0.875rem;
    }

    .divida-divider::before,
    .divida-divider::after {
      content: '';
      flex: 1;
      border-bottom: 1px solid var(--acc-color-border);
    }

    .divida-divider:not(:empty)::before {
      margin-right: 0.75rem;
    }

    .divida-divider:not(:empty)::after {
      margin-left: 0.75rem;
    }

    /* Modal nova pendência */
    .overlay {
      position: fixed;
      inset: 0;
      background-color: rgba(0, 0, 0, 0.45);
      backdrop-filter: blur(2px);
      z-index: 1000;
    }

    .modal {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      width: 100%;
      max-width: 480px;
      background-color: var(--acc-color-surface);
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-lg);
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.15);
      z-index: 1001;
      animation: slideUp 0.25s ease-out;
      display: flex;
      flex-direction: column;
    }

    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem 1.5rem 1rem;
      border-bottom: 1px solid var(--acc-color-border);
    }

    .modal-body {
      padding: 1.5rem;
    }

    .modal-title {
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .modal-actions {
      display: flex;
      gap: 0.75rem;
      justify-content: flex-end;
      padding: 1rem 1.5rem 1.5rem;
      border-top: 1px solid var(--acc-color-border);
    }

    .aviso-exclusao {
      margin-top: 0.5rem;
      font-size: 0.8125rem;
      color: var(--acc-color-text-muted);
    }

    @keyframes slideUp {
      from { opacity: 0; transform: translate(-50%, -40%); }
      to { opacity: 1; transform: translate(-50%, -50%); }
    }

    @media (max-width: 640px) {
      .page-container {
        padding: 1rem;
      }

      .form-actions,
      .modal-actions {
        flex-direction: column;
      }

      .form-actions button,
      .modal-actions button {
        width: 100%;
      }
    }
  `,
})
export class ClienteDetalhesPage implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly integra = inject(IntegraContadorService);
  protected readonly divida = inject(DividaAtivaService);
  protected readonly router = inject(Router);
  protected readonly route = inject(ActivatedRoute);
  protected readonly fb = inject(FormBuilder);

  protected readonly cliente = signal<ClienteDto | null>(null);

  protected readonly carregando = this.clienteService.carregando;
  protected readonly erroGeral = this.clienteService.erro;

  // Confirmação de exclusão
  protected readonly confirmarExclusaoAberta = signal(false);

  // Pendências locais (simulação, sem backend)
  protected readonly pendencias = signal<PendenciaDto[]>([
    {
      id: '1',
      descricao: 'Débito de ICMS - Referente a 2024',
      valor: 1250.00,
      vencimento: '2025-06-30',
      status: 'ABERTA',
      origem: 'Dívida Ativa',
    },
    {
      id: '2',
      descricao: 'Multa por atraso na entrega de declaração',
      valor: 520.50,
      vencimento: '2025-05-15',
      status: 'PAGA',
      origem: 'SICALC',
    },
    {
      id: '3',
      descricao: 'IPTU comercial atrasado',
      valor: 890.00,
      vencimento: '2025-04-10',
      status: 'VENCIDA',
      origem: 'Municipal',
    },
  ]);

  protected readonly novaPendenciaAberta = signal(false);

  protected readonly servicosIntegra = signal<ServicoIntegraContador[]>(SERVICOS_INTEGRA_CONTADOR);

  protected readonly form = this.fb.group({
    tipo: ['', [Validators.required]],
    cpfCnpj: ['', [Validators.required]],
    nomeRazaoSocial: ['', [Validators.required, Validators.minLength(3)]],
    nomeFantasia: [''],
    porteRegime: [''],
    email: ['', [Validators.email]],
    telefone: [''],
  });

  protected readonly formIntegra = this.fb.group({
    documentoContribuinte: ['', [Validators.required]],
    idSistema: ['', [Validators.required]],
    idServico: ['', [Validators.required]],
    verbo: ['', [Validators.required]],
    versaoSistema: [''],
    dados: ['', [Validators.required]],
    confirmacaoExplicita: [false],
    chaveIdempotencia: [''],
  });

  protected readonly formDivida = this.fb.group({
    documento: ['', [Validators.required]],
  });

  protected readonly formInscricao = this.fb.group({
    numeroInscricao: ['', [Validators.required]],
  });

  protected readonly formNovaPendencia = this.fb.group({
    descricao: ['', [Validators.required]],
    valor: [null, [Validators.required, Validators.min(0)]],
    vencimento: ['', [Validators.required]],
    status: ['ABERTA', [Validators.required]],
    origem: ['', [Validators.required]],
  });

  ngOnInit(): void {
    const id = this.route.snapshot.paramMap.get('id');
    if (id) {
      this.carregarCliente(id);
    }

    // Quando o sistema muda, filtra os serviços disponíveis
    this.formIntegra.get('idSistema')?.valueChanges?.subscribe(() => {
      this.formIntegra.get('idServico')?.setValue('');
      this.formIntegra.get('verbo')?.setValue('');
      this.formIntegra.get('versaoSistema')?.setValue('');
    });

    // Quando o serviço muda, preenche verbo e versão automaticamente
    this.formIntegra.get('idServico')?.valueChanges?.subscribe((idServico) => {
      const servico = this.servicosIntegra().find((s) => s.idServico === idServico);
      if (servico) {
        this.formIntegra.get('verbo')?.setValue(servico.verbo, { emitEvent: false });
        this.formIntegra.get('versaoSistema')?.setValue(servico.versaoSistema ?? '', { emitEvent: false });
      }
    });
  }

  protected servicosFiltrados(): ServicoIntegraContador[] {
    const sistema = this.formIntegra.get('idSistema')?.value;
    if (!sistema) return [];
    return this.servicosIntegra().filter((s) => s.idSistema === sistema);
  }

  protected isInvalid(field: string): boolean {
    const c = this.form.get(field);
    return !!(c?.invalid && c.touched);
  }

  protected cpfCnpjErro(): string | null {
    const c = this.form.get('cpfCnpj');
    if (c?.touched && c?.invalid) {
      return 'CPF/CNPJ inválido.';
    }
    return null;
  }

  private async carregarCliente(id: string): Promise<void> {
    try {
      const cliente = await this.clienteService.obter(id);
      this.cliente.set(cliente);
      this.form.patchValue({
        tipo: cliente.tipo,
        cpfCnpj: cliente.cpfCnpj,
        nomeRazaoSocial: cliente.nomeRazaoSocial,
        nomeFantasia: cliente.nomeFantasia ?? '',
        porteRegime: cliente.porteRegime ?? '',
        email: cliente.email ?? '',
        telefone: cliente.telefone ?? '',
      });
    } catch (err: any) {
      this.clienteService.definirErro(err.message ?? 'Falha ao carregar cliente.');
    }
  }

  protected async onSalvar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const id = this.route.snapshot.paramMap.get('id');
    if (!id) return;

    const cpfCnpj = sanitizeDocument(this.form.get('cpfCnpj')?.value ?? '');
    const telefone = (this.form.get('telefone')?.value ?? '').replace(/\D/g, '');
    const email = (this.form.get('email')?.value ?? '').trim().toLowerCase() || undefined;

    const payload: UpdateClienteRequestDto = {
      tipo: (this.form.get('tipo')?.value ?? 'PF') as TipoCliente,
      porteRegime: (this.form.get('porteRegime')?.value || undefined) as PorteRegime | undefined,
      cpfCnpj,
      nomeRazaoSocial: (this.form.get('nomeRazaoSocial')?.value ?? '').trim(),
      nomeFantasia: (this.form.get('nomeFantasia')?.value ?? '').trim() || undefined,
      email,
      telefone: telefone || undefined,
    };

    try {
      await this.clienteService.atualizar(id, payload);
      this.router.navigate(['/clientes']);
    } catch {
      // erro já é exibido pelo clienteService.erro()
    }
  }

  protected onCancelar(): void {
    this.router.navigate(['/clientes']);
  }

  protected async onExecutarIntegra(): Promise<void> {
    if (this.formIntegra.invalid) {
      this.formIntegra.markAllAsTouched();
      return;
    }

    const raw = this.formIntegra.getRawValue();
    const request = {
      documentoContribuinte: raw.documentoContribuinte!,
      verbo: raw.verbo as 'Consultar' | 'Declarar' | 'Emitir' | 'Apoiar' | 'Monitorar',
      pedidoDados: {
        idSistema: raw.idSistema!,
        idServico: raw.idServico!,
        versaoSistema: raw.versaoSistema!,
        dados: raw.dados!,
      },
      confirmacaoExplicita: raw.confirmacaoExplicita ?? false,
      chaveIdempotencia: raw.chaveIdempotencia || undefined,
    };

    try {
      await this.integra.executar(request);
    } catch {
      // erro já é exibido pelo integra.erro()
    }
  }

  protected async onConsultarDivida(): Promise<void> {
    if (this.formDivida.invalid) {
      this.formDivida.markAllAsTouched();
      return;
    }

    try {
      await this.divida.consultarDevedor(
        this.formDivida.get('documento')?.value ?? ''
      );
    } catch {
      // erro já é exibido pelo divida.erro()
    }
  }

  protected async onConsultarInscricao(): Promise<void> {
    if (this.formInscricao.invalid) {
      this.formInscricao.markAllAsTouched();
      return;
    }

    try {
      await this.divida.consultarInscricao(
        this.formInscricao.get('numeroInscricao')?.value ?? ''
      );
    } catch {
      // erro já é exibido pelo divida.erro()
    }
  }

  // === Pendências ===

  protected abrirNovaPendencia(): void {
    this.formNovaPendencia.reset({
      descricao: '',
      valor: null,
      vencimento: '',
      status: 'ABERTA',
      origem: '',
    });
    this.novaPendenciaAberta.set(true);
  }

  protected fecharNovaPendencia(): void {
    this.novaPendenciaAberta.set(false);
  }

  protected confirmarExclusao(): void {
    this.confirmarExclusaoAberta.set(true);
  }

  protected fecharConfirmarExclusao(): void {
    this.confirmarExclusaoAberta.set(false);
  }

  protected async excluirCliente(): Promise<void> {
    const id = this.route.snapshot.paramMap.get('id');
    if (!id) return;

    try {
      await this.clienteService.excluir(id);
      this.router.navigate(['/clientes']);
    } catch {
      // erro já é exibido pelo clienteService.erro()
      this.fecharConfirmarExclusao();
    }
  }

  protected async onSalvarPendencia(): Promise<void> {
    if (this.formNovaPendencia.invalid) {
      this.formNovaPendencia.markAllAsTouched();
      return;
    }

    const raw = this.formNovaPendencia.getRawValue();
    const nova: PendenciaDto = {
      id: Date.now().toString(),
      descricao: raw.descricao!,
      valor: raw.valor!,
      vencimento: raw.vencimento!,
      status: raw.status as 'ABERTA' | 'PAGA' | 'VENCIDA',
      origem: raw.origem!,
    };

    this.pendencias.update((atual) => [nova, ...atual]);
    this.fecharNovaPendencia();
  }

  protected excluirPendencia(id: string): void {
    this.pendencias.update((atual) => atual.filter((p) => p.id !== id));
  }

  protected formatarData(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    });
  }
}
