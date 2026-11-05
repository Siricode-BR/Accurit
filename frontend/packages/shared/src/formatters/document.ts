import { sanitizeDocument } from '../validators/cpf';

export function formatCPF(cpf: string | null | undefined): string {
  const clean = sanitizeDocument(cpf).slice(0, 11);
  if (!clean) return '';
  return clean
    .replace(/^(\d{3})(\d)/, '$1.$2')
    .replace(/^(\d{3})\.(\d{3})(\d)/, '$1.$2.$3')
    .replace(/^(\d{3})\.(\d{3})\.(\d{3})(\d{1,2})$/, '$1.$2.$3-$4');
}

export function formatCNPJ(cnpj: string | null | undefined): string {
  const clean = sanitizeDocument(cnpj).slice(0, 14);
  if (!clean) return '';
  return clean
    .replace(/^(\d{2})(\d)/, '$1.$2')
    .replace(/^(\d{2})\.(\d{3})(\d)/, '$1.$2.$3')
    .replace(/^(\d{2})\.(\d{3})\.(\d{3})(\d)/, '$1.$2.$3/$4')
    .replace(/^(\d{2})\.(\d{3})\.(\d{3})\/(\d{4})(\d{1,2})$/, '$1.$2.$3/$4-$5');
}

export function formatDocument(doc: string | null | undefined): string {
  const clean = sanitizeDocument(doc);
  if (clean.length <= 11) {
    return formatCPF(clean);
  }
  return formatCNPJ(clean);
}

