import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

import {
  NotificacaoDto,
  RequisicaoCriarNotificacao,
  RequisicaoMarcarLida,
  RequisicaoPreferenciaNotificacao,
  PreferenciaNotificacaoDto,
  RespostaContagemNaoLidas,
  TemplateNotificacaoDto,
} from '@accurit/core';

/**
 * Serviço de notificações (FASE C).
 * Toda rota exige autenticação com audiência `client` e isolamento por
 * `escritorio_id`.
 */
@Injectable({ providedIn: 'root' })
export class NotificacoesService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/notificacoes';

  #notificacoes = signal<NotificacaoDto[]>([]);
  #carregando = signal(false);
  #erro = signal<string | null>(null);
  #contagemNaoLidas = signal(0);

  readonly notificacoes = this.#notificacoes.asReadonly();
  readonly carregando = this.#carregando.asReadonly();
  readonly erro = this.#erro.asReadonly();
  readonly contagemNaoLidas = this.#contagemNaoLidas.asReadonly();

  readonly naoLidas = computed(() =>
    this.#notificacoes().filter((n) => !n.lida).length,
  );

  async listar(apenasNaoLidas = true): Promise<void> {
    this.#carregando.set(true);
    this.#erro.set(null);
    try {
      const resposta = await firstValueFrom<
        NotificacaoDto[]
      >(
        this.http.get<NotificacaoDto[]>(this.baseUrl, {
          params: apenasNaoLidas ? { apenasNaoLidas: 'true' } : undefined,
        }),
      );
      this.#notificacoes.set(resposta);
      this.#contagemNaoLidas.set(
        resposta.filter((n) => !n.lida).length,
      );
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao carregar notificações.',
      );
    } finally {
      this.#carregando.set(false);
    }
  }

  async contarNaoLidas(): Promise<void> {
    try {
      const resposta = await firstValueFrom<
        RespostaContagemNaoLidas
      >(this.http.get<RespostaContagemNaoLidas>(`${this.baseUrl}/contagem-nao-lidas`));
      this.#contagemNaoLidas.set(resposta.naoLidas);
    } catch {
      // Silencioso - não bloqueia a UI
    }
  }

  async marcarComoLida(id: string): Promise<void> {
    try {
      await firstValueFrom(
        this.http.patch(`${this.baseUrl}/${encodeURIComponent(id)}/ler`, {}),
      );
      this.#notificacoes.update((lista) =>
        lista.map((n) => (n.id === id ? { ...n, lida: true } : n)),
      );
      this.#contagemNaoLidas.update((c) => Math.max(0, c - 1));
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao marcar como lida.',
      );
      throw err;
    }
  }

  async marcarTodasComoLidas(): Promise<void> {
    try {
      await firstValueFrom(
        this.http.post(`${this.baseUrl}/marcar-todas-lidas`, {}),
      );
      this.#notificacoes.update((lista) =>
        lista.map((n) => ({ ...n, lida: true })),
      );
      this.#contagemNaoLidas.set(0);
    } catch (err: any) {
      this.#erro.set(
        err.error?.message ?? err.message ?? 'Falha ao marcar todas como lidas.',
      );
      throw err;
    }
  }

  async buscarPreferencia(): Promise<PreferenciaNotificacaoDto | null> {
    try {
      return await firstValueFrom<
        PreferenciaNotificacaoDto | null
      >(this.http.get<PreferenciaNotificacaoDto | null>('/api/v1/preferencias-notificacao'));
    } catch {
      return null;
    }
  }

  async salvarPreferencia(
    requisicao: RequisicaoPreferenciaNotificacao,
  ): Promise<PreferenciaNotificacaoDto> {
    return await firstValueFrom(
      this.http.post<PreferenciaNotificacaoDto>(
        '/api/v1/preferencias-notificacao',
        requisicao,
      ),
    );
  }

  async listarTemplates(): Promise<TemplateNotificacaoDto[]> {
    try {
      return await firstValueFrom<
        TemplateNotificacaoDto[]
      >(this.http.get<TemplateNotificacaoDto[]>('/api/v1/templates-notificacao'));
    } catch {
      return [];
    }
  }

  limpar(): void {
    this.#notificacoes.set([]);
    this.#carregando.set(false);
    this.#erro.set(null);
    this.#contagemNaoLidas.set(0);
  }
}
