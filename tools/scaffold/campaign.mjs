// campaign.mjs --dir <campaign dir> --base <campaign server URL> [--cells <uxui-cells>] [--top 30]
//              [--shots 5] [--repo <repository root>] [--product <product.json of the current page>]
//
// One improvement campaign over the documentation landing page, through the scaffold funnel:
//   stage 0  lift the current sources (lift/docs-landing/lift.mjs), `ess ui check` the document,
//            and check that every selector of binding.json matches one element of the built page
//   stage 1  every variant of the SOURCE space (space.json: product.json fields and landing.css
//            custom properties), lifted to ess-ui/1, checked and drawn in process by uxui-cells
//            on the terminal renderer at a wide and a narrow size; hard rules prune, a KPI ranks
//   stage 2  the top --top of stage 1 (and the current page) rendered in Chromium from their
//            patched sources over the built site (lift/docs-landing/landing-page.js), measured at
//            1440x900, 1024x768 and 390x844 in light and dark; Kendall's tau between the stages
//   stage 3  screenshots of the top --shots and the current page, a side-by-side compare.html
// Every step is printed as one NDJSON line for campaign-server.mjs: the studio's climb.
// Scripts that measure run without `slot` (repository rule); Chromium is killed by PID at exit.

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { HERE, launch, parseArgs } from './cdp.mjs';
import { materialize, liftDocument, variants, digest, baseChoice, pageOf, contentOf } from './lift/docs-landing/lift.mjs';

const { opt } = parseArgs(process.argv.slice(2));
const DIR = resolve(opt.dir || 'target/campaign');
const BASE = opt.base || 'http://127.0.0.1:5200';
const REPO = resolve(opt.repo || join(HERE, '..', '..'));
const TOP = +(opt.top || 30);
const SHOTS = +(opt.shots || 5);
const LIFT = join(HERE, 'lift', 'docs-landing');
const S1 = join(DIR, 's1');
mkdirSync(join(S1, 'state'), { recursive: true });
mkdirSync(join(DIR, 'shots'), { recursive: true });

const emit = (o) => process.stdout.write(JSON.stringify(o) + '\n');
const log = (text) => emit({ t: 'log', text });
const now = () => new Date().toISOString();
const ms = (t) => Math.round(performance.now() - t);

const space = JSON.parse(readFileSync(join(LIFT, 'space.json'), 'utf8'));
const product = JSON.parse(readFileSync(opt.product || join(REPO, 'website', 'product.json'), 'utf8'));
const status = JSON.parse(readFileSync(join(REPO, 'website', 'data', 'status.json'), 'utf8'));
const cssText = readFileSync(join(LIFT, 'landing.css'), 'utf8');
const binding = JSON.parse(readFileSync(join(LIFT, 'binding.json'), 'utf8'));
if (opt.cells && !existsSync(opt.cells)) {
  // A missing cell reader would silently turn stage 1 into the static reading; refuse instead.
  console.error(`no cell reader at ${opt.cells}`);
  process.exit(2);
}

const CAMPAIGN_ID = 'cp-docs-landing-' + new Date().toISOString().slice(0, 10);
// Each run's climb points carry their own ids, so a page left open shows a new run's steps as new rows.
const RUN = Date.now().toString(36);
const times = {};
let browser = null;
async function shutdown(code) {
  if (browser) await browser.close();
  browser = null;
  process.exit(code);
}
process.on('SIGTERM', () => shutdown(143));
process.on('SIGINT', () => shutdown(130));

const CONTENT = 'content.items_kept';
const STAGE1_SPACE = 'stage1-cells';
const STAGE2_SPACE = 'stage2-chromium';
const CONTRAST_MIN = 'a11y.contrast_min';
const KPI_KEYS = ['load.words_before_action', 'attention.primary_top_px', 'effort.actions_per_journey', 'attention.regions_crossed', 'a11y.tap_targets_under_44px', CONTENT];
// Higher is better for the content KPI, lower for the others.
const HIGHER = new Set([CONTENT]);
const vecOf = (k) => KPI_KEYS.filter((x) => k && x in k).map((x) => (HIGHER.has(x) ? -k[x] : k[x]));
const SCORE_NOTE = 'mean over the six KPIs of value / value of the current page (content: current / value); lower is better; +1000 per failed hard rule';
// Content hard rule: every section and card of the current page stays present (lift.mjs contentOf).
const missingOf = (ids, base) => base.filter((x) => !ids.includes(x));

