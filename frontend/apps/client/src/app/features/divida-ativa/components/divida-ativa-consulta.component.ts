import {
  ChangeDetectionStrategy,
  Component,
  OnInit,
  computed,
  inject,
  input,
  linkedSignal,
  signal,
} from '@angular/core';
import {
  AccButtonComponent,
  AccCardComponent,
  AccDocumentFieldComponent,
  AccEmptyStateComponent,
} from '@accurit/ui';
import { isValidDocument, sanitizeDocument } from '@accurit/shared';
import { DividaAtivaService } from '../../../data/divida-ativa.service';
import { SerproAmbienteService } from '../../../data/serpro.ambiente.service';
import { ConsultaModo, ModoOpcao } from '../divida-ativa.types';

/**
 * Formulário + resultado da Dívida Ativa, compartilhado entre a consulta de
 * cliente cadastrado (documento bloqueado) e a consulta avulsa (documento livre).
 * O bloqueio vale para o documento do devedor; o número de inscrição é sempre editável.
 */
@Component({
  selector: 'app-divida-ativa-consulta',
  standalone: true,
  imports: [AccButtonComponent, AccCardComponent, AccDocumentFieldComponent, AccEmptyStateComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './divida-ativa-consulta.component.html',
  styleUrl: './divida-ativa-consulta.component.css',
})
export class DividaAtivaConsultaComponent implements OnInit {
  private readonly servico = inject(DividaAtivaService);
  private readonly ambiente = inject(SerproAmbienteService);

  readonly documentoInicial = input<string>('');
  readonly documentoBloqueado = input<boolean>(false);

  protected readonly modos: readonly ModoOpcao[] = [
    { id: 'devedor', label: 'Por devedor (CPF/CNPJ)' },
    { id: 'inscricao', label: 'Por inscrição' },
  ];

  protected readonly modo = signal<ConsultaModo>('devedor');
  protected readonly documento = linkedSignal(() => this.documentoInicial());
  protected readonly inscricao = signal('');
  protected readonly consultou = signal(false);

  protected readonly carregando = this.servico.carregando;
  protected readonly erro = this.servico.erro;
  protected readonly devedor = this.servico.ultimoDevedor;
  protected readonly inscricaoResultado = this.servico.ultimaInscricao;
  protected readonly isTrial = this.ambiente.isTrial;

  /** Aviso consultivo permanente (ADR-0002): vem do resultado da consulta. */
  protected readonly avisoConsultivo = computed(
    () => this.devedor()?.aviso ?? this.inscricaoResultado()?.aviso ?? null,
  );

  protected readonly podeConsultar = computed(() => {
    if (this.carregando()) return false;
    if (this.modo() === 'inscricao') return !!sanitizeDocument(this.inscricao());
    const limpo = sanitizeDocument(this.documento());
    return this.isTrial() ? limpo.length === 11 || limpo.length === 14 : isValidDocument(limpo);
  });

  ngOnInit(): void {
    this.servico.limpar();
    this.ambiente.obter().subscribe({ error: () => undefined });
  }

  protected selecionarModo(modo: ConsultaModo): void {
    this.modo.set(modo);
    this.consultou.set(false);
    this.servico.limpar();
  }

  protected async consultar(): Promise<void> {
    if (!this.podeConsultar()) return;

    this.servico.limpar();
    this.consultou.set(true);
    try {
      if (this.modo() === 'devedor') {
        await this.servico.consultarDevedor(sanitizeDocument(this.documento()));
      } else {
        await this.servico.consultarInscricao(sanitizeDocument(this.inscricao()));
      }
    } catch {
      // mensagem de erro já publicada pelo serviço
    }
  }

  protected limpar(): void {
    this.consultou.set(false);
    this.servico.limpar();
  }
}
