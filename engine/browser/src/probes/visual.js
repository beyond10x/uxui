// The page made still for the visual ratchet's screenshots (`visual.rs`), and put back after them.
//
// `freeze`: the text caret is made transparent (a constructed style sheet adopted by the document,
// no element added); every animation and transition the page runs (`document.getAnimations()`) is
// finished when it ends, or paused at its start when it repeats forever; the page is scrolled to
// 0, 0; then fonts and two frames are awaited. Returns the boxes of the elements the mask
// selectors match, in CSS pixels of the document ([left, top, right, bottom], boxes of no area
// left out), and the document's scroll height.
// `thaw`: the style sheet is dropped and the paused animations play again.
(async function (step, masks) {
  if (step === 'thaw') {
    const sheet = window.__uxbVisualSheet;
    if (sheet) document.adoptedStyleSheets = document.adoptedStyleSheets.filter((s) => s !== sheet);
    for (const a of window.__uxbVisualPaused || []) a.play();
    delete window.__uxbVisualSheet;
    delete window.__uxbVisualPaused;
    return true;
  }
  const sheet = new CSSStyleSheet();
  sheet.replaceSync('*, *::before, *::after { caret-color: transparent !important; }');
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  window.__uxbVisualSheet = sheet;
  const paused = [];
  for (const a of document.getAnimations()) {
    const end = a.effect ? a.effect.getComputedTiming().endTime : Infinity;
    if (Number.isFinite(end)) {
      a.finish();
    } else {
      a.pause();
      a.currentTime = 0;
      paused.push(a);
    }
  }
  window.__uxbVisualPaused = paused;
  window.scrollTo(0, 0);
  await document.fonts.ready;
  await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  const boxes = [];
  for (const selector of masks) {
    for (const el of document.querySelectorAll(selector)) {
      const r = el.getBoundingClientRect();
      if (r.width > 0 && r.height > 0) {
        boxes.push([r.left + scrollX, r.top + scrollY, r.right + scrollX, r.bottom + scrollY]);
      }
    }
  }
  const root = document.documentElement;
  const height = Math.max(root.scrollHeight, document.body ? document.body.scrollHeight : 0);
  return { boxes, height };
})
