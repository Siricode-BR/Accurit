import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { ConsumoClienteDto, RetencaoLgpdDto } from '@accurit/core';

@Injectable({ providedIn: 'root' })
export class ConsumoService {
  private readonly http = inject(HttpClient);

  readonly consumo = signal<ConsumoClienteDto | null>(null);
  readonly carregando = signal(false);
  readonly erro = signal<string | null>(null);

  async carregar(): Promise<void> {
    this.carregando.set(true);
    this.erro.set(null);
    try {
      this.consumo.set(await firstValueFrom(this.http.get<ConsumoClienteDto>('/api/v1/consumo')));
    } catch (erro) {
      this.erro.set(
        (erro as { status?: number }).status === 404
          ? 'Nenhum plano de consumo foi atribuído ao escritório. Fale com o administrador da plataforma.'
          : 'Não foi possível carregar o consumo. Tente novamente.',
      );
    } finally {
      this.carregando.set(false);
    }
  }

  async obterRetencao(): Promise<RetencaoLgpdDto> {
    return firstValueFrom(this.http.get<RetencaoLgpdDto>('/api/v1/lgpd/retencao'));
  }

  async salvarRetencao(payload: RetencaoLgpdDto): Promise<RetencaoLgpdDto> {
    return firstValueFrom(this.http.put<RetencaoLgpdDto>('/api/v1/lgpd/retencao', payload));
  }
}
