// stage2.mjs <lift-dir> <url> --stage1 <stage1.json> [--top 3] [--spread 7] [--out dir] [--shots]
// Stage 2 of the scaffold funnel: projects chosen stage-1 variants of the lifted document back
// onto the real page through the binding table (node path -> CSS selector) and the runtime, then
// measures them in headless Chromium with sweep.mjs. Reports whether the browser agrees with the
// stage-1 ranking (top-k overlap and Kendall's tau over the projected variants).

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { HERE, parseArgs } from './cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
const [liftArg, url] = pos;
if (!liftArg || !url || !opt.stage1) {
  console.error('usage: stage2.mjs <lift-dir> <url> --stage1 <stage1.json> [--top 3] [--spread 7] [--out dir] [--shots]');
  process.exit(2);
}
const lift = resolve(liftArg);
const out = resolve(opt.out || join(lift, '..', 'stage2-out'));
mkdirSync(out, { recursive: true });
const space = JSON.parse(readFileSync(join(lift, 'space.json'), 'utf8'));
const binding = JSON.parse(readFileSync(join(lift, 'binding.json'), 'utf8'));
const s1 = JSON.parse(readFileSync(opt.stage1, 'utf8'));
const sel = (path) => binding.nodes[path];
const pagePath = `pages/${space.page}`;
const sectionSel = (n) => sel(`${pagePath}/sections/${n}`);

// variant of the lifted document -> runtime variant (DOM operations through the binding table)
export function project(choice) {
  const ops = [];
  const v = { ops, scope: binding.root };
  const base = space.axes.section_order?.original;
  const order = space.axes.section_order?.[choice.section_order] ?? base;
  const layout = choice.layout ? space.axes.layout[choice.layout] : 'stack';
  // sections of one column (one DOM parent) are reordered among their own slots
  const parentOf = (n) => document_parent[n];
  const document_parent = {};
  const columns = layout?.columns ?? space.axes.layout?.columns?.columns ?? [{ name: 'all', sections: base }];
  for (const c of columns) for (const n of c.sections) document_parent[n] = c.name;
  for (const c of columns) {
    const items = order.filter((n) => parentOf(n) === c.name && sectionSel(n));
    const orig = base.filter((n) => parentOf(n) === c.name && sectionSel(n));
    if (items.join() !== orig.join()) ops.push({ op: 'order', items: items.map(sectionSel) });
  }
  const primaryName = space.primary.split('/').pop();
  const actionsPath = space.primary.slice(0, space.primary.lastIndexOf('/actions/'));
  for (const [axis, key] of Object.entries(choice)) {
    const val = space.axes[axis][key];
    if (axis === 'actions') {
      for (const name of space.axes.actions.all) if (!val.includes(name)) ops.push({ op: 'hide', sel: sel(`${actionsPath}/actions/${name}`) });
    } else if (axis === 'primary_label') {
      if (key !== Object.keys(space.axes.primary_label)[0]) ops.push({ op: 'text', sel: sel(space.primary), text: val });
    } else if (axis === 'density') {
      if (val !== 1) v.density = val;
    } else if (axis === 'layout') {
      if (val?.columns) ops.push({ op: 'style', sel: binding.root, props: { 'grid-template-columns': 'minmax(0, 1.6fr) minmax(0, 1fr)', 'column-gap': '2rem', 'align-items': 'start', 'max-width': 'none' } });
    } else if (val?.remove && sectionSel(axis)) ops.push({ op: 'hide', sel: sectionSel(axis) });
  }
  void primaryName;
  return v;
}

// pick: the top k of stage 1 and a spread over the rest of its ranking
const rows = s1.rows;
const ranked = [...rows].sort((a, b) => a.kpi - b.kpi || a.seq - b.seq);
const top = +(opt.top || 3), spread = +(opt.spread || 7);
const picked = ranked.slice(0, top);
for (let i = 1; i <= spread; i++) {
  const r = ranked[Math.min(ranked.length - 1, top + Math.floor(((ranked.length - top - 1) * i) / spread))];
  if (!picked.includes(r)) picked.push(r);
}
const list = picked.map((r) => ({ name: `s1#${r.seq}`, variant: project(r.variant), stage1: { seq: r.seq, kpi: r.kpi, rank: ranked.indexOf(r) + 1, choice: r.variant } }));
const listFile = join(out, 'projected.json');
writeFileSync(listFile, JSON.stringify(list, null, 1));

const reqFile = join(out, 'require.json');
// the same hard rules as stage 1: each journey's target and each required node is present and rendered
writeFileSync(reqFile, JSON.stringify([...space.journeys.map((j) => ({ name: j.name, sel: sel(j.act) })), ...space.required.map((p) => ({ name: p.split('/').pop(), sel: sel(p) }))]));
const args = [join(HERE, 'sweep.mjs'), url, '--variants', listFile, '--primary', sel(space.primary), '--require', reqFile, '--out', join(out, 'stage2.ndjson')];
if (opt.shots) args.push('--shots', join(out, 'shots'));
const t0 = performance.now();
const res = spawnSync(process.execPath, args, { encoding: 'utf8', env: process.env, maxBuffer: 1 << 26 });
const wall = Math.round(performance.now() - t0);
process.stderr.write(res.stderr);
const lines = res.stdout.trim().split('\n').map((l) => JSON.parse(l));
const climb = lines.filter((l) => l.seq !== undefined);
const summary = lines.find((l) => l.summary)?.summary;

// agreement between the stage-1 and the stage-2 ranking over the projected variants
const pairs = climb.filter((c) => c.seq > 0).map((c) => ({ ...c, s1: list[c.seq - 1].stage1 }));
const byS2 = [...pairs].sort((a, b) => a.kpi - b.kpi);
const byS1 = [...pairs].sort((a, b) => a.s1.kpi - b.s1.kpi || a.s1.seq - b.s1.seq);
let concord = 0, discord = 0, ties = 0;
for (let i = 0; i < pairs.length; i++)
  for (let j = i + 1; j < pairs.length; j++) {
    const a = Math.sign(pairs[i].s1.kpi - pairs[j].s1.kpi), b = Math.sign(pairs[i].kpi - pairs[j].kpi);
    if (!a || !b) ties++;
    else if (a === b) concord++;
    else discord++;
  }
const k = Math.min(top, pairs.length);
const topS1 = new Set(byS1.slice(0, k).map((p) => p.s1.seq));
const report = {
  projected: pairs.length,
  stage2_ms_per_variant_median: summary?.variant_ms_median,
  stage2_total_ms: wall,
  kendall_tau: +((concord - discord) / Math.max(1, concord + discord + ties)).toFixed(3),
  pairs: { concordant: concord, discordant: discord, tied: ties },
  top_k: k,
  top_k_overlap: byS2.slice(0, k).filter((p) => topS1.has(p.s1.seq)).length,
  table: byS2.map((p, i) => ({ s2_rank: i + 1, s1_rank: p.s1.rank, s1_seq: p.s1.seq, s1_kpi: p.s1.kpi, s2_kpi: p.kpi, s2_fails: p.fails, s2_metrics: p.metrics, choice: p.s1.choice })),
};
writeFileSync(join(out, 'agreement.json'), JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
