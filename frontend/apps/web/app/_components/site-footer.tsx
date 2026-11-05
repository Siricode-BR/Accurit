import Link from "next/link";

export function SiteFooter() {
  return (
    <footer className="border-t border-[var(--acc-color-border)]/70 bg-[var(--acc-color-bg)]">
      <div className="mx-auto flex max-w-6xl flex-col gap-6 px-4 py-10 sm:flex-row sm:items-center sm:justify-between sm:px-6">
        <div>
          <div className="flex items-center gap-2.5">
            <span
              aria-hidden
              className="flex h-7 w-7 items-center justify-center rounded-md bg-[var(--acc-color-primary)] text-xs font-bold text-white"
            >
              A
            </span>
            <span className="text-base font-bold tracking-tight">Accurit</span>
          </div>
          <p className="mt-2 text-sm text-[var(--acc-color-text-muted)]">
            A plataforma que conecta o seu escritório contábil ao que importa.
          </p>
        </div>

        <nav className="flex flex-wrap items-center gap-x-8 gap-y-3 text-sm font-medium text-[var(--acc-color-text-muted)]">
          <a href="/#recursos" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Recursos
          </a>
          <Link href="/contato" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Fale Conosco
          </Link>
          <a
            href="/app/"
            className="font-semibold text-[var(--acc-color-primary)] transition-colors hover:text-[var(--acc-color-primary-hover)]"
          >
            Portal do Escritório →
          </a>
        </nav>
      </div>

      <div className="border-t border-[var(--acc-color-border)]/70">
        <p className="mx-auto max-w-6xl px-4 py-4 text-xs text-[var(--acc-color-text-muted)] sm:px-6">
          © 2026 Accurit. Todos os direitos reservados.
        </p>
      </div>
    </footer>
  );
}
