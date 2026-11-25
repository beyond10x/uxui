// The element that has focus, as the focus-mark check reads it (`checks::focus_marked` decides):
// whether it matches `:focus-visible`, its computed outline style, width (px) and colour and its
// box-shadow, and a short description of it. Focus inside a shadow root is followed to its element.
(function () {
  let e = document.activeElement;
  while (e && e.shadowRoot && e.shadowRoot.activeElement) e = e.shadowRoot.activeElement;
  if (!e || e === document.body || e === document.documentElement) {
    return { element: 'none', focus_visible: false, outline_style: 'none', outline_width: 0, outline_color: '', box_shadow: 'none' };
  }
  let s = e.tagName.toLowerCase();
  if (e.id) s += '#' + e.id;
  const cls = [...e.classList].filter((c) => !/_[A-Za-z0-9-]{4}$/.test(c)).slice(0, 2);
  if (cls.length) s += '.' + cls.join('.');
  const t = (e.getAttribute('aria-label') || e.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 40);
  const cs = getComputedStyle(e);
  let visible = false;
  try {
    visible = e.matches(':focus-visible');
  } catch (_) {
    visible = false;
  }
  return {
    element: t ? s + ' "' + t + '"' : s,
    focus_visible: visible,
    outline_style: cs.outlineStyle,
    outline_width: parseFloat(cs.outlineWidth) || 0,
    outline_color: cs.outlineColor,
    box_shadow: cs.boxShadow,
  };
})()
