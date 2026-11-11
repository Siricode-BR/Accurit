// Configuração do Karma para o @accurit/ui em CI/Windows.
// O ChromeHeadless padrão não sobe em alguns ambientes por falta de sandbox;
// usamos um launcher explícito com as flags necessárias.
module.exports = function (config) {
  config.set({
    frameworks: ['jasmine'],
    plugins: [
      require('karma-jasmine'),
      require('karma-chrome-launcher'),
      require('karma-jasmine-html-reporter'),
    ],
    browsers: ['ChromeHeadlessCI'],
    customLaunchers: {
      ChromeHeadlessCI: {
        base: 'ChromeHeadless',
        flags: ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'],
      },
    },
    singleRun: true,
  });
};
