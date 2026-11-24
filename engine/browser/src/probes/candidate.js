// One element that may be an always-visible item or a keyboard target, called with `this` bound to
// it: whether it lies inside the first match of `scope`, whether it is visible (rendered, not
// `visibility: hidden`, a box of some size), its box in viewport coordinates, and whether the
// element at the centre of that box is it or inside it.
function (scope) {
  const e = this;
  const r = e.getBoundingClientRect();
  const root = scope ? document.querySelector(scope) : null;
  const inScope = !scope || (!!root && root.contains(e));
  const cs = getComputedStyle(e);
  const rendered = typeof e.checkVisibility === 'function' ? e.checkVisibility() : true;
  const visible = rendered && cs.visibility === 'visible' && r.width > 0 && r.height > 0;
  const hitEl = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
  return {
    in_scope: inScope,
    visible,
    box: { x: r.x, y: r.y, width: r.width, height: r.height },
    hit: !!hitEl && (hitEl === e || e.contains(hitEl)),
  };
}
