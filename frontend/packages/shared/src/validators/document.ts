import { sanitizeDocument, isValidCPF } from './cpf';
import { isValidCNPJ } from './cnpj';

export type DocumentType = 'CPF' | 'CNPJ' | 'INVALID';

export function getDocumentType(document: string | null | undefined): DocumentType {
  const clean = sanitizeDocument(document);
  if (clean.length === 11) return 'CPF';
  if (clean.length === 14) return 'CNPJ';
  return 'INVALID';
}

export function isValidDocument(document: string | null | undefined): boolean {
  const clean = sanitizeDocument(document);
  if (clean.length === 11) return isValidCPF(clean);
  if (clean.length === 14) return isValidCNPJ(clean);
  return false;
}