// ---- stage 0 --------------------------------------------------------------------------------
const all = variants(space);
const base0 = baseChoice(space);
const records = all.map((choice, seq) => {
  const v = materialize(product, space, choice);
  return { seq, choice, ...v, digest: digest(v), baseline: Object.entries(choice).every(([a, x]) => x === base0[a]) };
});
const baseline = records.find((r) => r.baseline);
const baseContent = contentOf(product, status);
for (const r of records) {
  r.content = contentOf(r.product, status);
  r.contentMissing = missingOf(r.content, baseContent);
}
const t0 = performance.now();
const baseDoc = liftDocument(product, status);
writeFileSync(join(S1, 'base.ui.json'), JSON.stringify(baseDoc));
const chk = spawnSync('ess', ['ui', 'check', '--path', join(S1, 'base.ui.json'), '--format', 'json'], { encoding: 'utf8' });
let baseFindings = [];
try {
  baseFindings = JSON.parse(chk.stdout).findings.filter((f) => f.severity === 'error');
} catch {
  baseFindings = [{ message: chk.stderr.slice(0, 300) }];
}
times.stage0_lift_ms = ms(t0);

const axisValues = Object.entries(space.axes).flatMap(([axis, a]) =>
  Object.keys(a.values).map((name) => ({ axis_value_id: `av-${axis}-${name}`, axis_id: `ax-${axis}`, name, state: 'Declared' })),
);
emit({
  t: 'campaign',
  campaign: {
    campaign_id: CAMPAIGN_ID,
    target_id: space.target.id,
    design_space_id: 'ds-docs-landing',
    journeys: space.journeys.map((j) => j.name),
    environments: ['cells-120x40', 'cells-48x50', ...['1440x900', '1024x768', '390x844'].flatMap((v) => [`${v}-light`, `${v}-dark`])],
    kpis: KPI_KEYS,
    strategy: 'funnel',
    beam_width: TOP,
    seed: 0,
    budget: { variants: all.length, stage2: TOP, screenshots: SHOTS },
    ended_because: null,
    state: 'Running',
    base_digest: baseline.digest,
  },
  target: { target_id: space.target.id, base_document_id: 'doc-docs-landing', name: space.target.name, scope: 'page', node_path: space.target.page, host_slot: false, state: 'Declared' },
  space: { design_space_id: 'ds-docs-landing', target_id: space.target.id, name: `${Object.keys(space.axes).length} source axes, ${all.length} variants`, component_treatment: 'fixed', space_digest: 'sha256:' + digest({ product: space, css: {} }).slice(7), state: 'Frozen' },
  axisValues,
});
log(`stage 0: ${all.length} variants, lift + check ${times.stage0_lift_ms} ms, ${baseFindings.length} errors`);

