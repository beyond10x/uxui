// stage1.mjs <lift-dir> --out <dir> [--cells <uxui-cells binary>] [--size 80x24] [--no-cli]
// Stage 1 of the scaffold funnel: brute force over the variants of a lifted ess-ui/1 document,
// with no browser and no model call. For every combination of the axes in <lift-dir>/space.json:
// materialize the variant document, run `ess ui check` and the journeys with `ess ui test` on the
// terminal renderer, and (with --cells) draw it in-process on the terminal renderer to read cell
// geometry: words before the first action, rows to the primary action, nodes above the fold.
// Prints one NDJSON line per variant and a summary with the time per variant of each step.

import { readFileSync, writeFileSync, mkdirSync, cpSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { parseArgs } from './cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
const lift = resolve(pos[0] || '');
if (!pos[0] || !opt.out) {
  console.error('usage: stage1.mjs <lift-dir> --out <dir> [--cells bin] [--size 80x24] [--no-cli]');
  process.exit(2);
}
const out = resolve(opt.out);
const [W, H] = (opt.size || '80x24').split('x').map(Number);
const space = JSON.parse(readFileSync(join(lift, 'space.json'), 'utf8'));
const base = JSON.parse(readFileSync(join(lift, space.document), 'utf8'));
mkdirSync(join(out, 'tests'), { recursive: true });
if (existsSync(join(lift, 'fixtures'))) cpSync(join(lift, 'fixtures'), join(out, 'fixtures'), { recursive: true });

const axisNames = Object.keys(space.axes);
function product() {
  let acc = [{}];
  for (const a of axisNames) {
    const next = [];
    for (const c of acc) for (const v of Object.keys(space.axes[a])) next.push({ ...c, [a]: v });
    acc = next;
  }
  return acc;
}

// Mutation operators over the document. Axis kinds by name: section_order, actions, primary_label,
// layout, density (projection only: the terminal draws no tokens); any other axis names a section
// and chooses its component ({component, style}) or removes it ({remove: true}).
export function materialize(choice) {
  const d = structuredClone(base);
  const page = d.pages[space.page];
  const removed = page.sections.filter((s) => s.remove);
  const byName = new Map(page.sections.filter((s) => !s.remove).map((s) => [s.name, s]));
  let order = space.axes.section_order ? space.axes.section_order[choice.section_order] : [...byName.keys()];
  const primaryName = space.primary.split('/').pop();
  for (const [axis, key] of Object.entries(choice)) {
    const val = space.axes[axis][key];
    if (axis === 'actions') {
      for (const s of byName.values()) if (s.actions?.some((a) => a.name === primaryName)) s.actions = s.actions.filter((a) => val.includes(a.name));
    } else if (axis === 'primary_label') {
      for (const s of byName.values()) for (const a of s.actions || []) if (a.name === primaryName) a.label = val;
    } else if (byName.has(axis)) {
      if (val.remove) order = order.filter((n) => n !== axis);
      else Object.assign(byName.get(axis), val);
    }
  }
  page.sections = [...removed, ...order.map((n) => byName.get(n))];
  const layout = choice.layout ? space.axes.layout[choice.layout] : null;
  if (layout === 'stack') delete page.layout;
  else if (layout?.columns) {
    const columns = layout.columns
      .map((c) => ({ ...c, sections: order.filter((n) => c.sections.includes(n)) }))
      .filter((c) => c.sections.length);
    if (columns.length > 1) page.layout = { columns };
    else delete page.layout;
  }
  return d;
}

const words = (s) => (s.match(/[\p{L}\p{N}][\p{L}\p{N}'’.-]*/gu) || []).length;
const name = (i) => 'v' + String(i).padStart(3, '0');
const variants = product();
const ms = (t) => +(performance.now() - t).toFixed(2);

// Static reading of a variant (no renderer): words a reader passes before the primary action, in
// declaration order (the title, then each section above the one holding the action), and how many
// sections come before it. Fixture rows are read from JSON fixture files.
const fixtureRows = new Map();
function rowsOf(reads) {
  if (!reads?.fixture) return [];
  const file = join(lift, reads.fixture);
  if (!fixtureRows.has(file)) fixtureRows.set(file, existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')).views : {});
  return fixtureRows.get(file)[reads.placeholder || reads.view]?.rows || [];
}
function staticMetrics(doc) {
  const page = doc.pages[space.page];
  const primaryName = space.primary.split('/').pop();
  let w = words(page.title || ''), before = 0, found = false;
  for (const s of page.sections.filter((x) => !x.remove)) {
    if ((s.actions || []).some((a) => a.name === primaryName)) {
      found = true;
      break;
    }
    before++;
    w += words(s.title || '');
    const rs = rowsOf(s.reads);
    for (const f of s.fields || []) w += words(f.label || '') + words(String(rs[0]?.[f.field] ?? ''));
    for (const c of Array.isArray(s.columns) ? s.columns : []) w += words(c.label || c.field || '') + rs.reduce((n, r) => n + words(String(r[c.field] ?? '')), 0);
  }
  return { words_before_action: found ? w : null, sections_before_action: found ? before : null };
}

const rows = [];
const tGen = performance.now();
for (let i = 0; i < variants.length; i++) {
  const doc = materialize(variants[i]);
  const tS = performance.now();
  const st = staticMetrics(doc);
  const staticUs = Math.round((performance.now() - tS) * 1000);
  writeFileSync(join(out, name(i) + '.ui.json'), JSON.stringify(doc));
  const tests = {
    format: 'ess-ui-test/1',
    document: '../' + name(i) + '.ui.json',
    tests: space.journeys.map((j) => ({ name: j.name, steps: [{ open: space.page }, { act: j.act }, { expect: j.expect }] })),
  };
  writeFileSync(join(out, 'tests', name(i) + '.yaml'), JSON.stringify(tests));
  rows.push({ seq: i, name: name(i), variant: variants[i], static: st, static_us: staticUs });
}
const genMs = ms(tGen);

// CLI: one `ess ui check` and one `ess ui test` process per variant.
let cliMs = 0;
if (!opt['no-cli']) {
  for (const r of rows) {
    const t0 = performance.now();
    const c = spawnSync('ess', ['ui', 'check', '--path', join(out, r.name + '.ui.json'), '--format', 'json'], { encoding: 'utf8' });
    r.check_ms = ms(t0);
    let errors = -1;
    try {
      errors = JSON.parse(c.stdout).findings.filter((f) => f.severity === 'error').length;
    } catch {}
    r.check_errors = errors;
    const t1 = performance.now();
    const t = spawnSync('ess', ['ui', 'test', '--path', join(out, r.name + '.ui.json'), join(out, 'tests', r.name + '.yaml'), '--format', 'json'], { encoding: 'utf8' });
    r.test_ms = ms(t1);
    try {
      r.journeys = Object.fromEntries(JSON.parse(t.stdout).tests.map((x) => [x.name, x.status === 'passed']));
    } catch {
      r.journeys = { error: (t.stderr || t.stdout).slice(0, 200) };
    }
    cliMs += r.check_ms + r.test_ms;
  }
}

// In-process: check + open + draw every variant in one process.
let cellsMs = 0;
if (opt.cells) {
  const t0 = performance.now();
  const state = join(out, 'state');
  mkdirSync(state, { recursive: true });
  const res = spawnSync(opt.cells, [`${W}x${H}`, state, ...rows.map((r) => join(out, r.name + '.ui.json'))], { encoding: 'utf8', maxBuffer: 1 << 28 });
  cellsMs = ms(t0);
  if (res.status !== 0) console.error(res.stderr);
  const lines = res.stdout.trim().split('\n').map((l) => JSON.parse(l));
  lines.forEach((c, i) => {
    const r = rows[i];
    r.inproc_us = (c.check_us || 0) + (c.open_us || 0) + (c.draw_us || 0);
    r.inproc = { check_us: c.check_us, open_us: c.open_us, draw_us: c.draw_us };
    if (c.error) {
      r.cells_error = c.error;
      return;
    }
    const sectionPath = space.primary.slice(0, space.primary.lastIndexOf('/actions/'));
    const reg = (p) => c.regions.find((x) => x.path === p);
    const prim = reg(space.primary) || reg(sectionPath);
    const body = H - 2; // the last two rows are the status and hint lines
    r.cells = {
      primary_row: prim ? prim.y : null,
      primary_in_view: !!prim && prim.y + Math.min(prim.h, 2) <= body,
      words_before_action: prim ? c.screen.slice(1, prim.y).reduce((n, l) => n + words(l), 0) : null,
      nodes_above_fold: c.regions.filter((x) => x.y + x.h <= body).length,
      regions: c.regions.length,
      required_missing: space.required.filter((p) => !reg(p)).length,
    };
  });
}

// Hard rules and KPI. Lower is better. With cells: 1e6 x failures + words before the action + 2.4 x
// rows to it (a terminal row stands for about 24 CSS px; the browser KPI counts px / 10). Without:
// 1e6 x failures + the static words before the action.
for (const r of rows) {
  const fails = [];
  if (r.check_errors > 0) fails.push('check');
  for (const [j, ok] of Object.entries(r.journeys || {})) if (!ok) fails.push('journey:' + j);
  if (r.cells) {
    if (!r.cells.primary_in_view) fails.push('primary_in_view');
    if (r.cells.required_missing) fails.push('required_missing');
  }
  r.fails = fails;
  r.kpi_static = fails.length * 1e6 + (r.static.words_before_action ?? 999);
  r.kpi_cells = r.cells ? +(fails.length * 1e6 + (r.cells.words_before_action ?? 999) + 2.4 * (r.cells.primary_row ?? 99)).toFixed(1) : null;
  r.kpi = r.kpi_cells ?? r.kpi_static;
  console.log(JSON.stringify({ seq: r.seq, variant: r.variant, fails, kpi: r.kpi, static: r.static, cells: r.cells, check_ms: r.check_ms, test_ms: r.test_ms, inproc_us: r.inproc_us }));
}
const ranked = [...rows].sort((a, b) => a.kpi - b.kpi || a.seq - b.seq);
const med = (xs) => {
  const s = xs.filter((x) => x != null).sort((a, b) => a - b);
  return s.length ? s[Math.floor(s.length / 2)] : null;
};
const summary = {
  variants: rows.length,
  generate_ms_per_variant: +(genMs / rows.length).toFixed(3),
  cli_check_ms_median: med(rows.map((r) => r.check_ms)),
  cli_test_ms_median: med(rows.map((r) => r.test_ms)),
  cli_total_ms: Math.round(cliMs),
  static_us_median: med(rows.map((r) => r.static_us)),
  inproc_us_median: med(rows.map((r) => r.inproc_us)),
  inproc_total_ms: Math.round(cellsMs),
  passing: rows.filter((r) => !r.fails.length).length,
  top: ranked.slice(0, 10).map((r) => ({ seq: r.seq, variant: r.variant, kpi: r.kpi, static: r.static, cells: r.cells })),
};
writeFileSync(join(out, 'stage1.json'), JSON.stringify({ summary, rows }, null, 1));
console.log(JSON.stringify({ summary }));
