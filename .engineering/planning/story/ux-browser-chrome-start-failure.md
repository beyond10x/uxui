---
format: aep.planning-md/3
id: story:ux-browser-chrome-start-failure
kind: story
status: implemented
title: Chromium's own words when it does not start, and what a hosted runner needs
relations:
- decomposes: epic:engine-slices
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T11:11:53Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-25T11:11:54Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-25T11:13:56Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Chromium's own words when it does not start, and what a hosted runner needs

## Outcome

When Chromium ends before it listens, or prints no DevTools endpoint in time, `ux-browser` prints Chromium's last lines on stderr with the error, and `docs/browser-adapter.md` says what a consumer does on a hosted CI runner.

## Context

Consumer: llm (2026-11-25, beyond10x/llm pull request 107, run 45597987448). ux-browser 0.8.1 on a GitHub `ubuntu-latest` runner printed only `ux-browser: Chromium exited before it listened: signal: 6 (SIGABRT) (core dumped)`, exit 3; Playwright's Chromium of the same build, started without the sandbox, passed in the same job. `Process::start` (`engine/browser/src/chrome.rs`) reads Chromium's stderr only for the DevTools line and drops the rest, so the cause is not printed. llm's reading, inferred: the runner image refuses unprivileged user namespaces, so Chromium's sandbox aborts; llm adds `chrome_args: [--no-sandbox]`. `git grep -i sandbox` finds nothing in this repository.

## Acceptance

With a stand-in Chromium that prints lines on stderr and ends before it listens, the error names the exit status and the last lines it printed (at most 20); the same for one that prints no endpoint within the deadline; `docs/browser-adapter.md` says that the default switches keep Chromium's sandbox, that a runner which refuses unprivileged user namespaces makes Chromium abort, and that `chrome_args: [--no-sandbox]` is the consumer's choice there.
