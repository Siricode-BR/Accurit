import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export interface Tenant {
  id: string;
  slug: string;
  nome: string;
  cnpj?: string;
  modoTrialSerpro: boolean;
  criadoEm: string;
}

export interface CreateTenantRequest {
  slug: string;
  nome: string;
  emailTitular: string;
  cpfTitular: string;
  senhaTitular: string;
}

export interface CreateTenantResponse {
  escritorioId: string;
  slug: string;
  titularId: string;
  emailTitular: string;
}

@Injectable({ providedIn: 'root' })
export class TenantService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin';

  listar(): Observable<Tenant[]> {
    return this.http.get<Tenant[]>(`${this.baseUrl}/escritorios`);
  }

  criar(request: CreateTenantRequest): Observable<CreateTenantResponse> {
    return this.http.post<CreateTenantResponse>(`${this.baseUrl}/escritorios`, request);
  }
}