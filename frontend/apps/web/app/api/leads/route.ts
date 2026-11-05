import { NextRequest, NextResponse } from 'next/server';
import { isValidEmail, formatPhone } from '@accurit/shared';

export interface LeadPayload {
  nome: string;
  email: string;
  telefone: string;
  escritorio: string;
  mensagem?: string;
}

export async function POST(request: NextRequest) {
  try {
    const body = (await request.json()) as Partial<LeadPayload>;
    const { nome, email, telefone, escritorio, mensagem } = body;

    if (!nome || !email || !telefone || !escritorio) {
      return NextResponse.json(
        { error: 'Campos obrigatórios: nome, email, telefone, escritorio.' },
        { status: 400 }
      );
    }

    if (!isValidEmail(email)) {
      return NextResponse.json(
        { error: 'E-mail em formato inválido.' },
        { status: 400 }
      );
    }

    const leadFormatado = {
      nome: nome.trim(),
      email: email.trim().toLowerCase(),
      telefone: formatPhone(telefone),
      escritorio: escritorio.trim(),
      mensagem: mensagem?.trim() ?? '',
      recebidoEm: new Date().toISOString(),
    };

    // Logging do lead capturado (backend externo receberá via webhook ou pipeline)
    console.info('[Lead Captured]', leadFormatado);

    return NextResponse.json({ success: true, data: leadFormatado });
  } catch (error) {
    console.error('[Lead Error]', error);
    return NextResponse.json(
      { error: 'Erro interno ao processar requisição.' },
      { status: 500 }
    );
  }
}

