import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export type TipoCliente = 'PF' | 'PJ';
export type PorteRegime = 'MEI' | 'SIMPLES_NACIONAL' | 'LUCRO_PRESUMIDO' | 'LUCRO_REAL';
export type FiltroClientes = 'todos' | 'cpf' | 'cnpj_mei' | 'cnpj_simples' | 'cnpj_presumido' | 'cnpj_real';

export interface Cliente {
  id: string;
  tipo: TipoCliente;
  porteRegime?: PorteRegime;
  cpfCnpj: string;
  nomeRazaoSocial: string;
  nomeFantasia?: string;
  email?: string;
  telefone?: string;
  criadoEm: string;
  atualizadoEm: string;
}

export interface ListagemClientes {
  itens: Cliente[];
  total: number;
  pagina: number;
  tamanho: number;
}

export interface CreateClienteRequest {
  tipo: TipoCliente;
  porteRegime?: PorteRegime;
  cpfCnpj: string;
  nomeRazaoSocial: string;
  nomeFantasia?: string;
  email?: string;
  telefone?: string;
}

export interface UpdateClienteRequest {
  tipo?: TipoCliente;
  porteRegime?: PorteRegime | null;
  cpfCnpj?: string;
  nomeRazaoSocial?: string;
  nomeFantasia?: string | null;
  email?: string | null;
  telefone?: string | null;
}

@Injectable({ providedIn: 'root' })
export class ClienteService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/clientes';

  listar(filtro: FiltroClientes = 'todos', pagina = 1, tamanho = 20): Observable<ListagemClientes> {
    let params = new HttpParams()
      .set('pagina', pagina.toString())
      .set('tamanho', tamanho.toString());

    if (filtro !== 'todos') {
      params = params.set('filtro', filtro);
    }

    return this.http.get<ListagemClientes>(this.baseUrl, { params });
  }

  obter(id: string): Observable<Cliente> {
    return this.http.get<Cliente>(`${this.baseUrl}/${id}`);
  }

  criar(request: CreateClienteRequest): Observable<Cliente> {
    return this.http.post<Cliente>(this.baseUrl, request);
  }

  atualizar(id: string, request: UpdateClienteRequest): Observable<Cliente> {
    return this.http.put<Cliente>(`${this.baseUrl}/${id}`, request);
  }

  excluir(id: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/${id}`);
  }
}