import { expect, test, type Page } from '@playwright/test';

const CLIENT_EMAIL = process.env.E2E_CLIENT_EMAIL ?? 'titular@e2e.test';
const CLIENT_PASSWORD = process.env.E2E_CLIENT_PASSWORD ?? 'e2e-user-123';
const LIMITE_CHAMADAS = process.env.E2E_CHAMADAS_LIMITE ?? '1[,.]000';

function cpfComDv(base: string): string {
  const digitos = base.split('').map(Number);
  const calcular = (parcela: number[], peso: number) => {
    const soma = parcela.reduce((acc, digito, indice) => acc + digito * (peso - indice), 0);
    const resto = (soma * 10) % 11;
    return resto === 10 ? 0 : resto;
  };
  const primeiro = calcular(digitos, 10);
  const segundo = calcular([...digitos, primeiro], 11);
  return `${base}${primeiro}${segundo}`;
}

function cpfTerminandoEm(final: string): string {
  const inicio = 100_000_000 + (Date.now() % 700_000_000) + Math.floor(Math.random() * 1000);
  for (let numero = inicio; numero < inicio + 20_000; numero += 1) {
    const cpf = cpfComDv(String(numero).padStart(9, '0'));
    if (cpf.endsWith(final)) return cpf;
  }
  throw new Error('não foi possível gerar CPF');
}

async function login(page: Page): Promise<void> {
  await page.goto('/login');
  await page.locator('#email').fill(CLIENT_EMAIL);
  await page.locator('#senha').fill(CLIENT_PASSWORD);
  await page.getByRole('button', { name: 'Entrar', exact: true }).click();
  await expect(page).toHaveURL(/\/dashboard/);
}

test.describe('Portal do escritório', () => {
  test('login, CRUD de cliente, Dívida Ativa, procuração Trial, relatório e consumo', async ({ page, context }) => {
    await login(page);

    const cpf = cpfTerminandoEm('1');
    const nome = `Cliente E2E ${Date.now()}`;

    await page.goto('/clientes');
    await page.getByRole('button', { name: 'Novo cliente' }).click();
    await page.locator('#tipo').selectOption('PF');
    await page.locator('#cpfCnpj').fill(cpf);
    await page.locator('#nomeRazaoSocial').fill(nome);
    await page.locator('#email').fill(`cliente-${Date.now()}@e2e.test`);
    await page.getByRole('button', { name: 'Cadastrar Cliente' }).click();

    await expect(page).toHaveURL(/\/clientes\/[0-9a-f-]+$/);
    await expect(page.getByText(nome).first()).toBeVisible();

    // Valida procuração no Trial (mock determinístico; CPF termina em 1).
    await page.getByRole('button', { name: /Validar procura/i }).click();
    await expect(page.getByText(/AUTORIZADA|Autorizada/i).first()).toBeVisible({ timeout: 20_000 });

    // Consulta Dívida Ativa no simulador Trial.
    await page.goto('/consultas/divida-ativa');
    const campoDevedor = page.locator('acc-document-field input');
    await campoDevedor.click();
    await campoDevedor.pressSequentially('12345679891');
    await expect(page.getByRole('button', { name: 'Consultar', exact: true })).toBeEnabled();
    await page.getByRole('button', { name: 'Consultar', exact: true }).click();
    await expect(page.getByText('Simulado', { exact: true })).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText('Devedor', { exact: true })).toBeVisible();
    await expect(page.getByText('COM_PENDENCIAS').first()).toBeVisible();

    // Relatório PDF + link público acessível sem sessão.
    await page.goto('/relatorios');
    await page.getByRole('button', { name: 'Gerar relatório' }).click();
    await page.getByRole('button', { name: 'Gerar', exact: true }).click();
    await expect(page.getByText(/Concluído|Concluido/i).first()).toBeVisible({ timeout: 30_000 });

    await page.getByRole('button', { name: 'Compartilhar' }).first().click();
    const link = await page.locator('acc-modal input[readonly]').inputValue();
    expect(link).toContain('/relatorios/compartilhado/');

    const anonimo = await context.browser()!.newContext();
    const publica = await anonimo.newPage();
    await publica.goto(link);
    await expect(publica.getByText('Baixar relatório')).toBeVisible({ timeout: 20_000 });
    await anonimo.close();

    // Consumo do mês corrente e limites do plano.
    await page.goto('/consumo');
    await expect(page.getByRole('heading', { name: 'Consumo SERPRO' })).toBeVisible();
    await expect(page.getByText('Chamadas no mês')).toBeVisible();
    await expect(page.getByText(new RegExp(LIMITE_CHAMADAS)).first()).toBeVisible();
  });
});
