import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { App } from './app/app';

bootstrapApplication(App, appConfig).catch(() => {
  // Nunca logar o objeto de erro (pode conter stack/URLs internas).
  console.error('Falha ao iniciar a aplicação.');
});
