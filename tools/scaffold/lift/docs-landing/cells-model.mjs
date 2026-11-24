// cells-model.mjs: stage 1 of the landing campaign, the measurement over uxui-cells' output.
//
// uxui-cells draws a lifted document on ESS's terminal renderer: one row per text node (a longer
// text is cut), one column per character, no type scale and no spacing tokens. reflow() is uxui's
// own layout model on top of that drawing: it keeps the renderer's order and nesting and gives each
// row a height in body lines from facts known before rendering:
//   wrap     a text node takes as many lines as greedy word wrap needs at its region's width
//   fonts    a text of font size f rem has width / f characters per line and f x its line height /
//            the body line height rows per line (sizes from space.json `type`, the theme's values)
//   heading  the page title and the section titles are scaled by the variant's heading scale
//   density  the renderer's border rows stand for spacing and are scaled by the variant's density
// With every input off, reflow() is the identity: a row is one row, a text is one line.
//
// measure() reads the KPIs of stage 1 from a reflowed drawing; campaign.mjs and agreement.mjs both
// call it, so the campaign and the offline agreement check share one model.

export const INPUTS = ['wrap', 'fonts', 'heading', 'density'];
export const ALL = Object.fromEntries(INPUTS.map((k) => [k, true]));
export const SIZES = [
  { name: 'cells-120x40', w: 120, view: 40, wide: true },
  { name: 'cells-48x50', w: 48, view: 50, wide: false },
];
export const DRAW_H = 600;

export const CONTENT = 'content.items_kept';
export const KPI_KEYS = ['load.words_before_action', 'attention.primary_top_px', 'effort.actions_per_journey', 'attention.regions_crossed', 'a11y.tap_targets_under_44px', CONTENT];
// Higher is better for the content KPI, lower for the others.
export const HIGHER = new Set([CONTENT]);

