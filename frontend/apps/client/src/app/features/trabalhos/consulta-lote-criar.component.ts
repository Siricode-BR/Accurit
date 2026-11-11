import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
} from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import {
  FormsModule,
  ReactiveFormsModule,
  FormBuilder,
  Validators,
} from '@angular/forms';
import {
  CriarConsultaLoteRequest,
  ClienteDto,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import { ClienteService } from '../../data/cliente.service';
import {
  AccAlertComponent,
  AccButtonComponent,
  AccCheckboxComponent,
  AccInputComponent,
  AccSelectComponent,
  AccSpinnerComponent,
  AccCardComponent,
} from '@accurit/ui';

const PRODUTOS: { value: string; label: string }[] = [
  { value: 'INTEGRA_CONTADOR', label: 'Integra Contador' },
  { value: 'DIVIDA_ATIVA', label: 'Dívida Ativa' },
];

const SERVICOS: { value: string; label: string }[] = [
  { value: 'OBTERPROCURACAO41', label: 'OBTERPROCURACAO41 (Procurações)' },
  { value: 'DEVEDOR', label: 'DEVEDOR (Dívida Ativa)' },
  { value: 'INSCRICAO', label: 'INSCRICAO (Dívida Ativa)' },
];

const VERBOS: { value: string; label: string }[] = [
  { value: 'Consultar', label: 'Consultar' },
  { value: 'Emitir', label: 'Emitir' },
  { value: 'Declarar', label: 'Declarar' },
  { value: 'Apoiar', label: 'Apoiar' },
  { value: 'Monitorar', label: 'Monitorar' },
];

@Component({
  selector: 'app-consulta-lote-criar',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    AccAlertComponent,
    AccButtonComponent,
    AccCardComponent,
    AccCheckboxComponent,
    AccInputComponent,
    AccSelectComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './consulta-lote-criar.component.html',
  styleUrl: './consulta-lote-criar.component.css',
})
export class ConsultaLoteCriarComponent {
  protected readonly fb = inject(FormBuilder);
  protected readonly router = inject(Router);
  protected readonly trabalhosService = inject(TrabalhosService);
  protected readonly clienteService = inject(ClienteService);

  protected readonly clientes = this.clienteService.clientes;
  protected readonly carregandoClientes = this.clienteService.carregando;

  protected async ngOnInit(): Promise<void> {
    this.clienteService.listar('todos', 1, 100);
  }

  protected clienteSelecionado(cliente: ClienteDto): boolean {
    return this.documentos().includes(cliente.cpfCnpj);
  }

  /** Seleção de clientes do escritório: adiciona/remove o documento na lista. */
  protected alternarCliente(cliente: ClienteDto): void {
    const documento = cliente.cpfCnpj;
    this.documentos.update((atual) => {
      const limpos = atual.filter((d) => d.trim().length > 0);
      if (limpos.includes(documento)) {
        return limpos.filter((d) => d !== documento);
      }
      return [...limpos, documento];
    });
  }

  protected readonly produtos = PRODUTOS;
  protected readonly servicos = SERVICOS;
  protected readonly verbos = VERBOS;

  protected readonly documentos = signal<string[]>(['']);
  protected readonly carregando = this.trabalhosService.carregandoLotes;
  protected readonly erroServico = this.trabalhosService.erroLotes;
  protected readonly erroValidacao = signal<string | null>(null);

  protected readonly form = this.fb.group({
    nome: ['', [Validators.required, Validators.minLength(3)]],
    produto: ['INTEGRA_CONTADOR', [Validators.required]],
    servico: ['OBTERPROCURACAO41', [Validators.required]],
    verbo: ['Consultar', [Validators.required]],
  });

  async criar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const listaDocumentos = this.documentos()
      .map((d) => d.trim())
      .filter((d) => d.length > 0);

    if (listaDocumentos.length === 0) {
      this.erroValidacao.set('Informe ao menos um documento.');
      return;
    }

    const requisicao: CriarConsultaLoteRequest = {
      nome: this.form.value.nome!,
      produto: this.form.value.produto!,
      servico: this.form.value.servico!,
      verbo: this.form.value.verbo!,
      documentos: listaDocumentos.map((documento) => ({
        documento,
        tipo: documento.length === 11 ? 'PF' : 'PJ',
      })),
    };

    try {
      await this.trabalhosService.criarConsultaLote(requisicao);
      await this.router.navigate(['/consultas-lote']);
    } catch {
      // Erro já foi definido pelo serviço
    }
  }

  async voltar(): Promise<void> {
    await this.router.navigate(['/consultas-lote']);
  }

  protected adicionarDocumento(): void {
    this.documentos.update((atual) => [...atual, '']);
  }

  protected removerDocumento(index: number): void {
    this.documentos.update((atual) => atual.filter((_, i) => i !== index));
  }

  protected atualizarValorDocumento(index: number, valor: string): void {
    this.documentos.update((atual) => {
      const copia = [...atual];
      copia[index] = valor;
      return copia;
    });
  }

  protected rotuloProduto(valor: string): string {
    return this.produtos.find((p) => p.value === valor)?.label ?? valor;
  }

  protected rotuloServico(valor: string): string {
    return this.servicos.find((s) => s.value === valor)?.label ?? valor;
  }

  protected tipoDocumento(documento: string): string {
    return documento.length === 11 ? 'PF' : 'PJ';
  }
}

