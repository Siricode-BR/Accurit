import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { AccBackButtonComponent, AccPageHeaderComponent } from '@accurit/ui';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { IntegraContadorConsultaComponent } from './components/integra-contador-consulta.component';

/** Consulta avulsa do Integra Contador (`/consultas/integra-contador`): documento livre. */
@Component({
  selector: 'app-integra-contador-avulsa-page',
  standalone: true,
  imports: [RouterLink, AccBackButtonComponent, AccPageHeaderComponent, IntegraContadorConsultaComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consulta-avulsa.page.html',
  styleUrl: './consulta-avulsa.page.css',
})
export class IntegraContadorAvulsaPage {
  protected readonly ambienteEfetivo = inject(SerproAmbienteService).ambienteEfetivo;
}
