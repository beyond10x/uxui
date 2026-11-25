// The element a journey's `click` or `type` step acts on: the first element `css` matches that is
// rendered (`checkVisibility()`, a box of some size). It is scrolled into view when part of it lies
// outside the viewport (at once, whatever the page's `scroll-behavior`). For `click` (focus false)
// answers the centre of its box, where the click goes, or an error when the element at that point
// is neither it nor inside it. For `type` (focus true) it is focused and its content selected, so
// that the typed text takes its place; an element that does not take focus is an error.
// Answers {x, y}, {focused: true} or {error}.
(function (css, focus) {
  let all;
  try {
    all = document.querySelectorAll(css);
  } catch (e) {
    return { error: 'is not a valid selector' };
  }
  const rendered = (e) => {
    if (typeof e.checkVisibility === 'function' && !e.checkVisibility()) return false;
    const r = e.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  };
  const el = Array.from(all).find(rendered);
  if (!el) return { error: all.length ? 'matches no rendered element' : 'matches no element' };
  const vw = document.documentElement.clientWidth;
  const vh = document.documentElement.clientHeight;
  let r = el.getBoundingClientRect();
  if (r.left < 0 || r.top < 0 || r.right > vw || r.bottom > vh) {
    el.scrollIntoView({ block: 'center', inline: 'center', behavior: 'instant' });
    r = el.getBoundingClientRect();
  }
  const desc = (e) => {
    let s = e.tagName.toLowerCase();
    if (e.id) s += '#' + e.id;
    const cls = Array.from(e.classList).slice(0, 2);
    if (cls.length) s += '.' + cls.join('.');
    return s;
  };
  if (focus) {
    el.focus();
    if (document.activeElement !== el) return { error: 'does not take focus' };
    if (typeof el.select === 'function') {
      el.select();
    } else if (el.isContentEditable) {
      const range = document.createRange();
      range.selectNodeContents(el);
      const sel = getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    }
    return { focused: true };
  }
  const x = r.left + r.width / 2;
  const y = r.top + r.height / 2;
  const at = document.elementFromPoint(x, y);
  if (!at || (at !== el && !el.contains(at))) {
    return { error: 'is covered at its centre by ' + (at ? desc(at) : 'nothing') };
  }
  return { x, y };
})
