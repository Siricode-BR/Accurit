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
  templateUrl: './cliente-detalhes-modal.component.html',
  styleUrl: './cliente-detalhes-modal.component.css',
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
