// Runs before any script of each document when the configuration declares `clock`: the page's clock
// starts at the declared instant `at` (milliseconds since the epoch). `Date` (`Date()`, `new Date()`,
// `Date.now()`) reads `at` plus the time since the document's time origin, `performance.now()`, or
// `at` alone when the clock does not advance; `performance.timeOrigin` is `at`, so the two clocks
// agree. `Intl.DateTimeFormat`'s `format()` and `formatToParts()` without a date, and `Temporal.Now`
// where the browser has it, read the same clock. Timers, frames and `performance.now()` itself run
// as before: the ready wait, the settle times and the layout-shift times do not change.
(function (at, advance) {
  const Real = Date;
  const now = advance ? () => at + Math.floor(performance.now()) : () => at;
  const Fixed = function Date(...args) {
    if (!new.target) return new Real(now()).toString();
    return Reflect.construct(Real, args.length ? args : [now()], new.target);
  };
  Object.defineProperty(Fixed, 'length', { value: Real.length });
  Fixed.prototype = Real.prototype;
  Fixed.now = function now_() { return now(); };
  Object.defineProperty(Fixed.now, 'name', { value: 'now' });
  Fixed.parse = Real.parse;
  Fixed.UTC = Real.UTC;
  Object.defineProperty(Real.prototype, 'constructor', { value: Fixed, writable: true, configurable: true });
  window.Date = Fixed;
  try {
    Object.defineProperty(performance, 'timeOrigin', { configurable: true, get: () => at });
  } catch (e) {}
  const dtf = Intl.DateTimeFormat.prototype;
  const format = Object.getOwnPropertyDescriptor(dtf, 'format');
  const bound = new WeakMap();
  Object.defineProperty(dtf, 'format', {
    configurable: true,
    get() {
      let f = bound.get(this);
      if (!f) {
        const real = format.get.call(this);
        f = (d) => real(d === undefined ? now() : d);
        bound.set(this, f);
      }
      return f;
    },
  });
  const formatToParts = dtf.formatToParts;
  dtf.formatToParts = function formatToParts_(d) {
    return formatToParts.call(this, d === undefined ? now() : d);
  };
  if (typeof Temporal === 'object' && Temporal.Now) {
    const N = Temporal.Now;
    const instant = () => Temporal.Instant.fromEpochMilliseconds(now());
    const zoned = (tz) => instant().toZonedDateTimeISO(tz === undefined ? N.timeZoneId() : tz);
    N.instant = instant;
    N.zonedDateTimeISO = zoned;
    N.plainDateTimeISO = (tz) => zoned(tz).toPlainDateTime();
    N.plainDateISO = (tz) => zoned(tz).toPlainDate();
    N.plainTimeISO = (tz) => zoned(tz).toPlainTime();
  }
})
