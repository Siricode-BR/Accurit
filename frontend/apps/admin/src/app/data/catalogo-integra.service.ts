import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, tap } from 'rxjs';
import { LinhaCatalogoServicoDto, RequisicaoHabilitarServicoDto } from '@accurit/core';
import { mensagemDeErro } from '../core/http/mensagem-erro';

@Injectable({ providedIn: 'root' })
export class CatalogoIntegraService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/admin/integra-contador/catalogo';

  readonly servicos = signal<LinhaCatalogoServicoDto[]>([]);
  readonly carregando = signal(false);
  readonly erro = signal<string | null>(null);

  listar(): Observable<LinhaCatalogoServicoDto[]> {
    this.carregando.set(true);
    this.erro.set(null);
    return this.http.get<LinhaCatalogoServicoDto[]>(this.baseUrl).pipe(
      tap({
        next: (linhas) => {
          this.servicos.set(linhas);
          this.carregando.set(false);
        },
        error: (err: { error?: { message?: string } }) => {
          this.erro.set(mensagemDeErro(err, 'Falha ao carregar catálogo.'));
          this.carregando.set(false);
        },
      }),
    );
  }

  definirHabilitado(
    idSistema: string,
    idServico: string,
    habilitado: boolean,
  ): Observable<LinhaCatalogoServicoDto> {
    const req: RequisicaoHabilitarServicoDto = { habilitado };
    return this.http
      .patch<LinhaCatalogoServicoDto>(`${this.baseUrl}/${encodeURIComponent(idSistema)}/${encodeURIComponent(idServico)}`, req)
      .pipe(
        tap((atualizado) => {
          this.servicos.update((lista) =>
            lista.map((item) =>
              item.idSistema === atualizado.idSistema && item.idServico === atualizado.idServico
                ? atualizado
                : item,
            ),
          );
        }),
      );
  }
}
