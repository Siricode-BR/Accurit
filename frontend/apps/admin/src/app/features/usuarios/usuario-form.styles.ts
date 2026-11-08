/** Estilos compartilhados pelos formulários de usuário (criação, edição e senha). */
export const USUARIO_FORM_STYLES = `
  .form {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
    width: 100%;
  }
  @media (min-width: 560px) {
    .form { grid-template-columns: 1fr 1fr; }
  }
  .field--full, .alert-error, .actions { grid-column: 1 / -1; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; min-width: 0; }
  .label { font-size: 0.875rem; font-weight: 500; color: var(--acc-color-text-main); }
  .hint { font-size: 0.75rem; color: var(--acc-color-text-muted); }
  .field-error { font-size: 0.75rem; color: var(--acc-color-danger); }
  .input {
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--acc-color-border);
    border-radius: var(--acc-radius-md);
    font-size: 0.875rem;
    font-family: inherit;
    outline: none;
    background-color: var(--acc-color-surface);
  }
  .input:focus {
    border-color: var(--acc-color-primary);
    box-shadow: 0 0 0 3px rgb(30 64 175 / 0.14);
  }
  .input--error { border-color: var(--acc-color-danger); }
  .alert-error {
    padding: 0.75rem 1rem;
    border-radius: var(--acc-radius-md);
    background-color: var(--acc-color-danger-bg);
    color: var(--acc-color-danger);
    font-size: 0.875rem;
  }
  .actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
`;
