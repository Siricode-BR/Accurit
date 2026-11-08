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
  CriarTrabalhoRequest,
  TipoTrabalho,
} from '@accurit/core';
import { TrabalhosService } from '../../data/trabalhos.service';
import {
  AccButtonComponent,
  AccSpinnerComponent,
} from '@accurit/ui';

const TIPOS_TRABALHO: { valor: TipoTrabalho; label: string }[] = [
  { valor: 'consulta_divida_ativa', label: 'Consulta Dívida Ativa' },
  { valor: 'validar_procuracao', label: 'Validar Procuração' },
  { valor: 'consulta_integra_contador', label: 'Consulta Integra Contador' },
];

const AMBIENTES: { valor: string; label: string }[] = [
  { valor: 'TRIAL', label: 'Trial' },
  { valor: 'PRODUCAO', label: 'Produção' },
];

@Component({
  selector: 'app-trabalho-criar',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    AccButtonComponent,
    AccSpinnerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './trabalho-criar.component.html',
  styleUrl: './trabalho-criar.component.css',
})
export class TrabalhoCriarComponent {
  protected readonly fb = inject(FormBuilder);
  protected readonly router = inject(Router);
  protected readonly trabalhosService = inject(TrabalhosService);

  protected readonly tiposTrabalho = TIPOS_TRABALHO;
  protected readonly ambientes = AMBIENTES;

  protected readonly form = this.fb.group({
    nome: ['', [Validators.required, Validators.minLength(3)]],
    tipo: ['consulta_divida_ativa', [Validators.required]],
    produto: ['INTEGRA_CONTADOR', [Validators.required]],
    ambiente: ['TRIAL', [Validators.required]],
    cronExpression: [
      '',
      [Validators.required, Validators.pattern(/^\S+\s+\S+\s+\S+\s+\S+\s+\S+$/)],
    ],
  });

  protected readonly carregando = this.trabalhosService.carregandoTrabalhos;
  protected readonly erro = this.trabalhosService.erroTrabalhos;

  protected readonly tiposTrabalhoRotulo = {
    consulta_divida_ativa: 'Consulta Dívida Ativa',
    validar_procuracao: 'Validar Procuração',
    consulta_integra_contador: 'Consulta Integra Contador',
  };

  async ngOnInit(): Promise<void> {
    this.form.get('tipo')?.valueChanges.subscribe((tipo) => {
      const produto =
        tipo === 'consulta_divida_ativa' ? 'DIVIDA_ATIVA' : 'INTEGRA_CONTADOR';
      this.form.get('produto')?.setValue(produto);
    });
  }

  protected rotuloTipo(valor: TipoTrabalho): string {
    return this.tiposTrabalhoRotulo[valor] ?? valor;
  }

  protected rotuloAmbiente(valor: string): string {
    return valor === 'TRIAL' ? 'Ambiente de avaliação' : 'Produção';
  }

  async criar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const requisicao: CriarTrabalhoRequest = {
      nome: this.form.value.nome!,
      tipo: this.form.value.tipo! as TipoTrabalho,
      produto: this.form.value.produto!,
      ambiente: this.form.value.ambiente!,
      cronExpression: this.form.value.cronExpression!,
    };

    try {
      await this.trabalhosService.criarTrabalho(requisicao);
      await this.router.navigate(['/trabalhos']);
    } catch {
      // Erro já foi definido pelo serviço
    }
  }

  async voltar(): Promise<void> {
    await this.router.navigate(['/trabalhos']);
  }
}
