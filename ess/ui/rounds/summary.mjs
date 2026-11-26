// summary.mjs <round id> <evidence dir> <round.json to write>
// Reads what round.sh left in the evidence dir (versions.txt, page-checks.evidence.json, trial/,
// audit/) and writes the round's numbers as one file: the round.json of plugin/skills/
// ui-iteration-loop § 3.2. Deterministic code, no model, no browser. Every count is over the gating
// viewports and both modes unless its key says otherwise.
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [round, dir, target] = process.argv.slice(2);
if (!round || !dir || !target) {
  console.error('usage: summary.mjs <round id> <evidence dir> <round.json to write>');
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
    missed: run.steps.filter((s) => (s.expect || s.absent) && !s.met).map((s) => ({ step: s.step, expect: s.expect ?? null, absent: s.absent ?? null, found: s.found ?? null, text: s.text ?? null, rows: s.rows ?? null })),
    moments: run.moments.map((m) => ({ moment: m.moment, after_step: m.after_step, shot: m.shot, url: m.url, notices: m.notices.map((n) => n.text), dialogs: m.dialogs.map((d) => ({ label: d.label, focus_inside: d.focus_inside })), focus: m.focus, scroll_y: m.scroll.y })),
  };
}
const names = Object.keys(scenarios);
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
    'audit.pages_with_document_scroll': pagesWhere((c) => c.document_scrolls).length,
    'audit.pages_with_scrollable_element': pagesWhere((c) => c.scrollable_elements > 0).length,
    'audit.pages_without_h1': pagesWhere((c) => c.h1 === 0).length,
    'audit.text_under_14px.share': Math.round((sum(cells.map((c) => c.text_under_14px)) / sum(cells.map((c) => c.text_runs))) * 1000) / 1000,
    'audit.text_low_contrast.runs_per_page.max': max(cells.map((c) => c.text_low_contrast)),
    'audit.controls_unnamed.max': max(cells.map((c) => c.controls_unnamed)),
    'audit.inputs_placeholder_only.pages': pagesWhere((c) => c.inputs_placeholder_only > 0).length,
    'audit.tab_stops_unmarked.max': max(cells.map((c) => c.tab_stops_unmarked)),
    'audit.pages_tab_starts_outside_navigation': pagesWhere((c) => c.tab_first_region !== 'nav').length,
    'trial.scenarios': names.length,
    'trial.scenarios_not_completed': notCompleted.length,
    'trial.runs_failed': runs.filter((r) => r.failed).length,
    'trial.runs_expectation_missed': runs.filter((r) => !r.failed && !r.completed).length,
    'trial.looks.total': sum(runs.map((r) => r.looks)),
    'trial.actions.max': max(runs.map((r) => r.actions)),
  },
  page_checks: outputs,
  page_check_misses: misses,
  audit: pages,
  trial: scenarios,
};
writeFileSync(target, JSON.stringify(summary, null, 1) + '\n');
console.log(`summary: ${round}: ${names.length - notCompleted.length} of ${names.length} scenarios completed in every run, written to ${target}`);
