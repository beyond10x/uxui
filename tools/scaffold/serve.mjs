// serve.mjs <url> [--port 5301] [--snapshot] [--variants list.json]
// Serves any page with the scaffold runtime and its control panel injected, at
// http://127.0.0.1:<port>/. Default: a reverse proxy to the page's origin that injects the scripts
// into every HTML answer, so the page keeps its own scripts. --snapshot: the page's DOM after load
// in headless Chromium, scripts removed, with a <base href> (for pages whose proxy fails).
// /__uxui/ explains the panel and offers a bookmarklet that injects it into any open tab.
// GET and HEAD only: nothing is posted to the origin.

import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { HERE, launch, parseArgs } from './cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
let target = pos[0] && new URL(pos[0]);
if (!target) {
  console.error('usage: serve.mjs <url> [--port 5301] [--snapshot] [--variants list.json]');
  process.exit(2);
}
const port = +(opt.port || 5301);
// follow redirects once (www.example.org -> example.org) so the proxy serves the final origin
if (!opt.snapshot) target = new URL((await fetch(target.href, { method: "HEAD", redirect: "follow" })).url);
const origin = target.origin;
const variants = opt.variants ? readFileSync(opt.variants, 'utf8') : null;
const bundle = () => readFileSync(join(HERE, 'runtime.js'), 'utf8') + '\n' + readFileSync(join(HERE, 'panel.js'), 'utf8');
const self = (req) => `http://${req.headers.host || '127.0.0.1:' + port}`;

function inject(html, req, absolute) {
  const base = absolute ? self(req) : '';
  const tag =
    `<script>window.__uxuiAutoPanel=true;${variants ? `window.__uxuiVariants=${variants.trim()};` : ''}</script>` +
    `<script src="${base}/__uxui/bundle.js"></script>`;
  return /<\/head>/i.test(html) ? html.replace(/<\/head>/i, tag + '</head>') : tag + html;
}

let snapshot = null;
if (opt.snapshot) {
  const b = await launch();
  try {
    const p = await b.newPage({ runtime: false });
    await p.viewport(1440, 900);
    await p.goto(target.href);
    snapshot = await p.eval(`(() => {
      document.querySelectorAll('script').forEach((s) => s.remove());
      const base = document.createElement('base'); base.href = location.href;
      document.head.prepend(base);
      return '<!doctype html>' + document.documentElement.outerHTML;
    })()`);
  } finally {
    await b.close();
  }
}

function helpPage(req) {
  const loader = `javascript:(()=>{window.__uxuiAutoPanel=1;const s=document.createElement('script');s.src='${self(req)}/__uxui/bundle.js';document.documentElement.appendChild(s)})()`;
  const inline = 'javascript:' + encodeURIComponent(bundle() + ';window.__uxui.panel()');
  return `<!doctype html><meta charset="utf-8"><title>uxui scaffold</title>
<style>body{font:16px/1.5 system-ui,sans-serif;max-width:46em;margin:2em auto;padding:0 1em}a.bm{display:inline-block;padding:.3em .8em;border:1px solid #555;border-radius:6px;margin:.2em 0}</style>
<h1>uxui scaffold</h1>
<p>The page <a href="/">${target.href}</a> is served here with the scaffold runtime and its panel injected.
Move a control in the panel (bottom right): the page changes at once and the hard rules and metrics below the controls are measured again.</p>
<p>Any other tab: drag one of these to the bookmarks bar, open a page, click it.</p>
<p><a class="bm" href="${loader.replace(/"/g, '&quot;')}">uxui scaffold (loads from this server)</a><br>
<a class="bm" href="${inline.replace(/"/g, '&quot;')}">uxui scaffold (self-contained, ${Math.round(inline.length / 1024)} KB)</a></p>
<p>A page with a strict Content-Security-Policy refuses the first; the second runs where the browser lets bookmarklets run.</p>`;
}

const DROP = new Set(['content-encoding', 'content-length', 'transfer-encoding', 'connection', 'keep-alive', 'content-security-policy', 'content-security-policy-report-only', 'strict-transport-security', 'set-cookie', 'alt-svc']);

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url, 'http://x');
    if (url.pathname === '/__uxui/bundle.js') {
      res.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8', 'cache-control': 'no-store', 'access-control-allow-origin': '*' });
      return res.end(bundle());
    }
    if (url.pathname === '/__uxui/' || url.pathname === '/__uxui') {
      res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      return res.end(helpPage(req));
    }
    if (snapshot) {
      if (url.pathname !== '/') {
        res.writeHead(302, { location: new URL(url.pathname + url.search, target).href });
        return res.end();
      }
      res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      return res.end(inject(snapshot, req, true));
    }
    if (req.method !== 'GET' && req.method !== 'HEAD') {
      res.writeHead(405);
      return res.end('read-only proxy');
    }
    if (url.pathname === '/' && target.pathname !== '/') {
      res.writeHead(302, { location: target.pathname + target.search });
      return res.end();
    }
    const up = await fetch(origin + url.pathname + url.search, {
      redirect: 'manual',
      headers: { accept: req.headers.accept || '*/*', 'accept-language': req.headers['accept-language'] || 'en', 'user-agent': req.headers['user-agent'] || 'uxui-scaffold' },
    });
    const headers = {};
    up.headers.forEach((v, k) => {
      if (!DROP.has(k)) headers[k] = v;
    });
    if (headers.location?.startsWith(origin)) headers.location = headers.location.slice(origin.length) || '/';
    const type = up.headers.get('content-type') || '';
    if (/text\/html/.test(type)) {
      const html = (await up.text()).split(origin + '/').join('/');
      res.writeHead(up.status, headers);
      return res.end(inject(html, req, false));
    }
    if (/text\/css/.test(type)) {
      res.writeHead(up.status, headers);
      return res.end((await up.text()).split(origin + '/').join('/'));
    }
    res.writeHead(up.status, headers);
    res.end(Buffer.from(await up.arrayBuffer()));
  } catch (e) {
    res.writeHead(502, { 'content-type': 'text/plain' });
    res.end(String(e));
  }
});
server.listen(port, '127.0.0.1', () => console.log(`uxui scaffold: http://127.0.0.1:${port}/  (help: /__uxui/)  -> ${target.href}${snapshot ? ' (snapshot)' : ''}`));