// ---- stage 1 --------------------------------------------------------------------------------
const words = (s) => (s.match(/[\p{L}\p{N}][\p{L}\p{N}'’.-]*/gu) || []).length;
const SIZES = [
  { name: 'cells-120x40', w: 120, view: 40, wide: true },
  { name: 'cells-48x50', w: 48, view: 50, wide: false },
];
const DRAW_H = 600;

// Links of a lifted document to a page: [{label}] from hero actions and link primitives.
function linksTo(doc, page) {
  const out = [];
  for (const s of doc.pages.landing.sections) {
    for (const a of s.actions || []) if (a.navigate?.to === page) out.push({ label: a.label, path: `pages/landing/sections/${s.name}/actions/${a.name}`, section: s.name });
    for (const c of s.children || []) if (c.primitive === 'link' && c.to?.to === page) out.push({ label: c.text, path: `pages/landing/sections/${s.name}/children/${c.name}`, section: s.name });
  }
  return out;
}
const HERO = new Set(['kicker', 'lede', 'actions', 'meta', 'kpis']);
// Fixed routes outside the lifted page, the same model as stage 2's navbar.
function navbarRoute(target, wide) {
  const direct = space.navbar.links.includes(target);
  const viaNav = (space.journeys.find((j) => j.targets.includes(target))?.via || []).some((v) => space.navbar.links.includes(v));
  const menu = wide ? 0 : space.navbar.narrow_menu_actions;
  if (direct) return { cost: 1 + menu, regions: 1, where: 'navbar' };
  if (viaNav) return { cost: 2 + menu + (wide ? 0 : space.navbar.narrow_sidebar_actions), regions: 2, where: 'navbar via' };
  return null;
}

function stage1Cells(batch, size) {
  if (!opt.cells || !existsSync(opt.cells)) return null;
  const res = spawnSync(opt.cells, [`${size.w}x${DRAW_H}`, join(S1, 'state'), ...batch.map((r) => join(S1, `v${String(r.seq).padStart(3, '0')}.ui.json`))], { encoding: 'utf8', maxBuffer: 1 << 28 });
  if (res.status !== 0) log('cells: ' + res.stderr.slice(0, 300));
  return res.stdout.trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
}

function stage1Measure(r, doc, cell, size) {
  const body = size.view - 2;
  const screen = cell.screen || [];
  const rowOf = (label) => {
    const needle = label.slice(0, Math.min(label.length, size.w - 8, 18));
    const i = screen.findIndex((l) => l.includes(needle));
    return i < 0 ? null : i;
  };
  const reg = (p) => (cell.regions || []).find((x) => x.path === p);
  const sectionTop = (name) => reg(`pages/landing/sections/${name}`)?.y ?? null;
  const primaryAction = doc.pages.landing.sections.find((s) => s.name === 'actions').actions[0];
  const pr = reg(`pages/landing/sections/actions/actions/${primaryAction.name}`);
  const primaryRow = pr ? pr.y : rowOf(primaryAction.label);
  const regionTops = ['hero', ...doc.pages.landing.sections.filter((s) => s.name.startsWith('s_')).map((s) => s.name)].map((n) =>
    n === 'hero' ? 0 : sectionTop(n),
  );
  const journeys = {};
  for (const j of space.journeys) {
    let best = navbarRoute(j.targets[0], size.wide);
    for (const l of j.targets.flatMap((t) => linksTo(doc, pageOf(t)))) {
      const p = reg(l.path);
      const row = p ? p.y : rowOf(l.label);
      if (row === null) continue;
      const cost = 1 + Math.floor(row / body);
      const regions = 1 + regionTops.filter((y) => y !== null && y <= row).length;
      if (!best || cost < best.cost || (cost === best.cost && regions < best.regions)) best = { cost, regions, where: l.section };
    }
    journeys[j.name] = best;
  }
  return {
    primary_row: primaryRow,
    primary_in_view: primaryRow !== null && primaryRow + 1 <= body,
    words_before_action: primaryRow === null ? null : screen.slice(1, primaryRow).reduce((n, l) => n + words(l), 0),
    actions_total: Object.values(journeys).reduce((n, x) => n + (x?.cost ?? 99), 0),
    regions_total: Object.values(journeys).reduce((n, x) => n + (x?.regions ?? 99), 0),
    journeys,
    check_errors: (cell.check_errors || []).length,
    error: cell.error || null,
    us: (cell.check_us || 0) + (cell.open_us || 0) + (cell.draw_us || 0),
  };
}

// Static reading, used for tap targets and when no cell reader is given.
function staticMeasure(doc) {
  const secs = doc.pages.landing.sections.filter((s) => !s.remove);
  let w = words(doc.pages.landing.title);
  let found = false;
  for (const s of secs) {
    if (s.name === 'actions') {
      found = true;
      break;
    }
    for (const c of s.children || []) w += words(c.text || '');
  }
  const links = secs.reduce((n, s) => n + (s.actions || []).filter((a) => a.as === 'link').length + (s.children || []).filter((c) => c.primitive === 'link').length, 0);
  return { words_before_action: found ? w : null, small_targets: links };
}

let seq = 0;
const points = [];
function point(p) {
  const pt = { climb_point_id: `pt-${RUN}-${String(++seq).padStart(3, '0')}`, seq, at: now(), ...p };
  points.push(pt);
  emit({ t: 'point', point: pt });
}

const dominated = (a, b) => a.every((v, i) => v >= b[i]) && a.some((v, i) => v > b[i]);
function pareto(list, vec) {
  return list.filter((x) => !list.some((y) => y !== x && dominated(vec(x), vec(y))));
}

const t1 = performance.now();
const BATCH = 24;
let evaluated = 0, pruned = 0, insufficient = 0;
for (const r of records) {
  r.doc = liftDocument(r.product, status);
  writeFileSync(join(S1, `v${String(r.seq).padStart(3, '0')}.ui.json`), JSON.stringify(r.doc));
  r.static = staticMeasure(r.doc);
}
times.stage1_materialize_ms = ms(t1);
const cellsUs = [];
for (let i = 0; i < records.length; i += BATCH) {
  const batch = records.slice(i, i + BATCH);
  const per = {};
  for (const size of SIZES) {
    const cells = stage1Cells(batch, size);
    batch.forEach((r, k) => {
      r.s1 = r.s1 || {};
      r.s1[size.name] = cells ? stage1Measure(r, r.doc, cells[k], size) : null;
      if (cells) cellsUs.push(r.s1[size.name].us);
    });
  }
  for (const r of batch) {
    const m = Object.values(r.s1).filter(Boolean);
    const fails = [];
    if (m.some((x) => x.check_errors || x.error)) fails.push('ess_ui_check');
    if (m.some((x) => !x.primary_in_view)) fails.push('primary_in_view');
    for (const j of space.journeys) if (m.some((x) => !x.journeys[j.name])) fails.push('journey:' + j.name);
    if (r.contentMissing.length) fails.push('content_missing');
    r.s1fails = fails;
    r.k1 = m.length
      ? {
          'load.words_before_action': Math.max(...m.map((x) => x.words_before_action ?? 999)),
          'attention.primary_top_px': Math.max(...m.map((x) => x.primary_row ?? 99)),
          'effort.actions_per_journey': Math.max(...m.map((x) => x.actions_total)),
          'attention.regions_crossed': Math.max(...m.map((x) => x.regions_total)),
          'a11y.tap_targets_under_44px': r.static.small_targets,
          [CONTENT]: baseContent.length - r.contentMissing.length,
        }
      : { 'load.words_before_action': r.static.words_before_action, 'a11y.tap_targets_under_44px': r.static.small_targets, [CONTENT]: baseContent.length - r.contentMissing.length };
    evaluated++;
    if (fails.length) pruned++;
    if (m.some((x) => x.error)) insufficient++;
  }
  const done = records.slice(0, i + batch.length);
  const bk = baseline.k1 || done[0].k1;
  for (const r of done) r.score1 = score(r.k1, baseline.k1 || bk, r.s1fails.length);
  const passing = done.filter((r) => !r.s1fails.length);
  const front = pareto(passing, (r) => vecOf(r.k1));
  const best = [...done].sort((a, b) => a.score1 - b.score1 || a.seq - b.seq)[0];
  point({ stage: 1, evaluated, pruned, insufficient, frontier_size: front.length, best: best.digest, best_score: +best.score1.toFixed(4) });
}
times.stage1_ms = ms(t1);
times.stage1_cells_us_median = cellsUs.length ? cellsUs.sort((a, b) => a - b)[Math.floor(cellsUs.length / 2)] : null;

function score(k, b, fails) {
  const keys = KPI_KEYS.filter((x) => k && b && x in k && x in b && b[x]);
  const s = keys.reduce((n, x) => n + (HIGHER.has(x) ? (k[x] ? b[x] / k[x] : 10) : k[x] / b[x]), 0) / (keys.length || 1);
  return fails * 1000 + s;
}

const ranked1 = [...records].sort((a, b) => a.score1 - b.score1 || a.seq - b.seq);
ranked1.forEach((r, i) => (r.rank1 = i + 1));
const chosen = ranked1.filter((r) => !r.s1fails.length).slice(0, TOP);
if (!chosen.includes(baseline)) chosen.push(baseline);
for (const r of records) {
  r.state = chosen.includes(r) ? 'Candidate' : 'Pruned';
  emit({ t: 'variant', variant: variantRecord(r) });
}
pruned = records.length - chosen.length;
point({ stage: 1, evaluated, pruned, insufficient, frontier_size: chosen.length, best: ranked1[0].digest, best_score: +ranked1[0].score1.toFixed(4), note: `stage 1 kept ${chosen.length}` });
log(`stage 1: ${records.length} variants in ${times.stage1_ms} ms (cells median ${times.stage1_cells_us_median} us), ${records.filter((r) => r.s1fails.length).length} fail hard rules, ${chosen.length} go to stage 2`);

function variantRecord(r) {
  const evals = [];
  for (const [env, m] of Object.entries(r.s1 || {})) {
    if (!m) continue;
    evals.push({
      evaluation_key: `ev-${r.digest.slice(7, 15)}-${env}`,
      state_space_key: STAGE1_SPACE,
      environment_id: env,
      values: [
        { metric_key: 'load.words_before_action', at: 'page', value: m.words_before_action },
        { metric_key: 'attention.primary_row', at: 'page', value: m.primary_row },
        ...Object.entries(m.journeys).map(([j, x]) => ({ metric_key: 'effort.actions', at: j, value: x?.cost ?? null })),
        ...Object.entries(m.journeys).map(([j, x]) => ({ metric_key: 'attention.regions_crossed', at: j, value: x?.regions ?? null })),
      ],
      insufficient_because: m.error,
      state: m.error ? 'Insufficient' : 'Computed',
    });
  }
  for (const [env, m] of Object.entries(r.s2 || {})) {
    evals.push({
      evaluation_key: `ev-${r.digest.slice(7, 15)}-${env}`,
      state_space_key: STAGE2_SPACE,
      environment_id: env,
      values: [
        { metric_key: 'load.words_before_action', at: 'page', value: m.metrics.words_before_action },
        { metric_key: 'attention.primary_top_px', at: 'page', value: m.metrics.primary_top_px },
        ...Object.entries(m.j.journeys).map(([j, x]) => ({ metric_key: 'effort.actions', at: j, value: x.cost })),
        ...Object.entries(m.j.journeys).map(([j, x]) => ({ metric_key: 'attention.regions_crossed', at: j, value: x.regions })),
        { metric_key: 'a11y.tap_targets_under_44px', at: 'page', value: m.j.small_targets },
        { metric_key: CONTENT, at: 'page', value: m.content.kept },
        { metric_key: CONTRAST_MIN, at: 'page', value: m.metrics.contrast_min },
        { metric_key: 'a11y.apca_lc_min', at: 'page', value: m.j.apca.lc_min },
        { metric_key: 'layout.page_height_px', at: 'page', value: m.metrics.page_height },
      ],
      insufficient_because: null,
      state: 'Computed',
    });
  }
  const verdicts = [];
  if (r.s2) {
    const envs = Object.entries(r.s2);
    const rule = (id, failName, strength = 'hard') => {
      const bad = envs.filter(([, m]) => m.fails.some((f) => f === failName || f.startsWith(failName)));
      verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-${id}`, rule_id: id, strength, verdict: bad.length ? 'fail' : 'pass', findings: bad.map(([env, m]) => ({ node_path: env, note: m.fails.join(', ') })), score: null });
    };
    rule('primary-action-in-view', 'primary');
    rule('no-horizontal-scroll', 'horizontal_scroll');
    rule('contrast-wcag2-4.5', 'contrast');
    rule('every-control-named', 'name_missing');
    rule('every-base-section-and-card-present', 'content_missing');
    const lcMin = Math.min(...envs.map(([, m]) => m.j.apca.lc_min ?? 999));
    verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-apca`, rule_id: 'apca-body-text-lc60', strength: 'soft', verdict: lcMin >= 60 ? 'pass' : 'violated', findings: envs.filter(([, m]) => m.j.apca.under_60).map(([env, m]) => ({ node_path: env, note: `${m.j.apca.under_60} texts under Lc 60, min ${m.j.apca.lc_min}` })), score: lcMin === 999 ? null : lcMin });
  } else if (r.s1) {
    verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-check`, rule_id: 'ess-ui-check', strength: 'hard', verdict: r.s1fails.includes('ess_ui_check') ? 'fail' : 'pass', findings: [], score: null });
    verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-content1`, rule_id: 'every-base-section-and-card-present (sources)', strength: 'hard', verdict: r.contentMissing.length ? 'fail' : 'pass', findings: r.contentMissing.map((x) => ({ node_path: x, note: 'missing' })), score: baseContent.length - r.contentMissing.length });
    verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-prim1`, rule_id: 'primary-action-in-view (cells)', strength: 'hard', verdict: r.s1fails.includes('primary_in_view') ? 'fail' : 'pass', findings: [], score: null });
  }
  verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-score1`, rule_id: 'stage 1 score (rank ' + r.rank1 + ')', strength: 'soft', verdict: 'scored', findings: [], score: r.score1 == null ? null : +r.score1.toFixed(4) });
  if (r.score2 != null) verdicts.push({ rule_verdict_id: `rv-${r.digest.slice(7, 13)}-score2`, rule_id: 'stage 2 score (rank ' + r.rank2 + ')', strength: 'soft', verdict: 'scored', findings: [], score: +r.score2.toFixed(4) });
  return {
    variant_digest: r.digest,
    choice: r.choice,
    patch: r.patch,
    baseline: r.baseline,
    state: r.state,
    spec: { product: r.product, css: r.css },
    evaluations: evals,
    verdicts,
  };
}