export const words = (s) => (s.match(/[\p{L}\p{N}][\p{L}\p{N}'’.-]*/gu) || []).length;

// Greedy word wrap: the number of lines `text` takes at `cols` characters per line.
export function lines(text, cols) {
  const c = Math.max(1, Math.floor(cols));
  let n = 1, used = 0;
  for (const w of String(text).split(/\s+/).filter(Boolean)) {
    let len = w.length;
    if (used && used + 1 + len <= c) {
      used += 1 + len;
      continue;
    }
    if (used) n++;
    while (len > c) {
      n++;
      len -= c;
    }
    used = len;
  }
  return n;
}

const SECTION = /^pages\/landing\/sections\/([^/]+)$/;
const CHILD = /^pages\/landing\/sections\/([^/]+)\/children\/([^/]+)$/;

function fontOf(type, section, child) {
  const nodes = type.nodes || {};
  return nodes[`${section}/${child}`] || (section.startsWith('s_') && nodes[`s_*/${child}`]) || type.body;
}
const remOf = (f, size) => (typeof f.rem === 'object' ? f.rem[size.wide ? 'wide' : 'narrow'] : f.rem);

// Rows (in body lines) a text takes at `cols` columns in font `f`, scaled by `scale`.
function textRows(text, cols, f, size, type, on, scale = 1) {
  const rem = (on.fonts ? remOf(f, size) : 1) * scale;
  const n = on.wrap ? lines(text, cols / rem) : 1;
  return on.fonts ? (n * rem * f.line_height) / type.body.line_height : n * rem;
}

// A reflowed drawing: `y(row)` maps a row of the terminal drawing to its top in body lines, and
// `texts` lists every text node with its reflowed top and its full text.
export function reflow(cell, doc, size, css, type, on = ALL) {
  const page = doc.pages.landing;
  const regions = cell.regions || [];
  const n = Math.max(2, ...regions.map((r) => r.y + r.h)) + 1;
  const height = new Array(n).fill(1);
  const hs = on.heading ? css.heading_scale : 1;
  const gap = on.density ? css.density : 1;
  // row 1: the page title, a heading over the whole width
  height[1] = textRows(page.title, size.w, type.title, size, type, on, hs);
  const sections = new Map(page.sections.map((s) => [s.name, s]));
  for (const r of regions) {
    const m = SECTION.exec(r.path);
    if (m && r.h >= 2) {
      const s = sections.get(m[1]);
      const title = s?.title ? textRows(s.title, r.w - 4, type.section_title, size, type, on, hs) - 1 : 0;
      height[r.y] = gap + title;
      height[r.y + r.h - 1] = gap;
      continue;
    }
    const c = CHILD.exec(r.path);
    if (c && r.h === 1) {
      const node = sections.get(c[1])?.children?.find((x) => x.name === c[2]);
      if (node?.text != null) height[r.y] = textRows(node.text, r.w, fontOf(type, c[1], c[2]), size, type, on);
    }
  }
  const top = [0];
  for (let i = 0; i < n; i++) top.push(top[i] + height[i]);
  const y = (row) => (row == null ? null : top[Math.min(Math.max(0, row), n)]);
  const texts = [{ path: 'pages/landing', y: y(1), text: page.title }];
  for (const r of regions) {
    const c = CHILD.exec(r.path);
    const s = c && sections.get(c[1]);
    const node = s?.children?.find((x) => x.name === c[2]);
    if (node?.text != null) texts.push({ path: r.path, y: y(r.y), text: node.text });
    const m = SECTION.exec(r.path);
    if (m && sections.get(m[1])?.title) texts.push({ path: r.path, y: y(r.y), text: sections.get(m[1]).title });
  }
  return { y, texts, rows: top[n] };
}

// Links of a lifted document to a page: [{label, path, section}] from hero actions and link primitives.
export function linksTo(doc, page) {
  const out = [];
  for (const s of doc.pages.landing.sections) {
    for (const a of s.actions || []) if (a.navigate?.to === page) out.push({ label: a.label, path: `pages/landing/sections/${s.name}/actions/${a.name}`, section: s.name });
    for (const c of s.children || []) if (c.primitive === 'link' && c.to?.to === page) out.push({ label: c.text, path: `pages/landing/sections/${s.name}/children/${c.name}`, section: s.name });
  }
  return out;
}

// Fixed routes outside the lifted page, the same model as stage 2's navbar.
export function navbarRoute(space, target, wide) {
  const direct = space.navbar.links.includes(target);
  const viaNav = (space.journeys.find((j) => j.targets.includes(target))?.via || []).some((v) => space.navbar.links.includes(v));
  const menu = wide ? 0 : space.navbar.narrow_menu_actions;
  if (direct) return { cost: 1 + menu, regions: 1, where: 'navbar' };
  if (viaNav) return { cost: 2 + menu + (wide ? 0 : space.navbar.narrow_sidebar_actions), regions: 2, where: 'navbar via' };
  return null;
}

// The KPIs of one drawing (one size). `on` selects the inputs of reflow(); `pageOf` maps an href
// to a page of the lifted document. Rows are body lines after reflow (fractional).
export function measure({ space, doc, cell, size, css, on = ALL, pageOf }) {
  const body = size.view - 2;
  const screen = cell.screen || [];
  const t0 = performance.now();
  const flow = reflow(cell, doc, size, css, space.type, on);
  const rowOf = (label) => {
    const needle = label.slice(0, Math.min(label.length, size.w - 8, 18));
    const i = screen.findIndex((l) => l.includes(needle));
    return i < 0 ? null : i;
  };
  const reg = (p) => (cell.regions || []).find((x) => x.path === p);
  const sectionTop = (name) => flow.y(reg(`pages/landing/sections/${name}`)?.y ?? null);
  const primaryAction = doc.pages.landing.sections.find((s) => s.name === 'actions').actions[0];
  const pr = reg(`pages/landing/sections/actions/actions/${primaryAction.name}`);
  const primaryCell = pr ? pr.y : rowOf(primaryAction.label);
  const primaryRow = flow.y(primaryCell);
  const regionTops = ['hero', ...doc.pages.landing.sections.filter((s) => s.name.startsWith('s_')).map((s) => s.name)].map((n) =>
    n === 'hero' ? 0 : sectionTop(n),
  );
  const journeys = {};
  for (const j of space.journeys) {
    let best = navbarRoute(space, j.targets[0], size.wide);
    for (const l of j.targets.flatMap((t) => linksTo(doc, pageOf(t)))) {
      const p = reg(l.path);
      const row = flow.y(p ? p.y : rowOf(l.label));
      if (row === null) continue;
      const cost = 1 + Math.floor(row / body);
      const regions = 1 + regionTops.filter((y) => y !== null && y <= row).length;
      if (!best || cost < best.cost || (cost === best.cost && regions < best.regions)) best = { cost, regions, where: l.section };
    }
    journeys[j.name] = best;
  }
  // Words before the primary action: with wrapping every text is drawn whole, so the words of each
  // text node above it; without, the words the terminal drew on the rows above it.
  const wordsBefore =
    primaryRow === null
      ? null
      : on.wrap
        ? flow.texts.filter((t) => t.y < primaryRow).reduce((n, t) => n + words(t.text), 0)
        : screen.slice(1, primaryCell).reduce((n, l) => n + words(l), 0);
  const modelUs = Math.round((performance.now() - t0) * 1000);
  return {
    primary_row: primaryRow === null ? null : +primaryRow.toFixed(2),
    primary_in_view: primaryRow !== null && primaryRow + 1 <= body,
    words_before_action: wordsBefore,
    actions_total: Object.values(journeys).reduce((n, x) => n + (x?.cost ?? 99), 0),
    regions_total: Object.values(journeys).reduce((n, x) => n + (x?.regions ?? 99), 0),
    journeys,
    check_errors: (cell.check_errors || []).length,
    error: cell.error || null,
    us: (cell.check_us || 0) + (cell.open_us || 0) + (cell.draw_us || 0),
    model_us: modelUs,
  };
}

// Static reading, used for tap targets and when no cell reader is given.
export function staticMeasure(doc) {
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

// Hard rules of stage 1 over the measures of every size.
export function fails1(m, space, contentMissing) {
  const fails = [];
  if (m.some((x) => x.check_errors || x.error)) fails.push('ess_ui_check');
  if (m.some((x) => !x.primary_in_view)) fails.push('primary_in_view');
  for (const j of space.journeys) if (m.some((x) => !x.journeys[j.name])) fails.push('journey:' + j.name);
  if (contentMissing.length) fails.push('content_missing');
  return fails;
}

// Stage-1 KPIs of a variant from its measures at every size: the worst value over the sizes.
export function k1Of(m, stat, kept) {
  return m.length
    ? {
        'load.words_before_action': Math.max(...m.map((x) => x.words_before_action ?? 999)),
        'attention.primary_top_px': Math.max(...m.map((x) => x.primary_row ?? 99)),
        'effort.actions_per_journey': Math.max(...m.map((x) => x.actions_total)),
        'attention.regions_crossed': Math.max(...m.map((x) => x.regions_total)),
        'a11y.tap_targets_under_44px': stat.small_targets,
        [CONTENT]: kept,
      }
    : { 'load.words_before_action': stat.words_before_action, 'a11y.tap_targets_under_44px': stat.small_targets, [CONTENT]: kept };
}

// The ranking score: the mean over the KPIs of value / value of the current page (higher-is-better
// KPIs: current / value), plus 1000 per failed hard rule. Lower is better.
export function score(k, b, fails) {
  const ks = KPI_KEYS.filter((x) => k && b && x in k && x in b && b[x]);
  const s = ks.reduce((n, x) => n + (HIGHER.has(x) ? (k[x] ? b[x] / k[x] : 10) : k[x] / b[x]), 0) / (ks.length || 1);
  return fails * 1000 + s;
}

export function kendall(xs, ys) {
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
