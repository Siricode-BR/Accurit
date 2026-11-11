import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

import {
  CriarConsultaLoteRequest,
  CriarTrabalhoRequest,
  ConsultaLoteDto,
  ExecucaoTrabalhoDto,
  ItemConsultaLoteDto,
  TrabalhoAgendadoDto,
  AlternarTrabalhoRequest,
} from '@accurit/core';
import { mensagemDeErro } from '../core/http/mensagem-erro';

/**
 * Serviço de trabalhos agendados e consultas em lote (FASE B).
 * Toda rota exige autenticação com audiência `client` e isolamento por
 * `escritorio_id` — o backend responde 404 para recursos de outro escritório
 * (sem vazar existência).
 */
@Injectable({ providedIn: 'root' })
export class TrabalhosService {
  private readonly http = inject(HttpClient);
  private readonly baseUrlTrabalhos = '/api/v1/trabalhos';
  private readonly baseUrlExecucoes = '/api/v1/execucoes';
  private readonly baseUrlLotes = '/api/v1/consultas-lote';

  // ===== Trabalhos agendados =====

  #trabalhos = signal<TrabalhoAgendadoDto[]>([]);
  #carregandoTrabalhos = signal(false);
  #erroTrabalhos = signal<string | null>(null);

  readonly trabalhos = this.#trabalhos.asReadonly();
  readonly carregandoTrabalhos = this.#carregandoTrabalhos.asReadonly();
  readonly erroTrabalhos = this.#erroTrabalhos.asReadonly();

  readonly trabalhosPorTipo = computed(() => {
    const mapa: Record<string, TrabalhoAgendadoDto[]> = {};
    for (const t of this.#trabalhos()) {
      (mapa[t.tipo] ??= []).push(t);
    }
    return mapa;
  });

  readonly trabalhosHabilitados = computed(() =>
    this.#trabalhos().filter((t) => t.habilitado),
  );