// ---- stage 2 --------------------------------------------------------------------------------
const VIEWPORTS = [[1440, 900], [1024, 768], [390, 844]];
const MODES = ['light', 'dark'];
const t2 = performance.now();
browser = await launch();
writeFileSync(join(DIR, 'chromium.pid'), String(browser.pid));
const page = await browser.newPage();
await page.send('Page.addScriptToEvaluateOnNewDocument', { source: readFileSync(join(LIFT, 'landing-page.js'), 'utf8') });
await page.viewport(...VIEWPORTS[0]);
const tLoad = performance.now();
await page.goto(`${BASE}/uxui/`);
times.stage2_page_load_ms = ms(tLoad);
// The theme fades colours on a mode switch; a measurement taken inside the fade reads a colour
// between the two modes (seen as a false contrast failure). The harness turns transitions off.
await page.eval(`(() => { const s = document.createElement('style'); s.id = 'uxui-no-motion'; s.textContent = '*,*::before,*::after{transition:none!important;animation:none!important}'; document.head.appendChild(s); return true; })()`);
await page.call('scaffold', { primary: space.primary });
// stage 0, second half: the binding resolves on the built page
const drift = await page.eval(`(${JSON.stringify(binding.nodes)} && Object.entries(${JSON.stringify(binding.nodes)}).map(([p, s]) => [p, document.querySelectorAll(s).length]).filter(([, n]) => n !== 1))`);
log(`stage 0: binding drift ${JSON.stringify(drift)}`);
times.stage0_drift = drift.length;

