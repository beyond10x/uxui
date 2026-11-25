// Runs before any script of each document when the ready condition is a global: `window[name]`
// becomes an accessor that keeps the page's value and sends `uxb:ready` when the page sets it true,
// so the ready probe waits for an event and polls nothing.
(function (name) {
  let value = window[name];
  try {
    Object.defineProperty(window, name, {
      configurable: true,
      enumerable: true,
      get() { return value; },
      set(v) { value = v; if (v === true) window.dispatchEvent(new Event('uxb:ready')); },
    });
  } catch (e) {}
})
