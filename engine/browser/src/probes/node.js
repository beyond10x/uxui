// The render probe of one declared node: how many elements its selector matches, and the first
// one's box (CSS pixels, viewport coordinates) and computed values of the declared properties.
(function (selector, styles) {
  const all = document.querySelectorAll(selector);
  if (all.length === 0) return { matches: 0 };
  const e = all[0];
  const r = e.getBoundingClientRect();
  const cs = getComputedStyle(e);
  const values = {};
  for (const p of styles) values[p] = cs.getPropertyValue(p);
  return { matches: all.length, box: { x: r.x, y: r.y, width: r.width, height: r.height }, styles: values };
})
