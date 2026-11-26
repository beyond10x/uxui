// summary.mjs <round id> <evidence dir> <round.json to write> [<audit bars file>]
// Reads what round.sh left in the evidence dir (versions.txt, page-checks.evidence.json, trial/,
// audit/) and writes the round's numbers as one file: the round.json of plugin/skills/
// ui-iteration-loop § 3.2. Deterministic code, no model, no browser. Every count is over the gating
// viewports and both modes unless its key says otherwise.
// With an audit bars file (studio.audit.json) it also counts the cells of the page audit that miss a
// bar. `bars` holds the most one cell may show per key: `audit.cells_over_a_bar` and `audit_bars`.
// `floors` holds the least one cell must show per key: `audit.cells_under_a_floor` and
// `audit_floors`. A cell is one page at one viewport in one mode. It prints one line per bar and per
// floor and never fails on one: a round is a measurement.
// `page_bars` holds bars of one page (`{<page path>: {<key>: <bar>}}`), for what only some pages are
// held to, such as fitting the viewport (`document_overflow`: the document's height past the
// viewport's, in px): `audit.cells_over_a_page_bar` and `audit_page_bars`.
// Where round.sh ran the page checks more than once (repeat/<n>.evidence.json), it reads the layout
// shift after ready of every run, the first included: per page the value of each run
// (`page_checks_repeat`), the largest of all runs (`page_checks.layout_shift.max_of_runs`) and the
// largest spread of one page between its runs (`page_checks.layout_shift.spread`). A measure that
// repeated runs agree on has a spread under the tolerance of the skill, 0.01.
// Where the scenario cards hold an expectation on notices (studio.scenarios.json `notices`, recorded
// per run by trial.mjs), it counts the runs that showed a notice missing it:
// `trial.runs_notice_expectation_missed` and `trial_notices`. A run is not less completed for it.
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [round, dir, target, barsFile] = process.argv.slice(2);
if (!round || !dir || !target) {
  console.error('usage: summary.mjs <round id> <evidence dir> <round.json to write> [<audit bars file>]');
  process.exit(2);
}

const json = (file) => JSON.parse(readFileSync(file, 'utf8'));
const list = (sub) => readdirSync(join(dir, sub)).filter((f) => f.endsWith('.json')).sort().map((f) => json(join(dir, sub, f)));
const sum = (values) => values.reduce((a, b) => a + b, 0);
const max = (values) => values.reduce((a, b) => Math.max(a, b), 0);

const versions = {};
for (const line of readFileSync(join(dir, 'versions.txt'), 'utf8').split('\n')) {
  const at = line.indexOf(':');
  if (at > 0) versions[line.slice(0, at).trim()] = line.slice(at + 1).trim();
}

const evidence = json(join(dir, 'page-checks.evidence.json')).evidence;
const outputs = {};
const misses = {};
for (const result of evidence.results) {
  if (result.value_type !== 'number') {
    // The set of misses of a page: one entry per rule, node and reason, with the places it was seen.
    for (const item of result.items ?? []) {
      const key = `${item.identity.rule} ${item.identity.node_path.replace(/^[0-9x]+\/(light|dark)\//, '')}: ${item.fields.reason.replace(/ at [0-9]+$/, '')}`;
      ((misses[result.item] ??= {})[key] ??= []).push(item.identity.node_path.split('/').slice(0, 2).join('.'));
    }
    continue;
  }
  const value = Number(result.value);
  const output = (outputs[result.output] ??= { unit: result.unit, total: 0, max: 0, pages: {} });
  output.total = Math.round((output.total + value) * 10000) / 10000;
  output.max = Math.max(output.max, value);
  if (value !== 0) output.pages[result.item] = { value, viewports: Object.fromEntries(Object.entries(result.details?.viewports ?? {}).map(([k, v]) => [k, Number(v)])) };
}

