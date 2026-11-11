import http from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';

const port = Number(process.env.PORT ?? 4201);
const root = path.resolve(process.env.DIST ?? 'apps/admin/dist/admin/browser');
const backend = process.env.BACKEND ?? 'http://127.0.0.1:8080';

const tipos = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.ico': 'image/x-icon',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
};

async function arquivo(caminho) {
  try {
    const info = await stat(caminho);
    if (info.isFile()) return info;
  } catch {
    // inexistente: cai no fallback da SPA
  }
  return null;
}

const servidor = http.createServer(async (req, res) => {
  const url = new URL(req.url ?? '/', `http://127.0.0.1:${port}`);

  if (url.pathname.startsWith('/api/')) {
    try {
      const alvo = new URL(url.pathname + url.search, backend);
      const corpo = ['GET', 'HEAD'].includes(req.method ?? 'GET')
        ? undefined
        : await new Promise((resolve) => {
            const partes = [];
            req.on('data', (parte) => partes.push(parte));
            req.on('end', () => resolve(Buffer.concat(partes)));
          });
      const resposta = await fetch(alvo, {
        method: req.method,
        headers: Object.fromEntries(
          Object.entries(req.headers).filter(
            ([nome, valor]) =>
              valor !== undefined &&
              ![
                'host',
                'connection',
                'content-length',
                'accept-encoding',
                'transfer-encoding',
                'upgrade',
              ].includes(nome.toLowerCase()),
          ),
        ),
        body: corpo,
        redirect: 'manual',
      });
      res.statusCode = resposta.status;
      for (const [nome, valor] of resposta.headers) {
        if (nome.toLowerCase() === 'content-encoding') continue;
        res.setHeader(nome, valor);
      }
      res.end(Buffer.from(await resposta.arrayBuffer()));
    } catch (erro) {
      res.statusCode = 502;
      res.setHeader('content-type', 'application/json');
      res.end(JSON.stringify({ error: { code: 'BACKEND_INDISPONIVEL', message: String(erro) } }));
    }
    return;
  }

  let destino = path.join(root, decodeURIComponent(url.pathname));
  if (!destino.startsWith(root)) {
    res.statusCode = 403;
    res.end('forbidden');
    return;
  }
  if (!(await arquivo(destino))) {
    destino = path.join(root, 'index.html');
  }
  try {
    const conteudo = await readFile(destino);
    res.statusCode = 200;
    res.setHeader('content-type', tipos[path.extname(destino)] ?? 'application/octet-stream');
    res.end(conteudo);
  } catch {
    res.statusCode = 404;
    res.end('not found');
  }
});

servidor.listen(port, '127.0.0.1', () => {
  console.log(`static ${root} em http://127.0.0.1:${port} proxy ${backend}`);
});
