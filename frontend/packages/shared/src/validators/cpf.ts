export function sanitizeDocument(value: string | null | undefined): string {
  if (!value) return '';
  return value.replace(/\D/g, '');
}

export function isValidCPF(cpf: string | null | undefined): boolean {
  const clean = sanitizeDocument(cpf);
  if (clean.length !== 11 || /^(\d)\1{10}$/.test(clean)) {
    return false;
  }

  let sum = 0;
  for (let i = 0; i < 9; i++) {
    sum += parseInt(clean[i]!, 10) * (10 - i);
  }
  let rest = (sum * 10) % 11;
  if (rest === 10 || rest === 11) rest = 0;
  if (rest !== parseInt(clean[9]!, 10)) return false;

  sum = 0;
  for (let i = 0; i < 10; i++) {
    sum += parseInt(clean[i]!, 10) * (11 - i);
  }
  rest = (sum * 10) % 11;
  if (rest === 10 || rest === 11) rest = 0;
  return rest === parseInt(clean[10]!, 10);
}

