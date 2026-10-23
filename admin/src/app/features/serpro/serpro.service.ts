import { HttpClient } from '@angular/common/http';
import { HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export type AmbienteSerpro = 'TRIAL' | 'PRODUCAO';

export interface SerproConfiguracao {
  escritorioId: string;
  modoTrialSerpro: boolean;
  ambienteEfetivo: AmbienteSerpro;
  versao: number;
  atualizadoEm: string;
}

export interface SerproConfiguracaoAlterada {
  modoTrialSerpro: boolean;
  ambienteEfetivo: AmbienteSerpro;
  versaoAtual: number;
  atualizadoEm: string;
}

export interface AlterarModoTrialPedido {
  escritorioId: string;
  modoTrialSerpro: boolean;
  versao: number;
}

@Injectable({ providedIn: 'root' })
export class SerproService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin';

  obterConfiguracao(escritorioId: string): Observable<SerproConfiguracao> {
    const params = new HttpParams().set('escritorioId', escritorioId);
    return this.http.get<SerproConfiguracao>(`${this.baseUrl}/serpro/configuracao`, { params });
  }

  alterarModoTrial(pedido: AlterarModoTrialPedido): Observable<SerproConfiguracaoAlterada> {
    return this.http.patch<SerproConfiguracaoAlterada>(`${this.baseUrl}/serpro/trial`, pedido);
  }
}
