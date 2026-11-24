// lift.mjs: the documentation landing page as sources and as an ess-ui/1 document.
//
//   materialize(base, space, choice) -> { product, css, patch }   a variant of the SOURCES:
//       a patched b10x-product-landing/1 object and the values of landing.css's custom properties
//   liftDocument(product, status)    -> an ess-ui/1 document of the whole landing page (hero and
//       every section), with stub pages for every link target; stage 1 checks and draws it
//   variants(space)                  -> every combination of the axes, in a fixed order
//   digest(variant)                  -> "sha256:<12 hex>" of the patched sources
//
// CLI: node lift.mjs <product.json> <status.json> [--out landing.ui.json]
// writes the lifted document of the current sources.
//
// The lift extends tools/scaffold/lift/docs-hero: the hero keeps its node names (kicker, lede,
// actions, meta, kpis) and uses the `text` and `link` primitives that docs-hero lacked.

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export function variants(space) {
  let acc = [{}];
  for (const axis of Object.keys(space.axes)) {
    const next = [];
    for (const c of acc) for (const v of Object.keys(space.axes[axis].values)) next.push({ ...c, [axis]: v });
    acc = next;
  }
  return acc;
}

export function baseChoice(space) {
  return Object.fromEntries(Object.keys(space.axes).map((a) => [a, Object.keys(space.axes[a].values)[0]]));
}

export function materialize(base, space, choice) {
  const product = structuredClone(base);
  const patch = [];
  const ax = (name) => space.axes[name].values[choice[name]];
  if (choice.actions) {
    const byHref = new Map(base.product.actions.map((a) => [a.href, a]));
    const list = ax('actions').map((h) => byHref.get(h)).filter(Boolean);
    if (JSON.stringify(list) !== JSON.stringify(base.product.actions)) patch.push({ op: 'replace', path: '/product/actions', value: list });
    product.product.actions = list;
  }
  if (choice.lede) {
    const lede = ax('lede');
    if (lede) {
      product.product.lede = lede;
      patch.push({ op: 'replace', path: '/product/lede', value: lede });
    }
  }
  if (choice.kpis) {
    const k = ax('kpis');
    if (JSON.stringify(k) !== JSON.stringify(base.product.kpis)) patch.push({ op: 'replace', path: '/product/kpis', value: k });
    product.product.kpis = k;
  }
  if (choice.features) {
    const n = ax('features');
    const f = product.sections.find((s) => s.kind === 'features');
    if (f && f.items.length !== n) {
      f.items = f.items.slice(0, n);
      patch.push({ op: 'replace', path: `/sections/${base.sections.findIndex((s) => s.kind === 'features')}/items`, value: `first ${n}` });
    }
  }
  if (choice.section_order) {
    const order = ax('section_order');
    const byKind = new Map(product.sections.map((s) => [s.kind, s]));
    const sorted = order.map((k) => byKind.get(k)).filter(Boolean);
    for (const s of product.sections) if (!order.includes(s.kind)) sorted.push(s);
    if (sorted.map((s) => s.kind).join() !== base.sections.map((s) => s.kind).join()) patch.push({ op: 'move', path: '/sections', value: sorted.map((s) => s.kind) });
    product.sections = sorted;
  }
  const css = { density: choice.density ? ax('density') : 1, heading_scale: choice.heading_scale ? ax('heading_scale') : 1 };
  if (css.density !== 1) patch.push({ op: 'css', path: '--uxui-density', value: css.density });
  if (css.heading_scale !== 1) patch.push({ op: 'css', path: '--uxui-heading-scale', value: css.heading_scale });
  return { product, css, patch };
}

export function digest(v) {
  const h = createHash('sha256').update(JSON.stringify({ product: v.product, css: v.css })).digest('hex');
  return 'sha256:' + h.slice(0, 12);
}

// The content of a landing: one id per section and per card (a KPI tile, a feature card, a flow
// step, a status item, a related tool). Hero actions are navigation, not content. The same ids
// are read from the rendered page by landing-page.js `content()`.
export function contentOf(product, status) {
  const ids = [];
  for (const k of product.product.kpis || []) ids.push('kpi:' + (typeof k === 'string' ? k : k.count));
  for (const s of product.sections) {
    ids.push('section:' + s.kind);
    if (s.kind === 'features') for (const f of s.items) ids.push('feature:' + f.title);
    if (s.kind === 'flow') for (const st of s.steps) ids.push('step:' + st.title);
    if (s.kind === 'status') for (const i of status?.items || []) ids.push('status:' + i.label);
    if (s.kind === 'related') for (const t of s.tools || []) ids.push('related:' + (t.id || t.name));
  }
  return ids;
}

