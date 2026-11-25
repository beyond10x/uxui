// A journey's `wait` step: until `css` matches a rendered element (`checkVisibility()`, a box of some
// size), watched with a `MutationObserver` on the document (nodes, attributes, text), not polled, for
// at most `ms`. Answers {met} or {invalid} for a selector the browser does not parse.
(function (css, ms) {
  try {
    document.querySelector(css);
  } catch (e) {
    return { invalid: true };
  }
  const rendered = (e) => {
    if (typeof e.checkVisibility === 'function' && !e.checkVisibility()) return false;
    const r = e.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  };
  const holds = () => Array.from(document.querySelectorAll(css)).some(rendered);
  if (holds()) return { met: true };
  return new Promise((resolve) => {
    let timer;
    const watch = new MutationObserver(() => {
      if (holds()) {
        watch.disconnect();
        clearTimeout(timer);
        resolve({ met: true });
      }
    });
    watch.observe(document, { subtree: true, childList: true, attributes: true, characterData: true });
    timer = setTimeout(() => {
      watch.disconnect();
      resolve({ met: holds() });
    }, ms);
  });
})
