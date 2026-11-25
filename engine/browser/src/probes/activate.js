// The activation of a focused target, in three calls (`measure.rs`; `checks::acted` decides):
// `watch` notes the focused element and its state and starts a quiet watch of the document;
// `mark` ends the quiet watch (a key is pressed next); `report` answers what changed since the last
// `mark` or `report`: focus moved to another element, the noted element's state changed, or the
// document changed on a node that did not change during the quiet watch. A change of the document
// is an element added or removed, or `hidden`, `open`, `aria-hidden` or `disabled` changed. Whether
// the page navigated is the browser's to say (its events), not this probe's.
(function (cmd) {
  const active = () => {
    let e = document.activeElement;
    while (e && e.shadowRoot && e.shadowRoot.activeElement) e = e.shadowRoot.activeElement;
    return e;
  };
  const STATE = ['aria-expanded', 'aria-pressed', 'aria-selected', 'aria-checked'];
  const state = (e) =>
    e ? STATE.map((a) => String(e.getAttribute(a))).concat([String(e.checked), String(e.value), String(e.open)]).join('|') : '';
  const ATTRS = ['hidden', 'open', 'aria-hidden', 'disabled'];
  if (cmd === 'watch') {
    const w = { el: active(), quiet: new Set(), seen: [], phase: 'quiet' };
    w.state = state(w.el);
    w.take = (records) => {
      for (const r of records) {
        const counts =
          r.type === 'childList'
            ? [...r.addedNodes, ...r.removedNodes].some((n) => n.nodeType === 1)
            : r.type === 'attributes' && ATTRS.includes(r.attributeName);
        if (!counts) continue;
        if (w.phase === 'quiet') w.quiet.add(r.target);
        else w.seen.push(r.target);
      }
    };
    w.obs = new MutationObserver(w.take);
    w.obs.observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ATTRS });
    window.__uxbActivate = w;
    return true;
  }
  const w = window.__uxbActivate;
  if (!w) return null;
  w.take(w.obs.takeRecords());
  if (cmd === 'mark') {
    w.phase = 'key';
    w.seen = [];
    return true;
  }
  const now = state(w.el);
  const out = {
    navigated: false,
    focus_moved: active() !== w.el,
    state_changed: now !== w.state,
    dom_changed: w.seen.some((n) => !w.quiet.has(n)),
  };
  w.state = now;
  w.seen = [];
  return out;
})
