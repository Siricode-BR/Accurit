export function urlRetornoValida(valor: string | null | undefined): string | null {
  if (!valor || !valor.startsWith('/') || valor.startsWith('//')) {
    return null;
  }

  const candidatos = [valor];
  try {
    candidatos.push(decodeURIComponent(valor));
  } catch {
    return null;
  }

  for (const candidato of candidatos) {
    if (
      !candidato.startsWith('/') ||
      candidato.startsWith('//') ||
      candidato.includes('\\') ||
      candidato.includes('://') ||
      /^[a-z][a-z0-9+.-]*:/i.test(candidato)
    ) {
      return null;
    }
  }

  return valor;
}
