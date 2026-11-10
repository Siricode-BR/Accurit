import { ChangeDetectionStrategy, Component, OnInit, computed, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { formatDocument, formatPtBrDateTime } from '@accurit/shared';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCtaButtonComponent,
  AccEmptyStateComponent,
  AccIconComponent,
  AccIconName,
  AccPageHeaderComponent,
  AccSkeletonComponent,
  AccStatCardComponent,
} from '@accurit/ui';
import { SessionStore } from '../../core/auth/session.store';
import { TenantService } from '../../data/tenant.service';

interface AcaoRapida {
  id: string;
  titulo: string;
  descricao: string;
  icone: AccIconName;
  rota: string;
  params?: Record<string, string | number>;
}

/**
 * Dashboard do backoffice (`/dashboard`) — destino inicial após o login do administrador.
 * Exibe apenas dados reais: escritórios cadastrados, ambiente SERPRO e atualizações recentes.
 */
@Component({
  selector: 'app-admin-dashboard-page',
  standalone: true,
  imports: [
    RouterLink,
    AccBadgeComponent,
    AccButtonComponent,
    AccCtaButtonComponent,
    AccEmptyStateComponent,
    AccIconComponent,
    AccPageHeaderComponent,
    AccSkeletonComponent,
    AccStatCardComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './dashboard.page.html',
  styleUrl: './dashboard.page.css',
})
export class DashboardPage implements OnInit {
  private readonly tenantService = inject(TenantService);
  private readonly sessao = inject(SessionStore);

  protected readonly acoes: readonly AcaoRapida[] = [
    { id: 'novo', titulo: 'Cadastrar escritório', descricao: 'Crie um novo tenant com titular.', icone: 'plus', rota: '/tenants', params: { novo: 1 } },
    { id: 'lista', titulo: 'Ver escritórios', descricao: 'Lista, busca e ambiente SERPRO.', icone: 'building', rota: '/tenants' },
  ];

  protected readonly carregando = this.tenantService.carregando;
  protected readonly erro = this.tenantService.erro;
  protected readonly total = this.tenantService.total;
  protected readonly totalProducao = this.tenantService.totalProducao;
  protected readonly totalTrial = this.tenantService.totalTrial;

  /** Percentual de escritórios em produção entre os carregados (0–100). */
  protected readonly percentualProducao = computed(() => {
    const carregados = this.tenantService.tenants().length;
    return carregados ? Math.round((this.totalProducao() / carregados) * 100) : 0;
  });

  protected readonly recentes = computed(() =>
    [...this.tenantService.tenants()]
      .sort((a, b) => b.atualizadoEm.localeCompare(a.atualizadoEm))
      .slice(0, 5),
  );

  protected readonly saudacao = computed(() => {
    const hora = new Date().getHours();
    const periodo = hora < 12 ? 'Bom dia' : hora < 18 ? 'Boa tarde' : 'Boa noite';
    return `${periodo}!`;
  });

  protected readonly subtitulo = computed(() => {
    const nome = this.sessao.user()?.nome;
    return nome ? `Backoffice SaaS · ${nome}` : 'Backoffice SaaS';
  });

  ngOnInit(): void {
    this.tenantService.listar();
  }

  protected recarregar(): void {
    this.tenantService.listar();
  }

  protected formatarData(iso: string): string {
    return formatPtBrDateTime(iso);
  }

  protected formatarCnpj(cnpj: string): string {
    return formatDocument(cnpj);
  }
}
