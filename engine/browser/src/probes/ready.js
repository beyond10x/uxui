(function (kind, value, ms) {
  // The page's ready condition: an element that matches `value`, or `window[value] === true` (set
  // through the trap of probes/ready-global.js, which sends `uxb:ready`). Watched, not polled, for at
  // most `ms`; then fonts loaded and two frames drawn. Answers {met, ready} or {invalid}.
  if (kind === 'selector') {
    try { document.querySelector(value); } catch (e) { return { invalid: true }; }
  }
  const holds = kind === 'selector' ? () => document.querySelector(value) !== null : () => window[value] === true;
  const met = holds() ? Promise.resolve(true) : new Promise((resolve) => {
    let watch = null;
    let timer;
    const finish = (ok) => {
      if (watch) watch.disconnect();
      removeEventListener('uxb:ready', check);
      clearTimeout(timer);
      resolve(ok);
    };
    const check = () => { if (holds()) finish(true); };
    if (kind === 'selector') {
      watch = new MutationObserver(check);
      watch.observe(document, { subtree: true, childList: true, attributes: true });
    }
    addEventListener('uxb:ready', check);
    timer = setTimeout(() => finish(holds()), ms);
  });
  return met.then(async (ok) => {
    if (!ok) return { met: false };
    await document.fonts.ready;
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    return { met: true, ready: performance.now() };
  });
})