const audits = list('audit');
const pages = {};
for (const a of audits) {
  const page = (pages[a.path] ??= {});
  page[`${a.viewport}.${a.mode}`] = {
    document_height: a.document.scroll_height,
    viewport_height: a.document.client_height,
    document_scrolls: a.document.scroll_height > a.document.client_height + 1 || a.document.scroll_width > a.document.client_width + 1,
    document_overflow: Math.max(0, a.document.scroll_height - a.document.client_height),
    scrollable_elements: a.scrollers.length,
    sideways_scrollable_elements: a.scrollers.filter((s) => s.scroll_width > s.client_width + 1).length,
    h1: a.headings.filter((h) => h.startsWith('h1 ')).length,
    text_runs: a.text.runs,
    text_under_14px: a.text.under_14,
    text_low_contrast: a.text.low_contrast,
    min_contrast: a.text.min_contrast,
    controls: a.controls.count,
    controls_unnamed: a.controls.unnamed.length,
    controls_under_24px: a.controls.small.length,
    inputs_placeholder_only: a.controls.placeholder_only.length,
    tab_stops: a.tab.stops,
    tab_first_region: a.tab.first_region,
    tab_stops_unmarked: a.tab.unmarked.length,
  };
}
const cells = Object.values(pages).flatMap((page) => Object.values(page));
const pagesWhere = (test) => Object.entries(pages).filter(([, page]) => Object.values(page).some(test)).map(([path]) => path);

// The audit against its bars: per key the cells over the bar, and the worst of them.
const bars = barsFile ? json(barsFile).bars : null;
const auditBars = {};
for (const [key, bar] of Object.entries(bars ?? {})) {
  const over = Object.entries(pages).flatMap(([path, page]) => Object.entries(page).filter(([, cell]) => cell[key] > bar).map(([cell, values]) => ({ page: path, cell, value: values[key] })));
  if (cells.some((cell) => typeof cell[key] !== 'number')) {
    console.error(`summary: ${barsFile}: the page audit has no number "${key}"`);
    process.exit(2);
  }
  auditBars[key] = { bar, cells: cells.length, cells_over: over.length, worst: max(cells.map((cell) => cell[key])), over };
  console.log(`summary: bar ${key} <= ${bar}: ${over.length === 0 ? 'held' : 'missed'}, ${over.length} of ${cells.length} cells over it, worst ${auditBars[key].worst}`);
}

// The audit against its floors: per key the cells under the floor, and the least of them.
const floors = barsFile ? json(barsFile).floors ?? {} : {};
const auditFloors = {};
for (const [key, floor] of Object.entries(floors)) {
  const under = Object.entries(pages).flatMap(([path, page]) => Object.entries(page).filter(([, cell]) => cell[key] < floor).map(([cell, values]) => ({ page: path, cell, value: values[key] })));
  if (cells.some((cell) => typeof cell[key] !== 'number')) {
    console.error(`summary: ${barsFile}: the page audit has no number "${key}"`);
    process.exit(2);
  }
  auditFloors[key] = { floor, cells: cells.length, cells_under: under.length, least: cells.reduce((least, cell) => Math.min(least, cell[key]), Infinity), under };
  console.log(`summary: floor ${key} >= ${floor}: ${under.length === 0 ? 'held' : 'missed'}, ${under.length} of ${cells.length} cells under it, least ${auditFloors[key].least}`);
}
const hasFloors = Object.keys(floors).length > 0;

// The audit against the bars of single pages: per page and key the cells over the bar.
const pageBars = barsFile ? json(barsFile).page_bars ?? {} : {};
const auditPageBars = {};
for (const [path, keys] of Object.entries(pageBars)) {
  const page = pages[path];
  if (!page) {
    console.error(`summary: ${barsFile}: the page audit has no page "${path}"`);
    process.exit(2);
  }
  for (const [key, bar] of Object.entries(keys)) {
    const own = Object.entries(page);
    if (own.some(([, cell]) => typeof cell[key] !== 'number')) {
      console.error(`summary: ${barsFile}: the page audit has no number "${key}"`);
      process.exit(2);
    }
    const over = own.filter(([, cell]) => cell[key] > bar).map(([cell, values]) => ({ cell, value: values[key] }));
    (auditPageBars[path] ??= {})[key] = { bar, cells: own.length, cells_over: over.length, worst: max(own.map(([, cell]) => cell[key])), over };
    console.log(`summary: bar ${key} <= ${bar} on ${path}: ${over.length === 0 ? 'held' : 'missed'}, ${over.length} of ${own.length} cells over it, worst ${auditPageBars[path][key].worst}`);
  }
}
const hasPageBars = Object.keys(pageBars).length > 0;

