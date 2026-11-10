import { describe, expect, it } from 'vitest';
import { mensagemDeErro } from '@accurit/shared';

describe('mensagemDeErro', () => {
  it('usa a mensagem do servidor apenas para códigos da allowlist', () => {
    expect(
      mensagemDeErro({ status: 400, error: { code: 'VALIDACAO', message: 'Informe o CNPJ' } }),
    ).toBe('Informe o CNPJ');
    expect(
      mensagemDeErro({ status: 403, error: { code: 'ACESSO_NEGADO', message: 'Sem procuração' } }),
    ).toBe('Sem procuração');
  });

  it('nunca repassa HttpErrorResponse.message nem detalhes técnicos', () => {
    const erro = {
      status: 500,
      message: 'Http failure response for /api/v1/x: 500 Internal Server Error',
      error: { code: 'ERRO_INTERNO', message: 'diesel SELECT constraint panicked' },
    };
    const texto = mensagemDeErro(erro, 'Padrão');
    expect(texto).not.toContain('Http failure');
    expect(texto.toLowerCase()).not.toContain('diesel');
    expect(texto).not.toContain('SELECT');
  });

  it('mostra código de suporte em 5xx quando há requestId no envelope', () => {
    const texto = mensagemDeErro({
      status: 503,
      error: { code: 'BANCO_INDISPONIVEL', message: 'Banco indisponível', requestId: 'req-123' },
    });
    expect(texto).toContain('Código de suporte: req-123');
  });

  it('mapeia por status quando o código não é amigável', () => {
    expect(mensagemDeErro({ status: 0, error: {} })).toContain('Sem conexão');
    expect(mensagemDeErro({ status: 401, error: {} })).toContain('Sessão expirada');
    expect(mensagemDeErro({ status: 403, error: {} })).toContain('permissão');
    expect(mensagemDeErro({ status: 404, error: {} })).toContain('não foi encontrado');
    expect(mensagemDeErro({ status: 409, error: {} })).toContain('Conflito');
    expect(mensagemDeErro({ status: 429, error: {} })).toContain('Muitas tentativas');
  });
});
