import {
  Component,
  OnInit,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router } from '@angular/router';
import { TenantFormComponent } from './tenant-form.component';
import { TenantService } from '../../data/tenant.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccCtaButtonComponent,
  AccFilterChipsComponent,
  AccFilterOption,
  AccIconComponent,
  AccModalComponent,
  AccSearchFieldComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { formatPtBrDateTime } from '@accurit/shared';
import { CriarTenantResponseDto, TenantDto } from '@accurit/core';

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
    AccModalComponent,
    AccSearchFieldComponent,
    TenantFormComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tenant-list.component.html',
  styleUrl: './tenant-list.component.css',
})
export class TenantListComponent implements OnInit {
  protected readonly tenantService = inject(TenantService);
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  protected readonly modalNovoAberto = signal(false);

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
    this.modalNovoAberto.set(true);
  }

  protected fecharModalNovo(): void {
    this.modalNovoAberto.set(false);
  }

  /** Após cadastrar, fecha o popup e abre a página do escritório criado. */
  protected aoCadastrar(novo: CriarTenantResponseDto): void {
    this.modalNovoAberto.set(false);
    void this.router.navigate(['/tenants', novo.escritorioId], {
      state: { titularEmail: novo.emailTitular },
    });
  }

  protected abrirDetalhes(tenant: TenantDto): void {
    void this.router.navigate(['/tenants', tenant.id]);
  }

  ngOnInit(): void {
    this.tenantService.listar();
    // Atalhos (dashboard, links antigos) chegam com ?novo=1 para abrir o cadastro direto.
    if (this.route.snapshot.queryParamMap.has('novo')) {
      this.modalNovoAberto.set(true);
      void this.router.navigate([], { queryParams: { novo: null }, replaceUrl: true });
    }
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

