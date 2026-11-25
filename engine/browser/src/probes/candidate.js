// One element that may be an always-visible item or a keyboard target, called with `this` bound to
// it: whether it lies inside the first match of `scope`, whether it is visible (rendered, not
// `visibility: hidden`, a box of some size), its box in viewport coordinates, whether the element at
// each of five points of that box is it or inside it (`hit`: at every point; `missed`: the names of
// the points where it is not), and its `aria-current` attribute (null without one).
//
// The points: the centre, then the top left, top right, bottom right and bottom left corners, each
// moved in along both axes by 1 px plus r (1 - 1/sqrt(2)), r the corner's used radius along that
// axis. Hit testing follows `border-radius`, and that puts the point inside the corner's curve. No
// point is moved past the centre.
function (scope) {
  const e = this;
  const r = e.getBoundingClientRect();
  const root = scope ? document.querySelector(scope) : null;
  const inScope = !scope || (!!root && root.contains(e));
  const cs = getComputedStyle(e);
  const rendered = typeof e.checkVisibility === 'function' ? e.checkVisibility() : true;
  const visible = rendered && cs.visibility === 'visible' && r.width > 0 && r.height > 0;
  // The radii of each corner as computed, horizontal and vertical: a length, or a percentage of
  // the box.
  const length = (t, side) => {
    const n = parseFloat(t) || 0;
    return t.endsWith('%') ? (n / 100) * side : n;
  };
  const [tl, tr, br, bl] = [
    cs.borderTopLeftRadius,
    cs.borderTopRightRadius,
    cs.borderBottomRightRadius,
    cs.borderBottomLeftRadius,
  ].map((v) => {
    const [h, w = h] = v.trim().split(/\s+/);
    return [length(h, r.width), length(w, r.height)];
  });
  // Used radii: all scaled by one factor when two on one side add up to more than that side
  // (CSS Backgrounds and Borders 3, "Overlapping Curves").
  const fit = (side, a, b) => (a + b > side ? side / (a + b) : 1);
  const f = Math.min(
    fit(r.width, tl[0], tr[0]),
    fit(r.width, bl[0], br[0]),
    fit(r.height, tl[1], bl[1]),
    fit(r.height, tr[1], br[1]),
  );
  const k = 1 - Math.SQRT1_2;
  const inset = ([h, v]) => [
    Math.min(1 + h * f * k, r.width / 2),
    Math.min(1 + v * f * k, r.height / 2),
  ];
  const [itl, itr, ibr, ibl] = [tl, tr, br, bl].map(inset);
  const right = r.left + r.width;
  const bottom = r.top + r.height;
  const points = [
    ['centre', r.left + r.width / 2, r.top + r.height / 2],
    ['top-left', r.left + itl[0], r.top + itl[1]],
    ['top-right', right - itr[0], r.top + itr[1]],
    ['bottom-right', right - ibr[0], bottom - ibr[1]],
    ['bottom-left', r.left + ibl[0], bottom - ibl[1]],
  ];
  const missed = points
    .filter(([, x, y]) => {
      const at = document.elementFromPoint(x, y);
      return !at || (at !== e && !e.contains(at));
    })
    .map(([name]) => name);
  return {
    in_scope: inScope,
    visible,
    box: { x: r.x, y: r.y, width: r.width, height: r.height },
    hit: missed.length === 0,
    missed,
    current: e.getAttribute('aria-current'),
  };
}
