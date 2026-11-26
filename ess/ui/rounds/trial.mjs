// trial.mjs <origin> <scenarios file> <out dir> --pages <path,path,...> [--viewports 1366x768,1280x720] [--modes light,dark]
// The trial of a round without a person (plugin/skills/ui-iteration-loop, § 3.1 and § 7): plays each
// scripted scenario of the scenarios file in headless Chromium, takes a screenshot 0.75 s after each
// moment and records what the screen shows there; then audits each page of --pages once per
// viewport and mode (scroll, text size and contrast, names of controls, the Tab order).
// Deterministic code, no model. Every step names its node by `data-ui-path` or a CSS selector, not
// by role and name, and a control outside the viewport is scrolled to and counted as a look: this
// is a driver, not the screen-only user of § 7.
// Writes <out dir>/trial/<scenario>.<viewport>.<mode>.json, <out dir>/shots/<scenario>.<n>-<moment>.<viewport>.<mode>.png
// and <out dir>/audit/<page>.<viewport>.<mode>.json. Exit 0 when every run was played to its end
// (a failed step is recorded, not fatal), 3 when the browser failed.

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { launch, parseArgs } from '../../../tools/scaffold/cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
const [origin, scenariosFile, out] = pos;
if (!origin || !scenariosFile || !out || typeof opt.pages !== 'string') {
  console.error('usage: trial.mjs <origin> <scenarios file> <out dir> --pages <path,path,...> [--viewports WxH,WxH] [--modes light,dark]');
  process.exit(2);
}
const viewports = (opt.viewports || '1366x768,1280x720').split(',');
const modes = (opt.modes || 'light,dark').split(',');
const spec = JSON.parse(readFileSync(scenariosFile, 'utf8'));
const MOMENT_MS = 750;
for (const dir of ['trial', 'shots', 'audit']) mkdirSync(join(out, dir), { recursive: true });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const css = (step) => step.css ?? `[data-ui-path="${step.path}"]`;

