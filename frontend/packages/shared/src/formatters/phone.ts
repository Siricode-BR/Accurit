export function formatPhone(phone: string | null | undefined): string {
  if (!phone) return '';
  const clean = phone.replace(/\D/g, '').slice(0, 11);
  if (clean.length <= 10) {
    return clean.replace(/^(\d{2})(\d{4})(\d{0,4})$/, '($1) $2-$3').trim();
  }
  return clean.replace(/^(\d{2})(\d{5})(\d{0,4})$/, '($1) $2-$3').trim();
}

export function formatPtBrDate(dateStr: string | Date | null | undefined): string {
  if (!dateStr) return '';
  const date = typeof dateStr === 'string' ? new Date(dateStr) : dateStr;
  if (isNaN(date.getTime())) return '';
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  }).format(date);
}

export function formatPtBrDateTime(dateStr: string | Date | null | undefined): string {
  if (!dateStr) return '';
  const date = typeof dateStr === 'string' ? new Date(dateStr) : dateStr;
  if (isNaN(date.getTime())) return '';
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(date);
}