// The layout shift after ready over repeated runs of the page checks, the first run included.
const repeatDir = join(dir, 'repeat');
const repeatFiles = (() => {
  try {
    return readdirSync(repeatDir).filter((f) => f.endsWith('.evidence.json')).sort();
  } catch {
    return [];
  }
})();
const shiftsOf = (doc) => Object.fromEntries(doc.evidence.results.filter((r) => r.output === 'layout_shift.after_ready@1').map((r) => [r.item, Number(r.value)]));
let repeat = null;
if (repeatFiles.length > 0) {
  const runsOfShift = [shiftsOf({ evidence }), ...repeatFiles.map((f) => shiftsOf(json(join(repeatDir, f))))];
  const perPage = {};
  for (const run of runsOfShift) for (const [page, value] of Object.entries(run)) (perPage[page] ??= []).push(value);
  const spreads = Object.fromEntries(Object.entries(perPage).map(([page, values]) => [page, Math.round((Math.max(...values) - Math.min(...values)) * 10000) / 10000]));
  repeat = {
    runs: runsOfShift.length,
    max_of_runs: max(Object.values(perPage).flat()),
    spread: max(Object.values(spreads)),
    pages: Object.fromEntries(Object.entries(perPage).filter(([, values]) => values.some((v) => v !== 0)).map(([page, values]) => [page, { values, spread: spreads[page] }])),
  };
  console.log(`summary: layout shift after ready over ${repeat.runs} runs: largest ${repeat.max_of_runs}, largest spread of a page ${repeat.spread}`);
}

const runs = list('trial');
const scenarios = {};
for (const run of runs) {
  const scenario = (scenarios[run.scenario] ??= { card: run.card, source: run.source, runs: {} });
  scenario.runs[`${run.viewport}.${run.mode}`] = {
    result: run.completed ? 'completed' : run.failed ? 'failed' : 'expectation missed',
    failed: run.failed,
    actions: run.actions,
    looks: run.looks,
    expectations: run.expectations,
    ...(run.notices ? { notices_missed: run.notices.missed } : {}),
    missed: run.steps.filter((s) => (s.expect || s.absent) && !s.met).map((s) => ({ step: s.step, expect: s.expect ?? null, absent: s.absent ?? null, found: s.found ?? null, text: s.text ?? null, rows: s.rows ?? null })),
    moments: run.moments.map((m) => ({ moment: m.moment, after_step: m.after_step, shot: m.shot, url: m.url, notices: m.notices.map((n) => n.text), dialogs: m.dialogs.map((d) => ({ label: d.label, focus_inside: d.focus_inside })), focus: m.focus, scroll_y: m.scroll.y })),
  };
}
const names = Object.keys(scenarios);
// The expectation on notices, where the cards hold one: the runs that showed a notice missing it.
const noticed = runs.filter((r) => r.notices);
const missedNotices = {};
for (const run of noticed) for (const text of run.notices.missed) missedNotices[text] = (missedNotices[text] ?? 0) + 1;
const trialNotices = noticed.length === 0 ? null : {
  not_text: noticed[0].notices.not_text,
  runs: noticed.length,
  runs_missed: noticed.filter((r) => r.notices.missed.length > 0).length,
  scenarios_missed: [...new Set(noticed.filter((r) => r.notices.missed.length > 0).map((r) => r.scenario))],
  missed: missedNotices,
};
if (trialNotices) console.log(`summary: no notice contains "${trialNotices.not_text}": ${trialNotices.runs_missed === 0 ? 'held' : 'missed'}, in ${trialNotices.runs_missed} of ${trialNotices.runs} runs`);
const notCompleted = names.filter((name) => Object.values(scenarios[name].runs).some((r) => r.result !== 'completed'));

