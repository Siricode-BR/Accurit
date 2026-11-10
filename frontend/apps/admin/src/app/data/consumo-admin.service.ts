import { Injectable, inject } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  AtribuirPlanoRequestDto,
  AtribuirPlanoResponseDto,
  ConsumoAdminDto,
  CriarPlanoRequestDto,
  PlanoEscritorioDto,
} from '@accurit/core';

export interface FiltrosConsumo {
  inicio?: string;
  fim?: string;
  produto?: string;
  ambiente?: string;
  escritorioId?: string;
}

@Injectable({ providedIn: 'root' })
export class ConsumoAdminService {
  private readonly http = inject(HttpClient);

  private params(filtros: FiltrosConsumo): HttpParams {
    let params = new HttpParams();
    if (filtros.inicio) params = params.set('inicio', filtros.inicio);
    if (filtros.fim) params = params.set('fim', filtros.fim);
    if (filtros.produto) params = params.set('produto', filtros.produto);
    if (filtros.ambiente) params = params.set('ambiente', filtros.ambiente);
    if (filtros.escritorioId) params = params.set('escritorioId', filtros.escritorioId);
    return params;
  }

  consumo(filtros: FiltrosConsumo): Promise<ConsumoAdminDto> {
    return firstValueFrom(
      this.http.get<ConsumoAdminDto>('/api/v1/admin/consumo', { params: this.params(filtros) }),
    );
  }

  exportarCsv(filtros: FiltrosConsumo): Promise<Blob> {
    return firstValueFrom(
      this.http.get('/api/v1/admin/consumo/export.csv', {
        params: this.params(filtros),
        responseType: 'blob',
      }),
    );
  }

  planos(): Promise<PlanoEscritorioDto[]> {
    return firstValueFrom(this.http.get<PlanoEscritorioDto[]>('/api/v1/admin/planos'));
  }

  criarPlano(payload: CriarPlanoRequestDto): Promise<PlanoEscritorioDto> {
    return firstValueFrom(this.http.post<PlanoEscritorioDto>('/api/v1/admin/planos', payload));
  }

  atribuirPlano(
    escritorioId: string,
    payload: AtribuirPlanoRequestDto,
  ): Promise<AtribuirPlanoResponseDto> {
    return firstValueFrom(
      this.http.put<AtribuirPlanoResponseDto>(
        `/api/v1/admin/planos/escritorios/${escritorioId}`,
        payload,
      ),
    );
  }
}
