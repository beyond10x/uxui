# Not available here: the colleague's private tools

From a colleague's loop (shared 2026-10-19). The colleague's own scripts (a `ux.mjs` command with the
subcommands below, a recorder, a ratchet and the bots) are private. Build the equivalent on
Playwright, axe-core, rrweb and ffmpeg.

| tool | does | build on |
|---|---|---|
| recorder (`?ux=1`) | records the DOM plus events `moment`, `visible`, `act`, `region`, `survey`, `end` from `data-ux-region` marks, and posts them per session | rrweb |
| record writer | stores one JSONL per session (a header line, then the events) and the audio; prunes by retention | server side |
| `ux round <dir>` | round dir to `report.md`, `round.json`, `shots/`, `trend.md`; flags for shot count, sheet size, trend file, baseline file, no shots | Playwright (headless Chromium) |
| `ux report <file>` | one recording to markdown with worst moments linked to replay, or metrics JSON | – |
| `ux replay <file>` | local replay page with seek, or one PNG at a given time | rrweb |
| `ux views` | table of the 18 metrics per view | – |
| `ux probe <file>` | L5: a per-moment screenshot to a vision model, scored against the cue truth; writes PNGs, `probe.md`, `probe.json` | a vision model API (paid) |
| `ux gate [-update] [-decision <id>]` | the ratchet of SKILL.md § 6; `-update` rewrites the baseline | git, for `main`'s baseline |
| `ux video <file> <mix.wav>` | highlight MP4 and contact sheet from a recording and its audio | ffmpeg |
| metric catalogue | the 18 metrics, one function per key | – |
| layout contract | viewport profiles, no scroll, critical regions, axe, keyboard-only, layout shift, plus break-tests that prove each check can fail | Playwright, axe-core |
| screen-only bot | SKILL.md § 7 | Playwright accessibility locators |
| keyed replay | L4: recorded sessions replayed N = 3 times with 2-of-3 stability | the UI's paid backends |
| synthetic caller audio | a robot-call script to WAV, timeline and transcript | a TTS engine |
| closed-loop simulator | SKILL.md § 7, phase B | – |
| findings test | reads `ux/findings/` and fails on a closed S1/S2 record without a regression asset | the project's test runner |
