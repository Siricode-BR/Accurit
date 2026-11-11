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
  RequisicaoEmitirGuia,
  TipoGuia,
} from '@accurit/core';
import { GuiasService } from '../../data/guias.service';
import {
  AccButtonComponent,
  AccCardComponent,
} from '@accurit/ui';

@Component({
  selector: 'app-guia-emitir',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    AccButtonComponent,
    AccCardComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './guia-emitir.component.html',
  styleUrl: './guia-emitir.component.css',
})
export class GuiaEmitirComponent {
  protected readonly fb = inject(FormBuilder);
  protected readonly router = inject(Router);
  protected readonly service = inject(GuiasService);

  protected readonly tiposGuia: { valor: TipoGuia; label: string }[] = [
    { valor: 'DAS', label: 'DAS' },
    { valor: 'DARF', label: 'DARF' },
  ];

  protected readonly carregando = this.service.carregando;
  protected readonly erro = this.service.erro;

  protected readonly form = this.fb.group({
    clienteId: ['', [Validators.required]],
    tipoGuia: ['DAS', [Validators.required]],
    competencia: ['', [Validators.required, Validators.pattern(/^\d{4}-\d{2}$/)]],
    valorTotal: [0, [Validators.required, Validators.min(0)]],
    idempotenciaChave: ['', [Validators.required]],
  });

  async emitir(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    const requisicao: RequisicaoEmitirGuia = {
      clienteId: this.form.value.clienteId!,
      tipoGuia: this.form.value.tipoGuia! as TipoGuia,
      competencia: this.form.value.competencia!,
      valorTotal: this.form.value.valorTotal!,
      idempotenciaChave: this.form.value.idempotenciaChave!,
    };

    try {
      await this.service.emitir(requisicao);
      await this.router.navigate(['/guias']);
    } catch {
      // Erro já foi definido pelo serviço
    }
  }

  async voltar(): Promise<void> {
    await this.router.navigate(['/guias']);
  }
}
