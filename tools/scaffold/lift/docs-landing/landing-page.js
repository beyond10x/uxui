// Injected into the built landing page (/uxui/) with its scripts removed, beside runtime.js.
// window.__landing.render(spec) rebuilds the page from patched sources: spec.product is a
// b10x-product-landing/1 object, spec.css the values of landing.css's custom properties and
// spec.cssText the text of landing.css. The mapping mirrors the docs-system renderer
// (ProductHero, ProductSection, FeatureGrid): actions are keyed by href, sections by kind,
// features by title; the kicker of a section is its number and eyebrow; an empty `kpis` drops
// the KPI row; the feature grid takes featureColumns(count) columns.
// window.__landing.journeys(spec) measures the journeys, tap targets and APCA contrast.
(() => {
  if (window.__landing) return;
  let base = null;
  const strip = (href) => {
    try {
      const u = new URL(href, location.href);
      return u.pathname.replace(/^\/uxui/, '').replace(/\/$/, '') || '/';
    } catch {
      return href;
    }
  };
  const norm = (h) => h.replace(/\/$/, '') || '/';
  const featureColumns = (n) => (n === 4 || n <= 2 ? 2 : n === 7 || n === 8 ? 4 : 3);

  function root() {
    return document.querySelector('.b10x-product');
  }
  function reset() {
    const r = root();
    if (!r) throw new Error('no .b10x-product on this page');
    if (!base) base = r.cloneNode(true);
    else r.replaceWith(base.cloneNode(true));
    document.getElementById('uxui-landing-css')?.remove();
  }

  function render(spec) {
    const t0 = performance.now();
    reset();
    const r = root();
    const p = spec.product.product;
    // hero: lede, actions, kpis
    const lede = r.querySelector('.b10x-hero__lede');
    if (lede && lede.textContent !== p.lede) lede.textContent = p.lede;
    const box = r.querySelector('.b10x-hero__actions');
    if (box) {
      const byHref = new Map([...box.querySelectorAll(':scope > a')].map((a) => [strip(a.getAttribute('href')), a]));
      box.replaceChildren(
        ...p.actions.map((a) => {
          const el = byHref.get(norm(a.href));
          if (!el) throw new Error('no built action for ' + a.href);
          el.className = a.kind === 'primary' ? 'b10x-button' : 'b10x-text-link';
          return el;
        }),
      );
    }
    if (!(p.kpis || []).length) r.querySelector('.b10x-hero > .b10x-kpis')?.remove();
    // sections: order, numbers, feature count
    const hero = r.querySelector('.b10x-hero');
    const byKind = new Map(
      [...r.querySelectorAll(':scope > section.b10x-product-section')].map((s) => [
        [...s.classList].find((c) => c.startsWith('b10x-product-section--')).slice('b10x-product-section--'.length),
        s,
      ]),
    );
    let after = hero;
    spec.product.sections.forEach((sec, i) => {
      const el = byKind.get(sec.kind);
      if (!el) throw new Error('no built section ' + sec.kind);
      const k = el.querySelector('.b10x-product-section__header .b10x-kicker');
      if (k) k.textContent = [String(i + 1).padStart(2, '0'), sec.eyebrow].filter(Boolean).join(' / ');
      if (sec.kind === 'features') {
        const grid = el.querySelector('.b10x-feature-grid');
        const byTitle = new Map([...grid.querySelectorAll(':scope > .b10x-feature')].map((li) => [li.querySelector('h3').textContent.trim(), li]));
        grid.replaceChildren(...sec.items.map((it) => byTitle.get(it.title)));
        grid.className = `b10x-feature-grid b10x-feature-grid--${featureColumns(sec.items.length)}`;
      }
      after.after(el);
      after = el;
      byKind.delete(sec.kind);
    });
    for (const el of byKind.values()) el.remove();
    // CSS
    const style = document.createElement('style');
    style.id = 'uxui-landing-css';
    style.textContent = (spec.cssText || '') + `\n:root{--uxui-density:${spec.css.density};--uxui-heading-scale:${spec.css.heading_scale}}`;
    document.head.appendChild(style);
    return { ms: +(performance.now() - t0).toFixed(2) };
  }

  // ---- measurement --------------------------------------------------------------------------
  const visible = (el) => {
    if (!el.isConnected) return false;
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true })) return false;
    const b = el.getBoundingClientRect();
    return b.width > 0 && b.height > 0;
  };
  function regions() {
    const out = [];
    const nav = document.querySelector('nav.navbar');
    if (nav) out.push(nav);
    const r = root();
    if (r) out.push(...r.querySelectorAll(':scope > .b10x-hero, :scope > section.b10x-product-section'));
    const f = document.querySelector('footer');
    if (f) out.push(f);
    return out.filter(visible).map((el) => ({ el, top: el.getBoundingClientRect().top + scrollY }));
  }

  // Cheapest route of a journey at this viewport: a click on a visible link to a target (plus
  // one action per screenful scrolled to reach it), a hidden navbar link behind the menu toggle
  // (plus the toggle), or a route to a `via` page whose sidebar links the target (plus one, and
  // on a narrow viewport one more to open the sidebar). Regions crossed: navbar, hero, sections
  // and footer whose top is at or above the link.
  function journeys(spec) {
    scrollTo(0, 0);
    const vw = innerWidth, vh = innerHeight;
    const narrow = vw < (spec.navbar?.wide_from || 997);
    const regs = regions();
    const toggle = document.querySelector('.navbar__toggle');
    const toggleShown = toggle && visible(toggle);
    const links = [...document.querySelectorAll('a[href]')];
    const routeTo = (targets) => {
      let best = null;
      for (const a of links) {
        const href = strip(a.getAttribute('href'));
        if (!targets.includes(href)) continue;
        let cost, crossed, where;
        if (visible(a)) {
          const b = a.getBoundingClientRect();
          const bottom = b.bottom + scrollY;
          const scrolls = bottom <= vh ? 0 : Math.ceil((bottom - vh) / vh);
          cost = 1 + scrolls;
          crossed = Math.max(1, regs.filter((x) => x.top <= b.top + scrollY + 1).length);
          where = a.closest('nav.navbar') ? 'navbar' : a.closest('footer') ? 'footer' : a.closest('.b10x-hero') ? 'hero' : (a.closest('section.b10x-product-section')?.id || 'page');
        } else if (a.closest('nav.navbar') && toggleShown) {
          cost = 2;
          crossed = 1;
          where = 'navbar menu';
        } else continue;
        if (!best || cost < best.cost || (cost === best.cost && crossed < best.regions)) best = { cost, regions: crossed, where, href };
      }
      return best;
    };
    const out = {};
    for (const j of spec.journeys) {
      let best = routeTo(j.targets.map(norm));
      const via = (j.via || []).length ? routeTo(j.via.map(norm)) : null;
      if (via) {
        const v = { cost: via.cost + 1 + (narrow ? spec.navbar?.narrow_sidebar_actions ?? 1 : 0), regions: via.regions + 1, where: 'via ' + via.where + ' ' + via.href, href: j.targets[0] };
        if (!best || v.cost < best.cost) best = v;
      }
      out[j.name] = best || { cost: null, regions: null, where: 'unreachable' };
    }
    // tap targets under 44 px in the landing body
    let small = 0, targets = 0;
    const r = root();
    if (r)
      for (const el of r.querySelectorAll('a[href],button,summary,[role=button]')) {
        if (!visible(el)) continue;
        targets++;
        const b = el.getBoundingClientRect();
        if (b.width < 44 || b.height < 44) small++;
      }
    return { viewport: vw + 'x' + vh, journeys: out, small_targets: small, targets, apca: apca() };
  }

  // APCA-W3 0.0.98G-4g lightness contrast of every visible text under 24 px against its surface.
  function lin(c) {
    return Math.pow(c / 255, 2.4);
  }
  function Y([r, g, b]) {
    return 0.2126729 * lin(r) + 0.7151522 * lin(g) + 0.072175 * lin(b);
  }
  function lc(txt, bg) {
    const clamp = (y) => (y > 0.022 ? y : y + Math.pow(0.022 - y, 1.414));
    const t = clamp(Y(txt)), b = clamp(Y(bg));
    if (Math.abs(b - t) < 0.0005) return 0;
    if (b > t) {
      const s = (Math.pow(b, 0.56) - Math.pow(t, 0.57)) * 1.14;
      return s < 0.1 ? 0 : (s - 0.027) * 100;
    }
    const s = (Math.pow(b, 0.65) - Math.pow(t, 0.62)) * 1.14;
    return s > -0.1 ? 0 : (s + 0.027) * 100;
  }
  const parse = (s) => {
    const m = s && s.match(/rgba?\(([^)]+)\)/);
    if (!m) return [0, 0, 0, 0];
    const p = m[1].split(/[\s,/]+/).filter(Boolean).map(Number);
    return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1];
  };
  const over = (fg, bg) => {
    const a = fg[3];
    return [fg[0] * a + bg[0] * (1 - a), fg[1] * a + bg[1] * (1 - a), fg[2] * a + bg[2] * (1 - a), 1];
  };
  function surface(el) {
    const stack = [];
    for (let e = el; e; e = e.parentElement) {
      const cs = getComputedStyle(e);
      if (cs.backgroundImage !== 'none' && !cs.backgroundImage.startsWith('linear-gradient')) return null;
      const c = parse(cs.backgroundColor);
      if (c[3] > 0) stack.push(c);
      if (c[3] >= 1) break;
    }
    let bg = [255, 255, 255, 1];
    for (let i = stack.length - 1; i >= 0; i--) bg = over(stack[i], bg);
    return bg;
  }
  function apca() {
    let min = 999, under60 = 0, checked = 0;
    const seen = new Set();
    const tw = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let t = tw.nextNode(); t; t = tw.nextNode()) {
      if (!t.data.trim()) continue;
      const el = t.parentElement;
      if (!el || seen.has(el) || ['SCRIPT', 'STYLE', 'NOSCRIPT'].includes(el.tagName)) continue;
      seen.add(el);
      if (!visible(el)) continue;
      const cs = getComputedStyle(el);
      if (parseFloat(cs.fontSize) >= 24) continue;
      const bg = surface(el);
      if (!bg) continue;
      const v = Math.abs(lc(over(parse(cs.color), bg), bg));
      checked++;
      if (v < min) min = v;
      if (v < 60) under60++;
    }
    return { lc_min: checked ? +min.toFixed(1) : null, under_60: under60, checked };
  }

  // The ids of lift.mjs contentOf(), read from what the page renders (present and visible).
  function content() {
    const r = root();
    if (!r) return [];
    const ids = [];
    const text = (el, sel) => (sel ? el.querySelector(sel) : el)?.textContent.trim().replace(/\s+/g, ' ') || '';
    for (const el of r.querySelectorAll('.b10x-hero .b10x-kpi')) if (visible(el)) ids.push('kpi:' + ([...el.classList].find((c) => c.startsWith('b10x-kpi--')) || '').slice(10));
    for (const s of r.querySelectorAll(':scope > section.b10x-product-section')) {
      if (!visible(s)) continue;
      const kind = ([...s.classList].find((c) => c.startsWith('b10x-product-section--')) || '').slice(22);
      ids.push('section:' + kind);
      for (const el of s.querySelectorAll('.b10x-feature')) if (visible(el)) ids.push('feature:' + text(el, 'h3'));
      for (const el of s.querySelectorAll('.b10x-flow__step')) if (visible(el)) ids.push('step:' + text(el, 'strong'));
      for (const el of s.querySelectorAll('.b10x-status-strip__items li')) if (visible(el)) ids.push('status:' + text(el, '.b10x-status-strip__label'));
      for (const el of s.querySelectorAll('.b10x-related__card')) if (visible(el)) ids.push('related:' + (([...el.classList].find((c) => c.startsWith('b10x-family--')) || '').slice(13) || text(el, 'strong')));
    }
    return ids;
  }

  window.__landing = { render, reset, journeys, content, featureColumns };
  if (window.__variantSpec) {
    const go = () => render(window.__variantSpec);
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', go);
    else go();
  }
})();
