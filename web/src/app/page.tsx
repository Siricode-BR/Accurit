const recursos = [
  {
    titulo: "Escritórios contábeis",
    descricao:
      "Organize a operação do escritório e acompanhe as demandas fiscais em um único ambiente.",
  },
  {
    titulo: "Clientes",
    descricao:
      "Acesse as informações e os documentos reunidos pelo seu escritório com mais clareza.",
  },
];

export default function Home() {
  return (
    <div className="flex min-h-dvh flex-col bg-slate-50 text-slate-900">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex w-full max-w-5xl items-center justify-between px-6 py-5">
          <span className="text-lg font-bold tracking-tight">Accurit</span>
          <a
            className="rounded-full border border-slate-300 px-4 py-2 text-sm font-medium transition-colors hover:border-slate-900 hover:text-slate-950"
            href="/app/login"
          >
            Acessar plataforma
          </a>
        </div>
      </header>

      <main className="flex-1">
        <section className="mx-auto grid w-full max-w-5xl gap-10 px-6 py-20 lg:grid-cols-[1.2fr_0.8fr] lg:items-center">
          <div>
            <span className="inline-flex rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-slate-600">
              Plataforma em evolução
            </span>
            <h1 className="mt-6 max-w-2xl text-4xl font-semibold leading-tight tracking-tight text-slate-950 sm:text-5xl">
              Gestão fiscal com mais clareza para o seu escritório.
            </h1>
            <p className="mt-6 max-w-xl text-lg leading-8 text-slate-600">
              O Accurit reúne a operação fiscal, os documentos e o atendimento
              aos clientes em uma experiência simples e confiável.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <a
                className="rounded-full bg-slate-950 px-5 py-3 text-sm font-semibold text-white transition-colors hover:bg-slate-800"
                href="/admin/login"
              >
                Sou do escritório
              </a>
              <a
                className="rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold transition-colors hover:border-slate-900"
                href="/app/login"
              >
                Sou cliente
              </a>
            </div>
          </div>

          <div className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-wider text-slate-500">
              Uma base para crescer
            </p>
            <div className="mt-5 space-y-5" id="base">
              {recursos.map((recurso) => (
                <article key={recurso.titulo}>
                  <h2 className="text-base font-semibold text-slate-950">
                    {recurso.titulo}
                  </h2>
                  <p className="mt-1 text-sm leading-6 text-slate-600">
                    {recurso.descricao}
                  </p>
                </article>
              ))}
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t border-slate-200">
        <div className="mx-auto flex w-full max-w-5xl flex-col gap-2 px-6 py-6 text-sm text-slate-500 sm:flex-row sm:items-center sm:justify-between">
          <span>Accurit</span>
          <span>Fase 1 &middot; Fundação da plataforma</span>
        </div>
      </footer>
    </div>
  );
}
