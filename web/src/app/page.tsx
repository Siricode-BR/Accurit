import Link from "next/link";

const beneficios = [
  {
    numero: "01",
    titulo: "Clientes em contexto",
    descricao:
      "Mantenha os dados dos clientes atendidos organizados por escritório.",
  },
  {
    numero: "02",
    titulo: "Consultas com clareza",
    descricao:
      "Acesse os serviços fiscais disponíveis e entenda o ambiente ativo.",
  },
  {
    numero: "03",
    titulo: "Acesso controlado",
    descricao:
      "Entre em um espaço de trabalho reservado à equipe do escritório.",
  },
];

export default function Home() {
  return (
    <div className="flex min-h-dvh flex-col bg-canvas text-text-primary">
      <header className="border-b border-border-subtle bg-surface">
        <div className="mx-auto flex w-full max-w-6xl items-center justify-between gap-4 px-5 py-4 sm:px-8">
          <Link
            aria-label="Accurit — página inicial"
            className="flex items-center gap-3 font-semibold tracking-tight"
            href="/"
          >
            <span className="grid size-10 place-items-center rounded-xl bg-primary text-lg font-bold text-white">
              A
            </span>
            <span className="text-xl">Accurit</span>
          </Link>
          <Link
            className="inline-flex min-h-11 items-center justify-center rounded-control bg-primary px-5 text-sm font-semibold text-white transition-colors hover:bg-(--action-primary-hover)"
            href="/app/login"
          >
            Acessar escritório
          </Link>
        </div>
      </header>

      <main className="flex-1">
        <section className="mx-auto grid w-full max-w-6xl gap-12 px-5 py-16 sm:px-8 sm:py-24 lg:grid-cols-[1.15fr_0.85fr] lg:items-center">
          <div>
            <span className="inline-flex items-center gap-2 rounded-full border border-border-subtle bg-surface px-3 py-1.5 text-sm font-medium text-text-secondary">
              <span
                aria-hidden="true"
                className="size-2 rounded-full bg-success"
              />
              Plataforma para escritórios contábeis
            </span>
            <h1 className="mt-6 max-w-3xl text-4xl font-semibold leading-tight tracking-tight sm:text-5xl lg:text-6xl">
              Sua operação fiscal, com mais{" "}
              <span className="text-primary">clareza.</span>
            </h1>
            <p className="mt-6 max-w-2xl text-lg leading-8 text-text-secondary">
              Organize os clientes do escritório e acesse, em um só lugar, as
              consultas fiscais habilitadas para sua operação.
            </p>
            <div className="mt-8 flex flex-wrap items-center gap-4">
              <Link
                className="inline-flex min-h-12 items-center justify-center rounded-control bg-primary px-6 text-base font-semibold text-white shadow-sm transition-colors hover:bg-(--action-primary-hover)"
                href="/app/login"
              >
                Entrar na plataforma
              </Link>
              <p className="text-sm text-text-secondary">
                Acesso exclusivo para equipes de escritórios contábeis
              </p>
            </div>
          </div>

          <aside
            aria-label="Recursos da plataforma"
            className="rounded-2xl border border-border-subtle bg-surface p-6 shadow-(--shadow-popover) sm:p-8"
          >
            <p className="text-sm font-semibold text-primary">
              Um espaço de trabalho mais organizado
            </p>
            <ul className="mt-6 space-y-6">
              {beneficios.map((beneficio) => (
                <li
                  className="flex gap-4 border-t border-border-subtle pt-5"
                  key={beneficio.numero}
                >
                  <span className="pt-0.5 text-sm font-semibold text-text-secondary">
                    {beneficio.numero}
                  </span>
                  <div>
                    <h2 className="font-semibold">{beneficio.titulo}</h2>
                    <p className="mt-1 text-sm leading-6 text-text-secondary">
                      {beneficio.descricao}
                    </p>
                  </div>
                </li>
              ))}
            </ul>
          </aside>
        </section>
      </main>

      <footer className="border-t border-border-subtle bg-surface">
        <div className="mx-auto flex w-full max-w-6xl flex-col gap-2 px-5 py-6 text-sm text-text-secondary sm:flex-row sm:items-center sm:justify-between sm:px-8">
          <span className="font-semibold text-text-primary">Accurit</span>
          <span>Gestão fiscal para escritórios contábeis</span>
        </div>
      </footer>
    </div>
  );
}
