import { HttpClient, HttpErrorResponse, HttpHeaders } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, catchError, firstValueFrom, map, of, shareReplay, tap, throwError } from 'rxjs';

import { API_BASE_URL } from '../api.config';
import { Credenciais, SessaoResposta, Usuario, UsuarioAtualResposta } from './auth.models';
import { SessionStore } from './session.store';

const CABECALHOS_REFRESH = new HttpHeaders({
  'Content-Type': 'application/json',
  'X-Requested-With': 'accurit',
});

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = inject(API_BASE_URL);
  private readonly sessao = inject(SessionStore);

  private renovacaoEmAndamento: Observable<void> | null = null;

  autenticado(): boolean {
    return this.sessao.autenticado();
  }

  accessToken(): string | null {
    return this.sessao.accessToken();
  }

  usuario(): Usuario | null {
    return this.sessao.usuario();
  }

  login(credenciais: Credenciais): Observable<Usuario> {
    return this.http
      .post<SessaoResposta>(`${this.baseUrl}/auth/client/login`, credenciais)
      .pipe(
        tap((resposta) => {
          this.sessao.definirSlug(credenciais.escritorio.trim().toLowerCase());
          this.sessao.definirSessao(resposta);
        }),
        map((resposta) => resposta.usuario),
      );
  }

  renovar(): Observable<void> {
    if (this.renovacaoEmAndamento) {
      return this.renovacaoEmAndamento;
    }

    this.renovacaoEmAndamento = this.http
      .post<SessaoResposta>(
        `${this.baseUrl}/auth/client/refresh`,
        {},
        { headers: CABECALHOS_REFRESH },
      )
      .pipe(
        tap((resposta) => this.sessao.definirSessao(resposta)),
        map(() => undefined),
        catchError((erro: unknown) => {
          this.sessao.limpar();
          return throwError(() => erro);
        }),
        shareReplay({ bufferSize: 1, refCount: false }),
        tap({
          finalize: () => {
            this.renovacaoEmAndamento = null;
          },
        }),
      );

    return this.renovacaoEmAndamento;
  }

  confirmarSessao(): Observable<void> {
    return this.http.get<UsuarioAtualResposta>(`${this.baseUrl}/auth/client/me`).pipe(
      tap((resposta) => this.sessao.definirUsuario(resposta.usuario)),
      map(() => undefined),
    );
  }

  restaurarSessao(): Promise<void> {
    return firstValueFrom(this.renovar().pipe(catchError(() => of(undefined))));
  }

  logout(): void {
    this.http
      .post<void>(`${this.baseUrl}/auth/client/logout`, {}, { headers: CABECALHOS_REFRESH })
      .subscribe({
        next: () => undefined,
        error: () => undefined,
      });
    this.sessao.limpar();
  }

  tratarErroDeAcesso(erro: unknown): boolean {
    return erro instanceof HttpErrorResponse && erro.status === 401;
  }
}
