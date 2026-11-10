import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import {
  ConsentimentoLgpdDto,
  RegistrarConsentimentoRequestDto,
} from '@accurit/core';

@Injectable({ providedIn: 'root' })
export class LgpdService {
  private readonly http = inject(HttpClient);

  exportarCliente(clienteId: string): Promise<Blob> {
    return firstValueFrom(
      this.http.get(`/api/v1/lgpd/clientes/${clienteId}/export`, { responseType: 'blob' }),
    );
  }

  anonimizarCliente(clienteId: string): Promise<void> {
    return firstValueFrom(
      this.http.delete<void>(`/api/v1/lgpd/clientes/${clienteId}`),
    );
  }

  listarConsentimentos(clienteId: string): Promise<ConsentimentoLgpdDto[]> {
    return firstValueFrom(
      this.http.get<ConsentimentoLgpdDto[]>(
        `/api/v1/lgpd/clientes/${clienteId}/consentimentos`,
      ),
    );
  }

  registrarConsentimento(
    clienteId: string,
    payload: RegistrarConsentimentoRequestDto,
  ): Promise<ConsentimentoLgpdDto> {
    return firstValueFrom(
      this.http.post<ConsentimentoLgpdDto>(
        `/api/v1/lgpd/clientes/${clienteId}/consentimentos`,
        payload,
      ),
    );
  }
}
