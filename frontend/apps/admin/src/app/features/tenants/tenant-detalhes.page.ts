import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { formatDocument, formatPtBrDateTime } from '@accurit/shared';
import {
  AccBackButtonComponent,
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccEmptyStateComponent,
  AccIconComponent,
  AccSkeletonComponent,
} from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { TenantService } from '../../data/tenant.service';

/** Página do escritório (`/tenants/:id`) — destino após o cadastro e a partir da lista. */
@Component({
  selector: 'app-admin-tenant-detalhes',
  standalone: true,
  imports: [
    RouterLink,
    AccBackButtonComponent,
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccEmptyStateComponent,
    AccIconComponent,
    AccSkeletonComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tenant-detalhes.page.html',
  styleUrl: './tenant-detalhes.page.css',
})
export class TenantDetalhesPage implements OnInit {
  private readonly tenantService = inject(TenantService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);

  private readonly id = this.route.snapshot.paramMap.get('id') ?? '';

  /** E-mail do titular só é conhecido logo após o cadastro (vem no state da navegação). */
  protected readonly titularEmail: string | null =
    (this.router.getCurrentNavigation()?.extras.state?.['titularEmail'] as string | undefined) ??
    (history.state?.titularEmail as string | undefined) ??
    null;

  protected readonly carregando = this.tenantService.carregando;
  protected readonly alterando = signal(false);
  protected readonly erroAcao = signal<string | null>(null);

  protected readonly tenant = computed(() => this.tenantService.tenants().find((t) => t.id === this.id));

  ngOnInit(): void {
    if (!this.tenant()) this.tenantService.listar();
  }

  protected formatarData(iso: string): string {
    return formatPtBrDateTime(iso);
  }

  protected formatarCnpj(cnpj: string): string {
    return formatDocument(cnpj);
  }

  protected async alternarAmbiente(): Promise<void> {
    const t = this.tenant();
    if (!t) return;
    this.alterando.set(true);
    this.erroAcao.set(null);
    try {
      await this.tenantService.alternarModoSerpro(t.id, !t.modoTrialSerpro, t.versao);
    } catch (e) {
      const erro = e as { error?: { message?: string }; message?: string };
      this.erroAcao.set(mensagemDeErro(e, 'Não foi possível alterar o ambiente.'));
    } finally {
      this.alterando.set(false);
    }
  }
}
