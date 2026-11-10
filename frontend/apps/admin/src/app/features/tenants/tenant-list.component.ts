import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TenantDto, CriarTenantResponseDto } from '@accurit/core';
import { formatDocument, formatPtBrDateTime } from '@accurit/shared';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccCardComponent,
  AccCtaButtonComponent,
  AccFilterChipsComponent,
  AccFilterOption,
  AccIconComponent,
  AccModalComponent,
  AccSearchFieldComponent,
  AccSpinnerComponent,
} from '@accurit/ui';
import { mensagemDeErro } from '../../core/http/mensagem-erro';
import { TenantService } from '../../data/tenant.service';
import { TenantFormComponent } from './tenant-form.component';

type FiltroTenentes = 'todos' | 'trial' | 'producao';

@Component({
  selector: 'app-tenant-list',
  standalone: true,
  imports: [
    AccBadgeComponent,
    AccButtonComponent,
    AccCardComponent,
    AccCtaButtonComponent,
    AccFilterChipsComponent,
    AccIconComponent,
    AccModalComponent,
    AccSearchFieldComponent,
    AccSpinnerComponent,
    TenantFormComponent,
    RouterLink,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tenant-list.component.html',
  styleUrl: './tenant-list.component.css',
})
export class TenantListComponent implements OnInit {
  protected readonly tenantService = inject(TenantService);

  protected readonly opcoesFiltro: readonly AccFilterOption<FiltroTenentes>[] = [
    { value: 'todos', label: 'Todos' },
    { value: 'trial', label: 'Trial' },
    { value: 'producao', label: 'Produção' },
  ];

  protected readonly termoBusca = signal('');
  protected readonly filtroAtivo = signal<FiltroTenentes>('todos');
  protected readonly modalNovoAberto = signal(false);
  protected readonly editando = signal<string | null>(null);
  protected readonly erroAcao = signal<string | null>(null);
  protected readonly aviso = signal<string | null>(null);

  protected readonly tenantsExibidos = computed(() => {
    const termo = this.termoBusca().trim().toLowerCase();
    return this.tenantService.tenants().filter((t) => {
      if (!termo) return true;
      return (
        t.nome.toLowerCase().includes(termo) ||
        t.cnpj.includes(termo)
      );
    });
  });

  ngOnInit(): void {
    this.tenantService.listar();
  }

  protected aplicarFiltro(filtro: FiltroTenentes): void {
    this.filtroAtivo.set(filtro);
    this.carregar();
  }

  private carregar(): void {
    const filtro = this.filtroAtivo();
    this.tenantService.listar();
    // Filtro por ambiente já é feito via computed no template
  }

  protected recarregar(): void {
    this.aplicarFiltro(this.filtroAtivo());
  }

  protected novo(): void {
    this.modalNovoAberto.set(true);
  }

  protected aoCriar(resposta: CriarTenantResponseDto): void {
    this.modalNovoAberto.set(false);
    this.aviso.set(
      `Escritório ${formatDocument(resposta.cnpj)} criado. Login do titular: ${resposta.emailTitular}.`
    );
  }

  protected formatarCnpj(cnpj: string): string {
    return formatDocument(cnpj);
  }

  protected formatarData(iso: string | null): string {
    return iso ? formatPtBrDateTime(iso) : 'Nunca atualizado';
  }
}