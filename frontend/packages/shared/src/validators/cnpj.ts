import { sanitizeDocument } from './cpf';

export function isValidCNPJ(cnpj: string | null | undefined): boolean {
  const clean = sanitizeDocument(cnpj);
  if (clean.length !== 14 || /^(\d)\1{13}$/.test(clean)) {
    return false;
  }

  const calcDigit = (slice: string, factors: number[]): number => {
    const total = slice
      .split('')
      .reduce((sum, num, index) => sum + parseInt(num, 10) * factors[index]!, 0);
    const rest = total % 11;
    return rest < 2 ? 0 : 11 - rest;
  };

  const d1 = calcDigit(clean.slice(0, 12), [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
  const d2 = calcDigit(clean.slice(0, 13), [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);

  return d1 === parseInt(clean[12]!, 10) && d2 === parseInt(clean[13]!, 10);
}

