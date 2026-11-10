import { Injectable, inject } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  AdministradorDto,
  AlterarSenhaAdministradorRequestDto,
  AtualizarAdministradorRequestDto,
  CriarAdministradorRequestDto,
  ListagemAdministradoresDto,
} from '@accurit/core';

/** Backoffice SaaS: gerencia apenas `administradores` (sem usuários de escritório). */
@Injectable({ providedIn: 'root' })
export class AdministradorService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin/administradores';

  async listar(pagina = 1, tamanho = 20, q = ''): Promise<ListagemAdministradoresDto> {
    let params = new HttpParams().set('page', String(pagina)).set('pageSize', String(tamanho));
    if (q.trim()) params = params.set('q', q.trim());
    return firstValueFrom(this.http.get<ListagemAdministradoresDto>(this.baseUrl, { params }));
  }

  async criar(payload: CriarAdministradorRequestDto): Promise<AdministradorDto> {
    return firstValueFrom(this.http.post<AdministradorDto>(this.baseUrl, payload));
  }

  async atualizar(id: string, payload: AtualizarAdministradorRequestDto): Promise<AdministradorDto> {
    return firstValueFrom(this.http.patch<AdministradorDto>(`${this.baseUrl}/${id}`, payload));
  }

  async alterarSenha(id: string, payload: AlterarSenhaAdministradorRequestDto): Promise<AdministradorDto> {
    return firstValueFrom(this.http.put<AdministradorDto>(`${this.baseUrl}/${id}`, payload));
  }
}
