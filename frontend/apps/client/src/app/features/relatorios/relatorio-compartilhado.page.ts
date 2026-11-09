import {
  Component,
  OnInit,
  inject,
  signal,
} from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';

import { CommonModule } from '@angular/common';
import { AccSpinnerComponent } from '@accurit/ui';

@Component({
  selector: 'app-relatorio-compartilhado-page',
  standalone: true,
  imports: [CommonModule, AccSpinnerComponent],
  templateUrl: './relatorio-compartilhado.page.html',
  styleUrl: './relatorio-compartilhado.page.css',
})
export class RelatorioCompartilhadoPage implements OnInit {
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  protected readonly carregando = signal(false);
  protected readonly erro = signal<string | null>(null);

  ngOnInit(): void {
    const token = this.route.snapshot.paramMap.get('token');
    if (!token) {
      this.erro.set('Link inválido.');
      return;
    }
    this.carregarRelatorio(token);
  }

  protected async carregarRelatorio(token: string): Promise<void> {
    this.carregando.set(true);
    try {
      const resposta = await fetch(
        `/api/v1/relatorios/compartilhado/${encodeURIComponent(token)}`,
      );
      if (!resposta.ok) {
        if (resposta.status === 404) {
          this.erro.set('Link não encontrado ou expirado.');
        } else {
          this.erro.set('Falha ao carregar o relatório.');
        }
        return;
      }
      const blob = await resposta.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const nome =
        resposta.headers.get('content-disposition')?.match(/filename="(.+)"/)?.[1] ??
        'relatorio';
      a.download = nome;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch {
      this.erro.set('Falha ao carregar o relatório.');
    } finally {
      this.carregando.set(false);
    }
  }
}