const summary = {
  about: `Round ${round} on the studio: the numbers of ess/ui/rounds/round.sh, written by ess/ui/rounds/summary.mjs. Lower is better for every metric.`,
  round,
  versions,
  measured: {
    pages: Object.keys(pages),
    page_checks: { viewports: evidence.conditions.other.viewports.split(' '), modes: evidence.conditions.other.modes.split(' '), coverage: evidence.coverage, completion: evidence.completion },
    trial_and_audit: { viewports: [...new Set(runs.map((r) => r.viewport))], modes: [...new Set(runs.map((r) => r.mode))], scenarios: names.length, runs: runs.length, audits: audits.length },
  },
  metrics: {
    'page_checks.page_scroll.total': outputs['layout.page_scroll@1']?.total ?? null,
    'page_checks.overflow.blocks': outputs['layout.overflow@1']?.total ?? null,
    'page_checks.always_visible.misses': outputs['always_visible.misses@1']?.total ?? null,
    'page_checks.keyboard.misses': outputs['keyboard.misses@1']?.total ?? null,
    'page_checks.layout_shift.max': outputs['layout_shift.after_ready@1']?.max ?? null,
    ...(repeat ? { 'page_checks.layout_shift.max_of_runs': repeat.max_of_runs, 'page_checks.layout_shift.spread': repeat.spread } : {}),
    'audit.pages_with_document_scroll': pagesWhere((c) => c.document_scrolls).length,
    'audit.pages_with_scrollable_element': pagesWhere((c) => c.scrollable_elements > 0).length,
    'audit.pages_without_h1': pagesWhere((c) => c.h1 === 0).length,
    'audit.text_under_14px.share': Math.round((sum(cells.map((c) => c.text_under_14px)) / sum(cells.map((c) => c.text_runs))) * 1000) / 1000,
    'audit.text_low_contrast.runs_per_page.max': max(cells.map((c) => c.text_low_contrast)),
    'audit.controls_unnamed.max': max(cells.map((c) => c.controls_unnamed)),
    'audit.inputs_placeholder_only.pages': pagesWhere((c) => c.inputs_placeholder_only > 0).length,
    'audit.tab_stops_unmarked.max': max(cells.map((c) => c.tab_stops_unmarked)),
    'audit.pages_tab_starts_outside_navigation': pagesWhere((c) => c.tab_first_region !== 'nav').length,
    ...(bars ? { 'audit.cells_over_a_bar': sum(Object.values(auditBars).map((b) => b.cells_over)) } : {}),
    ...(hasFloors ? { 'audit.cells_under_a_floor': sum(Object.values(auditFloors).map((f) => f.cells_under)) } : {}),
    ...(hasPageBars ? { 'audit.cells_over_a_page_bar': sum(Object.values(auditPageBars).flatMap((keys) => Object.values(keys).map((b) => b.cells_over))) } : {}),
    'trial.scenarios': names.length,
    'trial.scenarios_not_completed': notCompleted.length,
    'trial.runs_failed': runs.filter((r) => r.failed).length,
    'trial.runs_expectation_missed': runs.filter((r) => !r.failed && !r.completed).length,
    ...(trialNotices ? { 'trial.runs_notice_expectation_missed': trialNotices.runs_missed } : {}),
    'trial.looks.total': sum(runs.map((r) => r.looks)),
    'trial.actions.max': max(runs.map((r) => r.actions)),
  },
  page_checks: outputs,
  page_check_misses: misses,
  ...(repeat ? { page_checks_repeat: repeat } : {}),
  ...(bars ? { audit_bars: auditBars } : {}),
  ...(hasFloors ? { audit_floors: auditFloors } : {}),
  ...(hasPageBars ? { audit_page_bars: auditPageBars } : {}),
  ...(trialNotices ? { trial_notices: trialNotices } : {}),
  audit: pages,
  trial: scenarios,
};
writeFileSync(target, JSON.stringify(summary, null, 1) + '\n');
console.log(`summary: ${round}: ${names.length - notCompleted.length} of ${names.length} scenarios completed in every run, written to ${target}`);
