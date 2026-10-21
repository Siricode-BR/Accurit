import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Accurit — Gestão fiscal para escritórios contábeis",
  description:
    "Plataforma para centralizar a gestão fiscal, documental e as consultas do seu escritório contábil.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="pt-BR" className="h-full antialiased">
      <body className="flex min-h-full flex-col">{children}</body>
    </html>
  );
}
