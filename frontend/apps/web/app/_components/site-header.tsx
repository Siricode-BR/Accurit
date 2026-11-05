import Link from "next/link";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-[var(--acc-color-border)]/70 bg-white/90 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
        <Link href="/" className="flex items-center gap-2.5">
          <span
            aria-hidden
            className="flex h-8 w-8 items-center justify-center rounded-lg bg-[var(--acc-color-primary)] text-sm font-bold text-white"
          >
            A
          </span>
          <span className="text-lg font-bold tracking-tight">
            Accurit
          </span>
        </Link>

        <nav className="hidden items-center gap-8 text-sm font-medium text-[var(--acc-color-text-muted)] md:flex">
          <a href="/#recursos" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Recursos
          </a>
          <a href="/#como-funciona" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Como funciona
          </a>
          <a href="/#serpro" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Integração SERPRO
          </a>
          <Link href="/contato" className="transition-colors hover:text-[var(--acc-color-text-main)]">
            Fale Conosco
          </Link>
        </nav>

        <div className="flex items-center gap-4">
          <Link
            href="/contato"
            className="text-sm font-medium text-[var(--acc-color-text-muted)] transition-colors hover:text-[var(--acc-color-text-main)] md:hidden"
          >
            Fale Conosco
          </Link>
          <a
            href="/app/"
            className="inline-flex h-10 items-center rounded-lg bg-[var(--acc-color-primary)] px-4 text-sm font-semibold text-white shadow-sm transition-colors hover:bg-[var(--acc-color-primary-hover)]"
          >
            Acessar Portal
          </a>
        </div>
      </div>
    </header>
  );
}
