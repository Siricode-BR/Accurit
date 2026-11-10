import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { CenarioTrialMockDto, ServicoTrialMock } from '@accurit/core';

interface EstadoTrialMock {
  carregado: boolean;
  ativo: boolean;
  cenarios: CenarioTrialMockDto[];
}

/**
 * Cenários do simulador SERPRO em Trial.
 * Cache em memória e falha silenciosa: a ausência do endpoint nunca quebra a tela.
 */
@Injectable({ providedIn: 'root' })
export class TrialMockService {
  private readonly http = inject(HttpClient);
  private readonly estado = signal<EstadoTrialMock>({
    carregado: false,
    ativo: false,
    cenarios: [],
  });
  private promessa: Promise<void> | null = null;

  readonly ativo = () => this.estado().ativo;
  readonly cenarios = () => this.estado().cenarios;

  async carregar(): Promise<void> {
    if (this.estado().carregado) return;
    this.promessa ??= this.buscar();
    await this.promessa;
  }

  porServico(...servicos: ServicoTrialMock[]): CenarioTrialMockDto[] {
    return this.estado().cenarios.filter((c) => servicos.includes(c.servico));
  }

  private async buscar(): Promise<void> {
    try {
      const resposta = await firstValueFrom(
        this.http.get<{ ativo: boolean; cenarios: CenarioTrialMockDto[] }>(
          '/api/v1/serpro/mock/cenarios',
        ),
      );
      this.estado.set({
        carregado: true,
        ativo: resposta?.ativo === true,
        cenarios: Array.isArray(resposta?.cenarios) ? resposta.cenarios : [],
      });
    } catch {
      // Falha silenciosa: sem mock disponível, a tela segue normal.
      this.estado.set({ carregado: true, ativo: false, cenarios: [] });
    }
  }
}
