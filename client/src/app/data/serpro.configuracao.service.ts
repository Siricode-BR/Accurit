import { HttpClient } from '@angular/common/http';
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

/// Leitura do ambiente SERPRO do escritório do próprio token do portal
/// (SPEC §10.2): consulta é de exibição e não expõe segredos.
@Injectable({ providedIn: 'root' })
export class SerproConfiguracaoService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro';

  obterConfiguracao(): Observable<SerproConfiguracao> {
    return this.http.get<SerproConfiguracao>(`${this.baseUrl}/configuracao`);
  }
}
