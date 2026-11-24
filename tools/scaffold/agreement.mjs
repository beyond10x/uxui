// agreement.mjs --results <results.json> --s1 <campaign s1 dir> --cells <uxui-cells> [--state <dir>]
//                [--type <file>]   (replaces space.json `type`, for a sensitivity check)
//
// Stage agreement of a finished landing campaign, again, without a browser: stage 1 is recomputed
// for the variants stage 2 measured (their lifted documents in <s1 dir>, drawn by uxui-cells at
// both sizes) with each subset of reflow()'s inputs (cells-model.mjs), and Kendall's tau-b between
// that score and the stage-2 score stored in results.json is printed for each. The subset with no
// input is the drawing as it is, so its row must reproduce the campaign's own stage-1 scores.
// Prints one JSON line per subset and a summary line with the time per variant.

import { readFileSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { HERE, parseArgs } from './cdp.mjs';
import { variants, pageOf } from './lift/docs-landing/lift.mjs';
import { INPUTS, SIZES, DRAW_H, CONTENT, measure, staticMeasure, k1Of, fails1, score, kendall } from './lift/docs-landing/cells-model.mjs';

const { opt } = parseArgs(process.argv.slice(2));
if (!opt.results || !opt.s1 || !opt.cells || !existsSync(opt.cells)) {
  console.error('usage: agreement.mjs --results <results.json> --s1 <dir> --cells <uxui-cells> [--state <dir>]');
  process.exit(2);
}
const space = JSON.parse(readFileSync(join(HERE, 'lift', 'docs-landing', 'space.json'), 'utf8'));
if (opt.type) space.type = JSON.parse(readFileSync(resolve(opt.type), 'utf8'));
const results = JSON.parse(readFileSync(resolve(opt.results), 'utf8'));
const S1 = resolve(opt.s1);
const state = resolve(opt.state || join(S1, 'state'));
mkdirSync(state, { recursive: true });

const all = variants(space);
const seqOf = (choice) => all.findIndex((c) => Object.keys(c).every((a) => c[a] === choice[a]));
const rows = results.ranked.map((r) => {
  const seq = seqOf(r.choice);
  const doc = JSON.parse(readFileSync(join(S1, `v${String(seq).padStart(3, '0')}.ui.json`), 'utf8'));
  const css = { density: space.axes.density.values[r.choice.density], heading_scale: space.axes.heading_scale.values[r.choice.heading_scale] };
  return { seq, digest: r.digest, baseline: r.digest === results.baseline.digest, doc, css, score1_run: r.score1, score2: r.score2, kept: r.kpis_stage1[CONTENT] };
});

// One drawing per variant and size; every subset of inputs reads the same drawings.
const cells = {};
for (const size of SIZES) {
  const res = spawnSync(opt.cells, [`${size.w}x${DRAW_H}`, state, ...rows.map((r) => join(S1, `v${String(r.seq).padStart(3, '0')}.ui.json`))], { encoding: 'utf8', maxBuffer: 1 << 28 });
  if (res.status !== 0) {
    console.error(res.stderr.slice(0, 500));
    process.exit(1);
  }
  cells[size.name] = res.stdout.trim().split('\n').map((l) => JSON.parse(l));
}

const med = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const subsets = [];
for (let mask = 0; mask < 1 << INPUTS.length; mask++) subsets.push(Object.fromEntries(INPUTS.map((k, i) => [k, !!(mask & (1 << i))])));
const modelUs = [];
let reproduced = null;
for (const on of subsets) {
  for (const r of rows) {
    const m = SIZES.map((size, i) => measure({ space, doc: r.doc, cell: cells[size.name][rows.indexOf(r)], size, css: r.css, on, pageOf }));
    if (INPUTS.every((k) => on[k])) modelUs.push(...m.map((x) => x.model_us));
    r.k1 = k1Of(m, staticMeasure(r.doc), r.kept);
    r.fails = fails1(m, space, []);
  }
  const base = rows.find((r) => r.baseline);
  for (const r of rows) r.score1 = score(r.k1, base.k1, r.fails.length);
  if (INPUTS.every((k) => !on[k])) reproduced = rows.every((r) => +r.score1.toFixed(4) === r.score1_run);
  const k = kendall(rows.map((r) => r.score1), rows.map((r) => r.score2));
  console.log(JSON.stringify({ inputs: INPUTS.filter((x) => on[x]), tau_b: k.tau_b, tau_a: k.tau_a, concordant: k.concordant, discordant: k.discordant, tied_stage1_only: k.tied_stage1_only, tied_both: k.tied_both, stage1_fails: rows.filter((r) => r.fails.length).length }));
}
const cellsUs = SIZES.flatMap((s) => cells[s.name].map((c) => (c.check_us || 0) + (c.open_us || 0) + (c.draw_us || 0)));
console.log(JSON.stringify({ variants: rows.length, run_scores_reproduced: reproduced, cells_us_median: med(cellsUs), model_us_median: med(modelUs) }));