const jspec = { journeys: space.journeys, navbar: space.navbar };
async function measureVariant(r) {
  const out = {};
  const tr = await page.eval(`window.__landing.render(${JSON.stringify({ product: r.product, css: r.css, cssText })})`);
  for (const mode of MODES) {
    await page.eval(`document.documentElement.setAttribute('data-theme', ${JSON.stringify(mode)}), true`);
    for (const [w, h] of VIEWPORTS) {
      await page.viewport(w, h);
      const m = await page.call('measure', { primary: space.primary });
      const j = await page.eval(`window.__landing.journeys(${JSON.stringify(jspec)})`);
      const fails = [...m.fails];
      for (const [name, x] of Object.entries(j.journeys)) if (x.cost == null) fails.push('journey:' + name);
      const ids = await page.eval('window.__landing.content()');
      const missing = missingOf(ids, baseContent);
      if (missing.length) fails.push('content_missing');
      out[`${w}x${h}-${mode}`] = { fails, metrics: m.metrics, contrast_samples: m.contrast_samples, j, content: { kept: baseContent.length - missing.length, missing } };
    }
  }
  return { out, render_ms: tr.ms };
}
function k2Of(r) {
  const envs = Object.entries(r.s2);
  const touch = envs.filter(([e]) => e.startsWith('390x844'));
  return {
    'load.words_before_action': Math.max(...envs.map(([, m]) => m.metrics.words_before_action)),
    'attention.primary_top_px': Math.max(...envs.map(([, m]) => m.metrics.primary_top_px ?? 9999)),
    'effort.actions_per_journey': Math.max(...envs.map(([, m]) => Object.values(m.j.journeys).reduce((n, x) => n + (x.cost ?? 99), 0))),
    'attention.regions_crossed': Math.max(...envs.map(([, m]) => Object.values(m.j.journeys).reduce((n, x) => n + (x.regions ?? 99), 0))),
    'a11y.tap_targets_under_44px': Math.max(...touch.map(([, m]) => m.j.small_targets)),
    [CONTENT]: Math.min(...envs.map(([, m]) => m.content.kept)),
  };
}