// href -> page name of the lifted document
export const pageOf = (href) => 'docs.' + (href.replace(/^\/docs\/?/, '').replace(/[^a-z0-9]+/gi, '_').replace(/^_|_$/g, '') || 'index');
const nodeName = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '').slice(0, 40) || 'item';

export function liftDocument(product, status) {
  const p = product.product;
  const stubs = new Map();
  const stub = (href, label) => {
    const name = pageOf(href);
    if (!stubs.has(name)) stubs.set(name, { href, label });
    return name;
  };
  const text = (name, t) => ({ name, primitive: 'text', text: t });
  const link = (name, t, href) => ({ name, primitive: 'link', text: t, to: { to: stub(href, t) } });
  const statusItems = typeof status === 'object' && status ? status.items : [];
  const counts = { shipped: 0, decided: 0, planned: 0 };
  for (const i of statusItems) counts[i.status]++;
  const sections = [{ name: 'body', remove: true }];
  sections.push({ name: 'kicker', component: 'record', children: [text('line', p.eyebrow)] });
  sections.push({ name: 'lede', component: 'record', children: [text('line', p.lede)] });
  sections.push({
    name: 'actions',
    component: 'record',
    actions: p.actions.map((a) => ({
      name: nodeName(a.label),
      label: a.label,
      navigate: { to: stub(a.href, a.label) },
      ...(a.kind === 'primary' ? {} : { as: 'link' }),
    })),
  });
  if (p.meta) sections.push({ name: 'meta', component: 'record', children: [text('line', p.meta)] });
  if ((p.kpis || []).length) {
    sections.push({
      name: 'kpis',
      component: 'record',
      children: p.kpis.map((k, i) => text(`kpi_${k}`, `${String(i + 1).padStart(2, '0')} ${k} ${counts[k] ?? ''}`)),
    });
  }
  product.sections.forEach((s, i) => {
    const kicker = `${String(i + 1).padStart(2, '0')} / ${s.eyebrow}`;
    const head = [text('kicker', kicker), ...(s.lede ? [text('lede', s.lede)] : [])];
    let body = [];
    if (s.kind === 'features') {
      s.items.forEach((f, j) => {
        body.push(f.href ? link(`f${j + 1}`, f.title, f.href) : text(`f${j + 1}`, f.title));
        body.push(text(`f${j + 1}_status`, f.status || ''));
        body.push(text(`f${j + 1}_text`, f.description));
      });
    } else if (s.kind === 'flow') {
      s.steps.forEach((st, j) => {
        body.push(text(`step${j + 1}`, `${st.label ? st.label + ': ' : ''}${st.title}`));
        if (st.description) body.push(text(`step${j + 1}_text`, st.description));
      });
    } else if (s.kind === 'status') {
      body.push(text('summary', `${counts.shipped} shipped, ${counts.decided} decided, ${counts.planned} planned`));
      for (const st of ['shipped', 'decided', 'planned']) {
        body.push(text(`group_${st}`, st));
        statusItems.filter((x) => x.status === st).forEach((x, j) => {
          body.push(x.href ? link(`${st}${j + 1}`, x.label, x.href) : text(`${st}${j + 1}`, x.label));
          if (x.detail) body.push(text(`${st}${j + 1}_text`, x.detail));
        });
      }
    } else if (s.kind === 'related') {
      for (const t of s.tools || []) body.push(text(nodeName(t.id || t.name), `${t.id || t.name}: ${t.relation || t.description || ''}`));
    }
    sections.push({ name: `s_${s.kind}`, title: s.title, component: 'record', children: [...head, ...body] });
  });
  const pages = {
    landing: { kind: 'static_page', title: [...p.promise].join(' '), sections },
  };
  for (const [name, { label }] of stubs) {
    pages[name] = {
      kind: 'static_page',
      title: label,
      sections: [{ name: 'body', remove: true }, { name: 'text', component: 'record', children: [text('line', label)] }],
    };
  }
  return {
    format: 'ess-ui/1',
    app: 'site',
    title: 'uxui documentation',
    model: 'uxui.site',
    actor: 'anonymous',
    placement_profile: 'thin',
    shells: { app: { regions: { main: { kind: 'page_outlet' } } } },
    navigation: { home: 'landing', sections: [{ name: 'site', label: 'Site', pages: ['landing'] }], hidden: [...stubs.keys()] },
    pages,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [productFile, statusFile] = process.argv.slice(2);
  const out = process.argv.includes('--out') ? process.argv[process.argv.indexOf('--out') + 1] : null;
  const product = JSON.parse(readFileSync(productFile, 'utf8'));
  const status = JSON.parse(readFileSync(statusFile, 'utf8'));
  const doc = JSON.stringify(liftDocument(product, status), null, 1);
  if (out) writeFileSync(out, doc + '\n');
  else console.log(doc);
}
