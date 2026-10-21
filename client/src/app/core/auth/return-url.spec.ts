import { urlRetornoValida } from './return-url';

describe('urlRetornoValida', () => {
  it('aceita caminhos internos', () => {
    expect(urlRetornoValida('/inicio')).toBe('/inicio');
    expect(urlRetornoValida('/consultas?pagina=2')).toBe('/consultas?pagina=2');
  });

  it('rejeita destinos externos e esquemas', () => {
    expect(urlRetornoValida(null)).toBeNull();
    expect(urlRetornoValida('')).toBeNull();
    expect(urlRetornoValida('inicio')).toBeNull();
    expect(urlRetornoValida('//evil.test')).toBeNull();
    expect(urlRetornoValida('/\\evil.test')).toBeNull();
    expect(urlRetornoValida('https://evil.test')).toBeNull();
    expect(urlRetornoValida('javascript:alert(1)')).toBeNull();
  });

  it('rejeita tentativas codificadas', () => {
    expect(urlRetornoValida('/%2Fevil.test')).toBeNull();
    expect(urlRetornoValida('/%5Cevil.test')).toBeNull();
    expect(urlRetornoValida('/%3A%2F%2Fevil.test')).toBeNull();
  });
});
