import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  NotificacaoDto,
} from '@accurit/core';
import { NotificacoesService } from '../../data/notificacoes.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-notificacoes-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './notificacoes-list.component.html',
  styleUrl: './notificacoes-list.component.css',
})
export class NotificacoesListComponent {
  protected readonly service = inject(NotificacoesService);
  protected readonly router = inject(Router);

  protected readonly notificacoes = this.service.notificacoes;
  protected readonly carregando = this.service.carregando;
  protected readonly erro = this.service.erro;
  protected readonly contagemNaoLidas = this.service.contagemNaoLidas;

  async ngOnInit(): Promise<void> {
    await this.service.listar();
    await this.service.contarNaoLidas();
  }

  rotuloTipo(tipo: string): string {
    switch (tipo) {
      case 'pendencia_divida_ativa':
        return 'Pendência Dívida Ativa';
      case 'mensagem_caixa_postal':
        return 'Caixa Postal';
      case 'procuracao_vencendo':
        return 'Procuração Vencendo';
      case 'certificado_a1_vencendo':
        return 'Certificado A1 Vencendo';
      case 'das_vencendo':
        return 'DAS Vencendo';
      case 'falha_rotina':
        return 'Falha de Rotina';
      default:
        return tipo;
    }
  }

  rotuloCanal(canal: string): string {
    switch (canal) {
      case 'in_app':
        return 'App';
      case 'email':
        return 'E-mail';
      case 'whatsapp':
        return 'WhatsApp';
      default:
        return canal;
    }
  }

  async marcarComoLida(id: string): Promise<void> {
    await this.service.marcarComoLida(id);
  }

  async marcarTodasComoLidas(): Promise<void> {
    await this.service.marcarTodasComoLidas();
  }

  async recarregar(): Promise<void> {
    await this.service.listar();
    await this.service.contarNaoLidas();
  }
}