  async listarTrabalhos(): Promise<void> {
    this.#carregandoTrabalhos.set(true);
    this.#erroTrabalhos.set(null);
    try {
      const resposta = await firstValueFrom<
        TrabalhoAgendadoDto[]
      >(this.http.get<TrabalhoAgendadoDto[]>(this.baseUrlTrabalhos));
      this.#trabalhos.set(resposta);
    } catch (err: any) {
      this.#erroTrabalhos.set(
        mensagemDeErro(err, 'Falha ao carregar trabalhos agendados.'),
      );
    } finally {
      this.#carregandoTrabalhos.set(false);
    }
  }

  async criarTrabalho(requisicao: CriarTrabalhoRequest): Promise<TrabalhoAgendadoDto> {
    this.#erroTrabalhos.set(null);
    try {
      const resposta = await firstValueFrom<
        TrabalhoAgendadoDto
      >(this.http.post<TrabalhoAgendadoDto>(this.baseUrlTrabalhos, requisicao));
      this.#trabalhos.update((atual) => [resposta, ...atual]);
      return resposta;
    } catch (err: any) {
      this.#erroTrabalhos.set(
        mensagemDeErro(err, 'Falha ao criar trabalho agendado.'),
      );
      throw err;
    }
  }

  async buscarTrabalho(id: string): Promise<TrabalhoAgendadoDto> {
    this.#erroTrabalhos.set(null);
    try {
      return await firstValueFrom<TrabalhoAgendadoDto>(
        this.http.get<TrabalhoAgendadoDto>(`${this.baseUrlTrabalhos}/${encodeURIComponent(id)}`),
      );
    } catch (err: any) {
      this.#erroTrabalhos.set(
        mensagemDeErro(err, 'Falha ao buscar trabalho agendado.'),
      );
      throw err;
    }
  }

  async alternarTrabalho(
    id: string,
    requisicao: AlternarTrabalhoRequest,
  ): Promise<void> {
    this.#erroTrabalhos.set(null);
    try {
      await firstValueFrom(
        this.http.patch(
          `${this.baseUrlTrabalhos}/${encodeURIComponent(id)}/alternar`,
          requisicao,
        ),
      );
      this.#trabalhos.update((atual) =>
        atual.map((t) =>
          t.id === id ? { ...t, habilitado: requisicao.habilitado } : t,
        ),
      );
    } catch (err: any) {
      this.#erroTrabalhos.set(
        mensagemDeErro(err, 'Falha ao alterar o trabalho agendado.'),
      );
      throw err;
    }
  }

  // ===== Execuções de trabalho =====

  #execucoes = signal<ExecucaoTrabalhoDto[]>([]);
  #carregandoExecucoes = signal(false);
  #erroExecucoes = signal<string | null>(null);

  readonly execucoes = this.#execucoes.asReadonly();
  readonly carregandoExecucoes = this.#carregandoExecucoes.asReadonly();
  readonly erroExecucoes = this.#erroExecucoes.asReadonly();

  async listarExecucoesPorTrabalho(trabalhoId: string): Promise<void> {
    this.#carregandoExecucoes.set(true);
    this.#erroExecucoes.set(null);
    try {
      const resposta = await firstValueFrom<
        ExecucaoTrabalhoDto[]
      >(
        this.http.get<ExecucaoTrabalhoDto[]>(
          `${this.baseUrlExecucoes}/trabalho/${encodeURIComponent(trabalhoId)}`,
        ),
      );
      this.#execucoes.set(resposta);
    } catch (err: any) {
      this.#erroExecucoes.set(
        mensagemDeErro(err, 'Falha ao carregar execuções do trabalho.'),
      );
    } finally {
      this.#carregandoExecucoes.set(false);
    }
  }

  async listarExecucoesPorEscritorio(): Promise<void> {
    this.#carregandoExecucoes.set(true);
    this.#erroExecucoes.set(null);
    try {
      const resposta = await firstValueFrom<
        ExecucaoTrabalhoDto[]
      >(this.http.get<ExecucaoTrabalhoDto[]>(this.baseUrlExecucoes));
      this.#execucoes.set(resposta);
    } catch (err: any) {
      this.#erroExecucoes.set(
        mensagemDeErro(err, 'Falha ao carregar execuções do escritório.'),
      );
    } finally {
      this.#carregandoExecucoes.set(false);
    }
  }

  // ===== Consultas em lote =====

  #lotes = signal<ConsultaLoteDto[]>([]);
  #carregandoLotes = signal(false);
  #erroLotes = signal<string | null>(null);

  readonly lotes = this.#lotes.asReadonly();
  readonly carregandoLotes = this.#carregandoLotes.asReadonly();
  readonly erroLotes = this.#erroLotes.asReadonly();

  async listarLotes(): Promise<void> {
    this.#carregandoLotes.set(true);
    this.#erroLotes.set(null);
    try {
      const resposta = await firstValueFrom<ConsultaLoteDto[]>(
        this.http.get<ConsultaLoteDto[]>(this.baseUrlLotes),
      );
      this.#lotes.set(resposta);
    } catch (err: any) {
      this.#erroLotes.set(
        mensagemDeErro(err, 'Falha ao carregar consultas em lote.'),
      );
    } finally {
      this.#carregandoLotes.set(false);
    }
  }

  async criarConsultaLote(
    requisicao: CriarConsultaLoteRequest,
  ): Promise<ConsultaLoteDto> {
    this.#erroLotes.set(null);
    try {
      const resposta = await firstValueFrom<ConsultaLoteDto>(
        this.http.post<ConsultaLoteDto>(this.baseUrlLotes, requisicao),
      );
      this.#lotes.update((atual) => [resposta, ...atual]);
      return resposta;
    } catch (err: any) {
      this.#erroLotes.set(
        mensagemDeErro(err, 'Falha ao criar consulta em lote.'),
      );
      throw err;
    }
  }

  async buscarLote(id: string): Promise<ConsultaLoteDto> {
    this.#erroLotes.set(null);
    try {
      return await firstValueFrom<ConsultaLoteDto>(
        this.http.get<ConsultaLoteDto>(
          `${this.baseUrlLotes}/${encodeURIComponent(id)}`,
        ),
      );
    } catch (err: any) {
      this.#erroLotes.set(
        mensagemDeErro(err, 'Falha ao buscar consulta em lote.'),
      );
      throw err;
    }
  }

  async listarItensLote(loteId: string): Promise<ItemConsultaLoteDto[]> {
    this.#erroLotes.set(null);
    try {
      return await firstValueFrom<ItemConsultaLoteDto[]>(
        this.http.get<ItemConsultaLoteDto[]>(
          `${this.baseUrlLotes}/${encodeURIComponent(loteId)}/itens`,
        ),
      );
    } catch (err: any) {
      this.#erroLotes.set(
        mensagemDeErro(err, 'Falha ao carregar itens do lote.'),
      );
      throw err;
    }
  }

  /**
   * Execução manual (Fase B.1): devolve à fila os itens que falharam ou
   * foram bloqueados. O worker interno processa o lote no próximo poll.
   */
  async reexecutarLote(loteId: string): Promise<void> {
    this.#erroLotes.set(null);
    try {
      await firstValueFrom(
        this.http.post(
          `${this.baseUrlLotes}/${encodeURIComponent(loteId)}/executar`,
          {},
        ),
      );
      this.#lotes.update((atual) =>
        atual.map((lote) =>
          lote.id === loteId
            ? { ...lote, status: 'pendente', itensComErro: 0 }
            : lote,
        ),
      );
    } catch (err: any) {
      this.#erroLotes.set(
        mensagemDeErro(err, 'Falha ao reexecutar a consulta em lote.'),
      );
      throw err;
    }
  }

  limpar(): void {
    this.#trabalhos.set([]);
    this.#execucoes.set([]);
    this.#lotes.set([]);
    this.#erroTrabalhos.set(null);
    this.#erroExecucoes.set(null);
    this.#erroLotes.set(null);
  }
}
