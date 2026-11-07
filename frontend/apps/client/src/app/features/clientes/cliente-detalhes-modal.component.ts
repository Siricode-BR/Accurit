import {
  Component,
  inject,
  Input,
  Output,
  EventEmitter,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { AccButtonComponent } from '@accurit/ui';
import { formatPtBrDateTime } from '@accurit/shared';
import { ClienteDto } from '@accurit/core';

@Component({
  selector: 'app-cliente-detalhes-modal',
  standalone: true,
  imports: [CommonModule, AccButtonComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="overlay" (click)="fechar.emit()"></div>

    <div class="modal" role="dialog" aria-modal="true" aria-label="Detalhes do cliente">
      <div class="modal-header">
        <h2 class="modal-title">Detalhes do Cliente</h2>
        <button acc-button variant="ghost" size="sm" (click)="fechar.emit()">
          ×
        </button>
      </div>

      <div class="modal-body">
        <div class="detalhe-row">
          <span class="detalhe-label">Tipo</span>
          <span class="detalhe-valor">{{ cliente.tipo }}</span>
        </div>

        <div class="detalhe-row">
          <span class="detalhe-label">CPF / CNPJ</span>
          <span class="detalhe-valor">{{ formatarDoc(cliente.cpfCnpj) }}</span>
        </div>

        <div class="detalhe-row">
          <span class="detalhe-label">Nome / Razão Social</span>
          <span class="detalhe-valor">{{ cliente.nomeRazaoSocial }}</span>
        </div>

        @if (cliente.nomeFantasia) {
          <div class="detalhe-row">
            <span class="detalhe-label">Nome Fantasia</span>
            <span class="detalhe-valor">{{ cliente.nomeFantasia }}</span>
          </div>
        }

        @if (cliente.porteRegime) {
          <div class="detalhe-row">
            <span class="detalhe-label">Porte / Regime Tributário</span>
            <span class="detalhe-valor">{{ cliente.porteRegime }}</span>
          </div>
        }

        @if (cliente.email) {
          <div class="detalhe-row">
            <span class="detalhe-label">E-mail</span>
            <span class="detalhe-valor">{{ cliente.email }}</span>
          </div>
        }

        @if (cliente.telefone) {
          <div class="detalhe-row">
            <span class="detalhe-label">Telefone</span>
            <span class="detalhe-valor">{{ cliente.telefone }}</span>
          </div>
        }

        <div class="detalhe-row">
          <span class="detalhe-label">Criado em</span>
          <span class="detalhe-valor">{{ formatarData(cliente.criadoEm) }}</span>
        </div>

        <div class="detalhe-row">
          <span class="detalhe-label">Atualizado em</span>
          <span class="detalhe-valor">{{ formatarData(cliente.atualizadoEm) }}</span>
        </div>
      </div>

      <div class="modal-footer">
        <button acc-button variant="outline" size="md" (click)="fechar.emit()">
          Fechar
        </button>
      </div>
    </div>
  `,
  styles: `
    :host {
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      z-index: 1000;
      display: block;
    }

    .overlay {
      position: fixed;
      inset: 0;
      background-color: rgba(0, 0, 0, 0.45);
      backdrop-filter: blur(2px);
      animation: fadeIn 0.2s ease;
    }

    .modal {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      width: 100%;
      max-width: 480px;
      max-height: 90vh;
      background-color: var(--acc-color-surface);
      border: 1px solid var(--acc-color-border);
      border-radius: var(--acc-radius-lg);
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.15),
        0 10px 10px -5px rgba(0, 0, 0, 0.04);
      animation: slideUp 0.25s ease-out;
      display: flex;
      flex-direction: column;
      overflow-y: auto;
    }

    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem 1.5rem 1rem;
      border-bottom: 1px solid var(--acc-color-border);
    }

    .modal-title {
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--acc-color-text-main);
      margin: 0;
    }

    .modal-body {
      padding: 1.5rem;
      display: grid;
      gap: 1rem;
    }

    .detalhe-row {
      display: flex;
      gap: 0.75rem;
      border-bottom: 1px solid var(--acc-color-border);
      padding-bottom: 0.5rem;
      align-items: baseline;
    }

    .detalhe-label {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--acc-color-text-muted);
      min-width: 140px;
      white-space: nowrap;
    }

    .detalhe-valor {
      font-size: 0.9375rem;
      color: var(--acc-color-text-main);
      word-break: break-word;
    }

    .modal-footer {
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      padding: 1rem 1.5rem 1.5rem;
      border-top: 1px solid var(--acc-color-border);
    }

    @keyframes fadeIn {
      from { opacity: 0; }
      to { opacity: 1; }
    }

    @keyframes slideUp {
      from { opacity: 0; transform: translate(-50%, -40%); }
      to { opacity: 1; transform: translate(-50%, -50%); }
    }

    @media (max-width: 480px) {
      .modal {
        width: 92%;
        max-height: 95vh;
      }

      .detalhe-row {
        flex-direction: column;
        align-items: flex-start;
        gap: 0.25rem;
      }

      .detalhe-label {
        min-width: auto;
      }
    }
  `,
})
export class ClienteDetalhesModalComponent {
  @Input() cliente!: ClienteDto;
  @Output() fechar = new EventEmitter<void>();

  protected readonly formatarData = (iso: string): string =>
    formatPtBrDateTime(iso);

  protected readonly formatarDoc = (doc: string): string => {
    const limpo = doc.replace(/\D/g, '');
    if (limpo.length === 11) {
      return limpo.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, '$1.$2.$3-$4');
    }
    if (limpo.length === 14) {
      return limpo.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, '$1.$2.$3/$4-$5');
    }
    return doc;
  };
}
