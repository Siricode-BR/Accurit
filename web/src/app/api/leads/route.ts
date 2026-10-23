import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();

    const { nome, email, telefone, escritorio, mensagem } = body;

    if (!nome || !email || !telefone || !escritorio) {
      return NextResponse.json(
        { error: 'Campos obrigatórios: nome, email, telefone, escritorio' },
        { status: 400 }
      );
    }

    // In a real implementation, you would save this to a database
    // For now, we'll just log it
    console.log('Novo lead recebido:', {
      nome,
      email,
      telefone,
      escritorio,
      mensagem,
      timestamp: new Date().toISOString(),
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Erro ao processar lead:', error);
    return NextResponse.json(
      { error: 'Erro interno do servidor' },
      { status: 500 }
    );
  }
}