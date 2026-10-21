import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { API_BASE_URL } from '../core/api.config';

export interface SaudeResposta {
  status: 'ok' | 'unavailable';
  service: string;
  db: 'ok' | 'unavailable';
}

@Injectable({ providedIn: 'root' })
export class SaudeService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = inject(API_BASE_URL);

  verificar(): Observable<SaudeResposta> {
    return this.http.get<SaudeResposta>(`${this.baseUrl}/health`);
  }
}
