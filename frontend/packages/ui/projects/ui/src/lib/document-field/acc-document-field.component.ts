import { ChangeDetectionStrategy, Component, computed, input, model, signal } from '@angular/core';
import { formatDocument, isValidDocument, sanitizeDocument } from '@accurit/shared';
import { AccIconComponent } from '../icon/acc-icon.component';

let proximoId = 0;

/**
 * Campo de CPF/CNPJ do contribuinte.
 * - `bloqueado`: somente leitura (documento vem do cliente selecionado); o usuário não altera.
 * - editável: aplica máscara enquanto digita e valida os dígitos verificadores ao sair do campo.
 * `value` guarda sempre o texto mascarado; use `sanitizeDocument` de `@accurit/shared` para enviar.
 */
@Component({
  selector: 'acc-document-field',
  standalone: true,
  imports: [AccIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-document-field.component.html',
  styleUrl: './acc-document-field.component.css',
})
export class AccDocumentFieldComponent {
  readonly label = input<string>('Documento do contribuinte');
  readonly value = model<string>('');
  readonly bloqueado = input<boolean>(false);
  readonly hint = input<string | null>(null);
  readonly placeholder = input<string>('000.000.000-00 ou 00.000.000/0000-00');
  /** `false` valida só o tamanho (11/14 dígitos) — útil no Trial, que usa documentos de demonstração. */
  readonly validarDigitos = input<boolean>(true);

  protected readonly inputId = `acc-doc-${++proximoId}`;
  protected readonly tocado = signal(false);

  protected readonly exibicao = computed(() => formatDocument(this.value()));

  protected readonly mensagemErro = computed(() => {
    if (this.bloqueado() || !this.tocado()) return null;
    const limpo = sanitizeDocument(this.value());
    if (!limpo) return null;
    if (limpo.length !== 11 && limpo.length !== 14) {
      return 'Informe 11 dígitos (CPF) ou 14 dígitos (CNPJ).';
    }
    if (this.validarDigitos() && !isValidDocument(limpo)) {
      return 'CPF ou CNPJ inválido. Confira os dígitos.';
    }
    return null;
  });

  protected readonly descricaoId = computed(() =>
    this.mensagemErro() || this.hint() ? `${this.inputId}-msg` : null,
  );

  protected aoDigitar(event: Event): void {
    const alvo = event.target as HTMLInputElement;
    if (this.bloqueado()) {
      // Documento vinculado ao cliente: restaura o valor caso o DOM seja alterado por fora.
      alvo.value = this.exibicao();
      return;
    }
    const formatado = formatDocument(sanitizeDocument(alvo.value).slice(0, 14));
    alvo.value = formatado;
    this.value.set(formatado);
  }
}
