import {
  Component,
  OnInit,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { TenantService } from '../../data/tenant.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccCtaButtonComponent,
  AccFilterChipsComponent,
  AccFilterOption,
  AccIconComponent,
  AccSearchFieldComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { formatPtBrDateTime } from '@accurit/shared';
import { TenantDto } from '@accurit/core';

type FiltroAmbiente = 'todos' | 'producao' | 'trial';

@Component({
  selector: 'app-tenant-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccCtaButtonComponent,
    AccFilterChipsComponent,
    AccIconComponent,
    AccSearchFieldComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tenant-list.component.html',
  styleUrl: './tenant-list.component.css',
})
export class TenantListComponent implements OnInit {
  protected readonly tenantService = inject(TenantService);
  protected readonly router = inject(Router);

  protected readonly opcoesFiltro: readonly AccFilterOption<FiltroAmbiente>[] = [
    { value: 'todos', label: 'Todos' },
    { value: 'producao', label: 'Produção' },
    { value: 'trial', label: 'Trial SERPRO' },
  ];

  protected readonly termoBusca = signal('');
  protected readonly filtroAtivo = signal<FiltroAmbiente>('todos');

  protected readonly tenantsExibidos = computed(() => {
    const termo = this.termoBusca().trim().toLowerCase();
    const filtro = this.filtroAtivo();
    return this.tenantService.tenants().filter((t) => {
      if (filtro === 'trial' && !t.modoTrialSerpro) return false;
      if (filtro === 'producao' && t.modoTrialSerpro) return false;
      return !termo || t.nome.toLowerCase().includes(termo) || t.slug.toLowerCase().includes(termo);
    });
  });

  protected novoTenant(): void {
    this.router.navigate(['/tenants/novo']);
  }

  ngOnInit(): void {
    this.tenantService.listar();
  }

  recarregar(): void {
    this.tenantService.listar();
  }

  formatarData(iso: string): string {
    return formatPtBrDateTime(iso);
  }

  async alternarAmbiente(tenant: TenantDto): Promise<void> {
    const novoModo = !tenant.modoTrialSerpro;
    await this.tenantService.alternarModoSerpro(tenant.id, novoModo, tenant.versao);
  }
}

