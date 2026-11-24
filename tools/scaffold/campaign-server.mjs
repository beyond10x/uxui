// campaign-server.mjs --port 5200 --dir <campaign dir> --studio <studio www dir> --site <built site dir> [--run]
//
// One Node server, no packages, for a campaign over the documentation landing page:
//   /api/views/<view>?params=...   the studio's reads, answered from the campaign's state
//   /api/commands/<command>        StartCampaign (runs the campaign again), StopCampaign
//   /api/channels/climb            server-sent events: CampaignProgressed, FrontierChanged,
//                                  CampaignStopped, CampaignEnded, as the sweep runs
//   /api/state/...                 the studio's server-held state, in memory
//   /uxui/...                      the built site with its scripts removed; ?variant=<digest>
//                                  renders that variant from its patched sources
//   /compare, /shots/<file>        the side-by-side page and its screenshots
//   anything else                  the studio application (www/, every page path -> index.html)
// The campaign itself is campaign.mjs, run as a child process; its NDJSON lines are the state.

import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync, statSync, createReadStream, mkdirSync } from 'node:fs';
import { join, extname, normalize, resolve } from 'node:path';
import { HERE, parseArgs } from './cdp.mjs';

const { opt } = parseArgs(process.argv.slice(2));
const PORT = +(opt.port || 5200);
const DIR = resolve(opt.dir || 'target/campaign');
const STUDIO = resolve(opt.studio || 'www');
const SITE = resolve(opt.site || 'website/build');
const LIFT = join(HERE, 'lift', 'docs-landing');
mkdirSync(DIR, { recursive: true });
const STATE_FILE = join(DIR, 'state.json');

const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.map': 'application/json', '.ico': 'image/x-icon', '.txt': 'text/plain', '.xml': 'application/xml' };

// ---- state ----------------------------------------------------------------------------------
let S = fresh();
function fresh() {
  return { campaign: null, target: null, space: null, axisValues: [], variants: {}, points: [], frontier: [], promotions: [], judgements: [], state: {} };
}
if (existsSync(STATE_FILE)) {
  try {
    S = { ...fresh(), ...JSON.parse(readFileSync(STATE_FILE, 'utf8')) };
  } catch {}
}
let saveTimer = null;
function save() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => writeFileSync(STATE_FILE, JSON.stringify(S)), 300);
}

const clients = new Set();
let eventId = 0;
function push(event, payload) {
  const line = `id: ${++eventId}\ndata: ${JSON.stringify({ event, payload })}\n\n`;
  for (const res of clients) res.write(line);
}

function apply(msg) {
  const cid = S.campaign?.campaign_id;
  switch (msg.t) {
    case 'campaign':
      S = { ...fresh(), campaign: msg.campaign, target: msg.target, space: msg.space, axisValues: msg.axisValues, state: S.state };
      break;
    case 'variant':
      S.variants[msg.variant.variant_digest] = { ...(S.variants[msg.variant.variant_digest] || {}), ...msg.variant };
      break;
    case 'point':
      S.points.push(msg.point);
      push('uxsearch.search.CampaignProgressed', { ...msg.point, campaign_id: cid });
      break;
    case 'frontier': {
      const before = new Set(S.frontier.map((f) => f.variant_digest));
      S.frontier = msg.entries;
      for (const f of msg.entries) if (!before.has(f.variant_digest)) push('uxsearch.search.FrontierChanged', { frontier_entry_id: f.frontier_entry_id, campaign_id: cid, variant_digest: f.variant_digest, admitted: true });
      if (!msg.entries.length || before.size !== msg.entries.length) push('uxsearch.search.FrontierChanged', { frontier_entry_id: 'all', campaign_id: cid, admitted: true });
      break;
    }
    case 'promotion':
      S.promotions = [msg.promotion];
      break;
    case 'campaign_state':
      Object.assign(S.campaign, msg.patch);
      if (msg.patch.state === 'Ended') push('uxsearch.search.CampaignEnded', { campaign_id: cid, ended_because: msg.patch.ended_because });
      if (msg.patch.state === 'Stopped') push('uxsearch.search.CampaignStopped', { campaign_id: cid });
      break;
    case 'log':
      console.error('[campaign]', msg.text);
      break;
  }
  save();
}

