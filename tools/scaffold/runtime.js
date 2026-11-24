// uxui scaffold runtime: injected into any page. Plain script, no dependencies.
//
//   __uxui.scaffold()      -> uxui-scaffold/1 JSON of the page as it is now (regions, sections,
//                             headings, interactive elements, palette, type and spacing scales)
//   __uxui.apply(variant)  -> applies a variant instantly; returns {ms}
//   __uxui.reset()         -> undoes every change
//   __uxui.measure()       -> hard rules and metrics at the current viewport
//
// A variant is {axis: value}. Generic axes: mode (original|light|dark), density (spacing factor),
// font_scale, accent (#rrggbb), section_order ("original" | "reverse" | [section ids]), hide
// ([section ids]), max_width (px | "none"), and `ops`, a list of DOM operations a binding table
// produces: {op: "order", items: [selector...]}, {op: "hide", sel}, {op: "text", sel, text},
// {op: "style", sel, props: {css-property: value}}.
(() => {
  if (window.__uxui) return;
  const HOST_ID = '__uxui_panel';
  const SKIP = new Set(['SCRIPT', 'STYLE', 'LINK', 'META', 'NOSCRIPT', 'TEMPLATE', 'HEAD', 'TITLE', 'BR', 'WBR', 'SOURCE', 'TRACK']);
  const INTERACTIVE =
    'a[href],button,input:not([type=hidden]),select,textarea,summary,[role=button],[role=link],[role=tab],' +
    '[role=menuitem],[role=checkbox],[role=switch],[role=radio],[role=combobox],[tabindex]:not([tabindex="-1"])';
  const LANDMARKS =
    'header,nav,main,aside,footer,form[aria-label],form[aria-labelledby],section[aria-label],section[aria-labelledby],' +
    '[role=banner],[role=navigation],[role=main],[role=complementary],[role=contentinfo],[role=search],[role=region]';

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
  const toHex = (c) => '#' + hex2(c[0]) + hex2(c[1]) + hex2(c[2]) + (c[3] < 1 ? hex2(c[3] * 255) : '');
  const toCss = (c) => (c[3] >= 1 ? `rgb(${c[0]}, ${c[1]}, ${c[2]})` : `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${+c[3].toFixed(3)})`);
  const lin = (v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  const gam = (v) => 255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);
  function oklch(c) {
    const r = lin(c[0]), g = lin(c[1]), b = lin(c[2]);
    const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
    const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
    const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
    return [L, Math.hypot(A, B), ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360];
  }
  function fromOklch(L, C, h, alpha) {
    for (let i = 0; i < 12; i++) {
      const A = C * Math.cos((h * Math.PI) / 180), B = C * Math.sin((h * Math.PI) / 180);
      const l = (L + 0.3963377774 * A + 0.2158037573 * B) ** 3;
      const m = (L - 0.1055613458 * A - 0.0638541728 * B) ** 3;
      const s = (L - 0.0894841775 * A - 1.291485548 * B) ** 3;
      const rgb = [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
      ];
      if (rgb.every((v) => v >= -0.001 && v <= 1.001) || i === 11) {
        return [...rgb.map((v) => Math.round(Math.min(255, Math.max(0, gam(Math.min(1, Math.max(0, v))))))), alpha];
      }
      C *= 0.85;
    }
  }
  const luminance = (c) => 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
  const contrast = (a, b) => {
    const x = luminance(a), y = luminance(b);
    return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
  };
  const over = (fg, bg) => {
    const a = fg[3];
    return [fg[0] * a + bg[0] * (1 - a), fg[1] * a + bg[1] * (1 - a), fg[2] * a + bg[2] * (1 - a), 1];
  };
  const isChromatic = (c) => {
    if (c[3] < 0.3) return false;
    const [L, C] = oklch(c);
    return C > 0.06 && L > 0.2 && L < 0.92;
  };
  const hueDist = (a, b) => Math.min(Math.abs(a - b), 360 - Math.abs(a - b));

  // ---------- DOM helpers ----------
  const isHost = (el) => el.id === HOST_ID;
  function visible(el) {
    if (!el.isConnected) return false;
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true })) return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  }
  function selectorOf(el) {
    if (el === document.body) return 'body';
    const uniq = (id) => {
      try {
        return document.querySelectorAll('#' + CSS.escape(id)).length === 1;
      } catch {
        return false;
      }
    };
    if (el.id && uniq(el.id)) return '#' + CSS.escape(el.id);
    const parts = [];
    let n = el;
    while (n && n.nodeType === 1 && n !== document.documentElement) {
      if (n !== el && n.id && uniq(n.id)) {
        parts.unshift('#' + CSS.escape(n.id));
        break;
      }
      if (n === document.body) {
        parts.unshift('body');
        break;
      }
      let p = n.tagName.toLowerCase();
      const par = n.parentElement;
      if (par) {
        const same = [...par.children].filter((c) => c.tagName === n.tagName);
        if (same.length > 1) p += `:nth-of-type(${same.indexOf(n) + 1})`;
      }
      parts.unshift(p);
      n = par;
    }
    return parts.join(' > ');
  }
  const box = (el) => {
    const r = el.getBoundingClientRect();
    return { x: Math.round(r.left + scrollX), y: Math.round(r.top + scrollY), w: Math.round(r.width), h: Math.round(r.height) };
  };
  const words = (s) => (s.match(/[\p{L}\p{N}][\p{L}\p{N}'’.-]*/gu) || []).length;
  const clean = (s) => (s || '').replace(/\s+/g, ' ').trim();

  function textOf(n) {
    if (n.nodeType === 3) return n.data;
    if (n.nodeType !== 1) return '';
    if (n.getAttribute('aria-hidden') === 'true') return '';
    if (n.tagName === 'IMG') return n.getAttribute('alt') || '';
    if (n.tagName === 'svg' || n.tagName === 'SVG') {
      return n.getAttribute('aria-label') || n.querySelector('title')?.textContent || '';
    }
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
    const al = clean(el.getAttribute('aria-label'));
    if (al) return al;
    const tag = el.tagName;
    if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') {
      const t = (el.type || '').toLowerCase();
      if (['submit', 'button', 'reset'].includes(t)) return clean(el.value) || (t === 'submit' ? 'Submit' : '');
      if (t === 'image') return clean(el.alt);
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
  function roleOf(el) {
    const r = el.getAttribute('role');
    if (r) return r;
    const t = el.tagName.toLowerCase();
    if (t === 'a') return 'link';
    if (t === 'button' || t === 'summary') return 'button';
    if (t === 'select') return 'combobox';
    if (t === 'textarea') return 'textbox';
    if (t === 'input') {
      const ty = (el.type || 'text').toLowerCase();
      return { checkbox: 'checkbox', radio: 'radio', submit: 'button', button: 'button', reset: 'button', range: 'slider' }[ty] || 'textbox';
    }
    return t;
  }
  const landmarkRole = (el) =>
    el.getAttribute('role') ||
    { HEADER: 'banner', NAV: 'navigation', MAIN: 'main', ASIDE: 'complementary', FOOTER: 'contentinfo', FORM: 'form', SECTION: 'region' }[el.tagName] ||
    'region';

  function allElements() {
    const out = [];
    const walk = (n) => {
      for (let c = n.firstElementChild; c; c = c.nextElementSibling) {
        if (SKIP.has(c.tagName) || isHost(c)) continue;
        out.push(c);
        walk(c);
      }
    };
    if (document.body) {
      out.push(document.body);
      walk(document.body);
    }
    return out;
  }
  function textNodes(root = document.body) {
    const out = [];
    const tw = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: (t) => {
        if (!t.data.trim()) return NodeFilter.FILTER_REJECT;
        const p = t.parentElement;
        if (!p || SKIP.has(p.tagName) || p.closest('#' + HOST_ID)) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      },
    });
    for (let t = tw.nextNode(); t; t = tw.nextNode()) out.push(t);
    return out;
  }
  const textRect = (t) => {
    const r = document.createRange();
    r.selectNodeContents(t);
    return r.getBoundingClientRect();
  };

  // ---------- state ----------
  const S = {
    sections: [], // {id, el}
    regions: [],
    primary: null,
    primaryReason: '',
    accent: null, // [r,g,b,a]
    pageDark: false,
    bases: new Map(), // "WxH" -> capture
    touched: new Map(), // el -> original style attribute (or null)
    moved: [], // [{marker, el}] in original order
    texts: [], // [{node, data}]
    variant: null,
    last: null,
  };

  // ---------- section discovery ----------
  function sectionRoot() {
    const start = document.querySelector('main,[role=main]') || document.body;
    const vw = innerWidth;
    let best = null;
    const consider = (el, depth) => {
      if (depth > 8 || !visible(el)) return;
      const cb = el.getBoundingClientRect();
      if (cb.width < Math.min(vw * 0.4, 300)) return;
      const kids = [...el.children].filter((c) => !SKIP.has(c.tagName) && !isHost(c) && visible(c));
      const flow = kids.filter((c) => {
        const cs = getComputedStyle(c);
        if (cs.position === 'absolute' || cs.position === 'fixed') return false;
        const r = c.getBoundingClientRect();
        return r.width >= cb.width * 0.5 && r.height >= 16;
      });
      let score = 0;
      for (const c of flow) score += c.children.length > 0 ? 1 : 0.5;
      if (flow.length >= 2 && (!best || score > best.score + 0.01)) best = { el, score, kids: flow };
      for (const c of kids) consider(c, depth + 1);
    };
    consider(start, 0);
    if (!best && start !== document.body) consider(document.body, 0);
    return best;
  }
  function labelOf(el) {
    const h = el.matches('h1,h2,h3,h4,h5,h6') ? el : el.querySelector('h1,h2,h3,h4,h5,h6,[role=heading]');
    const t = clean(h ? h.textContent : el.getAttribute('aria-label') || el.textContent);
    return (t.length > 48 ? t.slice(0, 47) + '…' : t) || el.tagName.toLowerCase();
  }

  function buttonLike(el) {
    if (el.tagName === 'BUTTON' || el.getAttribute('role') === 'button') return true;
    if (typeof el.className === 'string' && /(^|[\s_-])(btn|button|cta)([\s_-]|$)/i.test(el.className)) return true;
    const cs = getComputedStyle(el);
    const bg = parseColor(cs.backgroundColor);
    const bordered = parseFloat(cs.borderTopWidth) >= 1 && parseColor(cs.borderTopColor)[3] > 0.3;
    const padded = parseFloat(cs.paddingLeft) >= 8 && parseFloat(cs.paddingTop) >= 4;
    return padded && (bg[3] > 0.5 || bordered);
  }
  function findPrimary(all) {
    const skipLink = (i) => (i.el.getAttribute('href') || '').startsWith('#') && /skip/i.test(i.name);
    const interactive = all.filter((i) => i.box.x >= 0 && i.box.y >= 0 && !skipLink(i));
    const outside = (el) => {
      for (let n = el; n && n !== document.body; n = n.parentElement) {
        if (/^(NAV|FOOTER|ASIDE)$/.test(n.tagName)) return false;
        if (['banner', 'navigation', 'contentinfo', 'complementary'].includes(n.getAttribute('role'))) return false;
        if (n.tagName === 'HEADER' && !n.parentElement.closest('main,article,section,[role=main]')) return false;
      }
      return true;
    };
    const content = interactive.filter((i) => outside(i.el));
    const pool = content.length ? content : interactive;
    const byPos = [...pool].sort((a, b) => a.box.y - b.box.y || a.box.x - b.box.x);
    const btn = byPos.find((i) => buttonLike(i.el));
    if (btn) return [btn, 'first button-like control in the content, top to bottom'];
    if (byPos[0]) return [byPos[0], 'first control in the content (no button-like control)'];
    return [null, 'no interactive element'];
  }

  function frequency(map, key, w) {
    map.set(key, (map.get(key) || 0) + w);
  }
  const top = (map, n, fmt = (k) => k) =>
    [...map.entries()].sort((a, b) => b[1] - a[1]).slice(0, n).map(([k, w]) => ({ value: fmt(k), weight: Math.round(w) }));

  function scaffold(opts = {}) {
    reset();
    const t0 = performance.now();
    scrollTo(0, 0);
    const els = allElements();
    const vw = innerWidth, vh = innerHeight;
    // regions
    S.regions = [...document.querySelectorAll(LANDMARKS)]
      .filter((el) => visible(el) && !isHost(el))
      .filter((el) => !(el.tagName === 'HEADER' || el.tagName === 'FOOTER') || !el.parentElement.closest('article,section,main,aside,nav'))
      .map((el, i) => ({ id: 'r' + i, el }));
    // sections
    const root = sectionRoot();
    S.sections = root ? root.kids.map((el, i) => ({ id: 's' + i, el })) : [];
    // headings
    const headings = [...document.querySelectorAll('h1,h2,h3,h4,h5,h6,[role=heading]')]
      .filter(visible)
      .map((el, i) => ({ id: 'h' + i, level: +(el.getAttribute('aria-level') || el.tagName.slice(1)) || 2, text: clean(el.textContent).slice(0, 120), selector: selectorOf(el), box: box(el) }));
    // interactive
    const inter = [...document.querySelectorAll(INTERACTIVE)]
      .filter((el) => !el.closest('#' + HOST_ID) && visible(el))
      .map((el, i) => ({ id: 'i' + i, el, role: roleOf(el), name: accName(el).slice(0, 80), box: box(el) }));
    if (opts.primary) {
      const el = document.querySelector(opts.primary);
      S.primary = el;
      S.primaryReason = 'given: ' + opts.primary;
    } else {
      const [p, why] = findPrimary(inter);
      S.primary = p ? p.el : null;
      S.primaryReason = why;
    }
    // palette, type, spacing
    const textCol = new Map(), bgCol = new Map(), sizes = new Map(), spacing = new Map(), accentVotes = new Map();
    for (const el of els) {
      const cs = getComputedStyle(el);
      if (cs.display === 'none') continue;
      let chars = 0;
      for (const c of el.childNodes) if (c.nodeType === 3) chars += c.data.trim().length;
      const r = el.getBoundingClientRect();
      if (chars) {
        frequency(textCol, toHex(parseColor(cs.color)), chars);
        frequency(sizes, Math.round(parseFloat(cs.fontSize) * 10) / 10, chars);
      }
      const bg = parseColor(cs.backgroundColor);
      if (bg[3] > 0) frequency(bgCol, toHex(bg), (r.width * r.height) / 1000);
      for (const p of ['marginTop', 'marginBottom', 'paddingTop', 'paddingBottom', 'paddingLeft', 'paddingRight', 'rowGap', 'columnGap']) {
        const v = parseFloat(cs[p]);
        if (v > 0) frequency(spacing, Math.round(v), 1);
      }
    }
    for (const i of inter) {
      const cs = getComputedStyle(i.el);
      for (const [c, w] of [[parseColor(cs.color), 1], [parseColor(cs.backgroundColor), 3], [parseColor(cs.borderTopColor), parseFloat(cs.borderTopWidth) > 0 ? 1 : 0]]) {
        if (w && isChromatic(c)) frequency(accentVotes, toHex([c[0], c[1], c[2], 1]), w);
      }
    }
    for (const h of document.querySelectorAll('h1,h2,h3')) {
      const c = parseColor(getComputedStyle(h).color);
      if (isChromatic(c)) frequency(accentVotes, toHex([c[0], c[1], c[2], 1]), 0.5);
    }
    const acc = top(accentVotes, 1)[0];
    S.accent = acc ? parseColor(acc.value) : null;
    const pageBg = effectiveBg(document.body);
    S.pageDark = luminance(pageBg) < 0.2;
    // custom properties on :root
    const custom = {};
    let customTotal = 0;
    const rootCs = getComputedStyle(document.documentElement);
    for (let i = 0; i < rootCs.length; i++) {
      const p = rootCs[i];
      if (!p.startsWith('--')) continue;
      customTotal++;
      if (Object.keys(custom).length < 200) custom[p] = rootCs.getPropertyValue(p).trim().slice(0, 120);
    }
    const out = {
      schema: 'uxui-scaffold/1',
      url: location.href,
      title: document.title,
      viewport: { width: vw, height: vh, dpr: devicePixelRatio },
      page_height: document.documentElement.scrollHeight,
      mode: S.pageDark ? 'dark' : 'light',
      regions: S.regions.map(({ id, el }) => ({ id, role: landmarkRole(el), name: accName(el).slice(0, 60), selector: selectorOf(el), box: box(el) })),
      section_root: root ? { selector: selectorOf(root.el), display: getComputedStyle(root.el).display, reorder: 'dom-move' } : null,
      sections: S.sections.map(({ id, el }) => ({
        id,
        label: labelOf(el),
        tag: el.tagName.toLowerCase(),
        selector: selectorOf(el),
        box: box(el),
        words: words(el.innerText || ''),
        interactive: el.querySelectorAll(INTERACTIVE).length,
      })),
      headings,
      interactive: inter.map(({ id, el, role, name, box: b }) => ({ id, role, name, selector: selectorOf(el), box: b, primary: el === S.primary || undefined })),
      primary: S.primary ? { selector: selectorOf(S.primary), name: accName(S.primary).slice(0, 80), reason: S.primaryReason } : null,
      palette: {
        text: top(textCol, 10),
        background: top(bgCol, 10),
        accent: acc ? { color: acc.value, hue: Math.round(oklch(S.accent)[2]) } : null,
      },
      font_sizes: top(sizes, 12).sort((a, b) => a.value - b.value),
      spacing: top(spacing, 14).sort((a, b) => a.value - b.value),
      custom_properties: { total: customTotal, values: custom },
      counts: { elements: els.length, interactive: inter.length, headings: headings.length, text_nodes: textNodes().length },
    };
    out.axes = suggestAxes(out);
    out.ms = Math.round(performance.now() - t0);
    S.last = out;
    return out;
  }

  function suggestAxes(sc) {
    const ax = {
      mode: sc.mode === 'dark' ? ['original', 'light'] : ['original', 'dark'],
      density: [0.8, 1, 1.25],
      font_scale: [0.9, 1, 1.15],
      max_width: ['original', 720, 1100],
    };
    if (sc.palette.accent) {
      const [L, C, h] = oklch(parseColor(sc.palette.accent.color));
      ax.accent = ['original', toHex(fromOklch(L, Math.max(C, 0.12), (h + 120) % 360, 1)), toHex(fromOklch(L, Math.max(C, 0.12), (h + 240) % 360, 1))];
    }
    if (sc.sections.length >= 2) {
      ax.section_order = ['original', 'reverse'];
      const big = [...sc.sections].sort((a, b) => b.box.h - a.box.h).find((s) => !sc.interactive.some((i) => i.primary && document.querySelector(s.selector)?.contains(document.querySelector(i.selector))));
      ax.hide = [[], ...(big ? [[big.id]] : [])];
    }
    return ax;
  }

  // ---------- capture of base styles at the current viewport ----------
  function effectiveBg(el) {
    const layers = [];
    for (let n = el; n && n.nodeType === 1; n = n.parentElement) {
      const c = parseColor(getComputedStyle(n).backgroundColor);
      if (c[3] > 0) layers.push(c);
      if (c[3] >= 1) break;
    }
    let bg = [255, 255, 255, 1];
    for (let i = layers.length - 1; i >= 0; i--) bg = over(layers[i], bg);
    return bg;
  }
  function capture() {
    const key = innerWidth + 'x' + innerHeight;
    const els = allElements();
    const hit = S.bases.get(key);
    if (hit && hit.count === els.length && hit.recs[0]?.el === els[0]) return hit;
    const recs = [];
    const idx = new Map();
    for (const el of els) {
      const cs = getComputedStyle(el);
      if (cs.display === 'none') continue;
      const parentRec = idx.get(el.parentElement);
      const bg = parseColor(cs.backgroundColor);
      const rec = {
        el,
        color: parseColor(cs.color),
        bg,
        bgImage: cs.backgroundImage.includes('gradient') ? cs.backgroundImage : null,
        shadow: cs.boxShadow !== 'none' ? cs.boxShadow : null,
        borders: ['Top', 'Right', 'Bottom', 'Left'].map((s) => (parseFloat(cs['border' + s + 'Width']) > 0 ? parseColor(cs['border' + s + 'Color']) : null)),
        fill: el instanceof SVGElement && cs.fill && cs.fill !== 'none' && !cs.fill.startsWith('url') ? parseColor(cs.fill) : null,
        stroke: el instanceof SVGElement && cs.stroke && cs.stroke !== 'none' && !cs.stroke.startsWith('url') ? parseColor(cs.stroke) : null,
        fontSize: parseFloat(cs.fontSize),
        lineHeight: cs.lineHeight.endsWith('px') ? parseFloat(cs.lineHeight) : null,
        pad: ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft'].map((p) => parseFloat(cs[p]) || 0),
        margin: [parseFloat(cs.marginTop) || 0, parseFloat(cs.marginBottom) || 0],
        gap: /flex|grid/.test(cs.display) ? [parseFloat(cs.rowGap) || 0, parseFloat(cs.columnGap) || 0] : null,
        maxWidth: cs.maxWidth.endsWith('px') ? parseFloat(cs.maxWidth) : null,
        width: el.getBoundingClientRect().width,
        surface: null,
      };
      // the nearest opaque surface this element's text sits on
      rec.surface = bg[3] >= 0.5 ? rec : parentRec ? parentRec.surface : null;
      idx.set(el, rec);
      recs.push(rec);
    }
    const base = { key, count: els.length, recs, html: parseColor(getComputedStyle(document.documentElement).backgroundColor) };
    S.bases.set(key, base);
    return base;
  }

  // ---------- apply / reset ----------
  function touch(el) {
    if (!S.touched.has(el)) S.touched.set(el, el.getAttribute('style'));
  }
  const setp = (el, prop, value) => {
    touch(el);
    el.style.setProperty(prop, value, 'important');
  };
  function reset() {
    for (const [el, style] of S.touched) {
      if (style === null) el.removeAttribute('style');
      else el.setAttribute('style', style);
    }
    S.touched.clear();
    for (const { node, data } of S.texts) node.data = data;
    S.texts = [];
    for (const { marker, el } of S.moved) if (marker.isConnected) marker.after(el);
    S.moved = [];
    document.documentElement.classList.remove('uxui-active');
    S.variant = null;
  }
  function ensureKillTransitions() {
    if (document.getElementById('__uxui_style')) return;
    const st = document.createElement('style');
    st.id = '__uxui_style';
    st.textContent = 'html.uxui-active *,html.uxui-active *::before,html.uxui-active *::after{transition:none!important;animation-duration:0s!important;animation-delay:0s!important}';
    (document.head || document.documentElement).appendChild(st);
  }

  function colorFn(variant) {
    const accentTo = variant.accent && variant.accent !== 'original' && S.accent ? oklch(parseColor(variant.accent)) : null;
    const accentFrom = S.accent ? oklch(S.accent) : null;
    const mode = variant.mode || 'original';
    const invert = (mode === 'dark' && !S.pageDark) || (mode === 'light' && S.pageDark);
    if (!accentTo && !invert) return null;
    const cache = new Map();
    return (c, role, keep) => {
      if (!c || c[3] === 0) return null;
      const k = c.join(',') + role + (keep ? 'k' : '');
      if (cache.has(k)) return cache.get(k);
      let [L, C, h] = oklch(c);
      let changed = false;
      if (accentTo && C > 0.04 && hueDist(h, accentFrom[2]) < 30) {
        L = Math.min(0.98, Math.max(0.05, L + (accentTo[0] - accentFrom[0])));
        C = C * (accentTo[1] / Math.max(accentFrom[1], 0.01));
        h = (accentTo[2] + (h - accentFrom[2]) + 360) % 360;
        changed = true;
      }
      if (invert && !keep) {
        L = S.pageDark ? Math.min(1, 1.1 - L) : 0.97 - 0.84 * L;
        if (role === 'text' && C > 0.04) L = Math.max(L, S.pageDark ? 0 : 0.72);
        changed = true;
      }
      const out = changed ? toCss(fromOklch(L, C, h, c[3])) : null;
      cache.set(k, out);
      return out;
    };
  }
  function ensureContrast(css, bg, target) {
    let c = parseColor(css);
    if (contrast(over(c, bg), bg) >= target) return css;
    let [L, C, h] = oklch(c);
    const dir = luminance(bg) < 0.2 ? 1 : -1;
    for (let i = 0; i < 40 && contrast(over(c, bg), bg) < target; i++) {
      L = Math.min(1, Math.max(0, L + dir * 0.015));
      c = fromOklch(L, C, h, c[3]);
    }
    return toCss(c);
  }
  const mapColorsIn =(str, f, role) => str.replace(/rgba?\([^)]*\)/g, (m) => f(parseColor(m), role, false) || m);

  function placeInSlots(items) {
    // items: elements in their new order; each goes to the original slot of the item at that index
    const orig = items.map((el) => {
      let marker = S.moved.find((m) => m.el === el)?.marker;
      if (!marker) {
        marker = document.createComment('uxui-slot');
        el.before(marker);
        S.moved.push({ marker, el });
      }
      return marker;
    });
    // order original markers by document position
    const slots = [...orig].sort((a, b) => (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1));
    items.forEach((el, k) => slots[k].after(el));
  }
  function setText(el, text) {
    const nodes = textNodes(el);
    if (!nodes.length) return;
    S.texts.push({ node: nodes[0], data: nodes[0].data });
    nodes[0].data = text;
    for (const n of nodes.slice(1)) {
      if (n.parentElement.closest('[aria-hidden=true]')) continue;
      S.texts.push({ node: n, data: n.data });
      n.data = '';
    }
  }

  function apply(variant = {}) {
    const t0 = performance.now();
    reset();
    if (!S.last) scaffold();
    const v = variant;
    const base = capture();
    ensureKillTransitions();
    document.documentElement.classList.add('uxui-active');
    const f = colorFn(v);
    const invert = (v.mode === 'dark' && !S.pageDark) || (v.mode === 'light' && S.pageDark);
    const d = v.density && v.density !== 1 ? +v.density : null;
    const fs = v.font_scale && v.font_scale !== 1 ? +v.font_scale : null;
    const scope = v.scope ? document.querySelector(v.scope) : null;
    if (f || d || fs) {
      for (const r of base.recs) {
        if (scope && !scope.contains(r.el)) continue;
        const el = r.el;
        if (f) {
          const keep = r.surface && r.surface !== r && isChromatic(r.surface.bg);
          const keepFill = isChromatic(r.bg);
          let c = f(r.color, 'text', keep || keepFill);
          if (c && invert && r.surface !== undefined) {
            // keep the text's contrast on its remapped surface: at least what it had, up to 4.5
            const s0 = r.surface ? over(r.surface.bg, [255, 255, 255, 1]) : [255, 255, 255, 1];
            const s1 = r.surface ? parseColor(f(r.surface.bg, 'bg', isChromatic(r.surface.bg)) || toCss(s0)) : parseColor(f([255, 255, 255, 1], 'bg', false));
            c = ensureContrast(c, over(s1, [0, 0, 0, 1]), Math.min(4.6, contrast(over(r.color, s0), s0)));
          }
          if (c) setp(el, 'color', c);
          const b = r.bg[3] > 0 ? f(r.bg, 'bg', keepFill) : null;
          if (b) setp(el, 'background-color', b);
          r.borders.forEach((bc, i) => {
            const m = bc && f(bc, 'line', keep);
            if (m) setp(el, `border-${['top', 'right', 'bottom', 'left'][i]}-color`, m);
          });
          if (r.fill) {
            const m = f(r.fill, 'text', keep);
            if (m) setp(el, 'fill', m);
          }
          if (r.stroke) {
            const m = f(r.stroke, 'text', keep);
            if (m) setp(el, 'stroke', m);
          }
          if (r.bgImage) setp(el, 'background-image', mapColorsIn(r.bgImage, f, 'bg'));
          if (r.shadow) setp(el, 'box-shadow', mapColorsIn(r.shadow, f, 'line'));
        }
        if (d) {
          ['top', 'right', 'bottom', 'left'].forEach((s, i) => r.pad[i] && setp(el, 'padding-' + s, r.pad[i] * d + 'px'));
          if (r.margin[0]) setp(el, 'margin-top', r.margin[0] * d + 'px');
          if (r.margin[1]) setp(el, 'margin-bottom', r.margin[1] * d + 'px');
          if (r.gap) {
            if (r.gap[0]) setp(el, 'row-gap', r.gap[0] * d + 'px');
            if (r.gap[1]) setp(el, 'column-gap', r.gap[1] * d + 'px');
          }
        }
        if (fs) {
          setp(el, 'font-size', r.fontSize * fs + 'px');
          if (r.lineHeight) setp(el, 'line-height', r.lineHeight * fs + 'px');
        }
      }
      if (f) {
        const htmlBg = base.html[3] > 0 ? over(base.html, [255, 255, 255, 1]) : [255, 255, 255, 1];
        setp(document.documentElement, 'background-color', f(htmlBg, 'bg', false) || toCss(htmlBg));
        setp(document.documentElement, 'color-scheme', v.mode === 'dark' ? 'dark' : 'light');
      }
    }
    if (v.max_width && v.max_width !== 'original') {
      const value = v.max_width === 'none' ? 'none' : v.max_width + 'px';
      const containers = base.recs.filter((r) => r.maxWidth && r.maxWidth >= 480 && r.width >= Math.min(innerWidth, r.maxWidth) * 0.6);
      if (containers.length) for (const r of containers) setp(r.el, 'max-width', value);
      else if (S.sections[0]) {
        const root = S.sections[0].el.parentElement;
        setp(root, 'max-width', value);
        setp(root, 'margin-left', 'auto');
        setp(root, 'margin-right', 'auto');
      }
    }
    const byId = (id) => S.sections.find((s) => s.id === id)?.el;
    if (v.section_order && v.section_order !== 'original' && S.sections.length > 1) {
      let ids = S.sections.map((s) => s.id);
      if (v.section_order === 'reverse') ids = ids.reverse();
      else if (Array.isArray(v.section_order)) ids = [...v.section_order, ...ids.filter((i) => !v.section_order.includes(i))];
      placeInSlots(ids.map(byId).filter(Boolean));
    }
    for (const id of v.hide || []) {
      const el = byId(id);
      if (el) setp(el, 'display', 'none');
    }
    for (const op of v.ops || []) {
      if (op.op === 'order') placeInSlots(op.items.map((s) => document.querySelector(s)).filter(Boolean));
      else if (op.op === 'hide') document.querySelectorAll(op.sel).forEach((el) => setp(el, 'display', 'none'));
      else if (op.op === 'text') document.querySelectorAll(op.sel).forEach((el) => setText(el, op.text));
      else if (op.op === 'style') document.querySelectorAll(op.sel).forEach((el) => Object.entries(op.props).forEach(([p, val]) => setp(el, p, String(val))));
    }
    S.variant = v;
    return { ms: +(performance.now() - t0).toFixed(2), touched: S.touched.size };
  }

  // ---------- measure ----------
  function measure(opts = {}) {
    const t0 = performance.now();
    scrollTo(0, 0);
    const vw = innerWidth, vh = innerHeight;
    const fails = [];
    // primary action
    const p = opts.primary ? document.querySelector(opts.primary) : S.primary;
    let primaryTop = null, primaryIn = false;
    if (p && visible(p)) {
      const r = p.getBoundingClientRect();
      primaryTop = Math.round(r.top + scrollY);
      primaryIn = r.top >= 0 && r.left >= 0 && r.bottom <= vh && r.right <= vw;
    }
    if (!primaryIn) fails.push(p ? (primaryTop === null ? 'primary_hidden' : 'primary_in_view') : 'primary_missing');
    // horizontal scroll
    const sw = document.documentElement.scrollWidth;
    if (sw > vw + 1) fails.push('horizontal_scroll');
    // text: contrast and words before the primary action
    let contrastMin = 99, contrastFail = 0, cantTell = 0, wordsBefore = 0, textChecked = 0;
    const samples = [];
    const bgMemo = new Map();
    const surface = (el) => {
      if (bgMemo.has(el)) return bgMemo.get(el);
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
      bgMemo.set(el, res);
      return res;
    };
    const seen = new Map();
    for (const t of textNodes()) {
      const el = t.parentElement;
      const r = textRect(t);
      if (r.width * r.height < 4) continue;
      if (!seen.has(el)) seen.set(el, visible(el));
      else if (!seen.get(el)) continue;
      else {
        if (p && primaryTop !== null && r.top + scrollY < primaryTop - 1 && r.right > 0 && r.left < vw && !p.contains(t)) wordsBefore += words(t.data);
        continue;
      }
      if (!seen.get(el)) continue;
      if (p && primaryTop !== null && r.top + scrollY < primaryTop - 1 && r.right > 0 && r.left < vw && !p.contains(t)) wordsBefore += words(t.data);
      const cs = getComputedStyle(el);
      const bg = surface(el);
      if (bg === null) {
        cantTell++;
        continue;
      }
      const fg = over(parseColor(cs.color), bg);
      const ratio = contrast(fg, bg);
      const size = parseFloat(cs.fontSize), bold = +cs.fontWeight >= 700;
      const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
      textChecked++;
      if (ratio < contrastMin) contrastMin = ratio;
      if (ratio < need - 0.005) {
        contrastFail++;
        if (samples.length < 5) samples.push({ sel: selectorOf(el), ratio: +ratio.toFixed(2), fg: toHex(fg), bg: toHex(bg), text: clean(t.data).slice(0, 40) });
      }
    }
    if (contrastFail) fails.push('contrast');
    // interactive elements
    let unnamed = 0, above = 0, small = 0;
    for (const el of document.querySelectorAll(INTERACTIVE)) {
      if (el.closest('#' + HOST_ID) || !visible(el)) continue;
      if (!accName(el)) unnamed++;
      const r = el.getBoundingClientRect();
      if (r.bottom > 0 && r.top < vh && r.right > 0 && r.left < vw) above++;
      if (r.width < 24 || r.height < 24) small++;
    }
    if (unnamed) fails.push('name_missing');
    // required nodes (for example the target of each journey's step): present and rendered
    for (const req of opts.require || []) {
      const el = document.querySelector(req.sel);
      if (!el || !visible(el)) fails.push('required:' + req.name);
    }
    let regions = 0;
    for (const x of [...S.regions, ...S.sections]) {
      if (!visible(x.el)) continue;
      const r = x.el.getBoundingClientRect();
      if (r.bottom > 0 && r.top < vh) regions++;
    }
    return {
      viewport: vw + 'x' + vh,
      fails,
      metrics: {
        words_before_action: wordsBefore,
        primary_top_px: primaryTop,
        interactive_above_fold: above,
        regions_above_fold: regions,
        contrast_min: contrastMin === 99 ? null : +contrastMin.toFixed(2),
        contrast_failures: contrastFail,
        contrast_cant_tell: cantTell,
        unnamed_interactive: unnamed,
        targets_under_24px: small,
        page_height: document.documentElement.scrollHeight,
        text_checked: textChecked,
      },
      contrast_samples: samples,
      ms: +(performance.now() - t0).toFixed(2),
    };
  }

  window.__uxui = {
    version: 1,
    scaffold,
    apply,
    reset: () => (reset(), true),
    measure,
    get variant() {
      return S.variant;
    },
    get state() {
      return S;
    },
    color: { parseColor, oklch, fromOklch, toHex, contrast },
    selectorOf,
    panel: () => {
      if (window.__uxuiPanel) return window.__uxuiPanel();
      console.warn('uxui: panel.js is not loaded');
    },
  };
})();
