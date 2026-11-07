import { ChangeDetectionStrategy, Component, OnInit, computed, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { formatDocument } from '@accurit/shared';
import {
  AccBadgeComponent,
  AccButtonComponent,
  AccEmptyStateComponent,
  AccIconComponent,
  AccPageHeaderComponent,
  AccSkeletonComponent,
  AccStatCardComponent,
} from '@accurit/ui';
import { SessionStore } from '../../core/auth/session.store';
import { ClienteService } from '../../data/cliente.service';
import { SerproAmbienteService } from '../../data/serpro.ambiente.service';
import { AcaoRapida } from './dashboard.types';

/**
 * Dashboard padrão do escritório (`/dashboard`) — destino inicial após o login.
 * Exibe apenas dados reais (total de clientes, ambiente SERPRO, clientes recentes).
 */
@Component({
  selector: 'app-dashboard-page',
  standalone: true,
  imports: [
    RouterLink,
    AccBadgeComponent,
    AccButtonComponent,
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
  private readonly clienteService = inject(ClienteService);
  private readonly ambiente = inject(SerproAmbienteService);
  private readonly sessao = inject(SessionStore);

  protected readonly acoes: readonly AcaoRapida[] = [
    { id: 'novo-cliente', titulo: 'Cadastrar cliente', descricao: 'Adicione um novo contribuinte.', icone: 'user-plus', rota: '/clientes/novo' },
    { id: 'clientes', titulo: 'Ver clientes', descricao: 'Lista e fichas dos clientes.', icone: 'clientes', rota: '/clientes' },
    { id: 'integra-contador', titulo: 'Consultar Integra Contador', descricao: 'Consulta avulsa por CPF/CNPJ.', icone: 'integra-contador', rota: '/consultas/integra-contador' },
    { id: 'divida-ativa', titulo: 'Consultar Dívida Ativa', descricao: 'Consulta avulsa por devedor ou inscrição.', icone: 'divida-ativa', rota: '/consultas/divida-ativa' },
  ];

  protected readonly isTrial = this.ambiente.isTrial;
  protected readonly carregandoClientes = this.clienteService.carregando;
  protected readonly erroClientes = this.clienteService.erro;

  protected readonly carregandoAmbiente = computed(() => this.ambiente.leitura().estado === 'carregando');

  protected readonly totalClientes = computed(() => this.clienteService.total());

  protected readonly rotuloAmbiente = computed(() => {
    const leitura = this.ambiente.leitura();
    if (leitura.estado === 'indisponivel') return 'Indisponível';
    const efetivo = this.ambiente.ambienteEfetivo();
    if (!efetivo) return '—';
    return efetivo === 'TRIAL' ? 'Trial (avaliação)' : 'Produção';
  });

  protected readonly recentes = computed(() =>
    [...this.clienteService.clientes()]
      .sort((a, b) => b.criadoEm.localeCompare(a.criadoEm))
      .slice(0, 5),
  );

  protected readonly saudacao = computed(() => {
    const hora = new Date().getHours();
    const periodo = hora < 12 ? 'Bom dia' : hora < 18 ? 'Boa tarde' : 'Boa noite';
    return `${periodo}!`;
  });

  protected readonly subtitulo = computed(() => {
    const email = this.sessao.user()?.email;
    return email ? `Painel do escritório · ${email}` : 'Painel do escritório';
  });

  ngOnInit(): void {
    this.clienteService.listar('todos', 1, 20);
    this.ambiente.obter().subscribe({ error: () => undefined });
  }

  protected formatarDocumento(valor: string): string {
    return formatDocument(valor);
  }
}
