// What the page uses, for the design-token checks (`tokens.rs`): the declared custom properties as
// the root holds them in this mode, and over every element the page draws (`checkVisibility` with
// `visibilityProperty`) its font size and text colour (an element with text of its own), its
// background and border colours, its paddings, the gap to the sibling box before it, and its corner
// radii. Each distinct value is returned once, with how many times it is used and the first node
// that uses it. What is off the tokens is decided in Rust, not here.
//
// Colours are sRGB [r, g, b, a] (0-255, alpha 0-1), indexes into `colours`: `rgb()` and `rgba()`
// as the computed style writes them, `color(srgb ...)` read by number, any other colour drawn on a
// 1 x 1 canvas and read back. A fully transparent colour is not used. The background of text is
// the background colours of the element and its ancestors composited down to the first opaque
// one; it is unknown (-1) when a background image lies in between or no opaque one is reached.
//
// properties: [pattern, [[name, colour | -1, text]]]; sizes: [px, count, node];
// usage: [colour, property, count, node]; text: [colour, background | -1, px, weight, count, node];
// gaps: [px, 'gap' | 'padding', count, node]; radii: [px, full, count, node], `full` when the
// radius is at least half the box's shorter side.
(function (patterns) {
  const cache = new Map();
  const st = (e) => {
    let s = cache.get(e);
    if (!s) {
      s = getComputedStyle(e);
      cache.set(e, s);
    }
    return s;
  };
  const round = (x) => Math.round(x * 100) / 100;
  const colours = [];
  const colourIndex = new Map();
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  const RGB = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)\s*(?:[,/]\s*([\d.]+)(%?)\s*)?\)$/;
  const SRGB = /^color\(srgb\s+([\d.e-]+)\s+([\d.e-]+)\s+([\d.e-]+)\s*(?:\/\s*([\d.]+)(%?)\s*)?\)$/;
  const alpha = (a, pct) => (a === undefined ? 1 : pct ? parseFloat(a) / 100 : parseFloat(a));
  const parse = (text) => {
    const t = (text || '').trim();
    if (!t) return null;
    let m = RGB.exec(t);
    if (m) return [+m[1], +m[2], +m[3], alpha(m[4], m[5])];
    m = SRGB.exec(t);
    if (m) return [m[1] * 255, m[2] * 255, m[3] * 255, alpha(m[4], m[5])];
    if (!CSS.supports('color', t)) return null;
    ctx.clearRect(0, 0, 1, 1);
    ctx.fillStyle = '#000';
    ctx.fillStyle = t;
    ctx.fillRect(0, 0, 1, 1);
    const d = ctx.getImageData(0, 0, 1, 1).data;
    return [d[0], d[1], d[2], Math.round((d[3] / 255) * 1000) / 1000];
  };
  const index = (c) => {
    const key = c.map((x) => Math.round(x * 1000) / 1000).join(',');
    let i = colourIndex.get(key);
    if (i === undefined) {
      i = colours.length;
      colourIndex.set(key, i);
      colours.push(c.map((x) => Math.round(x * 1000) / 1000));
    }
    return i;
  };
  const nodes = [];
  const nodeIndex = new Map();
  const desc = (el) => {
    let s = el.tagName.toLowerCase();
    if (el.id) s += '#' + el.id;
    const cls = [...el.classList].filter((c) => !/_[A-Za-z0-9-]{4}$/.test(c)).slice(0, 2);
    if (cls.length) s += '.' + cls.join('.');
    const t = (el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 40);
    return t && el !== document.documentElement && el !== document.body ? s + ' "' + t + '"' : s;
  };
  const node = (el) => {
    let i = nodeIndex.get(el);
    if (i === undefined) {
      i = nodes.length;
      nodeIndex.set(el, i);
      nodes.push(desc(el));
    }
    return i;
  };
  // Distinct keys in the order first seen: key -> [row without count and node, count, node].
  const table = () => new Map();
  const count = (t, key, row, el) => {
    const e = t.get(key);
    if (e) e[1] += 1;
    else t.set(key, [row, 1, node(el)]);
  };
  const out = (t) => [...t.values()].map(([row, n, i]) => [...row, n, i]);

  // The declared custom properties, as the root holds them.
  const rootStyle = getComputedStyle(document.documentElement);
  let names = null;
  const allNames = () => {
    if (names) return names;
    const set = new Set();
    for (let i = 0; i < rootStyle.length; i++) if (rootStyle[i].startsWith('--')) set.add(rootStyle[i]);
    const walk = (rules) => {
      for (const r of rules) {
        if (r.style) for (let i = 0; i < r.style.length; i++) if (r.style[i].startsWith('--')) set.add(r.style[i]);
        if (r.cssRules) walk(r.cssRules);
      }
    };
    for (const sheet of document.styleSheets) {
      let rules;
      try {
        rules = sheet.cssRules;
      } catch (e) {
        continue;
      }
      walk(rules);
    }
    names = [...set].sort();
    return names;
  };
  const read = (name) => {
    const text = rootStyle.getPropertyValue(name).trim();
    const c = text ? parse(text) : null;
    return [name, c ? index(c) : -1, text];
  };
  const properties = patterns.map((p) =>
    p.endsWith('*')
      ? [p, allNames().filter((n) => n.startsWith(p.slice(0, -1))).map(read).filter((e) => e[2] !== '')]
      : [p, [read(p)]]
  );

  // The background under an element: -1 unknown, else a colour index; memoised per element.
  const backs = new Map();
  const back = (el) => {
    if (!el) return -1;
    if (backs.has(el)) return backs.get(el);
    const s = st(el);
    let b;
    if (s.backgroundImage && s.backgroundImage !== 'none') b = -1;
    else {
      const c = parse(s.backgroundColor);
      if (c && c[3] >= 1) b = index(c);
      else {
        const under = back(el.parentElement);
        if (under < 0 || !c || c[3] <= 0) b = under;
        else {
          const u = colours[under];
          const a = c[3];
          b = index([c[0] * a + u[0] * (1 - a), c[1] * a + u[1] * (1 - a), c[2] * a + u[2] * (1 - a), 1]);
        }
      }
    }
    backs.set(el, b);
    return b;
  };

  const sizes = table();
  const usage = table();
  const text = table();
  const gaps = table();
  const radii = table();
  const use = (el, value, property) => {
    const c = parse(value);
    if (!c || c[3] <= 0) return;
    const i = index(c);
    count(usage, i + '|' + property, [i, property], el);
  };
  const ownText = (el) => {
    for (const n of el.childNodes) if (n.nodeType === 3 && n.textContent.trim()) return true;
    return false;
  };
  const SIDES = ['Top', 'Right', 'Bottom', 'Left'];
  const CORNERS = ['TopLeft', 'TopRight', 'BottomRight', 'BottomLeft'];
  const boxed = (s) => {
    const d = s.display;
    return d !== 'inline' && d !== 'contents' && d !== 'none' && !d.startsWith('table-') && d !== 'table-cell' &&
      s.position !== 'absolute' && s.position !== 'fixed';
  };
  const all = [document.documentElement, document.body, ...document.body.querySelectorAll('*')];
  let elements = 0;
  for (const el of all) {
    if (el.closest('svg, script, style, noscript, template, head') && el.localName !== 'svg') continue;
    if (el !== document.documentElement && !el.checkVisibility({ visibilityProperty: true })) continue;
    elements += 1;
    const s = st(el);
    const rect = el.getBoundingClientRect();
    if (ownText(el)) {
      const fg = parse(s.color);
      if (fg && fg[3] > 0) {
        const px = round(parseFloat(s.fontSize));
        const weight = parseInt(s.fontWeight, 10) || 400;
        count(sizes, String(px), [px], el);
        const f = index(fg);
        use(el, s.color, 'color');
        const b = back(el);
        count(text, [f, b, px, weight].join('|'), [f, b, px, weight], el);
      }
    }
    use(el, s.backgroundColor, 'background-color');
    for (const side of SIDES) {
      const w = parseFloat(s['border' + side + 'Width']);
      const style = s['border' + side + 'Style'];
      if (w > 0 && style !== 'none' && style !== 'hidden') use(el, s['border' + side + 'Color'], 'border-color');
    }
    for (const side of SIDES) {
      const p = round(parseFloat(s['padding' + side]));
      if (p > 0) count(gaps, p + '|padding', [p, 'padding'], el);
    }
    if (rect.width > 0 && rect.height > 0) {
      for (const corner of CORNERS) {
        const v = (s['border' + corner + 'Radius'] || '').trim().split(/\s+/)[0];
        let r = v.endsWith('%') ? (parseFloat(v) / 100) * rect.width : parseFloat(v);
        if (!(r > 0)) continue;
        const full = r >= Math.min(rect.width, rect.height) / 2;
        r = round(r);
        count(radii, r + '|' + full, [r, full], el);
      }
    }
    if (el !== document.documentElement && el !== document.body && boxed(s)) {
      // The box before it: the nearest sibling box the page draws, with no text and no drawn inline
      // element in between (a gap across them is not a gap between two boxes).
      let prev = null;
      for (let n = el.previousSibling; n; n = n.previousSibling) {
        if (n.nodeType === 3) {
          if (n.textContent.trim()) break;
          continue;
        }
        if (n.nodeType !== 1 || !n.checkVisibility({ visibilityProperty: true })) continue;
        if (boxed(st(n))) prev = n;
        break;
      }
      if (prev && rect.width > 0 && rect.height > 0) {
        const a = prev.getBoundingClientRect();
        let g = null;
        if (rect.top >= a.bottom - 0.5) g = rect.top - a.bottom;
        else if (rect.left >= a.right - 0.5) g = rect.left - a.right;
        else if (rect.right <= a.left + 0.5) g = a.left - rect.right;
        if (g !== null) {
          g = round(g);
          if (g > 0) count(gaps, g + '|gap', [g, 'gap'], el);
        }
      }
    }
  }
  return {
    elements,
    properties,
    colours,
    nodes,
    sizes: out(sizes),
    usage: out(usage),
    text: out(text),
    gaps: out(gaps),
    radii: out(radii),
  };
})
