// sweep.mjs <url> <axes.json | auto> [--variants list.json] [--primary selector] [--require req.json]
//           [--out results.ndjson]
//           [--shots dir] [--viewports 1440x900,1024x768,390x844] [--limit 48]
// Headless: applies every variant (the exhaustive product of the axes, or an explicit list of
// {name, variant}), measures hard rules and metrics at each viewport, prints one climb line per
// variant (NDJSON) and the ranked frontier at the end.

import { readFileSync, writeFileSync, mkdirSync, appendFileSync } from 'node:fs';
import { join } from 'node:path';
import { launch, parseArgs } from './cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
const [url, axesArg] = pos;
if (!url || (!axesArg && !opt.variants)) {
  console.error('usage: sweep.mjs <url> <axes.json|auto> [--variants list.json] [--primary sel] [--out file] [--shots dir]');
  process.exit(2);
}
const viewports = (opt.viewports || '1440x900,1024x768,390x844').split(',').map((v) => v.split('x').map(Number));
const limit = +(opt.limit || 48);
// [{name, sel}]: nodes that must be present and rendered (a journey step target), a hard rule each
const require = opt.require ? JSON.parse(readFileSync(opt.require, "utf8")) : [];

function product(axes) {
  let out = [{}];
  for (const [k, values] of Object.entries(axes)) {
    const next = [];
    for (const v of out) for (const x of values) next.push({ ...v, [k]: x });
    out = next;
  }
  return out;
}
const isDefault = (k, v) =>
  v === 'original' || (k === 'density' && v === 1) || (k === 'font_scale' && v === 1) || (k === 'hide' && Array.isArray(v) && !v.length);
const compact = (v) => Object.fromEntries(Object.entries(v).filter(([k, x]) => !isDefault(k, x)));

// KPI, declared: 1e6 x hard-rule failures + words before the primary action (worst viewport)
// + px from the top to the primary action (worst viewport) / 10. Lower is better.
const kpiOf = (r) => r.fails.length * 1e6 + (r.worst.words_before_action ?? 999) + (r.worst.primary_top_px ?? 9999) / 10;
// Frontier objectives: failures down, words before action down, primary top down, contrast up.
const objectives = (r) => [r.fails.length, r.worst.words_before_action ?? 1e9, r.worst.primary_top_px ?? 1e9, -(r.worst.contrast_min ?? 0)];
const dominates = (a, b) => {
  const x = objectives(a), y = objectives(b);
  return x.every((v, i) => v <= y[i]) && x.some((v, i) => v < y[i]);
};

