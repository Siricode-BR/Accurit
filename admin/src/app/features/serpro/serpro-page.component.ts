import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { Component, OnInit, inject, signal } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

import { SerproService, SerproConfiguracao } from './serpro.service';

@Component({
  selector: 'app-serpro-page',
  standalone: true,
  imports: [
    CommonModule,
    RouterLink,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>Ambiente SERPRO</mat-card-title>
        <mat-card-subtitle>
          Define em qual ambiente o escritório consome os serviços SERPRO.
        </mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <a mat-button routerLink="/tenants">
          <mat-icon>arrow_back</mat-icon>
          Voltar aos escritórios
        </a>

        @if (carregando()) {
          <div class="loading-container">
            <mat-spinner diameter="40"></mat-spinner>
            <p>Carregando configuração...</p>
          </div>
        } @else if (config() === null) {
          <div class="empty-state">
            <mat-icon>cloud_off</mat-icon>
            <h3>Configuração indisponível</h3>
            <p>Não foi possível carregar o ambiente SERPRO. Tente recarregar.</p>
          </div>
        } @else {
          <div class="estado">
            <span class="badge" [class.trial]="config()!.modoTrialSerpro" [class.producao]="!config()!.modoTrialSerpro">
              {{ rotuloAmbiente() }}
            </span>
            <p class="atualizacao">
              Ambiente efetivo das consultas, versão {{ config()!.versao }}, atualizada em {{ data() }}.
            </p>
          </div>

          <div class="header-actions">
            @if (confirmaProducao()) {
              <p class="aviso" role="alert">
                Migrar usa credenciais de produção e pode devolver erro se faltarem credencial ou
                certificado A1. A versão avança e o gestor precisa confirmar.
              </p>
              <button mat-raised-button color="warn" (click)="desistirProducao()">Não migrar</button>
              <button mat-raised-button color="primary" (click)="alternar(true)" [disabled]="alterando()">
                Confirmar migração
              </button>
            } @else if (config()!.modoTrialSerpro) {
              <button mat-raised-button color="warn" (click)="alternar(false)" [disabled]="alterando()">
                Migrar para produção
              </button>
            } @else {
              <button mat-raised-button (click)="alternar(true)" [disabled]="alterando()">
                Voltar ao TRIAL
              </button>
            }
          </div>
        }
      </mat-card-content>
    </mat-card>
  `,
  styles: `
    .loading-container,
    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 16px;
      padding: 32px;
    }

    .empty-state {
      text-align: center;
      color: #666;

      mat-icon {
        font-size: 64px;
        width: 64px;
        height: 64px;
      }
    }

    .estado {
      display: flex;
      gap: 16px;
      align-items: center;
      margin: 16px 0;
    }

    .atualizacao {
      color: #555;
      margin: 0;
    }

    .badge {
      display: inline-block;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 500;
      text-transform: uppercase;
    }

    .badge.trial {
      background-color: #fff3e0;
      color: #e65100;
    }

    .badge.producao {
      background-color: #e8f5e9;
      color: #2e7d32;
    }

    .header-actions {
      display: flex;
      gap: 12px;
      align-items: center;
    }

    .aviso {
      flex: 1;
      color: #b71c1c;
    }
  `,
})
export class SerproPageComponent implements OnInit {
  private readonly rota = inject(ActivatedRoute);
  private readonly serproService = inject(SerproService);
  private readonly snackBar = inject(MatSnackBar);

  protected readonly carregando = signal(true);
  protected readonly config = signal<SerproConfiguracao | null>(null);
  protected readonly alterando = signal(false);
  protected readonly confirmaProducao = signal(false);
  protected readonly escritorioId = signal<string | null>(null);

  protected readonly rotuloAmbiente = () =>
    this.config()?.modoTrialSerpro ? 'TRIAL' : 'PRODUÇÃO';

  protected readonly data = () =>
    new Date(this.config()?.atualizadoEm ?? '').toLocaleString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });

  ngOnInit(): void {
    const id = this.rota.snapshot.paramMap.get('escritorioId');
    if (!id) {
      this.carregando.set(false);
      return;
    }
    this.escritorioId.set(id);
    this.carregar(id);
  }

  private carregar(id: string): void {
    this.carregando.set(true);
    this.serproService.obterConfiguracao(id).subscribe({
      next: (config) => {
        this.config.set(config);
        this.carregando.set(false);
      },
      error: () => {
        this.config.set(null);
        this.carregando.set(false);
      },
    });
  }

  protected alternar(modoTrial: boolean): void {
    const id = this.escritorioId();
    const config = this.config();
    if (!id || !config || this.alterando()) return;

    if (!modoTrial && !this.confirmaProducao()) {
      this.confirmaProducao.set(true);
      return;
    }

    this.alterando.set(true);
    this.serproService
      .alterarModoTrial({ escritorioId: id, modoTrialSerpro: modoTrial, versao: config.versao })
      .subscribe({
        next: (resposta) => {
          this.config.set({
            ...config,
            modoTrialSerpro: resposta.modoTrialSerpro,
            ambienteEfetivo: resposta.ambienteEfetivo,
            versao: resposta.versaoAtual,
            atualizadoEm: resposta.atualizadoEm,
          });
          this.confirmaProducao.set(false);
          this.alterando.set(false);
          this.snackBar.open('Ambiente SERPRO atualizado.', 'Fechar', { duration: 3000 });
        },
        error: (erro: unknown) => {
          this.alterando.set(false);
          if (erro instanceof HttpErrorResponse && erro.status === 409) {
            this.carregar(id);
            this.snackBar.open(
              'Versão divergiu: a configuração mudou em outro lugar, recarregamos os dados.',
              'Fechar',
              { duration: 5000 },
            );
            return;
          }
          this.snackBar.open(this.mensagemErro(erro, 'Não foi possível alterar o ambiente.'), 'Fechar', {
            duration: 5000,
          });
        },
      });
  }

  protected desistirProducao(): void {
    this.confirmaProducao.set(false);
  }

  private mensagemErro(erro: unknown, padrao: string): string {
    const resposta = erro instanceof HttpErrorResponse ? erro.error : null;
    return resposta?.error?.message ?? padrao;
  }
}