// What the page answers; each runs in the page and returns JSON.
const IN_PAGE = `(() => {
  const text = (el) => (el.innerText || el.textContent || '').replace(/\\s+/g, ' ').trim();
  const box = (el) => { const r = el.getBoundingClientRect(); return { x: Math.round(r.x), y: Math.round(r.y), width: Math.round(r.width), height: Math.round(r.height) }; };
  const drawn = (el) => { const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0 && el.checkVisibility({ visibilityProperty: true }); };
  const inView = (el) => { const r = el.getBoundingClientRect(); return r.top >= -0.5 && r.left >= -0.5 && r.bottom <= innerHeight + 0.5 && r.right <= innerWidth + 0.5; };
  const first = (selector) => [...document.querySelectorAll(selector)].find(drawn) ?? null;
  const describe = (el) => {
    if (!el || el === document.body) return 'body';
    const path = el.closest('[data-ui-path]')?.getAttribute('data-ui-path');
    return el.tagName.toLowerCase() + (el.getAttribute('aria-label') ? ' "' + el.getAttribute('aria-label') + '"' : text(el) ? ' "' + text(el).slice(0, 40) + '"' : '') + (path ? ' in ' + path : '');
  };
  const name = (el) => {
    const labelled = (el.getAttribute('aria-labelledby') || '').split(' ').map((id) => document.getElementById(id)).filter(Boolean).map(text).join(' ');
    return el.getAttribute('aria-label') || labelled || (el.labels && [...el.labels].map(text).join(' ')) || (el.closest('label') ? text(el.closest('label')) : '') || text(el) || el.getAttribute('title') || '';
  };
  const parse = (c) => { const m = c.match(/rgba?\\(([^)]+)\\)/); if (!m) return null; const p = m[1].split(/[ ,\\/]+/).map(Number); return { r: p[0], g: p[1], b: p[2], a: p.length > 3 ? p[3] : 1 }; };
  const over = (top, under) => ({ r: top.r * top.a + under.r * (1 - top.a), g: top.g * top.a + under.g * (1 - top.a), b: top.b * top.a + under.b * (1 - top.a), a: 1 });
  const background = (el) => {
    const layers = [];
    for (let e = el; e; e = e.parentElement) {
      const style = getComputedStyle(e);
      if (style.backgroundImage !== 'none') layers.push(null);
      const c = parse(style.backgroundColor);
      if (c && c.a > 0) layers.push(c);
      if (c && c.a === 1) break;
    }
    let colour = { r: 255, g: 255, b: 255, a: 1 };
    let image = false;
    for (const layer of layers.reverse()) { if (layer === null) image = true; else colour = over(layer, colour); }
    return { colour, image };
  };
  const luminance = (c) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x); return (hi + 0.05) / (lo + 0.05); };
  return {
    target(selector) {
      const el = first(selector);
      if (!el) return { found: false, matches: document.querySelectorAll(selector).length };
      const b = box(el);
      const centre = document.elementFromPoint(b.x + b.width / 2, b.y + b.height / 2);
      return { found: true, text: text(el).slice(0, 200), box: b, in_view: inView(el), covered: inView(el) && !(centre === el || el.contains(centre)), disabled: el.disabled === true };
    },
    reveal(selector) { first(selector)?.scrollIntoView({ block: 'center', inline: 'nearest' }); return true; },
    select(selector, value) {
      const el = first(selector);
      if (!el) return false;
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(el, value);
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return el.value === value;
    },
    focusOn(selector) { const el = first(selector); const a = document.activeElement; return !!el && !!a && (a === el || el.contains(a)); },
    rows(selector) { return [...document.querySelectorAll(selector + ' tr[data-ui-path]')].map((r) => r.getAttribute('data-ui-path').split('/').pop()); },
    facts() {
      const a = document.activeElement;
      return {
        url: location.pathname + location.search,
        h1: [...document.querySelectorAll('h1')].filter(drawn).map(text),
        live: [...document.querySelectorAll('.ui-live')].map((e) => e.getAttribute('data-lifecycle')),
        notices: [...document.querySelectorAll('.ui-notifications li')].map((e) => ({ text: text(e), tone: (e.className.match(/ui-tone-(\\w+)/) || [])[1], in_view: inView(e), box: box(e) })),
        dialogs: [...document.querySelectorAll('[role=dialog]')].filter(drawn).map((e) => ({ label: e.getAttribute('aria-label'), path: e.getAttribute('data-ui-path'), box: box(e), focus_inside: e.contains(a) })),
        alerts: [...document.querySelectorAll('[role=alert], .ui-state-failed, .ui-form-error, .ui-field-error')].filter(drawn).map((e) => text(e).slice(0, 200)),
        states: [...document.querySelectorAll('.ui-state')].filter(drawn).map((e) => text(e).slice(0, 120)),
        focus: describe(a),
        scroll: { y: Math.round(scrollY), height: document.documentElement.scrollHeight, client: document.documentElement.clientHeight },
      };
    },
    audit() {
      const root = document.documentElement;
      const scrollers = [...document.querySelectorAll('body *')].filter((e) => drawn(e) && /(auto|scroll)/.test(getComputedStyle(e).overflowY + getComputedStyle(e).overflowX) && (e.scrollHeight > e.clientHeight + 1 || e.scrollWidth > e.clientWidth + 1)).map((e) => ({ node: describe(e), class: e.className, scroll_height: e.scrollHeight, client_height: e.clientHeight, scroll_width: e.scrollWidth, client_width: e.clientWidth }));
      const runs = [];
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        const el = n.parentElement;
        if (!n.nodeValue.trim() || !el || !drawn(el) || el.closest('script, style, [aria-hidden=true]')) continue;
        const style = getComputedStyle(el);
        const size = parseFloat(style.fontSize);
        const fg = parse(style.color);
        const bg = background(el);
        const colour = fg ? over(fg, bg.colour) : null;
        const large = size >= 24 || (size >= 18.66 && Number(style.fontWeight) >= 700);
        runs.push({ text: n.nodeValue.replace(/\\s+/g, ' ').trim().slice(0, 60), node: describe(el), class: el.className, size, weight: Number(style.fontWeight), large, colour: style.color, on: 'rgb(' + [bg.colour.r, bg.colour.g, bg.colour.b].map(Math.round).join(', ') + ')', on_image: bg.image, contrast: colour ? Math.round(contrast(colour, bg.colour) * 100) / 100 : null, disabled: el.closest(':disabled') !== null });
      }
      const controls = [...document.querySelectorAll('a[href], button, input, select, textarea, [role=button], [tabindex]')].filter(drawn);
      const sizes = {};
      for (const run of runs) sizes[run.size] = (sizes[run.size] || 0) + 1;
      const low = (r) => !r.disabled && !r.on_image && r.contrast !== null && r.contrast < (r.large ? 3 : 4.5);
      const key = (run) => run.class + '|' + run.size + '|' + run.colour + '|' + run.on;
      const once = (list) => { const seen = new Set(); return list.filter((run) => !seen.has(key(run)) && seen.add(key(run))); };
      return {
        url: location.pathname + location.search,
        document: { scroll_height: root.scrollHeight, client_height: root.clientHeight, scroll_width: root.scrollWidth, client_width: root.clientWidth },
        scrollers,
        headings: [...document.querySelectorAll('h1, h2, h3')].filter(drawn).map((e) => e.tagName.toLowerCase() + ' ' + text(e)),
        landmarks: [...document.querySelectorAll('main, nav, header, [role=navigation], [role=main], [role=banner]')].filter(drawn).map((e) => e.tagName.toLowerCase() + (e.getAttribute('aria-label') ? ' "' + e.getAttribute('aria-label') + '"' : '')),
        text: { runs: runs.length, sizes, under_14: runs.filter((r) => r.size < 14).length, under_14_examples: once(runs.filter((r) => r.size < 14)).slice(0, 12), low_contrast: runs.filter(low).length, low_contrast_examples: once(runs.filter(low)).slice(0, 12), unmeasured_contrast: once(runs.filter((r) => r.on_image)).map((r) => r.node), min_contrast: runs.filter((r) => !r.disabled && !r.on_image && r.contrast !== null).reduce((m, r) => Math.min(m, r.contrast), 99) },
        controls: { count: controls.length, unnamed: controls.filter((e) => !name(e)).map(describe), small: controls.filter((e) => { const r = e.getBoundingClientRect(); return r.width < 24 || r.height < 24; }).map((e) => describe(e) + ' ' + Math.round(e.getBoundingClientRect().width) + 'x' + Math.round(e.getBoundingClientRect().height)), placeholder_only: controls.filter((e) => /^(INPUT|TEXTAREA)$/.test(e.tagName) && !e.getAttribute('aria-label') && !(e.labels && e.labels.length) && !e.closest('label')).map((e) => describe(e) + ' placeholder "' + (e.getAttribute('placeholder') || '') + '"') },
        focus: describe(document.activeElement),
      };
    },
    focused() {
      const a = document.activeElement;
      if (!a || a === document.body) return null;
      const style = getComputedStyle(a);
      const marked = a.matches(':focus-visible') && ((style.outlineStyle !== 'none' && parseFloat(style.outlineWidth) > 0) || style.boxShadow !== 'none');
      return { node: describe(a), in_view: inView(a), marked, region: a.closest('main') ? 'main' : a.closest('nav.ui-nav') ? 'nav' : a.closest('[role=dialog]') ? 'dialog' : 'other' };
    },
  };
})()`;

