import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

import {
  CriarTenantRequest,
  CriarTenantResponse,
  Pagina,
  Tenant,
  TenantAlterado,
} from '../domain/tenant';

/** Camada de dados: acesso HTTP à API de admin (SPEC §10.1). */
@Injectable({ providedIn: 'root' })
export class TenantService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin';

  listar(pagina = 1, porPagina = 20): Observable<Pagina<Tenant>> {
    const params = new HttpParams()
      .set('page', pagina.toString())
      .set('pageSize', porPagina.toString());
    return this.http.get<Pagina<Tenant>>(`${this.baseUrl}/escritorios`, { params });
  }

  alterarTrial(
    id: string,
    pedido: { modoTrialSerpro: boolean; versao: number },
  ): Observable<TenantAlterado> {
    return this.http.patch<TenantAlterado>(
      `${this.baseUrl}/escritorios/${id}/serpro/trial`,
      pedido,
    );
  }

  criar(request: CriarTenantRequest): Observable<CriarTenantResponse> {
    return this.http.post<CriarTenantResponse>(`${this.baseUrl}/escritorios`, request);
  }
}