const order2 = [baseline, ...chosen.filter((r) => r !== baseline)];
const measured = [];
const renderMs = [];
for (const r of order2) {
  const tv = performance.now();
  const { out, render_ms } = await measureVariant(r);
  r.s2 = out;
  r.s2ms = ms(tv);
  renderMs.push(render_ms);
  r.s2fails = [...new Set(Object.entries(out).flatMap(([env, m]) => m.fails.map((f) => `${f}@${env}`)))];
  r.k2 = k2Of(r);
  measured.push(r);
  for (const x of measured) x.score2 = score(x.k2, baseline.k2, x.s2fails.length);
  const passing = measured.filter((x) => !x.s2fails.length);
  const front = pareto(passing, (x) => vecOf(x.k2));
  const best = [...measured].sort((a, b) => a.score2 - b.score2)[0];
  emit({ t: 'variant', variant: variantRecord(r) });
  point({ stage: 2, evaluated: evaluated + measured.length, pruned: pruned + measured.filter((x) => x.s2fails.length).length, insufficient, frontier_size: front.length, best: best.digest, best_score: +best.score2.toFixed(4) });
  emit({ t: 'frontier', entries: frontierEntries(front) });
}
times.stage2_ms = ms(t2);
times.stage2_variant_ms_median = measured.map((r) => r.s2ms).sort((a, b) => a - b)[Math.floor(measured.length / 2)];
times.stage2_render_ms_median = renderMs.sort((a, b) => a - b)[Math.floor(renderMs.length / 2)];

function frontierEntries(front) {
  return [...front]
    .sort((a, b) => a.score2 - b.score2)
    .map((r) => ({
      frontier_entry_id: `fe-${r.digest.slice(7, 15)}`,
      variant_digest: r.digest,
      kpi_values: [...KPI_KEYS.map((k) => ({ metric_key: k, value: r.k2[k] })), { metric_key: 'score', value: +r.score2.toFixed(4) }],
      evaluation_keys: Object.keys(r.s2).map((env) => `ev-${r.digest.slice(7, 15)}-${env}`),
    }));
}

// agreement
function kendall(xs, ys) {
  let c = 0, d = 0, tx = 0, ty = 0, txy = 0;
  for (let i = 0; i < xs.length; i++)
    for (let j = i + 1; j < xs.length; j++) {
      const a = Math.sign(xs[i] - xs[j]), b = Math.sign(ys[i] - ys[j]);
      if (a === 0 && b === 0) txy++;
      else if (a === 0) tx++;
      else if (b === 0) ty++;
      else if (a === b) c++;
      else d++;
    }
  const n0 = (xs.length * (xs.length - 1)) / 2;
  return {
    pairs: n0,
    concordant: c,
    discordant: d,
    tied_stage1_only: tx,
    tied_stage2_only: ty,
    tied_both: txy,
    tau_a: +((c - d) / n0).toFixed(3),
    tau_b: +((c - d) / Math.sqrt((c + d + tx) * (c + d + ty) || 1)).toFixed(3),
  };
}
const agreement = kendall(measured.map((r) => r.score1), measured.map((r) => r.score2));
const ranked2 = [...measured].sort((a, b) => a.score2 - b.score2);
ranked2.forEach((r, i) => (r.rank2 = i + 1));
const passing2 = measured.filter((x) => !x.s2fails.length);
const front2 = pareto(passing2, (x) => vecOf(x.k2));
for (const r of measured) emit({ t: 'variant', variant: variantRecord(r) });
emit({ t: 'frontier', entries: frontierEntries(front2) });
agreement.top3_overlap = ranked2.slice(0, 3).filter((r) => ranked1.slice(0, 3).includes(r)).length;
log(`stage 2: ${measured.length} variants in ${times.stage2_ms} ms; tau_b ${agreement.tau_b}`);

// winner: the best stage-2 score among passing variants; within 2 % of it, the smallest change
const top = ranked2.filter((r) => !r.s2fails.length);
const band = top.filter((r) => r.score2 <= top[0].score2 * 1.02);
const winner = [...band].sort((a, b) => a.patch.length - b.patch.length || a.score2 - b.score2)[0];

