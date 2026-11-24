// The geometry the overflow and page-scroll checks read (`checks.rs`): the viewport's client width,
// the page's scroll width, every non-blank text run and every replaced or wide element with the
// context it sits in and its block, and each block's right edge. What counts as overflow is decided
// in Rust, not here.
//
// items: [right, width, height, context, block]; context 0 flow, 1 fixed, 2 horizontal scroll
// container, 3 clipping box; block indexes `blocks` ([desc, right]), -1 outside the flow.
(function () {
  const vw = document.documentElement.clientWidth;
  const se = document.scrollingElement || document.documentElement;
  const cache = new Map();
  const st = (e) => {
    let s = cache.get(e);
    if (!s) {
      s = getComputedStyle(e);
      cache.set(e, s);
    }
    return s;
  };
  const desc = (el) => {
    let s = el.tagName.toLowerCase();
    if (el.id) s += '#' + el.id;
    const cls = [...el.classList].filter((c) => !/_[A-Za-z0-9-]{4}$/.test(c)).slice(0, 2);
    if (cls.length) s += '.' + cls.join('.');
    const t = (el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 40);
    return t ? s + ' "' + t + '"' : s;
  };
  const context = (e) => {
    for (let a = e; a && a !== document.body && a !== document.documentElement; a = a.parentElement) {
      const s = st(a);
      if (s.position === 'fixed') return 1;
      if (s.overflowX === 'auto' || s.overflowX === 'scroll') return 2;
      if (s.overflowX === 'hidden' || s.overflowX === 'clip') return 3;
    }
    return 0;
  };
  const blockOf = (e) => {
    for (let a = e; a && a !== document.body; a = a.parentElement) {
      const d = st(a).display;
      if (!d.startsWith('inline') && d !== 'contents') return a;
    }
    return document.body;
  };
  const blocks = [];
  const index = new Map();
  const blockIndex = (b) => {
    let i = index.get(b);
    if (i === undefined) {
      i = blocks.length;
      index.set(b, i);
      blocks.push([desc(b), b.getBoundingClientRect().right]);
    }
    return i;
  };
  const items = [];
  const add = (rect, from) => {
    const c = context(from);
    items.push([rect.right, rect.width, rect.height, c, c === 0 ? blockIndex(blockOf(from)) : -1]);
  };
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const range = document.createRange();
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const p = n.parentElement;
    if (!p || !(n.textContent || '').trim() || st(p).visibility === 'hidden') continue;
    if (p.closest('svg, script, style, noscript, template')) continue;
    range.selectNodeContents(n);
    add(range.getBoundingClientRect(), p);
  }
  for (const el of document.body.querySelectorAll('img, svg, video, canvas, iframe, input, select, textarea, table, pre')) {
    if (el.parentElement && el.parentElement.closest('svg')) continue;
    if (st(el).visibility === 'hidden') continue;
    add(el.getBoundingClientRect(), el.parentElement || el);
  }
  return { vw, scroll_width: se.scrollWidth, blocks, items };
})