const KEYS = {
  Tab: { key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 },
  Enter: { key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, text: '\r' },
  Space: { key: ' ', code: 'Space', windowsVirtualKeyCode: 32, text: ' ' },
  Escape: { key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 },
};

async function press(page, name) {
  const key = KEYS[name];
  await page.send('Input.dispatchKeyEvent', { type: key.text ? 'keyDown' : 'rawKeyDown', ...key });
  await page.send('Input.dispatchKeyEvent', { type: 'keyUp', key: key.key, code: key.code, windowsVirtualKeyCode: key.windowsVirtualKeyCode });
}

const ask = (page, call) => page.eval(`${IN_PAGE}.${call}`);
const arg = (value) => JSON.stringify(value);

async function waitFor(page, selector, timeoutMs) {
  const end = Date.now() + timeoutMs;
  for (;;) {
    const t = await ask(page, `target(${arg(selector)})`);
    if (t.found) return true;
    if (Date.now() > end) return false;
    await sleep(50);
  }
}

async function open(page, path, [width, height], mode) {
  await page.send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: mode }, { name: 'prefers-reduced-motion', value: 'reduce' }] });
  await page.send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await page.viewport(width, height);
  await page.goto(origin + path);
  await waitFor(page, spec.ready, 10000);
}

// Brings the target into view (a look when it was not) and clicks its centre.
async function click(page, selector, record) {
  let t = await ask(page, `target(${arg(selector)})`);
  Object.assign(record, { found: t.found, in_view: t.in_view ?? false, text: t.text });
  if (!t.found) return 'matches no drawn element';
  if (t.disabled) return 'the control is disabled';
  if (!t.in_view) {
    record.look = true;
    await ask(page, `reveal(${arg(selector)})`);
    await sleep(100);
    t = await ask(page, `target(${arg(selector)})`);
  }
  record.covered = t.covered;
  if (t.covered) return 'another element covers its centre';
  const x = t.box.x + t.box.width / 2;
  const y = t.box.y + t.box.height / 2;
  await page.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await page.send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
  await page.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
  return null;
}

