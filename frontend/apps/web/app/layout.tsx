import type { Metadata } from "next";
import { Inter } from "next/font/google";
import Link from "next/link";
import "./globals.css";
import { SiteHeader } from "./_components/site-header";
import { SiteFooter } from "./_components/site-footer";

const inter = Inter({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-inter",
  fallback: ["ui-sans-serif", "system-ui", "sans-serif"],
});

export const metadata: Metadata = {
  title: {
    default: "Accurit | Plataforma para Escritórios Contábeis",
    template: "%s | Accurit",
  },
  description:
    "Centralize a gestão de clientes e as consultas fiscais do seu escritório contábil — Integra Contador e Dívida Ativa com integração SERPRO, num único portal.",
  keywords: [
    "plataforma contábil",
    "escritório contábil",
    "Integra Contador",
    "Dívida Ativa RS",
    "SERPRO",
    "gestão de clientes",
  ],
  openGraph: {
    title: "Accurit | Plataforma para Escritórios Contábeis",
    description:
      "Gestão de clientes e consultas fiscais integradas ao SERPRO em um só portal.",
    type: "website",
    locale: "pt_BR",
    siteName: "Accurit",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="pt-BR"
      className={`h-full scroll-smooth antialiased ${inter.variable}`}
    >
      <body className="flex min-h-full flex-col bg-white font-sans text-[var(--acc-color-text-main)]">
        <SiteHeader />
        <main className="flex-1">{children}</main>
        <SiteFooter />
      </body>
    </html>
  );
}
