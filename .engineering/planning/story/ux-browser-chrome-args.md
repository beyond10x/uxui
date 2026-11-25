---
format: aep.planning-md/3
id: story:ux-browser-chrome-args
kind: story
status: implemented
title: Chrome switches and a low-load mode
relations:
- decomposes: epic:engine-slices
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T02:37:08Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-25T02:37:08Z", actor: "human:operator", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-25T03:05:59Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Chrome switches, and no GPU process by default

## Outcome

A run on a shared workstation starts no GPU process by default, and a consumer adds its own Chrome switches without a wrapper script.

## Context

Consumer: the second consumer, 0.5.0 on 2026-11-25. With `--headless=new` and the default 4 jobs one run put a GPU process at 137 % CPU and 4 renderers at 50 to 80 % each while the workstation's owner was using it. It needed a wrapper named by `UXUI_CHROME` to add `--disable-gpu`. Checked against the source at d060e30: the switches are the constant `SWITCHES` in `src/chrome.rs`; no setting adds to it. With `--jobs 1` and `--disable-gpu` the run took 61 s, against 19 to 27 s at 0.4.0 with the default jobs.

Ordered first among the four drafts of 2026-11-25 by the host operator, 03:35 CET: "a setting for Chrome switches and a default that starts no GPU process".

## Acceptance

- `SWITCHES` starts no GPU process by default (`--disable-gpu`, plus what else is needed so no process with `--type=gpu-process` runs); a live run on llm's documentation build shows no such process in the browser's process tree, and its values equal the example document's, or the example document is regenerated from a real run and the difference is named.
- `chrome_args` in the checks file and a repeatable `--chrome-arg` flag append switches after `SWITCHES`; a switch that `SWITCHES` already sets with another value (`--headless`, `--remote-debugging-port`) is refused with its name; `--enable-gpu` style switches that undo the default are allowed and named in the result.
- The switches used appear in `run.local_inputs`, outside the evidence, so documents of runs that differ only in switches are equal.
- The documentation names the default, the setting and the flag, and a low-load run (`--jobs 1`).
