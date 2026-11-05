import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Accurit | Plataforma para Escritórios Contábeis",
  description: "Organize clientes e centralize consultas fiscais em um só lugar.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="pt-BR" className="h-full antialiased">
      <body className="min-h-full flex flex-col font-sans">{children}</body>
    </html>
  );
}
