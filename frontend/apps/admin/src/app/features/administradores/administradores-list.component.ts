import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';

import { AdministradorDto, ListagemAdministradoresDto } from '@accurit/core';

import { AccButtonComponent, AccBadgeComponent } from '@accurit/ui';

type MetodoPagina = 'page' | 'pageSize' | 'q';

interface FiltrosTabela {
  pagina: number;
  pagina_tamanho: number;
  termo_busca: string;
}

@Component({
  selector: 'app-administradores-list',
  standalone: true,
  imports: [AccButtonComponent, AccBadgeComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="page-header">
      <h1>Administradores</h1>
      <p class="text-secondary">Gestão de administradores do SaaS Accurit</p>
    </div>

    <div class="table-responsive">
      <table class="table table-borderless mb-0">
        <thead>
          <tr>
            <th>Nome</th>
            <th>E-mail</th>
            <th>CPF</th>
            <th>Status</th>
            <th style="width: 180px;">Ações</th>
          </tr>
        </thead>
        <tbody>
          @for (a of administradores(); track a.id) {
          <tr>
            <td>{{ a.nome }}</td>
            <td>{{ a.email }}</td>
            <td>{{ a.cpf }}</td>
            <td>
              @if (a.ativo) {
                <acc-badge status="success">Ativo</acc-badge>
              } @else {
                <acc-badge status="danger">Inativo</acc-badge>
              }
            </td>
            <td>
              <button acc-button size="sm" variant="outline" (click)="editar(a.id)">Editar</button>
              <button acc-button size="sm" variant="danger" (click)="excluir(a.id)">Excluir</button>
            </td>
          </tr>
          }
        </tbody>
      </table>
    </div>

    <div class="mt-3">
      <button acc-button size="lg" variant="primary" (click)="novo()">
        <span class="icon-text"><i class="fas fa-user-plus me-2"></i> Novo administrador</span>
      </button>
    </div>
  `,
  styles: [],
})
export class AdministradoresListComponent {
  private readonly http = inject(HttpClient);

  readonly administradores = signal<AdministradorDto[]>([]);
  readonly carregando = signal(false);
  readonly erro = signal<string | null>(null);

  ngOnInit(): void {
    this.carregar();
  }

  private carregar(): void {
    this.carregando.set(true);
    const params = new HttpParams()
      .set('page', '1')
      .set('pageSize', '100');

    this.http
      .get<ListagemAdministradoresDto>('/api/v1/administradores', { params })
      .subscribe({
        next: (res) => {
          this.administradores.set(res.itens);
          this.carregando.set(false);
        },
        error: (err) => {
          this.erro.set(err.message ?? 'Falha ao carregar lista de administradores.');
          this.carregando.set(false);
        },
      });
  }

  novo(): void {
    // TODO: Abrir modal de criar administrador
    alert('Funcionalidade de criar administrador em desenvolvimento.');
  }

  editar(id: string): void {
    // TODO: Abrir modal de editar administrador
    alert(`Editar administrador ${id} em desenvolvimento.`);
  }

  excluir(id: string): void {
    // TODO: Confirmar exclusão e verificar regras de negócio
    if (confirm('Tem certeza que deseja excluir este administrador?')) {
      // Verificar regra: não pode excluir o último administrador ativo
      this.erro.set('Exclusão em desenvolvimento.');
    }
  }
}