// ---- the campaign child ---------------------------------------------------------------------
let child = null;
function run() {
  if (child) return false;
  const args = [join(HERE, 'campaign.mjs'), '--dir', DIR, '--base', `http://127.0.0.1:${PORT}`];
  for (const k of ['cells', 'top', 'shots', 'repo', 'product']) if (opt[k]) args.push('--' + k, opt[k]);
  child = spawn(process.execPath, args, { stdio: ['ignore', 'pipe', 'inherit'] });
  writeFileSync(join(DIR, 'campaign.pid'), String(child.pid));
  let buf = '';
  child.stdout.on('data', (d) => {
    buf += d;
    let i;
    while ((i = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, i);
      buf = buf.slice(i + 1);
      if (!line.trim()) continue;
      try {
        apply(JSON.parse(line));
      } catch (e) {
        console.error('bad line', line.slice(0, 200), e.message);
      }
    }
  });
  child.on('exit', (code) => {
    if (S.campaign && S.campaign.state === 'Running') apply({ t: 'campaign_state', patch: { state: 'Stopped', ended_because: code ? `exit ${code}` : 'stopped' } });
    child = null;
  });
  return true;
}

// ---- views ----------------------------------------------------------------------------------
const ids = () => ({ campaign_id: S.campaign?.campaign_id, target_id: S.target?.target_id });
function view(name, params) {
  const variants = Object.values(S.variants);
  const vrow = (v) => ({
    variant_digest: v.variant_digest,
    base_document_id: S.target?.base_document_id,
    target_id: S.target?.target_id,
    state: v.state,
    routes: [{ design_space_id: S.campaign?.design_space_id, origin: v.baseline ? 'manual' : 'search', choices: Object.entries(v.choice || {}).map(([axis, value]) => `${axis}=${value}`), applications: (v.patch || []).map((p) => `${p.op} ${p.path}`) }],
  });
  switch (name) {
    case 'uxsearch.search.Campaigns':
      return S.campaign ? [S.campaign] : [];
    case 'uxsearch.search.CampaignById':
      return S.campaign && (!params.campaign_id || params.campaign_id === S.campaign.campaign_id) ? [S.campaign] : [];
    case 'uxsearch.search.Climb':
      return S.campaign && params.campaign_id && params.campaign_id !== S.campaign.campaign_id ? [] : S.points;
    case 'uxsearch.search.Frontier':
      return S.campaign && params.campaign_id && params.campaign_id !== S.campaign.campaign_id ? [] : S.frontier;
    case 'uxsearch.design.Targets':
    case 'uxsearch.design.TargetCatalog':
      return S.target ? [S.target] : [];
    case 'uxsearch.design.BaseDocuments':
      return S.target ? [{ base_document_id: S.target.base_document_id, path: 'website/product.json', app: 'documentation site', document_digest: S.campaign?.base_digest, state: 'Registered' }] : [];
    case 'uxsearch.design.DesignSpaces':
      return S.space ? [S.space] : [];
    case 'uxsearch.design.AxisValues':
      return S.axisValues;
    case 'uxsearch.design.Variants':
      return variants.map(vrow);
    case 'uxsearch.design.VariantById':
      return variants.filter((v) => v.variant_digest === params.variant_digest).map(vrow);
    case 'uxsearch.design.AxisSwitches':
      return [];
    case 'uxsearch.evaluation.EvaluationsOfVariant':
      return S.variants[params.variant_digest]?.evaluations || [];
    case 'uxsearch.rules.VerdictsOfVariant':
      return S.variants[params.variant_digest]?.verdicts || [];
    case 'uxsearch.promotion.Promotions':
      return S.promotions;
    case 'uxsearch.promotion.PromotionById':
      return S.promotions.filter((p) => p.promotion_id === params.promotion_id);
    case 'uxsearch.promotion.Judgements':
      return S.judgements;
    case 'uxsearch.ratchet.GateDecisions':
      return [];
    default:
      return null;
  }
}

// ---- the site, with a variant ---------------------------------------------------------------
function siteFile(path) {
  let rel = decodeURIComponent(path.replace(/^\/uxui\/?/, ''));
  let file = normalize(join(SITE, rel));
  if (!file.startsWith(SITE)) return null;
  if (existsSync(file) && statSync(file).isDirectory()) file = join(file, 'index.html');
  else if (!existsSync(file) && existsSync(file + '.html')) file += '.html';
  return existsSync(file) ? file : null;
}
function siteHtml(file, query) {
  let html = readFileSync(file, 'utf8');
  html = html.replace(/<script\b[^>]*\bsrc=[^>]*>\s*<\/script>/g, '');
  const variant = query.get('variant');
  const mode = query.get('mode');
  let inject = '';
  if (mode === 'dark' || mode === 'light') inject += `<script>document.documentElement.setAttribute('data-theme',${JSON.stringify(mode)})</script>`;
  const v = variant && (S.variants[variant] || null);
  if (v && v.spec) inject += `<script>window.__variantSpec=${JSON.stringify({ ...v.spec, cssText: readFileSync(join(LIFT, 'landing.css'), 'utf8') }).replace(/</g, '\\u003c')}</script><script src="/__campaign/landing-page.js"></script>`;
  return html.replace('</body>', inject + '</body>');
}

