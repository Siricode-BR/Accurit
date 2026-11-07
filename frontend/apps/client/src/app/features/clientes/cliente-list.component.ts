import {
  Component,
  OnInit,
  DestroyRef,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { ClienteService } from '../../data/cliente.service';
import { ProcuracaoService } from '../../data/procuracao.service';
import {
  AccButtonComponent,
  AccBadgeComponent,
  AccCardComponent,
  AccSpinnerComponent,
  AccCtaButtonComponent,
  AccIconComponent,
  AccSearchFieldComponent,
  AccFilterChipsComponent,
  AccFilterOption,
} from '@accurit/ui';
import { isValidDocument, formatDocument } from '@accurit/shared';
import { ClienteDto, FiltroClientes } from '@accurit/core';

@Component({
  selector: 'app-cliente-list',
  standalone: true,
  imports: [
    CommonModule,
    AccButtonComponent,
    AccBadgeComponent,
    AccCardComponent,
    AccSpinnerComponent,
    AccCtaButtonComponent,
    AccIconComponent,
    AccSearchFieldComponent,
    AccFilterChipsComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './cliente-list.component.html',
  styleUrl: './cliente-list.component.css',
})
export class ClienteListComponent implements OnInit {
  protected readonly clienteService = inject(ClienteService);
  protected readonly procuracaoService = inject(ProcuracaoService);
  protected readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly opcoesFiltro: readonly AccFilterOption<FiltroClientes>[] = [
    { value: 'todos', label: 'Todos' },
    { value: 'cpf', label: 'Pessoa Física' },
    { value: 'cnpj_mei', label: 'MEI' },
    { value: 'cnpj_simples', label: 'Simples Nacional' },
    { value: 'cnpj_presumido', label: 'Lucro Presumido' },
    { value: 'cnpj_real', label: 'Lucro Real' },
  ];

  readonly termoBusca = signal<string>('');
  readonly filtroAtivo = signal<FiltroClientes>('todos');

  readonly clientesExibidos = computed(() => {
    const termo = this.termoBusca().trim();
    if (!termo) {
      return this.clienteService.clientes();
    }
    return this.clienteService.filtrarPorTextoOuDocumento(termo);
  });

  readonly validacaoDocumentoAviso = computed(() => {
    const termo = this.termoBusca().trim();
    if (termo.length >= 11 && /\d/.test(termo)) {
      if (!isValidDocument(termo)) {
        return 'O número de documento informado parece incompleto ou com dígito verificador inválido.';
      }
    }
    return null;
  });

  // Estado da validação de procuração e-CAC por cliente (sessão).
  // O serviço guarda o status (Autorizada/Pendente); aqui ficam
  // os clientes com requisição em andamento e a última mensagem de erro.
  readonly procuracaoValidando = signal<ReadonlySet<string>>(new Set());
  readonly erroValidacao = signal<string | null>(null);

  ngOnInit(): void {
    this.clienteService.listar();
  }

  aplicarFiltro(filtro: FiltroClientes): void {
    this.filtroAtivo.set(filtro);
    this.clienteService.listar(filtro);
  }

  recarregar(): void {
    this.clienteService.listar(this.filtroAtivo());
  }

  formatarDoc(doc: string): string {
    return formatDocument(doc);
  }

  abrirModalNovo(): void {
    this.router.navigate(['/clientes/novo']);
  }

  abrirDetalhes(cliente: ClienteDto): void {
    this.router.navigate(['/clientes', cliente.id]);
  }

  // Status exibido no acc-badge por cliente (ausência = Pendente).
  statusProcuracaoDe(clienteId: string): 'AUTORIZADA' | 'PENDENTE' {
    return this.procuracaoService.statusDe(clienteId);
  }

  private marcarValidando(clienteId: string, ativo: boolean): void {
    this.procuracaoValidando.update((atual) => {
      const proximo = new Set(atual);
      if (ativo) {
        proximo.add(clienteId);
      } else {
        proximo.delete(clienteId);
      }
      return proximo;
    });
  }

  // Valida a procuração e-CAC do cliente junto ao backend (AUTENTICAPROCURADOR).
  validarProcuracao(cliente: ClienteDto): void {
    this.erroValidacao.set(null);
    this.marcarValidando(cliente.id, true);

    this.procuracaoService
      .validarProcuracao(cliente.id)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: () => {
          this.marcarValidando(cliente.id, false);
        },
        error: (erro: { error?: { message?: string }; message?: string }) => {
          this.marcarValidando(cliente.id, false);
          this.erroValidacao.set(
            erro?.error?.message ?? erro?.message ?? 'Falha ao validar a procuração e-CAC.',
          );
        },
      });
  }
}
