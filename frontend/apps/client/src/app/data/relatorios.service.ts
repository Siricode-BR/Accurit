import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

import {
  CriarLinkCompartilhamentoRequestDto,
  CriarRelatorioRequestDto,
  LinkCompartilhamentoDto,
  RelatorioDto,
  RelatorioResumoDto,
} from '@accurit/core';

/**
 * Relatórios e link seguro de compartilhamento (Fase E).
 * Toda rota exige autenticação com audiência `client` e isolamento por
 * `escritorio_id`, exceto o link público de compartilhamento.
 */
@Injectable({ providedIn: 'root' })
export class RelatoriosService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/relatorios';

  #relatorios = signal<RelatorioResumoDto[]>([]);
  #carregando = signal(false);
  #erro = signal<string | null>(null);

  readonly relatorios = this.#relatorios.asReadonly();
  readonly carregando = this.#carregando.asReadonly();
  readonly erro = this.#erro.asReadonly();

  async listar(): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);
    try {
      const resposta = await firstValueFrom<RelatorioResumoDto[]>(
        this.http.get<RelatorioResumoDto[]>(this.baseUrl),
      );
      this.#relatorios.set(resposta);
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao carregar relatórios.',
      );
    } finally {
      this.#carregando.set(false);
    }
  }

  async gerar(
    requisicao: CriarRelatorioRequestDto,
  ): Promise<RelatorioDto> {
    this.#erro.set(null);
    try {
      const relatorio = await firstValueFrom<RelatorioDto>(
        this.http.post<RelatorioDto>(this.baseUrl, requisicao),
      );
      this.#relatorios.update((atual) => [relatorio, ...atual]);
      return relatorio;
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao gerar relatório.',
      );
      throw err;
    }
  }

  async obter(id: string): Promise<RelatorioDto> {
    try {
      return await firstValueFrom<RelatorioDto>(
        this.http.get<RelatorioDto>(
          `${this.baseUrl}/${encodeURIComponent(id)}`,
        ),
      );
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao buscar relatório.',
      );
      throw err;
    }
  }

  async baixar(id: string): Promise<Blob> {
    try {
      const resposta = await firstValueFrom<Blob>(
        this.http.get(`${this.baseUrl}/${encodeURIComponent(id)}/baixar`, {
          responseType: 'blob',
        }),
      );
      return resposta;
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao baixar relatório.',
      );
      throw err;
    }
  }

  async criarLinkCompartilhamento(
    id: string,
    requisicao: CriarLinkCompartilhamentoRequestDto,
  ): Promise<LinkCompartilhamentoDto> {
    this.#erro.set(null);
    try {
      const link = await firstValueFrom<LinkCompartilhamentoDto>(
        this.http.post<LinkCompartilhamentoDto>(
          `${this.baseUrl}/${encodeURIComponent(id)}/compartilhar`,
          requisicao,
        ),
      );
      return link;
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao criar link de compartilhamento.',
      );
      throw err;
    }
  }

  limpar(): void {
    this.#relatorios.set([]);
    this.#carregando.set(false);
    this.#erro.set(null);
  }
}