function sendFile(res, file, type) {
  res.writeHead(200, { 'content-type': type || MIME[extname(file)] || 'application/octet-stream', 'cache-control': 'no-store' });
  createReadStream(file).pipe(res);
}
function json(res, code, body) {
  res.writeHead(code, { 'content-type': 'application/json', 'cache-control': 'no-store' });
  res.end(JSON.stringify(body));
}

const server = createServer((req, res) => {
  const url = new URL(req.url, `http://127.0.0.1:${PORT}`);
  const path = url.pathname;
  try {
    if (path.startsWith('/api/views/')) {
      let params = {};
      try {
        params = JSON.parse(url.searchParams.get('params') || '{}') || {};
      } catch {}
      const rows = view(path.slice('/api/views/'.length), params);
      if (rows === null) return json(res, 404, { error: 'unknown view' });
      return json(res, 200, { rows, total: rows.length });
    }
    if (path.startsWith('/api/commands/') && req.method === 'POST') {
      const cmd = path.slice('/api/commands/'.length);
      if (cmd === 'uxsearch.search.StartCampaign') {
        const ok = run();
        return json(res, ok ? 200 : 409, ok ? { accepted: true } : { outcome: 'refused', error: 'already_running' });
      }
      if (cmd === 'uxsearch.search.StopCampaign') {
        if (child) child.kill('SIGTERM');
        return json(res, 200, { accepted: true });
      }
      return json(res, 200, { accepted: false, unapplied: 'this server answers StartCampaign and StopCampaign only' });
    }
    if (path.startsWith('/api/state/')) {
      const key = path.slice('/api/state/'.length);
      if (req.method === 'PUT') {
        let body = '';
        req.on('data', (d) => (body += d));
        req.on('end', () => {
          try {
            S.state[key] = JSON.parse(body);
          } catch {}
          json(res, 200, {});
        });
        return;
      }
      return key in S.state ? json(res, 200, S.state[key]) : json(res, 404, {});
    }
    if (path.startsWith('/api/channels/')) {
      res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-store', connection: 'keep-alive' });
      res.write(': live\n\n');
      if (path.endsWith('/climb')) {
        clients.add(res);
        req.on('close', () => clients.delete(res));
      }
      const ping = setInterval(() => res.write(': ping\n\n'), 15000);
      req.on('close', () => clearInterval(ping));
      return;
    }
    if (path === '/api/run' && req.method === 'POST') return json(res, 200, { started: run() });
    if (path === '/compare' || path === '/compare/') {
      const f = join(DIR, 'compare.html');
      return existsSync(f) ? sendFile(res, f, MIME['.html']) : json(res, 404, { error: 'no compare page yet' });
    }
    if (path.startsWith('/shots/')) {
      const f = normalize(join(DIR, 'shots', decodeURIComponent(path.slice(7))));
      if (f.startsWith(join(DIR, 'shots')) && existsSync(f)) return sendFile(res, f);
      return json(res, 404, {});
    }
    if (path === '/__campaign/landing-page.js') return sendFile(res, join(LIFT, 'landing-page.js'));
    if (path === '/__campaign/landing.css') return sendFile(res, join(LIFT, 'landing.css'));
    if (path === '/__campaign/state.json') return json(res, 200, S);
    if (path === '/uxui' || path.startsWith('/uxui/')) {
      const f = siteFile(path);
      if (!f) return json(res, 404, { error: 'not in the built site' });
      if (f.endsWith('.html')) {
        res.writeHead(200, { 'content-type': MIME['.html'], 'cache-control': 'no-store' });
        return res.end(siteHtml(f, url.searchParams));
      }
      return sendFile(res, f);
    }
    // the studio
    let f = normalize(join(STUDIO, decodeURIComponent(path)));
    if (!f.startsWith(STUDIO) || !existsSync(f) || statSync(f).isDirectory()) f = join(STUDIO, 'index.html');
    return sendFile(res, f);
  } catch (e) {
    json(res, 500, { error: String(e.message || e) });
  }
});

server.listen(PORT, '127.0.0.1', () => {
  console.error(`campaign server on http://127.0.0.1:${PORT}/ (dir ${DIR})`);
  if (opt.run) run();
});
process.on('SIGTERM', () => {
  if (child) child.kill('SIGTERM');
  process.exit(0);
});