const tStart = performance.now();
const browser = await launch();
const results = [];
try {
  const page = await browser.newPage();
  await page.viewport(...viewports[0]);
  const tLoad = performance.now();
  await page.goto(url);
  const loadMs = Math.round(performance.now() - tLoad);
  const sc = await page.call('scaffold', { primary: opt.primary });
  let list;
  if (opt.variants) list = JSON.parse(readFileSync(opt.variants, 'utf8'));
  else {
    const axes = axesArg === 'auto' ? sc.axes : JSON.parse(readFileSync(axesArg, 'utf8'));
    list = product(axes).map((v) => ({ variant: compact(v) }));
  }
  if (list.length > limit) {
    console.error(`${list.length} variants; only the first ${limit} run (--limit)`);
    list = list.slice(0, limit);
  }
  list.unshift({ name: 'baseline', variant: {} });
  console.error(
    JSON.stringify({ url, load_ms: loadMs, scaffold_ms: sc.ms, sections: sc.sections.length, primary: sc.primary, accent: sc.palette.accent, variants: list.length }),
  );
  if (opt.out) writeFileSync(opt.out, '');
  let best = null;
  for (let seq = 0; seq < list.length; seq++) {
    const { name, variant } = list[seq];
    const t0 = performance.now();
    const per = {};
    let applyMs = 0, measureMs = 0;
    for (const [w, h] of viewports) {
      await page.viewport(w, h);
      const a = await page.call('apply', variant);
      const m = await page.call('measure', { primary: opt.primary, require });
      applyMs += a.ms;
      measureMs += m.ms;
      per[m.viewport] = m;
    }
    const ms = Math.round(performance.now() - t0);
    const cells = Object.values(per);
    const worst = {
      words_before_action: Math.max(...cells.map((c) => c.metrics.words_before_action)),
      primary_top_px: cells.some((c) => c.metrics.primary_top_px === null) ? null : Math.max(...cells.map((c) => c.metrics.primary_top_px)),
      interactive_above_fold: Math.min(...cells.map((c) => c.metrics.interactive_above_fold)),
      regions_above_fold: Math.min(...cells.map((c) => c.metrics.regions_above_fold)),
      contrast_min: Math.min(...cells.map((c) => c.metrics.contrast_min ?? 99)),
      contrast_cant_tell: Math.max(...cells.map((c) => c.metrics.contrast_cant_tell)),
    };
    const fails = cells.flatMap((c) => c.fails.map((f) => `${f}@${c.viewport}`));
    const r = { seq, name, variant, ms, apply_ms: +applyMs.toFixed(1), measure_ms: +measureMs.toFixed(1), worst, fails, per };
    r.kpi = +kpiOf(r).toFixed(1);
    if (!best || r.kpi < best.kpi) best = { seq, kpi: r.kpi, name, variant };
    results.push(r);
    const line = { seq, name, variant, ms, apply_ms: r.apply_ms, measure_ms: r.measure_ms, metrics: worst, fails, kpi: r.kpi, best: { seq: best.seq, kpi: best.kpi } };
    console.log(JSON.stringify(line));
    if (opt.out) appendFileSync(opt.out, JSON.stringify(r) + '\n');
  }
  const frontier = results.filter((a) => !results.some((b) => b !== a && dominates(b, a))).sort((a, b) => a.kpi - b.kpi);
  const total = Math.round(performance.now() - tStart);
  const perVariant = results.map((r) => r.ms).sort((a, b) => a - b);
  const summary = {
    summary: {
      variants: results.length,
      viewports: viewports.length,
      total_ms: total,
      load_ms: loadMs,
      variant_ms_median: perVariant[Math.floor(perVariant.length / 2)],
      variant_ms_max: perVariant[perVariant.length - 1],
      frontier: frontier.map((r) => ({ seq: r.seq, name: r.name, variant: r.variant, kpi: r.kpi, fails: r.fails.length, metrics: r.worst })),
    },
  };
  console.log(JSON.stringify(summary));
  console.error('\nrank  seq   kpi        fails words top_px contrast variant');
  frontier.forEach((r, i) =>
    console.error(
      `${String(i + 1).padStart(4)}  ${String(r.seq).padStart(3)}  ${String(r.kpi).padEnd(10)} ${String(r.fails.length).padStart(5)} ${String(r.worst.words_before_action).padStart(5)} ${String(r.worst.primary_top_px).padStart(6)} ${String(r.worst.contrast_min).padStart(8)} ${r.name ? r.name + ' ' : ''}${JSON.stringify(r.variant)}`,
    ),
  );
  console.error(`total ${total} ms, median ${summary.summary.variant_ms_median} ms per variant (${viewports.length} viewports)`);
  if (opt.shots) {
    mkdirSync(opt.shots, { recursive: true });
    const pick = [results[0], ...frontier.filter((r) => r.seq !== 0).slice(0, 3)];
    for (const r of pick) {
      for (const [w, h] of [viewports[0], viewports[viewports.length - 1]]) {
        await page.viewport(w, h);
        await page.call('apply', r.variant);
        await page.eval('scrollTo(0,0)');
        await page.screenshot(join(opt.shots, `v${String(r.seq).padStart(2, '0')}-${w}x${h}.png`));
      }
    }
  }
} finally {
  await browser.close();
}
