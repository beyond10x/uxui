// probe.mjs <url> [--out file.json] [--viewport 1440x900] [--primary <css selector>]
// Loads the page in headless Chromium and writes its uxui-scaffold/1 JSON.

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { launch, parseArgs, scratchDir } from './cdp.mjs';

const { pos, opt } = parseArgs(process.argv.slice(2));
const url = pos[0];
if (!url) {
  console.error('usage: probe.mjs <url> [--out file.json] [--viewport WxH] [--primary selector]');
  process.exit(2);
}
const [w, h] = (opt.viewport || '1440x900').split('x').map(Number);
const out = opt.out || join(scratchDir(), 'scaffold-' + new URL(url).hostname + '.json');

const t0 = performance.now();
const browser = await launch();
try {
  const page = await browser.newPage();
  await page.viewport(w, h);
  const tLoad = performance.now();
  await page.goto(url);
  const loadMs = performance.now() - tLoad;
  const sc = await page.call('scaffold', { primary: opt.primary });
  const text = JSON.stringify(sc, null, 1);
  writeFileSync(out, text + '\n');
  console.log(
    JSON.stringify({
      out,
      bytes: text.length,
      regions: sc.regions.length,
      sections: sc.sections.length,
      headings: sc.headings.length,
      interactive: sc.interactive.length,
      elements: sc.counts.elements,
      primary: sc.primary,
      accent: sc.palette.accent,
      mode: sc.mode,
      load_ms: Math.round(loadMs),
      scaffold_ms: sc.ms,
      total_ms: Math.round(performance.now() - t0),
    }),
  );
} finally {
  await browser.close();
}