async function play(browser, scenario, viewport, mode) {
  const page = await browser.newPage({ runtime: false });
  const size = viewport.split('x').map(Number);
  const run = { scenario: scenario.name, card: scenario.card, source: scenario.source, viewport, mode, steps: [], moments: [], actions: 0, looks: 0, failed: null };
  let shots = 0;
  try {
    for (const [index, step] of scenario.steps.entries()) {
      const record = { step: index + 1, ...step };
      let error = null;
      if (step.open) {
        await open(page, step.open, size, mode);
      } else if (step.click) {
        error = await click(page, css(step.click), record);
        if (!error) run.actions += 1;
      } else if (step.type) {
        error = await click(page, css(step.type), record);
        if (!error) {
          await page.send('Input.insertText', { text: step.type.text });
          run.actions += 2;
        }
      } else if (step.select) {
        const ok = await ask(page, `select(${arg(css(step.select))}, ${arg(step.select.value)})`);
        if (!ok) error = 'the choice has no such value';
        else run.actions += 2;
      } else if (step.press) {
        await press(page, step.press);
        run.actions += 1;
      } else if (step.tab_to) {
        const max = step.tab_to.max ?? 60;
        record.tabs = 0;
        record.stops = [];
        while (record.tabs < max && !(await ask(page, `focusOn(${arg(css(step.tab_to))})`))) {
          await press(page, 'Tab');
          record.tabs += 1;
          const stop = await ask(page, 'focused()');
          if (record.stops.length < 8) record.stops.push(stop ? stop.node : 'nothing');
        }
        record.reached = await ask(page, `focusOn(${arg(css(step.tab_to))})`);
        record.focused = await ask(page, 'focused()');
        run.actions += record.tabs;
        if (!record.reached) error = `Tab did not reach it in ${max} presses`;
      } else if (step.wait) {
        const reached = await waitFor(page, css(step.wait), step.wait.timeout_ms ?? 5000);
        if (!reached) error = `no drawn element matched within ${step.wait.timeout_ms ?? 5000} ms`;
      } else if (step.reveal) {
        const t = await ask(page, `target(${arg(css(step.reveal))})`);
        record.in_view = t.in_view ?? false;
        if (t.found && !t.in_view) {
          record.look = true;
          await ask(page, `reveal(${arg(css(step.reveal))})`);
        }
      } else if (step.expect) {
        const t = await ask(page, `target(${arg(css(step.expect))})`);
        record.found = t.found;
        record.text = t.text;
        record.in_view = t.in_view ?? false;
        record.met = t.found && (step.expect.text === undefined || t.text.includes(step.expect.text)) && (step.expect.not_text === undefined || !t.text.includes(step.expect.not_text));
        const rows = await ask(page, `rows(${arg(css(step.expect))})`);
        if (rows.length > 0) record.rows = rows;
        if (step.expect.rows) record.met = record.met && JSON.stringify(rows) === JSON.stringify(step.expect.rows);
      } else if (step.absent) {
        const t = await ask(page, `target(${arg(css(step.absent))})`);
        record.met = !t.found;
      } else if (step.settle) {
        await sleep(step.settle);
      } else if (step.moment) {
        await sleep(MOMENT_MS);
        shots += 1;
        const file = `${scenario.name}.${shots}-${step.moment}.${viewport}.${mode}.png`;
        await page.screenshot(join(out, 'shots', file));
        run.moments.push({ moment: step.moment, after_step: index, shot: `shots/${file}`, ...(await ask(page, 'facts()')) });
      }
      if (record.look) run.looks += 1;
      if (error) record.error = error;
      run.steps.push(record);
      if (error && !step.optional) {
        run.failed = { step: index + 1, error };
        shots += 1;
        const file = `${scenario.name}.${shots}-failed.${viewport}.${mode}.png`;
        await page.screenshot(join(out, 'shots', file));
        run.moments.push({ moment: 'failed', after_step: index + 1, shot: `shots/${file}`, ...(await ask(page, 'facts()')) });
        break;
      }
    }
  } finally {
    await browser.conn.send('Target.closeTarget', { targetId: page.targetId }).catch(() => {});
  }
  const expects = run.steps.filter((s) => s.expect || s.absent);
  run.expectations = { total: expects.length, met: expects.filter((s) => s.met).length };
  run.completed = run.failed === null && expects.every((s) => s.met);
  return run;
}

