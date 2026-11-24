// Runs before any script of each document: records every layout shift without recent input as
// [startTime, value]; null where the browser has no layout-shift observer.
window.__uxbShifts = [];
try {
  new PerformanceObserver((list) => {
    for (const e of list.getEntries()) if (!e.hadRecentInput) window.__uxbShifts.push([e.startTime, e.value]);
  }).observe({ type: 'layout-shift', buffered: true });
} catch (e) {
  window.__uxbShifts = null;
}
