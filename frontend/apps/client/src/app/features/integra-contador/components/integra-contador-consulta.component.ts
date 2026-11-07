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
import { AccButtonComponent, AccCardComponent, AccDocumentFieldComponent, AccSelectComponent, AccSelectOption } from '@accurit/ui';
import { isValidDocument, sanitizeDocument } from '@accurit/shared';
import { IntegraContadorService } from '../../../data/integra-contador.service';
import { SerproAmbienteService } from '../../../data/serpro.ambiente.service';
import { CATALOGO, VERBOS, VERBO_PADRAO } from '../integra-contador.catalogo';
import { VerboIntegraContador } from '../integra-contador.types';

/**
 * Formulário + resultado do Integra Contador, compartilhado entre a consulta
 * de cliente cadastrado (documento bloqueado) e a consulta avulsa (documento livre).
 *
 * Os selects são encadeados por signals: verbo → sistema → serviço → versão.
 * Ao mudar um nível, os seguintes são recalculados; se restar uma única opção, ela é preenchida.
 */
@Component({
  selector: 'app-integra-contador-consulta',
  standalone: true,
  imports: [AccButtonComponent, AccCardComponent, AccDocumentFieldComponent, AccSelectComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './integra-contador-consulta.component.html',
  styleUrl: './integra-contador-consulta.component.css',
})
export class IntegraContadorConsultaComponent implements OnInit {
  private readonly servico = inject(IntegraContadorService);
  private readonly ambiente = inject(SerproAmbienteService);

  /** Documento inicial (ex.: do cliente selecionado). */
  readonly documentoInicial = input<string>('');
  /** Quando `true`, o usuário não pode alterar o documento. */
  readonly documentoBloqueado = input<boolean>(false);

  protected readonly opcoesVerbo: readonly AccSelectOption[] = VERBOS.map((v) => ({
    value: v.valor,
    label: v.label,
  }));

  protected readonly verbo = signal<string>(VERBO_PADRAO);

  protected readonly opcoesSistema = computed<AccSelectOption[]>(() => {
    const sistemas = new Set(CATALOGO.filter((s) => s.verbo === this.verbo()).map((s) => s.idSistema));
    return [...sistemas].map((s) => ({ value: s, label: s }));
  });

  protected readonly idSistema = linkedSignal<AccSelectOption[], string>({
    source: this.opcoesSistema,
    computation: (opcoes, anterior) => this.manterOuUnica(opcoes, anterior?.value),
  });

  protected readonly opcoesServico = computed<AccSelectOption[]>(() =>
    CATALOGO.filter((s) => s.verbo === this.verbo() && s.idSistema === this.idSistema()).map((s) => ({
      value: s.idServico,
      label: s.idServico,
    })),
  );

  protected readonly idServico = linkedSignal<AccSelectOption[], string>({
    source: this.opcoesServico,
    computation: (opcoes, anterior) => this.manterOuUnica(opcoes, anterior?.value),
  });

  protected readonly opcoesVersao = computed<AccSelectOption[]>(() => {
    const entrada = CATALOGO.find(
      (s) =>
        s.verbo === this.verbo() && s.idSistema === this.idSistema() && s.idServico === this.idServico(),
    );
    return entrada?.versaoSistema ? [{ value: entrada.versaoSistema, label: entrada.versaoSistema }] : [];
  });

  protected readonly versao = linkedSignal<AccSelectOption[], string>({
    source: this.opcoesVersao,
    computation: (opcoes, anterior) => this.manterOuUnica(opcoes, anterior?.value),
  });

  protected readonly documento = linkedSignal(() => this.documentoInicial());
  protected readonly dados = signal('');
  protected readonly confirmacaoExplicita = signal(false);

  protected readonly carregando = this.servico.carregando;
  protected readonly erro = this.servico.erro;
  protected readonly resultado = this.servico.ultimoResultado;
  protected readonly isTrial = this.ambiente.isTrial;

  protected readonly dadosInvalidos = computed(() => {
    const texto = this.dados().trim();
    if (!texto) return false;
    try {
      JSON.parse(texto);
      return false;
    } catch {
      return true;
    }
  });

  private readonly documentoValido = computed(() => {
    const limpo = sanitizeDocument(this.documento());
    if (this.isTrial()) return limpo.length === 11 || limpo.length === 14;
    return isValidDocument(limpo);
  });

  protected readonly podeExecutar = computed(
    () =>
      !this.carregando() &&
      this.documentoValido() &&
      !!this.idSistema() &&
      !!this.idServico() &&
      !this.dadosInvalidos(),
  );

  ngOnInit(): void {
    this.servico.limparResultado();
    this.ambiente.obter().subscribe({ error: () => undefined });
  }

  protected async executar(): Promise<void> {
    if (!this.podeExecutar()) return;

    this.servico.limparResultado();
    try {
      await this.servico.executar({
        documentoContribuinte: sanitizeDocument(this.documento()),
        verbo: this.verbo() as VerboIntegraContador,
        pedidoDados: {
          idSistema: this.idSistema(),
          idServico: this.idServico(),
          versaoSistema: this.versao(),
          dados: this.dados().trim(),
        },
        confirmacaoExplicita: this.confirmacaoExplicita(),
      });
    } catch {
      // mensagem de erro já publicada pelo serviço
    }
  }

  protected limpar(): void {
    this.servico.limparResultado();
  }

  /** Mantém a seleção anterior se ainda existir; senão preenche quando há uma única opção. */
  private manterOuUnica(opcoes: AccSelectOption[], anterior: string | undefined): string {
    if (anterior && opcoes.some((o) => o.value === anterior)) return anterior;
    return opcoes.length === 1 ? opcoes[0].value : '';
  }
}
