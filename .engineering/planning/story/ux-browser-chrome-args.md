---
format: aep.planning-md/3
id: story:ux-browser-chrome-args
kind: story
status: draft
title: Chrome switches and a low-load mode
relations:
- decomposes: epic:engine-slices
revision: 2
---
# Story: Chrome switches and a low-load mode

## Outcome

A consumer on a shared workstation runs ux-browser without a GPU process and with one page at a time, without a wrapper script.

## Context

Consumer: the second consumer, 0.5.0 on 2026-11-25. With `--headless=new` and the default 4 jobs one run put a GPU process at 137 % CPU and 4 renderers at 50 to 80 % each while the workstation's owner was using it. It needed a wrapper named by `UXUI_CHROME` to add `--disable-gpu`. Checked against the source at d060e30: the switches are the constant `SWITCHES` in `src/chrome.rs`; no setting adds to it. With `--jobs 1` and `--disable-gpu` the run took 61 s, against 19 to 27 s at 0.4.0 with the default jobs.

## Acceptance

`chrome_args` in the checks file and a repeatable `--chrome-arg` flag append switches after `SWITCHES`; a switch that `SWITCHES` already sets with another value (`--headless`, `--remote-debugging-port`) is refused; `--low-load` means `--jobs 1` plus `--disable-gpu`; the switches used appear in `run.local_inputs`, outside the evidence; a run with `--disable-gpu` on llm's build gives the example document's values; the documentation names the low-load mode.