// ---- stage 3 --------------------------------------------------------------------------------
const t3 = performance.now();
const shotList = [baseline, ...top.filter((r) => r !== baseline).slice(0, SHOTS)];
if (!shotList.includes(winner)) shotList.push(winner);
for (const r of shotList) {
  await page.eval(`window.__landing.render(${JSON.stringify({ product: r.product, css: r.css, cssText })})`);
  r.shots = {};
  for (const mode of MODES) {
    await page.eval(`document.documentElement.setAttribute('data-theme', ${JSON.stringify(mode)}), true`);
    for (const [w, h] of [VIEWPORTS[0], VIEWPORTS[2]]) {
      await page.viewport(w, h);
      await page.eval('scrollTo(0,0), true');
      const f = `${r.baseline ? 'current' : 'r' + String(top.indexOf(r) + 1).padStart(2, '0')}-${r.digest.slice(7, 15)}-${mode}-${w}.png`;
      await page.screenshot(join(DIR, 'shots', f));
      r.shots[`${mode}-${w}`] = f;
    }
  }
  await page.eval(`document.documentElement.setAttribute('data-theme', 'light'), true`);
  await page.viewport(...VIEWPORTS[0]);
  const f = `${r.baseline ? 'current' : 'r' + String(top.indexOf(r) + 1).padStart(2, '0')}-${r.digest.slice(7, 15)}-light-1440-full.png`;
  const hgt = await page.eval('document.documentElement.scrollHeight');
  await page.send('Emulation.setDeviceMetricsOverride', { width: 1440, height: hgt, deviceScaleFactor: 1, mobile: false });
  await page.screenshot(join(DIR, 'shots', f));
  r.shots['light-1440-full'] = f;
}
times.stage3_shots_ms = ms(t3);
await browser.close();
browser = null;

// ---- results --------------------------------------------------------------------------------
const delta = (r) => Object.fromEntries(KPI_KEYS.map((k) => [k, { value: r.k2[k], current: baseline.k2[k], delta: r.k2[k] - baseline.k2[k] }]));
const describe = (r) => ({
  digest: r.digest,
  rank1: r.rank1,
  rank2: r.rank2,
  score1: +r.score1.toFixed(4),
  score2: r.score2 == null ? null : +r.score2.toFixed(4),
  choice: r.choice,
  patch: r.patch,
  fails: r.s2fails,
  kpis: r.k2,
  kpis_stage1: r.k1,
  frontier: front2.includes(r),
  apca_lc_min: r.s2 ? Math.min(...Object.values(r.s2).map((m) => m.j.apca.lc_min ?? 999)) : null,
  contrast_min: r.s2 ? Math.min(...Object.values(r.s2).map((m) => m.metrics.contrast_min ?? 99)) : null,
  routes: r.s2 ? Object.fromEntries(Object.entries(r.s2).map(([e, m]) => [e, Object.fromEntries(Object.entries(m.j.journeys).map(([j, x]) => [j, `${x.cost} via ${x.where}`]))])) : null,
  shots: r.shots || null,
});
const results = {
  campaign_id: CAMPAIGN_ID,
  space: { axes: Object.fromEntries(Object.entries(space.axes).map(([a, x]) => [a, Object.keys(x.values)])), variants: all.length },
  stage0: { lift_errors: baseFindings, binding_drift: drift },
  content: { base_items: baseContent.length, base: baseContent },
  stage1: { variants: records.length, content_pruned: records.filter((r) => r.contentMissing.length).length, after_content_rule: records.filter((r) => !r.contentMissing.length).length, hard_fail: records.filter((r) => r.s1fails.length).length, fails_by_rule: countFails(records.map((r) => r.s1fails)), kept: chosen.length },
  stage2: { measured: measured.length, hard_fail: measured.filter((r) => r.s2fails.length).length, fails_by_rule: countFails(measured.map((r) => r.s2fails.map((f) => f.split('@')[0]))) },
  agreement,
  times,
  score: SCORE_NOTE,
  baseline: describe(baseline),
  frontier: front2.sort((a, b) => a.score2 - b.score2).map(describe),
  ranked: ranked2.map(describe),
  winner: { ...describe(winner), delta: delta(winner), product: winner.product, css: winner.css },
};
function countFails(lists) {
  const c = {};
  for (const l of lists) for (const f of new Set(l)) c[f] = (c[f] || 0) + 1;
  return c;
}
writeFileSync(join(DIR, 'results.json'), JSON.stringify(results, null, 1));
writeFileSync(join(DIR, 'compare.html'), compareHtml(shotList, top));
emit({
  t: 'promotion',
  promotion: {
    promotion_id: 'pr-docs-landing-1',
    campaign_id: CAMPAIGN_ID,
    variant_digest: winner.digest,
    rationale: `stage-2 rank ${winner.rank2}, score ${winner.score2.toFixed(3)} against 1.000 for the current page; ` + KPI_KEYS.map((k) => `${k.split('.')[1]} ${baseline.k2[k]} -> ${winner.k2[k]}`).join(', '),
    trade_off_group: null,
    accepted_by: null,
    rejection_reason: null,
    state: 'Proposed',
    diff: { changed: [...new Set(winner.patch.map((p) => (p.op === 'css' ? 'website/src/css/landing.css' : 'website/product.json')))] },
    propagates_to: [],
  },
});
emit({ t: 'campaign_state', patch: { state: 'Ended', ended_because: 'space_exhausted' } });
log(`done: winner ${winner.digest} ${JSON.stringify(winner.choice)}; times ${JSON.stringify(times)}`);
process.exit(0);

