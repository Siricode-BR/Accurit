'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { isValidEmail, formatPhone } from '@accurit/shared';

export default function ContatoPage() {
  const [formData, setFormData] = useState({
    nome: '',
    email: '',
    telefone: '',
    escritorio: '',
    mensagem: '',
  });

  const [status, setStatus] = useState<'idle' | 'submitting' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    if (name === 'telefone') {
      setFormData((prev) => ({ ...prev, telefone: formatPhone(value) }));
    } else {
      setFormData((prev) => ({ ...prev, [name]: value }));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMessage('');

    if (!isValidEmail(formData.email)) {
      setErrorMessage('Por favor, informe um e-mail válido.');
      return;
    }

    setStatus('submitting');

    try {
      const response = await fetch('/api/leads', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      });

      const json = await response.json();

      if (!response.ok) {
        throw new Error(json.error || 'Erro ao processar envio.');
      }

      setStatus('success');
      setFormData({ nome: '', email: '', telefone: '', escritorio: '', mensagem: '' });
    } catch (err: unknown) {
      setStatus('error');
      setErrorMessage(err instanceof Error ? err.message : 'Erro ao enviar. Tente novamente mais tarde.');
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md mx-auto bg-white rounded-xl shadow-md p-8">
        <div className="mb-6">
          <Link href="/" className="text-sm text-blue-700 font-semibold hover:underline">
            &larr; Voltar para o início
          </Link>
          <h1 className="text-2xl font-bold text-slate-900 mt-2">Fale Conosco</h1>
          <p className="text-sm text-slate-600 mt-1">
            Conheça a plataforma Accurit para o seu escritório contábil.
          </p>
        </div>

        {status === 'success' ? (
          <div className="rounded-lg bg-green-50 p-4 border border-green-200 text-green-800">
            <h3 className="font-semibold">Mensagem enviada com sucesso!</h3>
            <p className="text-sm mt-1">Nossa equipe entrará em contato em breve.</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {errorMessage && (
              <div className="rounded-lg bg-red-50 p-3 text-sm text-red-700 border border-red-200">
                {errorMessage}
              </div>
            )}

            <div>
              <label htmlFor="nome" className="block text-sm font-medium text-slate-700">
                Nome completo *
              </label>
              <input
                id="nome"
                name="nome"
                type="text"
                required
                value={formData.nome}
                onChange={handleChange}
                className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-600 focus:outline-none"
              />
            </div>

            <div>
              <label htmlFor="email" className="block text-sm font-medium text-slate-700">
                E-mail corporativo *
              </label>
              <input
                id="email"
                name="email"
                type="email"
                required
                value={formData.email}
                onChange={handleChange}
                className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-600 focus:outline-none"
              />
            </div>

            <div>
              <label htmlFor="telefone" className="block text-sm font-medium text-slate-700">
                Telefone / WhatsApp *
              </label>
              <input
                id="telefone"
                name="telefone"
                type="tel"
                required
                placeholder="(00) 00000-0000"
                value={formData.telefone}
                onChange={handleChange}
                className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-600 focus:outline-none"
              />
            </div>

            <div>
              <label htmlFor="escritorio" className="block text-sm font-medium text-slate-700">
                Nome do escritório *
              </label>
              <input
                id="escritorio"
                name="escritorio"
                type="text"
                required
                value={formData.escritorio}
                onChange={handleChange}
                className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-600 focus:outline-none"
              />
            </div>

            <div>
              <label htmlFor="mensagem" className="block text-sm font-medium text-slate-700">
                Mensagem
              </label>
              <textarea
                id="mensagem"
                name="mensagem"
                rows={3}
                value={formData.mensagem}
                onChange={handleChange}
                className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-600 focus:outline-none"
              ></textarea>
            </div>

            <button
              type="submit"
              disabled={status === 'submitting'}
              className="w-full rounded-md bg-blue-700 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-blue-800 disabled:opacity-50"
            >
              {status === 'submitting' ? 'Enviando...' : 'Solicitar Demonstração'}
            </button>
          </form>
        )}
      </div>
    </div>
  );
}

