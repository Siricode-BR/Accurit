/** Extrai a mensagem do envelope de erro da API (`{ error: { message } }`) ou do HttpErrorResponse. */
export function mensagemDeErro(erro: unknown, padrao: string): string {
  const e = erro as { error?: { error?: { message?: string }; message?: string }; message?: string };
  return e?.error?.error?.message ?? e?.error?.message ?? padrao;
}
