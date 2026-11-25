---
format: aep.planning-md/3
id: story:ux-browser-chrome-pin
kind: story
status: draft
title: Pin the Chrome build
relations:
- decomposes: epic:engine-slices
revision: 2
---
# Story: Pin the Chrome build

## Outcome

A consumer's runs use the Chrome build it named, and fail when that build is missing, rather than silently moving to another build.

## Context

Consumer: the second consumer, 0.5.0 on 2026-11-25. ux-browser takes the newest `chromium-<n>` under the Playwright cache (`src/chrome.rs`); `chromium-1254` (Chrome 158, used at 0.3.0) was removed from the cache, so 0.4.0 and 0.5.0 ran on `chromium-1228` (Chrome 149) without a word. The version is already in every document as `conditions.tools.chromium` (`Chrome/149.0.7827.55` in the llm example); the consumer asked for that and a pin. Only the pin is missing.

## Acceptance

`chrome: {build: <n>}` or `chrome: {version: <major>}` in the checks file selects that build from the cache; when it is missing the run exits non-zero naming the wanted and the found builds; without the key the newest build is taken as in 0.5.0; a test with a fake cache holding two builds covers pick, miss and default.
