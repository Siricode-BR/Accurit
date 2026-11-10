import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

import {
  GuiaEmissaoDto,
  RequisicaoEmitirGuia,
} from '@accurit/core';
import { mensagemDeErro } from '../core/http/mensagem-erro';

/**
 * Serviço de emissão de guias (FASE D).
 * Toda rota exige autenticação com audiência `client` e isolamento por
 * `escritorio_id`.
 */
@Injectable({ providedIn: 'root' })
export class GuiasService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/guias';

  #guias = signal<GuiaEmissaoDto[]>([]);
  #carregando = signal(false);
  #erro = signal<string | null>(null);

  readonly guias = this.#guias.asReadonly();
  readonly carregando = this.#carregando.asReadonly();
  readonly erro = this.#erro.asReadonly();

  async listar(): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);
    try {
      const resposta = await firstValueFrom<GuiaEmissaoDto[]>(
        this.http.get<GuiaEmissaoDto[]>(this.baseUrl),
      );
      this.#guias.set(resposta);
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao carregar guias.'),
      );
    } finally {
      this.#carregando.set(false);
    }
  }

  async emitir(requisicao: RequisicaoEmitirGuia): Promise<GuiaEmissaoDto> {
    this.#erro.set(null);
    try {
      const guia = await firstValueFrom<GuiaEmissaoDto>(
        this.http.post<GuiaEmissaoDto>(this.baseUrl, requisicao),
      );
      this.#guias.update((atual) => [guia, ...atual]);
      return guia;
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao emitir guia.'),
      );
      throw err;
    }
  }

  async buscar(id: string): Promise<GuiaEmissaoDto> {
    try {
      return await firstValueFrom<GuiaEmissaoDto>(
        this.http.get<GuiaEmissaoDto>(`${this.baseUrl}/${encodeURIComponent(id)}`),
      );
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao buscar guia.'),
      );
      throw err;
    }
  }

  async baixar(id: string): Promise<GuiaEmissaoDto> {
    try {
      const guia = await firstValueFrom<GuiaEmissaoDto>(
        this.http.post<GuiaEmissaoDto>(
          `${this.baseUrl}/${encodeURIComponent(id)}/baixar`,
          {},
        ),
      );
      this.#guias.update((atual) =>
        atual.map((g) => (g.id === id ? guia : g)),
      );
      return guia;
    } catch (err: any) {
      this.#erro.set(
        mensagemDeErro(err, 'Falha ao baixar guia.'),
      );
      throw err;
    }
  }

  limpar(): void {
    this.#guias.set([]);
    this.#carregando.set(false);
    this.#erro.set(null);
  }
}
