import { expect, test } from '@playwright/test';

const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? 'admin@e2e.test';
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? 'e2e-admin-123';
const ADMIN_NOME = process.env.E2E_ADMIN_NOME ?? 'Gestor E2E';
const CLIENT_EMAIL = process.env.E2E_CLIENT_EMAIL ?? 'titular@e2e.test';
const CLIENT_PASSWORD = process.env.E2E_CLIENT_PASSWORD ?? 'e2e-user-123';
const PLANO = process.env.E2E_PLANO ?? 'E2E';

test.describe('Backoffice', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.locator('#email').fill(ADMIN_EMAIL);
    await page.locator('#senha').fill(ADMIN_PASSWORD);
    await page.getByRole('button', { name: /Entrar como Administrador/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('login, consumo/planos e administradores', async ({ page }) => {
    await page.goto('/consumo');
    await expect(page.getByRole('heading', { name: 'Consumo e planos' })).toBeVisible();
    await expect(page.getByText('Chamadas', { exact: true }).first()).toBeVisible();

    await page.getByRole('button', { name: 'Planos', exact: true }).click();
    await expect(page.getByText(PLANO, { exact: true }).first()).toBeVisible();

    await page.goto('/administradores');
    await expect(page.getByRole('heading', { name: 'Administradores' })).toBeVisible();
    await expect(page.getByText(ADMIN_NOME)).toBeVisible();
  });

  test('token de escritório não acessa o backoffice', async ({ request }) => {
    const login = await request.post('http://127.0.0.1:4202/api/v1/auth/client/login', {
      data: { email: CLIENT_EMAIL, senha: CLIENT_PASSWORD },
    });
    expect(login.status()).toBe(200);
    const corpo = await login.json();
    const resposta = await request.get('http://127.0.0.1:4201/api/v1/admin/consumo', {
      headers: { authorization: `Bearer ${corpo.accessToken}` },
    });
    expect([401, 403]).toContain(resposta.status());
  });
});
