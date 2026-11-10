/**
 * Função centralizada de mapeamento de erros para o frontend.
 * NUNCA passa HttpErrorResponse.message diretamente.
 * Apenas usa error.error.message quando o code está na allowlist amigável.
 * Todos os usos de error.message/String(erro)/JSON.stringify(erro) devem ser
 * substituídos por esta função.
 *
 * Allowlist de códigos que podem exibir a mensagem raw do servidor:
 * - VALIDACAO
 * - CONFLITO
 * - ACESSO_NEGADO
 * - LIMITE_SERPRO
 * - PRODUCAO_BLOQUEADA
 * - ESTADO_INDETERMINADO
 */

const allowlist = [
  'VALIDACAO',
  'CONFLITO',
  'ACESSO_NEGADO',
  'LIMITE_SERPRO',
  'PRODUCAO_BLOQUEADA',
  'ESTADO_INDETERMINADO',
];

export function mensagemDeErro(
  erro: unknown,
  padrao: string = 'Ocorreu um erro inesperado.'
): string {
  // Se for HttpErrorResponse do Angular (erro de requisição HTTP)
  if (erro && typeof erro === 'object' && 'status' in erro && 'error' in erro) {
    const httpError = erro as { status: number; error: { code?: string; message?: string } };
    const code = httpError.error?.code;
    const rawMessage = httpError.error?.message;

    // Allowlist de códigos que podem exibir a mensagem raw do servidor
    if (code && allowlist.includes(code) && rawMessage) {
      // Código amigável - pode exibir a mensagem do servidor
      return rawMessage;
    }

    // Mapeamento por status HTTP
    const status = httpError.status;

    switch (status) {
      case 0:
        // Sem conexão / rede offline
        return 'Sem conexão com o servidor. Verifique sua rede e tente novamente.';

      case 400:
        // Bad request - revisar os dados
        return 'Requisição inválida. Por favor, verifique os dados informados e tente novamente.';

      case 401:
        // Sessão expirada
        return 'Sessão expirada. Faça login novamente para continuar.';

      case 403:
        // Sem permissão
        return 'Não tem permissão para acessar este recurso.';

      case 404:
        // Não encontrado
        return 'O recurso solicitado não foi encontrado.';

      case 409:
        // Já existe/alterado
        return 'Conflito: o recurso já existe ou foi alterado. Verifique os dados informados.';

      case 429:
        // Muitas tentativas
        return 'Muitas tentativas. Aguarde um momento e tente novamente.';

      case 500:
      case 502:
      case 503:
      case 504:
        // 5xx - indisponível
        // Em produção, NUNCA vazar stack, host ou detalhes técnicos
        // Apenas mostrar mensagem genérica + código de suporte
        const requestId =
          (httpError as { error?: { requestId?: string } }).error?.requestId ??
          (httpError as { requestId?: string }).requestId;
        if (requestId) {
          return `Serviço indisponível. Código de suporte: ${requestId}`;
        }
        return 'Serviço temporariamente indisponível. Tente novamente mais tarde.';

      default:
        // Outros erros - mensagem genérica
        return padrao;
    }
  }

  // Se for erro de aplicação com structure conhecida
  if (erro && typeof erro === 'object' && 'code' in erro && 'message' in erro) {
    const appError = erro as { code: string; message: string };
    const code = appError.code;

    // Allowlist de códigos de aplicação
    if (allowlist.includes(code) && appError.message) {
      return appError.message;
    }

    // Mensagem genérica para códigos não permitidos
    return padrao;
  }

  // Erro desconhecido - mensagem genérica
  return padrao;
}