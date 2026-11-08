import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { AccBackButtonComponent, AccPageHeaderComponent } from '@accurit/ui';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { DividaAtivaConsultaComponent } from './components/divida-ativa-consulta.component';

/** Consulta avulsa da Dívida Ativa (`/consultas/divida-ativa`): documento livre. */
@Component({
  selector: 'app-divida-ativa-avulsa-page',
  standalone: true,
  imports: [RouterLink, AccBackButtonComponent, AccPageHeaderComponent, DividaAtivaConsultaComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consulta-avulsa.page.html',
  styleUrl: './consulta-avulsa.page.css',
})
export class DividaAtivaAvulsaPage {
  protected readonly ambienteEfetivo = inject(SerproAmbienteService).ambienteEfetivo;
}
