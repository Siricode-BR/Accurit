import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  GuiaEmissaoDto,
} from '@accurit/core';
import { GuiasService } from '../../data/guias.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-guias-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './guias-list.component.html',
  styleUrl: './guias-list.component.css',
})
export class GuiasListComponent {
  protected readonly service = inject(GuiasService);
  protected readonly router = inject(Router);

  protected readonly guias = this.service.guias;
  protected readonly carregando = this.service.carregando;
  protected readonly erro = this.service.erro;

  async ngOnInit(): Promise<void> {
    await this.service.listar();
  }

  rotuloTipo(tipo: string): string {
    return tipo === 'DAS' ? 'DAS' : 'DARF';
  }

  rotuloStatus(status: string): string {
    switch (status) {
      case 'solicitada':
        return 'Solicitada';
      case 'emitida':
        return 'Emitida';
      case 'baixada':
        return 'Baixada';
      case 'falhou':
        return 'Falhou';
      default:
        return status;
    }
  }

  badgeStatus(status: string): 'success' | 'danger' | 'warning' | 'info' {
    switch (status) {
      case 'emitida':
        return 'success';
      case 'falhou':
        return 'danger';
      case 'solicitada':
        return 'info';
      case 'baixada':
      default:
        return 'warning';
    }
  }

  async abrirEmissao(): Promise<void> {
    await this.router.navigate(['/guias/novo']);
  }

  async baixarGuia(id: string): Promise<void> {
    await this.service.baixar(id);
    await this.service.listar();
  }

  async recarregar(): Promise<void> {
    await this.service.listar();
  }
}
