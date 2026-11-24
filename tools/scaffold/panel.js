// uxui scaffold panel: a floating control panel over the page, one control per axis of the
// scaffold. Moving a control applies the variant at once through __uxui.apply and measures it.
// Needs runtime.js. Opens with __uxui.panel(), or by itself when the URL has ?uxui-panel or
// window.__uxuiAutoPanel is set. window.__uxuiVariants = [{name, variant}] adds a list of
// prepared variants (for example the projected variants of a lifted document).
(() => {
  if (window.__uxuiPanel) return;
  const HOST_ID = '__uxui_panel';
  const CSS = `
:host{all:initial}
.p{position:fixed;right:12px;bottom:12px;z-index:2147483647;width:300px;max-height:85vh;overflow:auto;
 font:12px/1.4 system-ui,sans-serif;color:#111;background:#fff;border:1px solid #888;border-radius:8px;
 box-shadow:0 4px 18px rgba(0,0,0,.25);padding:8px 10px}
.p.min{width:auto;max-height:none}
.p.min .b{display:none}
h1{font-size:12px;margin:0 0 6px;display:flex;gap:6px;align-items:center;justify-content:space-between}
label{display:grid;grid-template-columns:78px 1fr 40px;gap:6px;align-items:center;margin:3px 0}
input[type=range]{width:100%}
select,button,input{font:inherit}
button{padding:2px 6px;border:1px solid #888;border-radius:4px;background:#f4f4f4;cursor:pointer}
.secs{margin:4px 0;border-top:1px solid #ddd;padding-top:4px}
.sec{display:flex;gap:4px;align-items:center;margin:2px 0}
.sec span{flex:1;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}
.m{margin-top:6px;border-top:1px solid #ddd;padding-top:4px;font:11px/1.35 ui-monospace,monospace;white-space:pre-wrap}
.bad{color:#a00}.ok{color:#060}
.row{display:flex;gap:6px;flex-wrap:wrap;margin-top:6px}`;

  function open() {
    if (document.getElementById(HOST_ID)) return;
    const U = window.__uxui;
    const sc = U.state.last || U.scaffold();
    const host = document.createElement('div');
    host.id = HOST_ID;
    document.documentElement.appendChild(host);
    const root = host.attachShadow({ mode: 'open' });
    const ax = sc.axes || {};
    const v = { mode: 'original', density: 1, font_scale: 1, accent: 'original', max_width: 'original', section_order: sc.sections.map((s) => s.id), hide: [] };
    let preset = null;
    const accent0 = sc.palette.accent ? sc.palette.accent.color.slice(0, 7) : '#3366cc';
    root.innerHTML = `<style>${CSS}</style><div class="p"><h1><span>uxui scaffold</span><span>
      <button data-a="min" title="Fold">_</button><button data-a="close" title="Close">x</button></span></h1>
      <div class="b">
      ${(window.__uxuiVariants || []).length ? `<label>variant<select data-k="preset" style="grid-column:2/4"><option value="">(axes below)</option>${window.__uxuiVariants
        .map((x, i) => `<option value="${i}">${x.name}</option>`)
        .join('')}</select></label>` : ''}
      <label>mode<select data-k="mode"><option>original</option><option>light</option><option>dark</option></select><span></span></label>
      <label>density<input type="range" data-k="density" min="0.6" max="1.6" step="0.05" value="1"><span data-o="density">1</span></label>
      <label>font scale<input type="range" data-k="font_scale" min="0.8" max="1.4" step="0.05" value="1"><span data-o="font_scale">1</span></label>
      <label>accent<input type="color" data-k="accent" value="${accent0}" ${sc.palette.accent ? '' : 'disabled'}><button data-a="accent0" title="Original accent">o</button></label>
      <label>max width<input type="range" data-k="max_width" min="0" max="1600" step="20" value="0"><span data-o="max_width">orig</span></label>
      <div class="secs">${sc.sections.length ? 'sections (order, hide)' : 'no sections found'}<div data-r="secs"></div></div>
      <div class="row"><button data-a="reset">Reset</button><button data-a="copy">Copy variant</button><button data-a="measure">Measure</button></div>
      <div class="m" data-r="m"></div></div></div>`;
    const $ = (s) => root.querySelector(s);
    const label = (id) => sc.sections.find((s) => s.id === id)?.label || id;
    function drawSections() {
      $('[data-r=secs]').innerHTML = v.section_order
        .map(
          (id, i) => `<div class="sec"><input type="checkbox" data-hide="${id}" ${v.hide.includes(id) ? '' : 'checked'} title="Shown">
        <span title="${label(id)}">${id} ${label(id)}</span><button data-up="${i}" ${i ? '' : 'disabled'}>↑</button><button data-dn="${i}" ${i < v.section_order.length - 1 ? '' : 'disabled'}>↓</button></div>`,
        )
        .join('');
    }
    function current() {
      if (preset) return preset;
      const out = {};
      if (v.mode !== 'original') out.mode = v.mode;
      if (+v.density !== 1) out.density = +v.density;
      if (+v.font_scale !== 1) out.font_scale = +v.font_scale;
      if (v.accent !== 'original') out.accent = v.accent;
      if (v.max_width !== 'original') out.max_width = v.max_width;
      if (v.section_order.join() !== sc.sections.map((s) => s.id).join()) out.section_order = v.section_order;
      if (v.hide.length) out.hide = v.hide;
      return out;
    }
    let pending = 0;
    function update() {
      cancelAnimationFrame(pending);
      pending = requestAnimationFrame(() => {
        const a = U.apply(current());
        const m = U.measure();
        const fails = m.fails.length ? `<span class="bad">fails: ${m.fails.join(', ')}</span>` : '<span class="ok">hard rules pass</span>';
        $('[data-r=m]').innerHTML =
          `${fails}\nviewport ${m.viewport}  apply ${a.ms} ms  measure ${m.ms} ms\n` +
          Object.entries(m.metrics)
            .map(([k, x]) => `${k} ${x}`)
            .join('\n') +
          `\n\nvariant ${JSON.stringify(current())}`;
      });
    }
    drawSections();
    root.addEventListener('input', (e) => {
      const k = e.target.dataset.k;
      if (!k) return;
      if (k === 'preset') {
        preset = e.target.value === '' ? null : window.__uxuiVariants[+e.target.value].variant;
      } else if (k === 'max_width') v.max_width = +e.target.value ? +e.target.value : 'original';
      else v[k] = e.target.value;
      const o = root.querySelector(`[data-o=${k}]`);
      if (o) o.textContent = k === 'max_width' && v.max_width === 'original' ? 'orig' : e.target.value;
      update();
    });
    root.addEventListener('change', (e) => {
      const id = e.target.dataset.hide;
      if (!id) return;
      v.hide = e.target.checked ? v.hide.filter((x) => x !== id) : [...v.hide, id];
      update();
    });
    root.addEventListener('click', (e) => {
      const t = e.target, a = t.dataset.a;
      if (t.dataset.up !== undefined || t.dataset.dn !== undefined) {
        const i = +(t.dataset.up ?? t.dataset.dn), j = t.dataset.up !== undefined ? i - 1 : i + 1;
        [v.section_order[i], v.section_order[j]] = [v.section_order[j], v.section_order[i]];
        drawSections();
        update();
      } else if (a === 'min') $('.p').classList.toggle('min');
      else if (a === 'close') {
        U.reset();
        host.remove();
      } else if (a === 'accent0') {
        v.accent = 'original';
        update();
      } else if (a === 'reset') {
        host.remove();
        U.reset();
        open();
      } else if (a === 'copy') navigator.clipboard?.writeText(JSON.stringify(current()));
      else if (a === 'measure') update();
    });
    let rt = 0;
    addEventListener('resize', () => {
      clearTimeout(rt);
      rt = setTimeout(update, 150);
    });
    update();
  }
  window.__uxuiPanel = open;
  const auto = () => {
    if (window.__uxuiAutoPanel || /[?&]uxui-panel\b/.test(location.search)) open();
  };
  if (document.readyState !== 'complete') addEventListener('load', () => setTimeout(auto, 50));
  else auto();
})();
