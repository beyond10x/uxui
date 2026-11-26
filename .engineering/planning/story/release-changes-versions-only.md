---
format: aep.planning-md/3
id: story:release-changes-versions-only
kind: story
status: draft
title: A release changes versions and nothing else
summary: One command sets the version; no browser re-record, re-bless or hand edit
revision: 2
---
# Story: A release changes versions and nothing else

## Outcome

Cutting a release is one command and one pull request: no browser run, no golden re-blessed by
hand, no version edited in a page by hand.

## Context

Release 0.11.0 (2026-11-26) needed a re-record of the landing campaign on the build node, two
re-blessed golden files, 32 replaced screenshots and four hand edits, because the version stands in
files that tests hold byte for byte. Found while preparing that release:

Forces a browser re-record:

- `website/product.json:6`: the eyebrow carries the plugin version, and the file is one of the
  campaign's two sources, so each release changes its digest.
- `engine/search/tests/fixtures/docs-landing/product.json:6` and `variant.product.json:6`: copies of
  that source, held to the site by `engine/search/tests/landing.rs`.
- `engine/search/tests/fixtures/docs-landing/measurements.json`: the adapter line holds the adapter's
  crate version (`engine/search/src/landing/run.rs`, `ux_browser::VERSION`) and the product digest;
  `report.txt` and `promote.evidence.json` follow from both.
- `docs/campaign-2026-11-26-docs-landing-engine/*.png`: all 32 show the eyebrow.

Forces a hand edit or a re-bless:

- `website/docs/index.md:14`, `website/data/status.json:10`, `docs/roadmap.md:27`: the version in
  prose that no generator writes and no check holds.
- `engine/browser/examples/llm-docs.evidence.json`, `uxui-landing.render.json`: the tool version,
  held by no test.
- `engine/static/tests/fixtures/ratchet-example/golden.evidence.json`: the tool version in a
  byte-for-byte golden.
- `tools/release-versions.sh` checks the two manifests, the crates and the changelog only; the
  places above are held by nothing that names the release.
- Whether an empty `## Unreleased` stays in `CHANGELOG.md` after a release is not written down.

Friction in the re-record itself:

- `tools/docs-landing-campaign.sh`: its default binary path does not exist on the build node; while
  it holds a variant's sources in `website/`, a second pool job of the same checkout reads the
  swapped file.
- The campaign document's "Run it again" names the kept screenshot directory as the out directory,
  which puts `measurements.json` there and fails a test until it is moved.
- The bless switches and the browser path are not passed to the build node by any file of the
  repository.

## Acceptance

On a copy of `main`, one command sets the next version, and after it `sh tools/gate.sh` passes
without a browser run, a re-bless or another edit; `sh tools/release-versions.sh <version>` fails
when any file that carries the version disagrees.

## Provenance

Drafted on 2026-11-26 from the preparation of release 0.11.0 (pull request 53).
