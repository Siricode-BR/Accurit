import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-acesso-negado',
  imports: [RouterLink],
  template: `
    <main class="pagina">
      <header class="cabecalho">
        <span class="marca">Accurit</span>
      </header>

      <section class="conteudo">
        <p class="sobretitulo">Acesso</p>
        <h1>Acesso negado</h1>
        <p class="descricao">
          Sua conta não tem permissão para acessar este recurso no Portal do
          Cliente.
        </p>
        <p><a routerLink="/inicio">Voltar ao início</a></p>
      </section>

      <footer class="rodape">Fase 2A &middot; Autenticação</footer>
    </main>
  `,
  styles: `
    .pagina {
      display: flex;
      flex-direction: column;
      gap: 2rem;
      max-width: 32rem;
      margin: 0 auto;
      padding: 4rem 1.5rem;
    }

    .cabecalho {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .marca {
      font-weight: 700;
    }

    .sobretitulo {
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      opacity: 0.7;
    }

    .descricao {
      opacity: 0.85;
    }

    .rodape {
      font-size: 0.75rem;
      opacity: 0.6;
    }
  `,
})
export class AcessoNegado {}
