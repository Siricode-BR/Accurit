import Link from "next/link";

const RECURSOS = [
  {
    title: "Gestão de clientes PF e PJ",
    description:
      "Cadastre pessoas físicas e jurídicas com validação automática de CPF/CNPJ, histórico completo e organização por escritório.",
    icon: (
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z"
      />
    ),
  },
  {
    title: "Consultas fiscais integradas",
    description:
      "Integra Contador e Dívida Ativa (RS) direto na plataforma. O escritório em trial começa a operar imediatamente e migra para produção quando estiver pronto.",
    icon: (
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    ),
  },
  {
    title: "Arquitetura multi-escritório",
    description:
      "Cada escritório opera em ambiente próprio, com seus usuários, permissões por papel e base de clientes isolada.",
    icon: (
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M13.5 21v-7.5a.75.75 0 0 1 .75-.75h3a.75.75 0 0 1 .75.75V21m-4.5 0H2.36m11.14 0H18m0 0h3.64m-1.39 0V9.349M3.75 21V9.349m0 0a3.001 3.001 0 0 0 3.75-.615A2.993 2.993 0 0 0 9.75 9.75c.896 0 1.7-.393 2.25-1.016a2.993 2.993 0 0 0 2.25 1.016c.896 0 1.7-.393 2.25-1.016a3.001 3.001 0 0 0 3.75.614m-16.5 0a3.004 3.004 0 0 1-.621-4.72l1.189-1.9A1.5 1.5 0 0 1 5.378 3h13.243a1.5 1.5 0 0 1 1.06.44l1.19 1.189a3 3 0 0 1-.621 4.72M6.75 18h3.75a.75.75 0 0 0 .75-.75V13.5a.75.75 0 0 0-.75-.75H6.75a.75.75 0 0 0-.75.75v3.75c0 .414.336.75.75.75Z"
      />
    ),
  },
  {
    title: "Segurança de ponta a ponta",
    description:
      "Sessões autenticadas por token, proteção contra força em logins, tráfego isolado do banco e conformidade com a LGPD como princípio.",
    icon: (
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z"
      />
    ),
  },
] as const;

const PASSOS = [
  {
    step: "01",
    title: "Crie o escritório",
    description:
      "Cadastro rápido com e-mail verificado. O ambiente da empresa fica pronto na hora.",
  },
  {
    step: "02",
    title: "Monte a carteira",
    description:
      "Convide a equipe e cadastre clientes com validação de CPF/CNPJ em tempo real.",
  },
  {
    step: "03",
    title: "Consulte e acompanhe",
    description:
      "Consulte Integra Contador e Dívida Ativa com o cliente vinculado e histórico nas consultas.",
  },
] as const;