function compareHtml(list, topList) {
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
  const label = (r) => (r.baseline ? 'Current page' : `Rank ${topList.indexOf(r) + 1}${r === winner ? ' (applied)' : ''}`);
  const kpiRow = (r) =>
    KPI_KEYS.map((k) => {
      const d = r.k2[k] - baseline.k2[k];
      const good = HIGHER.has(k) ? d > 0 : d < 0, bad = HIGHER.has(k) ? d < 0 : d > 0;
      return `<li><span>${esc(k.split('.')[1].replace(/_/g, ' '))}</span> <b>${r.k2[k]}</b>${r.baseline ? '' : ` <em class="${good ? 'good' : bad ? 'bad' : ''}">${d > 0 ? '+' : ''}${d}</em>`}</li>`;
    }).join('');
  const changes = (r) => (r.patch.length ? r.patch.map((p) => `<code>${esc(p.op)} ${esc(p.path)}${p.op === 'css' ? ' = ' + p.value : ''}</code>`).join(' ') : '<code>no change</code>');
  const rows = [
    ['light-1440', 'Light, 1440 x 900'],
    ['dark-1440', 'Dark, 1440 x 900'],
    ['light-390', 'Light, 390 x 844'],
    ['dark-390', 'Dark, 390 x 844'],
    ['light-1440-full', 'Light, 1440, whole page'],
  ];
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Campaign ${esc(CAMPAIGN_ID)}: compare</title>
<style>
body{font:14px/1.45 system-ui,sans-serif;margin:0;background:#f6f7f8;color:#1b1f23}
header{padding:16px 24px;border-bottom:1px solid #d0d7de;background:#fff}
h1{font-size:18px;margin:0 0 4px} p{margin:4px 0}
.grid{display:grid;grid-template-columns:120px repeat(${list.length},minmax(170px,1fr));gap:12px;padding:16px 24px;align-items:start}
.head{position:sticky;top:0;background:#f6f7f8;padding-bottom:8px;z-index:1}
.head h2{font-size:15px;margin:0} .head ul{list-style:none;padding:0;margin:6px 0;font-size:12px}
.head li span{color:#57606a} em.good{color:#1a7f37;font-style:normal} em.bad{color:#cf222e;font-style:normal}
.head code{display:inline-block;font-size:11px;background:#eaeef2;padding:1px 4px;border-radius:3px;margin:1px 0}
.rowlabel{font-weight:600;color:#57606a;padding-top:4px}
img{width:100%;border:1px solid #d0d7de;background:#fff;display:block}
.narrow img{max-width:200px}
a{color:#0969da}
</style></head><body>
<header><h1>Documentation landing page: the top ${list.length - 1} variants beside the current page</h1>
<p>Campaign <code>${esc(CAMPAIGN_ID)}</code>, ${all.length} variants of the sources; stage 2 measured ${measured.length}. KPIs are the worst value over 1440x900, 1024x768 and 390x844 in light and dark (tap targets: 390x844); lower is better; deltas against the current page.</p>
<p>Open a variant live (scripts off, rendered from its sources): the links under each heading. The studio: <a href="/campaigns/climb/${esc(CAMPAIGN_ID)}">live climb</a>, <a href="/campaigns/frontier/${esc(CAMPAIGN_ID)}">frontier</a>.</p></header>
<div class="grid">
<div class="head"></div>
${list
  .map(
    (r) => `<div class="head"><h2>${esc(label(r))}</h2><p><code>${esc(r.digest)}</code> score ${r.score2.toFixed(3)}${front2.includes(r) ? ' · frontier' : ''}</p><ul>${kpiRow(r)}</ul><p>${changes(r)}</p><p><a href="/uxui/?variant=${encodeURIComponent(r.digest)}&mode=light">light</a> · <a href="/uxui/?variant=${encodeURIComponent(r.digest)}&mode=dark">dark</a> · <a href="/variants/detail/${encodeURIComponent(r.digest)}">in the studio</a></p></div>`,
  )
  .join('\n')}
${rows
  .map(
    ([key, name]) =>
      `<div class="rowlabel">${esc(name)}</div>` +
      list.map((r) => `<div class="${key.includes('390') ? 'narrow' : ''}"><a href="/shots/${esc(r.shots[key])}"><img loading="lazy" src="/shots/${esc(r.shots[key])}" alt="${esc(label(r) + ', ' + name)}"></a></div>`).join(''),
  )
  .join('\n')}
</div></body></html>
`;
}
