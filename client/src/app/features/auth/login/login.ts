import { HttpErrorResponse } from '@angular/common/http';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';

import { AuthService } from '../../../core/auth/auth.service';
import { urlRetornoValida } from '../../../core/auth/return-url';
import { SessionStore } from '../../../core/auth/session.store';

@Component({
  selector: 'app-login',
  imports: [FormsModule],
  templateUrl: './login.html',
  styleUrl: './login.css',
})
export class Login {
  private readonly auth = inject(AuthService);
  private readonly sessao = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly rota = inject(ActivatedRoute);

  protected readonly escritorio = signal(this.sessao.slug() ?? '');
  protected readonly email = signal('');
  protected readonly senha = signal('');
  protected readonly enviando = signal(false);
  protected readonly erro = signal<string | null>(null);

  protected entrar(): void {
    if (this.enviando()) {
      return;
    }

    this.enviando.set(true);
    this.erro.set(null);

    this.auth
      .login({
        escritorio: this.escritorio(),
        email: this.email(),
        senha: this.senha(),
      })
      .subscribe({
        next: () => {
          this.enviando.set(false);
          const retorno = urlRetornoValida(
            this.rota.snapshot.queryParamMap.get('returnUrl'),
          );
          void this.router.navigateByUrl(retorno ?? '/inicio');
        },
        error: (erro: unknown) => {
          this.enviando.set(false);
          this.erro.set(this.mensagemDeErro(erro));
        },
      });
  }

  private mensagemDeErro(erro: unknown): string {
    if (erro instanceof HttpErrorResponse) {
      if (erro.status === 429) {
        return 'Muitas tentativas de login. Aguarde um instante e tente novamente.';
      }
      if (erro.status === 401) {
        return 'Credenciais inválidas. Confira escritório, e-mail e senha.';
      }
      if (erro.status === 503) {
        return 'Serviço temporariamente indisponível. Tente novamente em instantes.';
      }
    }
    return 'Não foi possível entrar agora. Tente novamente.';
  }
}