async function audit(browser, path, viewport, mode) {
  const page = await browser.newPage({ runtime: false });
  try {
    await open(page, path, viewport.split('x').map(Number), mode);
    await sleep(MOMENT_MS);
    const result = { path, viewport, mode, ...(await ask(page, 'audit()')) };
    // The Tab order from a fresh load: every stop until focus leaves the page or repeats.
    const stops = [];
    for (let i = 0; i < 80; i += 1) {
      await press(page, 'Tab');
      const stop = await ask(page, 'focused()');
      if (!stop || (stops.length > 0 && stop.node === stops[0].node)) break;
      stops.push(stop);
    }
    result.tab = { stops: stops.length, first_region: stops[0]?.region ?? null, order: [...new Set(stops.map((s) => s.region))], unmarked: stops.filter((s) => !s.marked).map((s) => s.node), first: stops.slice(0, 6).map((s) => s.node) };
    return result;
  } finally {
    await browser.conn.send('Target.closeTarget', { targetId: page.targetId }).catch(() => {});
  }
}

const fileName = (path) => path.replace(/^\//, '').replace(/\//g, '.').replace(/sha256%3A/g, '');

const browser = await launch({ profile: join(out, '..', `trial-profile-${process.pid}`) });
let code = 0;
try {
  for (const viewport of viewports) {
    for (const mode of modes) {
      for (const scenario of spec.scenarios) {
        const run = await play(browser, scenario, viewport, mode);
        writeFileSync(join(out, 'trial', `${scenario.name}.${viewport}.${mode}.json`), JSON.stringify(run, null, 1) + '\n');
        console.log(`trial: ${scenario.name} ${viewport} ${mode}: ${run.completed ? 'completed' : run.failed ? `failed at step ${run.failed.step}: ${run.failed.error}` : 'expectation missed'}, ${run.actions} actions, ${run.looks} looks`);
      }
      for (const path of opt.pages.split(',')) {
        const result = await audit(browser, path, viewport, mode);
        writeFileSync(join(out, 'audit', `${fileName(path)}.${viewport}.${mode}.json`), JSON.stringify(result, null, 1) + '\n');
      }
    }
  }
} catch (error) {
  console.error(`trial: ${error.stack ?? error}`);
  code = 3;
} finally {
  await browser.close();
}
process.exit(code);
