// The in-page probe of `ux-search campaign`, evaluated once in the declared page after it is
// ready, with the page's own scripts on. The colour, text and name functions are those of
// probes/landing.js; a text run and its contrast are counted as the loop round's audit counts
// them (one run per text node; a disabled control, a text over an image and a text whose colour
// has alpha 0 are not judged).
//
//   __uxuiCampaign.prepare(config)  keeps the declaration's selectors and sizes, sets the mode's
//                                   attribute when it has one, and counts what each selector
//                                   matches (-1: the browser does not read it)
//   __uxuiCampaign.apply(spec)      injects the variant's style sheet (spec.css) after the
//                                   page's own; null takes the injected sheet away. The run
//                                   calls it, parks the pointer, and then measures
//   __uxuiCampaign.measure(spec)    injects the same sheet when it is not there, checks every
//                                   property it sets (spec.checks), and returns the numbers of
//                                   one render, what the mode's expectation reads on the page,
//                                   the first elements behind four of the counts, and a digest
//                                   of every element's box and colours
//   __uxuiCampaign.still()          hides the caret before a screenshot, and lets what that
//                                   starts come to rest
(() => {
  if (window.__uxuiCampaign) return true;
  const STYLE_ID = 'uxui-campaign-css';
  const SKIP = new Set(['SCRIPT', 'STYLE', 'LINK', 'META', 'NOSCRIPT', 'TEMPLATE', 'HEAD', 'TITLE', 'BR', 'WBR', 'SOURCE', 'TRACK']);
  const INTERACTIVE =
    'a[href],button,input:not([type=hidden]),select,textarea,summary,[role=button],[role=link],[role=tab],' +
    '[role=menuitem],[role=checkbox],[role=switch],[role=radio],[role=combobox],[tabindex]:not([tabindex="-1"])';
  const TARGETS = 'a[href],button,summary,[role=button]';
  const BOXES = 'img,svg,video,canvas,iframe,input,select,textarea,button';
  let config = null;

  const clean = (s) => (s || '').replace(/\s+/g, ' ').trim();
  function visible(el) {
    if (!el.isConnected) return false;
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true, opacityProperty: true, visibilityProperty: true })) return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  }
  const all = (selector) => {
    try {
      return [...document.querySelectorAll(selector)];
    } catch {
      return null;
    }
  };

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

  // ---------- text and names ----------
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

  // ---------- naming an element ----------
  // An element as a note names it: its parent and itself as tag, id or the first two classes
  // (`main > p.pale`), then the first 40 characters of a text in quotes.
  const NAMED = 3;
  function short(el) {
    const tag = el.tagName.toLowerCase();
    if (el.id) return `${tag}#${el.id}`;
    const classes = clean(el.getAttribute('class')).split(' ').filter(Boolean).slice(0, 2);
    return classes.length ? `${tag}.${classes.join('.')}` : tag;
  }
  function named(el, text) {
    const parent = el.parentElement;
    const path = parent && parent !== document.body && parent !== document.documentElement ? `${short(parent)} > ${short(el)}` : short(el);
    const said = clean(text === undefined ? el.textContent : text).slice(0, 40);
    return said ? `${path} "${said}"` : path;
  }
  // The first NAMED of what was found, in document order: an element before what it holds.
  function first(found) {
    return found
      .sort((a, b) => (a.node === b.node ? 0 : a.node.compareDocumentPosition(b.node) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1))
      .slice(0, NAMED)
      .map((entry) => entry.says());
  }

  // ---------- what changes at all ----------
  // A digest of every element's box, rounded to px, and of the computed styles a theme's custom
  // properties usually end in (STYLED), for the element and for its `::before` and `::after`
  // when they are generated: a status dot is often one of those. Two renders of one page with
  // the same digest look the same in all of that; the run compares each value of an axis with
  // the shipped one by it. `opacity` and `transform` are left out: an animation without an end
  // moves them, and a digest that moves by itself says nothing about a variant.
  const STYLED = [
    'color', 'backgroundColor', 'backgroundImage', 'fontSize', 'fontWeight', 'borderRadius',
    'borderTopColor', 'borderRightColor', 'borderBottomColor', 'borderLeftColor', 'outlineColor',
    'boxShadow', 'fill', 'stroke',
  ];
  function digest() {
    let a = 0x811c9dc5, b = 0x9e3779b9;
    const feed = (s) => {
      for (let i = 0; i < s.length; i++) {
        const c = s.charCodeAt(i);
        a = Math.imul(a ^ c, 0x01000193);
        b = Math.imul(b ^ c, 0x85ebca6b);
      }
    };
    const styled = (cs) => STYLED.map((property) => cs[property]).join(';');
    for (const el of document.querySelectorAll('*')) {
      if (SKIP.has(el.tagName)) continue;
      const r = el.getBoundingClientRect();
      feed(`${Math.round(r.left)},${Math.round(r.top)},${Math.round(r.width)},${Math.round(r.height)};${styled(getComputedStyle(el))}|`);
      for (const pseudo of ['::before', '::after']) {
        const cs = getComputedStyle(el, pseudo);
        if (cs.content === 'none' || cs.content === 'normal') continue;
        feed(`${pseudo};${cs.width};${cs.height};${styled(cs)}|`);
      }
    }
    return (a >>> 0).toString(16).padStart(8, '0') + (b >>> 0).toString(16).padStart(8, '0');
  }

  // A change of the style sheet may start transitions; each is taken to its end, so a render
  // is measured in the state it comes to rest in.
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
  const frames = () => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));

  // The variant: its sheet is the last element of the head, so of two rules of the same weight
  // the variant's is in force. An empty sheet is still injected: the page as shipped is
  // measured through the same path as every other variant.
  function inject(spec) {
    let style = document.getElementById(STYLE_ID);
    if (spec === null) {
      style?.remove();
      return;
    }
    if (!style) {
      style = document.createElement('style');
      style.id = STYLE_ID;
    }
    if (document.head.lastElementChild !== style) document.head.append(style);
    if (style.textContent !== spec.css) style.textContent = spec.css;
  }

  // Every property the sheet sets must be in force on every element its selector matches.
  // A value that refers to another property (`var(`) is not compared: the page computes it.
  function check(spec) {
    const failed = [];
    for (const c of spec ? spec.checks : []) {
      const found = all(c.selector);
      if (found === null) {
        failed.push({ key: c.key, why: `the browser does not read the selector \`${c.selector}\`` });
        continue;
      }
      if (!found.length) {
        failed.push({ key: c.key, why: `the selector \`${c.selector}\` matches nothing on the page, so \`${c.property}\` is set for nothing` });
        continue;
      }
      if (c.value.includes('var(')) continue;
      for (const el of found) {
        const got = getComputedStyle(el).getPropertyValue(c.property).trim();
        if (got === c.value) continue;
        failed.push({ key: c.key, why: `\`${c.property}\` of \`${c.selector}\` is \`${got}\` on the page, not \`${c.value}\`: another rule of the page is in force` });
        break;
      }
    }
    return failed;
  }

  // ---------- one render ----------
  async function measure(spec) {
    inject(spec);
    settle();
    await document.fonts.ready;
    await frames();
    settle();
    scrollTo(0, 0);
    const failed = check(spec);
    const vw = innerWidth, vh = innerHeight;
    const notes = [];
    // The box an element may show in: the viewport, cut by every box around it that clips or
    // scrolls. An element that leaves it is cut off or needs scrolling to be reached.
    let clips = new Map();
    const clip = (el) => {
      if (!el || el === document.documentElement || el === document.body) return { left: 0, top: 0, right: vw, bottom: vh };
      if (clips.has(el)) return clips.get(el);
      let box = clip(el.parentElement);
      const cs = getComputedStyle(el);
      const x = cs.overflowX !== 'visible', y = cs.overflowY !== 'visible';
      if (x || y) {
        const r = el.getBoundingClientRect();
        box = { ...box };
        if (x) {
          box.left = Math.max(box.left, r.left + el.clientLeft);
          box.right = Math.min(box.right, r.left + el.clientLeft + el.clientWidth);
        }
        if (y) {
          box.top = Math.max(box.top, r.top + el.clientTop);
          box.bottom = Math.min(box.bottom, r.top + el.clientTop + el.clientHeight);
        }
      }
      clips.set(el, box);
      return box;
    };
    const inView = (el) => {
      const r = el.getBoundingClientRect(), box = clip(el.parentElement);
      return r.top >= box.top - 1 && r.left >= box.left - 1 && r.bottom <= box.bottom + 1 && r.right <= box.right + 1;
    };

    const primary = (all(config.primary) || []).find(visible) || null;
    let primaryTop = null, primaryBottom = null, primaryIn = false;
    if (primary) {
      const r = primary.getBoundingClientRect();
      primaryTop = Math.max(0, Math.round(r.top + scrollY));
      primaryBottom = Math.max(0, Math.round(r.bottom + scrollY));
      primaryIn = inView(primary);
    }
    if (!primaryIn) {
      const r = primary ? primary.getBoundingClientRect() : null;
      const box = primary ? clip(primary.parentElement) : null;
      notes.push('primary_in_view: ' + (primary ? `the primary action ends at ${Math.round(r.right)} x ${Math.round(r.bottom)} px, the box it shows in at ${Math.round(box.right)} x ${Math.round(box.bottom)}` : 'no visible primary action'));
    }
    const scrollWidth = document.documentElement.scrollWidth;
    const hScroll = scrollWidth > vw + 1;
    if (hScroll) notes.push(`no_h_scroll: the page is ${scrollWidth} px wide in a viewport of ${vw}`);

    let unnamed = 0;
    for (const el of document.querySelectorAll(INTERACTIVE)) {
      if (!visible(el) || accName(el)) continue;
      unnamed++;
      if (unnamed <= 3) notes.push(`named: no accessible name: <${el.tagName.toLowerCase()} class="${el.className}">`);
    }

    // The first elements behind a count, per count.
    const elements = {};
    const keep = (term, found) => {
      if (found.length) elements[term] = first(found);
    };

    let small = 0;
    const smalls = [];
    for (const el of document.querySelectorAll(TARGETS)) {
      if (!visible(el)) continue;
      const b = el.getBoundingClientRect();
      if (b.width < config.small_target_px || b.height < config.small_target_px) {
        small++;
        if (smalls.length < NAMED) smalls.push({ node: el, says: () => `${named(el)}: ${Math.round(b.width)} x ${Math.round(b.height)} px` });
      }
    }
    keep('small_targets', smalls);

    // The rows of a list that lie fully in view at the top of the page.
    let rowsInView = 0;
    if (config.rows !== null)
      for (const el of all(config.rows) || []) {
        if (visible(el) && inView(el)) rowsInView++;
      }

    let regions = 0;
    if (config.regions !== null)
      for (const el of all(config.regions) || []) {
        if (visible(el) && (primaryTop === null || el.getBoundingClientRect().top + scrollY <= primaryTop + 1)) regions++;
      }

    // The surface a text is composited on; null when an image lies under it.
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

    let runs = 0, under14 = 0, lowContrast = 0, overflow = 0;
    const pale = [], cut = [];
    const shown = new Map();
    for (const t of textNodes()) {
      const el = t.parentElement;
      if (!shown.has(el)) shown.set(el, visible(el) && !el.closest('[aria-hidden=true]'));
      if (!shown.get(el)) continue;
      const r = textRect(t);
      if (r.width <= 0 || r.height <= 0) continue;
      runs++;
      const cs = getComputedStyle(el);
      const size = parseFloat(cs.fontSize);
      if (size < 14) under14++;
      const within = clip(el).right;
      if (r.right > within + 1) {
        overflow++;
        cut.push({ node: t, says: () => `${named(el, t.data)}: the text ends at ${Math.round(r.right)} px, the box it shows in at ${Math.round(within)}` });
      }
      if (el.closest(':disabled')) continue;
      const bg = surface(el);
      if (bg === null) continue;
      // A colour of alpha 0 paints nothing: the page shows something else in the text's place,
      // an image or a mask, or nothing. No contrast of the text is there to judge.
      const ink = parseColor(cs.color);
      if (ink[3] === 0) continue;
      const fg = over(ink, bg);
      const ratio = contrast(fg, bg);
      const need = size >= 24 || (+cs.fontWeight >= 700 && size >= 18.66) ? 3 : 4.5;
      if (ratio < need - 0.005) {
        lowContrast++;
        if (lowContrast <= 3) notes.push(`contrast: ${ratio.toFixed(2)} under ${need}: ${toHex(fg)} on ${toHex(bg)}, "${clean(t.data).slice(0, 40)}"`);
        if (pale.length < NAMED) pale.push({ node: t, says: () => `${named(el, t.data)}: ${ratio.toFixed(2)} under ${need}, ${toHex(fg)} on ${toHex(bg)}` });
      }
    }
    for (const el of document.querySelectorAll(BOXES)) {
      if (!visible(el)) continue;
      const right = el.getBoundingClientRect().right, within = clip(el.parentElement).right;
      if (right > within + 1) {
        overflow++;
        cut.push({ node: el, says: () => `${named(el)}: ends at ${Math.round(right)} px, the box it shows in at ${Math.round(within)}` });
      }
    }
    keep('low_contrast', pale);
    keep('overflow', cut);

    // An item is there when one of its matches lies fully in the box it shows in, at the top of
    // the page and again at its end. An item with a container is there as well when it is
    // clipped by that container alone: the container (the item itself or the nearest element
    // around it that the selector matches) clips or scrolls, lies fully in view, and the part of
    // the item inside the container's scroll port is not empty and is cut by nothing between the
    // two.
    const clippedOnlyBy = (el, selector) => {
      const box = el.closest(selector);
      if (!box || !visible(box) || !inView(box)) return false;
      const cs = getComputedStyle(box);
      if (cs.overflowX === 'visible' && cs.overflowY === 'visible') return false;
      const b = box.getBoundingClientRect(), r = el.getBoundingClientRect();
      const left = b.left + box.clientLeft, top = b.top + box.clientTop;
      const part = {
        left: Math.max(r.left, left),
        top: Math.max(r.top, top),
        right: Math.min(r.right, left + box.clientWidth),
        bottom: Math.min(r.bottom, top + box.clientHeight),
      };
      if (part.right <= part.left || part.bottom <= part.top) return false;
      for (let at = el.parentElement; at && at !== box; at = at.parentElement) {
        const s = getComputedStyle(at);
        if (s.overflowX === 'visible' && s.overflowY === 'visible') continue;
        const a = at.getBoundingClientRect();
        const l = a.left + at.clientLeft, t = a.top + at.clientTop;
        if (part.left < l - 1 || part.top < t - 1 || part.right > l + at.clientWidth + 1 || part.bottom > t + at.clientHeight + 1) return false;
      }
      return true;
    };
    let misses = 0;
    if (config.always_visible.length) {
      const there = () => {
        clips = new Map();
        return config.always_visible.map((item) =>
          (all(item.selector) || []).some((el) => visible(el) && (inView(el) || (item.container !== null && clippedOnlyBy(el, item.container)))),
        );
      };
      const top = there();
      let end = top;
      const height = document.documentElement.scrollHeight;
      if (height > vh + 1) {
        scrollTo(0, height);
        end = there();
        scrollTo(0, 0);
      }
      misses = top.filter((at, i) => !at || !end[i]).length;
      // The items in the declaration's order, not the document's: an item is a name.
      const missed = [];
      config.always_visible.forEach((item, i) => {
        if (top[i] && end[i]) return;
        const where = top[i] ? 'at the end of the page' : end[i] ? 'at the top of the page' : 'at the top of the page and at its end';
        if (missed.length < NAMED) missed.push(`${item.name} (\`${item.selector}\`): not in view ${where}`);
      });
      if (missed.length) elements.always_visible_misses = missed;
    }

    // What the mode expects of the page, as the page has it now.
    let expect = null;
    if (config.expect) {
      const found = config.expect.selector === null ? null : all(config.expect.selector);
      expect = {
        attribute: config.expect.attribute === null ? null : document.documentElement.getAttribute(config.expect.attribute.name),
        selector: config.expect.selector === null ? null : found === null ? -1 : found.length,
      };
    }

    return {
      primary_in_view: primaryIn,
      h_scroll: hScroll,
      unnamed: unnamed,
      primary_px: primaryTop === null ? document.documentElement.scrollHeight : primaryTop,
      small_targets: small,
      regions: regions,
      text_runs: runs,
      text_under_14: under14,
      low_contrast: lowContrast,
      overflow: overflow,
      always_visible_misses: misses,
      rows_in_view: rowsInView,
      primary_bottom_px: primaryBottom === null ? document.documentElement.scrollHeight : primaryBottom,
      fonts: document.fonts.status,
      attribute: config.attribute ? document.documentElement.getAttribute(config.attribute.name) : null,
      expect,
      elements,
      digest: digest(),
      failed,
      notes,
    };
  }

  function apply(spec) {
    inject(spec);
    return true;
  }

  // Before a screenshot: a caret blinks in a field the page focused, on in one shot and off in
  // the next, so it is hidden. Nothing else of the page is changed, and no number is measured
  // after it.
  const STILL_ID = 'uxui-campaign-still';
  async function still() {
    if (!document.getElementById(STILL_ID)) {
      const style = document.createElement('style');
      style.id = STILL_ID;
      style.textContent = '*, *::before, *::after { caret-color: transparent !important; }';
      document.head.append(style);
    }
    settle();
    await frames();
    settle();
    return true;
  }

  // `config`: primary, regions (or null), rows (or null), always_visible (name, selector,
  // container or null), small_target_px,
  // attribute (name, value; or null), expect (attribute: name, value, or null; selector or
  // null; or null).
  function prepare(cfg) {
    config = cfg;
    if (cfg.attribute) document.documentElement.setAttribute(cfg.attribute.name, cfg.attribute.value);
    const count = (selector) => {
      const found = all(selector);
      return found === null ? -1 : found.length;
    };
    return {
      width: innerWidth,
      height: innerHeight,
      primary: count(cfg.primary),
      regions: cfg.regions === null ? null : count(cfg.regions),
      rows: cfg.rows === null ? null : count(cfg.rows),
      always_visible: cfg.always_visible.map((item) => count(item.selector)),
      containers: cfg.always_visible.map((item) => (item.container === null ? null : count(item.container))),
    };
  }

  window.__uxuiCampaign = { prepare, apply, measure, still };
  return true;
})()
