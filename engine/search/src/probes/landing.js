// The in-page probe of `ux-search landing`, evaluated once in the built landing page with the
// page's own scripts off (the static HTML a visitor's first paint shows). It derives from the
// spike's tools/scaffold/lift/docs-landing/landing-page.js and tools/scaffold/runtime.js.
//
//   __uxuiLanding.prepare(config, mode)  keeps the landing body as built and sets the colour
//                                        scheme as the page's inline script would
//   __uxuiLanding.render(spec)           rebuilds the body from the built one for a variant:
//                                        spec.actions (label, href, kind) and spec.sections
//                                        (kind, eyebrow) as product.json has them, and spec.css,
//                                        the text of landing.css; null gives the built page back
//   __uxuiLanding.measure(spec)          render(spec), then the numbers of one render
//
// The mapping mirrors the docs-system renderer: an action is keyed by its href and is a
// `b10x-button` when its kind is `primary`, else a `b10x-text-link`; a section is keyed by its
// kind, and its kicker is its number and eyebrow. An action the built page lacks is a copy of
// a built one with its label and href.
(() => {
  if (window.__uxuiLanding) return true;
  const STYLE_ID = 'uxui-landing-css';
  const PRIMARY = '.b10x-hero__actions > a.b10x-button';
  const SKIP = new Set(['SCRIPT', 'STYLE', 'LINK', 'META', 'NOSCRIPT', 'TEMPLATE', 'HEAD', 'TITLE', 'BR', 'WBR', 'SOURCE', 'TRACK']);
  const INTERACTIVE =
    'a[href],button,input:not([type=hidden]),select,textarea,summary,[role=button],[role=link],[role=tab],' +
    '[role=menuitem],[role=checkbox],[role=switch],[role=radio],[role=combobox],[tabindex]:not([tabindex="-1"])';
  let config = null;
  let built = null;
  let builtContent = [];

  const root = () => document.querySelector('.b10x-product');
  const clean = (s) => (s || '').replace(/\s+/g, ' ').trim();
  const words = (s) => (s.match(/[\p{L}\p{N}][\p{L}\p{N}'’.-]*/gu) || []).length;
  const norm = (path) => path.replace(/\/$/, '') || '/';
  // The site path of a link of this origin, without the base path: /uxui/docs/status -> /docs/status.
  const site = (href) => {
    let url;
    try {
      url = new URL(href, location.href);
    } catch {
      return null;
    }
    if (url.origin !== location.origin) return null;
    const base = norm(config.base_path);
    const path = url.pathname;
    if (path !== base && !path.startsWith(base + '/')) return null;
    return norm(path.slice(base.length));
  };
  function visible(el) {
    if (!el.isConnected) return false;
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true, opacityProperty: true, visibilityProperty: true })) return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  }

  // ---------- the variant ----------
  function reset() {
    const r = root();
    if (!r || !built) throw new Error('prepare() has not kept a landing body');
    r.replaceWith(built.cloneNode(true));
    document.getElementById(STYLE_ID)?.remove();
  }
  function setLabel(el, text) {
    for (const node of el.childNodes) {
      if (node.nodeType !== 3) continue;
      if (node.data !== text) node.data = text;
      return;
    }
    el.prepend(text);
  }
  function render(spec) {
    reset();
    if (!spec) return true;
    const r = root();
    const box = r.querySelector('.b10x-hero__actions');
    if (!box) throw new Error('the built page has no .b10x-hero__actions');
    const builtActions = [...box.querySelectorAll(':scope > a')];
    if (!builtActions.length) throw new Error('the built page has no hero action to copy');
    const byHref = new Map(builtActions.map((a) => [site(a.getAttribute('href')), a]));
    box.replaceChildren(
      ...spec.actions.map((action) => {
        let el = byHref.get(norm(action.href));
        if (el) byHref.delete(norm(action.href));
        else {
          el = builtActions[builtActions.length - 1].cloneNode(true);
          el.setAttribute('href', norm(config.base_path) + action.href);
        }
        const kind = action.kind === 'primary' ? 'b10x-button' : 'b10x-text-link';
        if (el.className !== kind) el.className = kind;
        setLabel(el, action.label);
        return el;
      }),
    );
    const byKind = new Map(
      [...r.querySelectorAll(':scope > section.b10x-product-section')].map((s) => [
        ([...s.classList].find((c) => c.startsWith('b10x-product-section--')) || '').slice('b10x-product-section--'.length),
        s,
      ]),
    );
    let after = r.querySelector(':scope > .b10x-hero');
    if (!after) throw new Error('the built page has no .b10x-hero');
    spec.sections.forEach((section, i) => {
      const el = byKind.get(section.kind);
      if (!el) throw new Error('the built page has no section ' + section.kind);
      const kicker = el.querySelector('.b10x-product-section__header .b10x-kicker');
      const text = [String(i + 1).padStart(2, '0'), section.eyebrow].filter(Boolean).join(' / ');
      if (kicker && kicker.textContent !== text) kicker.textContent = text;
      if (after.nextElementSibling !== el) after.after(el);
      after = el;
      byKind.delete(section.kind);
    });
    for (const el of byKind.values()) el.remove();
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = spec.css;
    document.head.appendChild(style);
    return true;
  }

  // ---------- content ----------
  // The sections and cards the page shows: KPI tiles, sections, feature cards, flow steps, status
  // items and related-tool cards, each present and visible.
  function content() {
    const r = root();
    if (!r) return [];
    const ids = [];
    const text = (el, sel) => clean((sel ? el.querySelector(sel) : el)?.textContent);
    const variantOf = (el, prefix) => ([...el.classList].find((c) => c.startsWith(prefix)) || '').slice(prefix.length);
    for (const el of r.querySelectorAll('.b10x-hero .b10x-kpi')) if (visible(el)) ids.push('kpi:' + variantOf(el, 'b10x-kpi--'));
    for (const s of r.querySelectorAll(':scope > section.b10x-product-section')) {
      if (!visible(s)) continue;
      ids.push('section:' + variantOf(s, 'b10x-product-section--'));
      for (const el of s.querySelectorAll('.b10x-feature')) if (visible(el)) ids.push('feature:' + text(el, 'h3'));
      for (const el of s.querySelectorAll('.b10x-flow__step')) if (visible(el)) ids.push('step:' + text(el, 'strong'));
      for (const el of s.querySelectorAll('.b10x-status-strip__items li')) if (visible(el)) ids.push('status:' + text(el, '.b10x-status-strip__label'));
      for (const el of s.querySelectorAll('.b10x-related__card')) if (visible(el)) ids.push('related:' + (variantOf(el, 'b10x-family--') || text(el, 'strong')));
    }
    return ids;
  }

  // ---------- colour ----------
  const parsed = new Map();
  let ctx2d = null;
  function parseColor(s) {
    if (!s || s === 'transparent' || s === 'none') return [0, 0, 0, 0];
    let c = parsed.get(s);
    if (c) return c;
    const m = s.match(/^rgba?\(([^)]+)\)$/);
    if (m) {
      const p = m[1].split(/[\s,/]+/).filter(Boolean).map(Number);
      c = [p[0], p[1], p[2], p.length > 3 ? p[3] : 1];
    } else {
      if (!ctx2d) ctx2d = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
      ctx2d.clearRect(0, 0, 1, 1);
      ctx2d.fillStyle = '#000';
      ctx2d.fillStyle = s;
      ctx2d.fillRect(0, 0, 1, 1);
      const d = ctx2d.getImageData(0, 0, 1, 1).data;
      c = [d[0], d[1], d[2], d[3] / 255];
    }
    parsed.set(s, c);
    return c;
  }
  const hex2 = (v) => Math.round(v).toString(16).padStart(2, '0');
  const toHex = (c) => '#' + hex2(c[0]) + hex2(c[1]) + hex2(c[2]);
  const lin = (v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  const luminance = (c) => 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
  const contrast = (a, b) => {
    const x = luminance(a), y = luminance(b);
    return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
  };
  const over = (fg, bg) => {
    const a = fg[3];
    return [fg[0] * a + bg[0] * (1 - a), fg[1] * a + bg[1] * (1 - a), fg[2] * a + bg[2] * (1 - a), 1];
  };

  // ---------- text ----------
  // The walker takes no filter: with the page's scripts off, the browser calls no callback.
  function textNodes() {
    const out = [];
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let t = walker.nextNode(); t; t = walker.nextNode()) {
      if (t.data.trim() && t.parentElement && !SKIP.has(t.parentElement.tagName)) out.push(t);
    }
    return out;
  }
  const textRect = (t) => {
    const range = document.createRange();
    range.selectNodeContents(t);
    return range.getBoundingClientRect();
  };
  function textOf(n) {
    if (n.nodeType === 3) return n.data;
    if (n.nodeType !== 1) return '';
    if (n.getAttribute('aria-hidden') === 'true') return '';
    if (n.tagName === 'IMG') return n.getAttribute('alt') || '';
    if (n.tagName === 'svg' || n.tagName === 'SVG') return n.getAttribute('aria-label') || n.querySelector('title')?.textContent || '';
    if (n.getAttribute('aria-label')) return n.getAttribute('aria-label');
    let s = '';
    for (const c of n.childNodes) s += ' ' + textOf(c);
    return s;
  }
  function accName(el) {
    const by = el.getAttribute('aria-labelledby');
    if (by) {
      const t = clean(by.split(/\s+/).map((id) => document.getElementById(id)?.textContent || '').join(' '));
      if (t) return t;
    }
    const label = clean(el.getAttribute('aria-label'));
    if (label) return label;
    const tag = el.tagName;
    if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') {
      const type = (el.type || '').toLowerCase();
      if (['submit', 'button', 'reset'].includes(type)) return clean(el.value) || (type === 'submit' ? 'Submit' : '');
      if (type === 'image') return clean(el.alt);
      if (el.id) {
        const l = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
        if (l && clean(l.textContent)) return clean(l.textContent);
      }
      const wrap = el.closest('label');
      if (wrap && clean(wrap.textContent)) return clean(wrap.textContent);
      return clean(el.getAttribute('title') || el.getAttribute('placeholder'));
    }
    return clean(textOf(el)) || clean(el.getAttribute('title'));
  }

  // ---------- journeys ----------
  function regions() {
    const out = [];
    const nav = document.querySelector('nav.navbar');
    if (nav) out.push(nav);
    const r = root();
    if (r) out.push(...r.querySelectorAll(':scope > .b10x-hero, :scope > section.b10x-product-section'));
    const footer = document.querySelector('footer');
    if (footer) out.push(footer);
    return out.filter(visible).map((el) => el.getBoundingClientRect().top + scrollY);
  }
  // The cheapest route of each journey at this viewport: a click on a visible link to a target,
  // plus one action per screenful scrolled to reach it; a navbar link behind the menu toggle,
  // plus the toggle; or a route to a `via` page whose sidebar links the target, plus one, and on
  // a narrow viewport one more to open the sidebar. Of two routes with the same actions, the one
  // that crosses fewer regions. Regions crossed: the navbar, the hero, the sections and the
  // footer that start at or above the link.
  function journeys() {
    const vw = innerWidth, vh = innerHeight;
    const narrow = vw < config.wide_from;
    const tops = regions();
    const toggle = document.querySelector('.navbar__toggle');
    const toggleShown = !!toggle && visible(toggle);
    const links = [...document.querySelectorAll('a[href]')].map((a) => [a, site(a.getAttribute('href'))]);
    const routeTo = (targets) => {
      let best = null;
      for (const [a, path] of links) {
        if (path === null || !targets.includes(path)) continue;
        let actions, crossed;
        if (visible(a)) {
          const b = a.getBoundingClientRect();
          const bottom = b.bottom + scrollY;
          actions = 1 + (bottom <= vh ? 0 : Math.ceil((bottom - vh) / vh));
          crossed = Math.max(1, tops.filter((top) => top <= b.top + scrollY + 1).length);
        } else if (a.closest('nav.navbar') && toggleShown) {
          actions = 2;
          crossed = 1;
        } else continue;
        if (!best || actions < best[0] || (actions === best[0] && crossed < best[1])) best = [actions, crossed];
      }
      return best;
    };
    return config.journeys.map((journey) => {
      let best = routeTo(journey.targets.map(norm));
      const via = journey.via.length ? routeTo(journey.via.map(norm)) : null;
      if (via) {
        const through = [via[0] + 1 + (narrow ? config.sidebar_actions : 0), via[1] + 1];
        if (!best || through[0] < best[0]) best = through;
      }
      return best;
    });
  }

  // A change of the style sheet or the colour scheme may start transitions; each is taken to
  // its end, so a render is measured in the state it comes to rest in.
  function settle() {
    void getComputedStyle(document.documentElement).color;
    for (const animation of document.getAnimations()) {
      try {
        animation.finish();
      } catch {
        // An animation without an end has no end state to take.
      }
    }
  }

  // ---------- one render ----------
  function measure(spec) {
    render(spec);
    settle();
    scrollTo(0, 0);
    const vw = innerWidth, vh = innerHeight;
    const notes = [];
    const primary = document.querySelector(PRIMARY);
    let primaryTop = null, primaryIn = false;
    if (primary && visible(primary)) {
      const r = primary.getBoundingClientRect();
      primaryTop = Math.round(r.top + scrollY);
      primaryIn = r.top >= 0 && r.left >= 0 && r.bottom <= vh && r.right <= vw;
    }
    if (!primaryIn) notes.push(primaryTop === null ? 'no visible primary action' : `the primary action ends at ${Math.round(primary.getBoundingClientRect().bottom)} px of ${vh}`);
    const scrollWidth = document.documentElement.scrollWidth;
    const hScroll = scrollWidth > vw + 1;
    if (hScroll) notes.push(`the page is ${scrollWidth} px wide in a viewport of ${vw}`);

    // Words above the primary action, and WCAG 2.x 1.4.3 of every visible text against the
    // surface it is composited on: 4.5:1, 3:1 for large text (24 px, or 18.66 px bold).
    const surfaces = new Map();
    const surface = (el) => {
      if (surfaces.has(el)) return surfaces.get(el);
      const cs = getComputedStyle(el);
      let res;
      if (cs.backgroundImage !== 'none') res = null;
      else {
        const c = parseColor(cs.backgroundColor);
        if (c[3] >= 1) res = c;
        else {
          const parent = el.parentElement ? surface(el.parentElement) : [255, 255, 255, 1];
          res = parent === null ? null : c[3] > 0 ? over(c, parent) : parent;
        }
      }
      if (el === document.documentElement && res === null) res = [255, 255, 255, 1];
      surfaces.set(el, res);
      return res;
    };
    let wordsBefore = 0, contrastFailures = 0;
    const shown = new Map();
    for (const t of textNodes()) {
      const el = t.parentElement;
      const r = textRect(t);
      if (r.width * r.height < 4) continue;
      const first = !shown.has(el);
      if (first) shown.set(el, visible(el));
      if (!shown.get(el)) continue;
      const above = primaryTop === null || r.top + scrollY < primaryTop - 1;
      if (above && r.right > 0 && r.left < vw && !(primary && primary.contains(t))) wordsBefore += words(t.data);
      if (!first) continue;
      const bg = surface(el);
      if (bg === null) continue;
      const cs = getComputedStyle(el);
      const fg = over(parseColor(cs.color), bg);
      const ratio = contrast(fg, bg);
      const size = parseFloat(cs.fontSize), bold = +cs.fontWeight >= 700;
      const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
      if (ratio < need - 0.005) {
        contrastFailures++;
        if (contrastFailures <= 3) notes.push(`contrast ${ratio.toFixed(2)} under ${need}: ${toHex(fg)} on ${toHex(bg)}, "${clean(t.data).slice(0, 40)}"`);
      }
    }

    let unnamed = 0;
    for (const el of document.querySelectorAll(INTERACTIVE)) {
      if (!visible(el) || accName(el)) continue;
      unnamed++;
      if (unnamed <= 3) notes.push(`no accessible name: <${el.tagName.toLowerCase()} class="${el.className}">`);
    }

    const now = new Set(content());
    const missing = builtContent.filter((id) => !now.has(id));
    if (missing.length) notes.push('not shown: ' + missing.slice(0, 3).join(', '));

    let small = 0;
    const r = root();
    if (r)
      for (const el of r.querySelectorAll('a[href],button,summary,[role=button]')) {
        if (!visible(el)) continue;
        const b = el.getBoundingClientRect();
        if (b.width < 44 || b.height < 44) small++;
      }

    const routes = journeys();
    routes.forEach((route, i) => {
      if (!route) notes.push('no route: ' + config.journeys[i].name);
    });
    return {
      primary_in_view: primaryIn,
      h_scroll: hScroll,
      contrast_failures: contrastFailures,
      unnamed,
      content_missing: missing.length,
      words: wordsBefore,
      primary_px: primaryTop === null ? document.documentElement.scrollHeight : primaryTop,
      small_targets: small,
      journeys: routes,
      fonts: document.fonts.status,
      notes,
    };
  }

  // `config`: base_path, journeys (name, targets, via), wide_from, sidebar_actions. The colour
  // scheme is set as the page's inline script sets it from the system's scheme.
  function prepare(cfg, mode) {
    config = cfg;
    const html = document.documentElement;
    html.setAttribute('data-theme', mode);
    html.setAttribute('data-theme-choice', 'system');
    const r = root();
    if (!r) throw new Error('no .b10x-product on this page');
    settle();
    built = r.cloneNode(true);
    builtContent = content();
    return {
      actions: r.querySelectorAll('.b10x-hero__actions > a').length,
      sections: r.querySelectorAll(':scope > section.b10x-product-section').length,
      content: builtContent.length,
      width: innerWidth,
      height: innerHeight,
      theme: getComputedStyle(html).colorScheme,
    };
  }

  window.__uxuiLanding = { prepare, render, measure, html: () => root().outerHTML };
  return true;
})()
