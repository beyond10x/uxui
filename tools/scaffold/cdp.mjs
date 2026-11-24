// Minimal Chrome DevTools Protocol client: starts Chromium, talks CDP over Node's global WebSocket.
// No third-party dependencies. Chromium is found under the Playwright cache or named by UXUI_CHROME.

import { spawn } from 'node:child_process';
import { mkdirSync, readdirSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { homedir } from 'node:os';
import { fileURLToPath } from 'node:url';

export const HERE = dirname(fileURLToPath(import.meta.url));

export function scratchDir() {
  const dir = process.env.UXUI_SCRATCH ?? join(process.cwd(), 'target', 'scaffold');
  mkdirSync(dir, { recursive: true });
  return dir;
}

export function chromePath() {
  if (process.env.UXUI_CHROME) return process.env.UXUI_CHROME;
  const base = join(homedir(), '.cache', 'ms-playwright');
  const dirs = existsSync(base) ? readdirSync(base).filter((d) => /^chromium-\d+$/.test(d)).sort().reverse() : [];
  for (const d of dirs) {
    for (const sub of ['chrome-linux64', 'chrome-linux']) {
      const p = join(base, d, sub, 'chrome');
      if (existsSync(p)) return p;
    }
  }
  throw new Error('no Chromium found; set UXUI_CHROME');
}

export function runtimeSource() {
  return readFileSync(join(HERE, 'runtime.js'), 'utf8');
}

class Connection {
  constructor(ws) {
    this.ws = ws;
    this.seq = 0;
    this.pending = new Map();
    this.listeners = new Set();
    ws.onmessage = (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id) {
        const p = this.pending.get(msg.id);
        if (!p) return;
        this.pending.delete(msg.id);
        if (msg.error) p.reject(new Error(`${p.method}: ${msg.error.message}`));
        else p.resolve(msg.result);
      } else {
        for (const l of this.listeners) l(msg);
      }
    };
    ws.onclose = () => {
      for (const p of this.pending.values()) p.reject(new Error('CDP connection closed'));
      this.pending.clear();
    };
  }

  static open(url) {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(url);
      ws.onopen = () => resolve(new Connection(ws));
      ws.onerror = () => reject(new Error(`cannot open ${url}`));
    });
  }

  send(method, params = {}, sessionId) {
    const id = ++this.seq;
    const msg = { id, method, params };
    if (sessionId) msg.sessionId = sessionId;
    this.ws.send(JSON.stringify(msg));
    return new Promise((resolve, reject) => this.pending.set(id, { resolve, reject, method }));
  }
}

export class Page {
  constructor(conn, sessionId, targetId) {
    this.conn = conn;
    this.sessionId = sessionId;
    this.targetId = targetId;
  }

  send(method, params) {
    return this.conn.send(method, params, this.sessionId);
  }

  waitFor(method, pred = () => true, timeoutMs = 20000) {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.conn.listeners.delete(l);
        reject(new Error(`timeout waiting for ${method}`));
      }, timeoutMs);
      const l = (msg) => {
        if (msg.sessionId !== this.sessionId || msg.method !== method || !pred(msg.params)) return;
        clearTimeout(timer);
        this.conn.listeners.delete(l);
        resolve(msg.params);
      };
      this.conn.listeners.add(l);
    });
  }

  async eval(expression) {
    const r = await this.send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.exceptionDetails) {
      throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    }
    return r.result.value;
  }

  // Calls window.__uxui.<fn>(...args) and returns its JSON value.
  call(fn, ...args) {
    return this.eval(`window.__uxui.${fn}(...${JSON.stringify(args)})`);
  }

  async goto(url, { timeoutMs = 20000, idleMs = 2500 } = {}) {
    const load = this.waitFor('Page.loadEventFired', () => true, timeoutMs);
    const nav = await this.send('Page.navigate', { url });
    if (nav.errorText) throw new Error(`navigate ${url}: ${nav.errorText}`);
    await load;
    await this.waitFor('Page.lifecycleEvent', (p) => p.name === 'networkIdle' && p.loaderId === nav.loaderId, idleMs).catch(() => {});
    await this.eval('document.fonts ? document.fonts.ready.then(() => true) : true');
  }

  async viewport(width, height) {
    await this.send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: width < 600 });
    await this.eval('new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => r(true))))');
  }

  async screenshot(path, { fullPage = false } = {}) {
    const params = { format: 'png' };
    if (fullPage) params.captureBeyondViewport = true;
    const { data } = await this.send('Page.captureScreenshot', params);
    writeFileSync(path, Buffer.from(data, 'base64'));
  }
}

// Starts Chromium and returns { pid, conn, newPage(), close() }.
export async function launch({ headless = true, profile = join(scratchDir(), `profile-${process.pid}`) } = {}) {
  mkdirSync(profile, { recursive: true });
  const args = [
    headless ? '--headless=new' : null,
    '--remote-debugging-port=0',
    `--user-data-dir=${profile}`,
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-extensions',
    '--disable-background-networking',
    '--disable-sync',
    '--hide-scrollbars',
    '--mute-audio',
    '--force-color-profile=srgb',
    'about:blank',
  ].filter(Boolean);
  const proc = spawn(chromePath(), args, { stdio: ['ignore', 'ignore', 'pipe'] });
  const wsUrl = await new Promise((resolve, reject) => {
    let buf = '';
    const timer = setTimeout(() => reject(new Error(`Chromium did not start:\n${buf}`)), 20000);
    proc.stderr.on('data', (d) => {
      if (buf === null) return;
      buf += d;
      const m = buf.match(/DevTools listening on (ws:\/\/\S+)/);
      if (m) {
        clearTimeout(timer);
        buf = null;
        resolve(m[1]);
      }
    });
    proc.on('exit', (code) => reject(new Error(`Chromium exited ${code}:\n${buf}`)));
  });
  const conn = await Connection.open(wsUrl);
  return {
    pid: proc.pid,
    conn,
    async newPage({ runtime = true } = {}) {
      const { targetId } = await conn.send('Target.createTarget', { url: 'about:blank' });
      const { sessionId } = await conn.send('Target.attachToTarget', { targetId, flatten: true });
      const page = new Page(conn, sessionId, targetId);
      await page.send('Page.enable');
      await page.send('Page.setLifecycleEventsEnabled', { enabled: true });
      if (runtime) await page.send('Page.addScriptToEvaluateOnNewDocument', { source: runtimeSource() });
      return page;
    },
    async close() {
      try {
        await conn.send('Browser.close');
      } catch {}
      await new Promise((r) => setTimeout(r, 200));
      try {
        process.kill(proc.pid, 'SIGKILL');
      } catch {}
    },
  };
}

export function parseArgs(argv) {
  const pos = [];
  const opt = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const [k, v] = a.slice(2).split('=', 2);
      if (v !== undefined) opt[k] = v;
      else if (i + 1 < argv.length && !argv[i + 1].startsWith('--')) opt[k] = argv[++i];
      else opt[k] = true;
    } else pos.push(a);
  }
  return { pos, opt };
}