export default function Home() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden bg-linear-to-b from-[#eff6ff] via-white to-white">
        <div className="mx-auto max-w-6xl px-4 pt-16 pb-20 sm:px-6 sm:pt-24 lg:pt-28 lg:pb-24">
          <div className="flex flex-col items-center text-center">
            <span className="inline-flex items-center gap-2 rounded-full border border-(--acc-color-primary)/20 bg-white px-4 py-1.5 text-xs font-semibold text-(--acc-color-primary) shadow-sm">
              <span aria-hidden className="h-1.5 w-1.5 rounded-full bg-(--acc-color-success)" />
              Plataforma para escritórios contábeis
            </span>

            <h1 className="mt-6 max-w-3xl text-4xl font-extrabold leading-tight tracking-tight sm:text-5xl lg:text-6xl">
              A gestão do seu escritório contábil,{" "}
              <span className="text-(--acc-color-primary)">
                centralizada e sem atrito.
              </span>
            </h1>

            <p className="mt-6 max-w-2xl text-lg text-(--acc-color-text-muted) sm:text-xl">
              Clientes, equipe e consultas fiscais com Integra Contador e
              Dívida Ativa em um só lugar. Menos retrabalho operacional, mais
              tempo dedicado ao cliente.
            </p>

            <div className="mt-10 flex w-full flex-col items-center gap-4 sm:w-auto sm:flex-row">
              <a
                href="/app/"
                className="flex h-12 w-full items-center justify-center gap-2 rounded-xl bg-(--acc-color-primary) px-8 text-base font-semibold text-white shadow-md shadow-blue-900/20 transition-colors hover:bg-(--acc-color-primary-hover) sm:w-auto"
              >
                Acessar Portal do Escritório
                <svg
                  aria-hidden
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  className="h-4 w-4"
                >
                  <path
                    fillRule="evenodd"
                    d="M3 10a.75.75 0 0 1 .75-.75h10.638L10.23 5.29a.75.75 0 1 1 1.04-1.08l5.5 5.5a.75.75 0 0 1 0 1.08l-5.5 5.5a.75.75 0 1 1-1.04-1.08l4.158-4.16H3.75A.75.75 0 0 1 3 10Z"
                    clipRule="evenodd"
                  />
                </svg>
              </a>
              <Link
                href="/contato"
                className="flex h-12 w-full items-center justify-center rounded-xl border border-(--acc-color-border) bg-white px-8 text-base font-semibold text-(--acc-color-text-main) transition-colors hover:border-(--acc-color-primary) hover:text-(--acc-color-primary) sm:w-auto"
              >
                Solicitar uma demonstração
              </Link>
            </div>

            <p className="mt-6 text-xs font-medium text-(--acc-color-text-muted)">
              Integra Contador · Dívida Ativa RS · Ambiente trial do SERPRO
            </p>
          </div>
        </div>
      </section>

      {/* Recursos */}
      <section id="recursos" className="scroll-mt-24 bg-white">
        <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6">
          <div className="max-w-2xl">
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Feito para a rotina do contador
            </h2>
            <p className="mt-4 text-lg text-(--acc-color-text-muted)">
              Cada recurso nasceu da operação real de escritórios de contabilidade.
            </p>
          </div>

          <div className="mt-12 grid gap-6 sm:grid-cols-2">
            {RECURSOS.map((recurso) => (
              <div
                key={recurso.title}
                className="rounded-2xl border border-(--acc-color-border) bg-(--acc-color-surface) p-8 transition-shadow hover:shadow-md"
              >
                <span className="flex h-11 w-11 items-center justify-center rounded-xl bg-(--acc-color-primary)/10 text-(--acc-color-primary)">
                  <svg
                    aria-hidden
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth={1.5}
                    stroke="currentColor"
                    className="h-6 w-6"
                  >
                    {recurso.icon}
                  </svg>
                </span>
                <h3 className="mt-5 text-lg font-bold">{recurso.title}</h3>
                <p className="mt-2 text-sm leading-6 text-(--acc-color-text-muted)">
                  {recurso.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* SERPRO */}
      <section id="serpro" className="scroll-mt-24 bg-(--acc-color-bg)">
        <div className="mx-auto grid max-w-6xl items-center gap-12 px-4 py-20 sm:px-6 lg:grid-cols-2">
          <div>
            <span className="text-xs font-bold uppercase tracking-widest text-(--acc-color-primary)">
              Integração SERPRO
            </span>
            <h2 className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl">
              Do trial à produção, sem trocar de ferramenta
            </h2>
            <p className="mt-4 text-lg text-(--acc-color-text-muted)">
              O Accurit conversa com os serviços do SERPRO por meio do
              <strong> Integra Contador</strong> e da
              <strong> consulta de Dívida Ativa do RS</strong>.
            </p>
            <ul className="mt-8 space-y-4">
              {[
                "Conexão autenticada e monitorada com os serviços do SERPRO",
                "Modo trial para o escritório testar antes de contratar",
                "Histórico de consultas por cliente e por escritório",
                "Backup de certificado criptografado no banco",
              ].map((item) => (
                <li key={item} className="flex items-start gap-3">
                  <svg
                    aria-hidden
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    className="mt-0.5 h-5 w-5 shrink-0 text-(--acc-color-success)"
                  >
                    <path
                      fillRule="evenodd"
                      d="M16.704 4.153a.75.75 0 0 1 .143 1.052l-8 10.5a.75.75 0 0 1-1.127.075l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 0 1 1.05-.143Z"
                      clipRule="evenodd"
                    />
                  </svg>
                  <span className="text-sm leading-6 text-(--acc-color-text-main)">
                    {item}
                  </span>
                </li>
              ))}
            </ul>
          </div>

          <div className="rounded-2xl border border-(--acc-color-border) bg-white p-8 shadow-sm">
            <div className="flex items-center justify-between">
              <span className="text-sm font-semibold text-(--acc-color-text-muted)">
                Status do ambiente
              </span>
              <span className="inline-flex items-center gap-1.5 rounded-full bg-(--acc-color-success-bg) px-3 py-1 text-xs font-semibold text-(--acc-color-success)">
                <span aria-hidden className="h-1.5 w-1.5 rounded-full bg-current" />
                Conectado
              </span>
            </div>
            <dl className="mt-6 space-y-5 text-sm">
              <div className="flex items-center justify-between border-b border-(--acc-color-border)/60 pb-4">
                <dt className="text-(--acc-color-text-muted)">Integra Contador</dt>
                <dd className="font-semibold">Ativo</dd>
              </div>
              <div className="flex items-center justify-between border-b border-(--acc-color-border)/60 pb-4">
                <dt className="text-(--acc-color-text-muted)">Dívida Ativa RS</dt>
                <dd className="font-semibold">Ativa</dd>
              </div>
              <div className="flex items-center justify-between">
                <dt className="text-(--acc-color-text-muted)">Procurações vigentes</dt>
                <dd className="font-semibold">Por escritório</dd>
              </div>
            </dl>
          </div>
        </div>
      </section>

      {/* Como funciona */}
      <section id="como-funciona" className="scroll-mt-24 bg-white">
        <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6">
          <div className="max-w-2xl">
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Comece a operar em três passos
            </h2>
          </div>
          <div className="mt-12 grid gap-8 md:grid-cols-3">
            {PASSOS.map((passo) => (
              <div key={passo.step} className="relative border-t-2 border-(--acc-color-border) pt-6">
                <span className="text-sm font-bold text-(--acc-color-primary)">
                  {passo.step}
                </span>
                <h3 className="mt-2 text-lg font-bold">{passo.title}</h3>
                <p className="mt-2 text-sm leading-6 text-(--acc-color-text-muted)">
                  {passo.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA final */}
      <section className="bg-white">
        <div className="mx-auto max-w-6xl px-4 pb-20 sm:px-6">
          <div className="rounded-3xl bg-linear-to-br from-(--acc-color-primary) to-[#1e3a8a] px-8 py-14 text-center shadow-lg sm:px-16">
            <h2 className="mx-auto max-w-2xl text-3xl font-bold text-white sm:text-4xl">
              Pronto para digitalizar o seu escritório?
            </h2>
            <p className="mx-auto mt-4 max-w-xl text-base text-blue-100/90 sm:text-lg">
              Se você já é um escritório parceiro, acesse o portal. Se ainda
              não conhece a plataforma, fale com a nossa equipe.
            </p>
            <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
              <a
                href="/app/"
                className="flex h-12 w-full items-center justify-center gap-2 rounded-xl bg-white px-8 text-base font-semibold text-(--acc-color-primary) shadow-md transition-colors hover:bg-blue-50 sm:w-auto"
              >
                Acessar Portal do Escritório Contábil
                <svg
                  aria-hidden
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  className="h-4 w-4"
                >
                  <path
                    fillRule="evenodd"
                    d="M3 10a.75.75 0 0 1 .75-.75h10.638L10.23 5.29a.75.75 0 1 1 1.04-1.08l5.5 5.5a.75.75 0 0 1 0 1.08l-5.5 5.5a.75.75 0 1 1-1.04-1.08l4.158-4.16H3.75A.75.75 0 0 1 3 10Z"
                    clipRule="evenodd"
                  />
                </svg>
              </a>
              <Link
                href="/contato"
                className="flex h-12 w-full items-center justify-center rounded-xl border border-white/40 px-8 text-base font-semibold text-white transition-colors hover:bg-white/10 sm:w-auto"
              >
                Escritório é novo por aqui
              </Link>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
