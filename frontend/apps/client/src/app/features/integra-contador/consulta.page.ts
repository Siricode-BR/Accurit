import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { ClienteDto } from '@accurit/core';
import {
  AccButtonComponent,
  AccEmptyStateComponent,
  AccPageHeaderComponent,
  AccSkeletonComponent,
} from '@accurit/ui';
import { ClienteService } from '../../data/cliente.service';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { IntegraContadorConsultaComponent } from './components/integra-contador-consulta.component';
import { EstadoClienteConsulta } from './integra-contador.types';

/**
 * Integra Contador para um cliente cadastrado (`/integra-contador?clienteId=`).
 * O documento do contribuinte vem do cliente e fica bloqueado para edição.
 * Sem `clienteId`, redireciona para a consulta avulsa.
 */
@Component({
  selector: 'app-integra-contador-consulta-page',
  standalone: true,
  imports: [
    RouterLink,
    AccButtonComponent,
    AccEmptyStateComponent,
    AccPageHeaderComponent,
    AccSkeletonComponent,
    IntegraContadorConsultaComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consulta.page.html',
  styleUrl: './consulta.page.css',
})
export class IntegraContadorConsultaPage implements OnInit {
  private readonly clienteService = inject(ClienteService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);

  protected readonly ambienteEfetivo = inject(SerproAmbienteService).ambienteEfetivo;
  protected readonly cliente = signal<ClienteDto | null>(null);
  protected readonly estado = signal<EstadoClienteConsulta>('carregando');

  async ngOnInit(): Promise<void> {
    const clienteId = this.route.snapshot.queryParamMap.get('clienteId');
    if (!clienteId) {
      await this.router.navigate(['/consultas/integra-contador'], { replaceUrl: true });
      return;
    }

    try {
      this.cliente.set(await this.clienteService.obter(clienteId));
      this.estado.set('pronto');
    } catch {
      this.estado.set('naoEncontrado');
    }
  }

  protected voltar(): void {
    const id = this.cliente()?.id;
    void this.router.navigate(id ? ['/clientes', id] : ['/clientes']);
  }
}
