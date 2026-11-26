---
format: aep.planning-md/3
id: story:studio-loop-round-3
kind: story
status: draft
title: 'Studio loop round 3: the live climb, titles, notices and values as text'
summary: Fix UXF-004, UXF-002, UXF-008, UXF-010 and UXF-005 on the studio and measure round R-03 against R-02
relations:
- decomposes: epic:bench
- depends_on: story:studio-loop-round-2
scope:
- confidence: cited
  path: ess/ui/findings
- confidence: cited
  path: ess/ui/rounds
- confidence: cited
  path: ess/ui/studio.ui.yaml
- confidence: cited
  path: ess/ui/tests/studio.yaml
- confidence: cited
  path: themes/studio.css
revision: 8
---
# Story: Studio loop round 3: the live climb, titles, notices and values as text

## Outcome

The live climb keeps Stop and its states in view, the decision of a promotion reads as text, and
round R-03 shows by how much each target metric moved against R-02.

## Context

Round R-02 (`ess/ui/rounds/R-02/`, `story:studio-loop-round-2`) left 20 findings open, 2 of them S1;
gates 1, 2 and 3 fail. Its triage (`ess/ui/rounds/R-02/triage.md`, "The change set") names five
changes: UXF-004 and UXF-002 (every S1), UXF-008, UXF-010 and UXF-015 (by priority).

An order of 2026-11-26, given on the owner's delegation with the design decision of R-02: UXF-005 goes into the next
round, so that the gate decision of a promotion reads as text and not as JSON. A change set holds at
most five changes, so UXF-005 takes the place of UXF-015, the lowest priority of the set.

So this round builds five changes:

- UXF-004: the live climb scrolls; Stop and the states leave the viewport (layout; the design
  decision of this round, `decision.md` of R-03).
- UXF-002: "Switch to this value" is clipped and cannot be clicked at 1280x720 (layout; it waits for
  its own decision if the change of UXF-004 does not settle it, and blocks no other change).
- UXF-008: four pages show no title (the document).
- UXF-010: a success notice names the command, not what happened (the document).
- UXF-005: structured values are drawn as raw JSON. A known loss of the stock generator (cost L in
  R-01's triage): the change is the studio's own handling in its document or theme, and what cannot
  be done there goes to the generator's maintainers as a need.

R-02 also named what a comparable round needs: the same fonts on every machine that measures. This
round serves the theme's font with the app, or records which machine measured, before it reads a
pixel value against R-02.

Shared surface: `ess/ui/studio.ui.yaml` and `ess/ui/tests/studio.yaml`, `themes/studio.css`,
`ess/ui/rounds/`, `ess/ui/findings/`. The declared campaign on the studio
(`tools/studio-campaign.sh`) reads the document and the theme: it is recorded again after this
round.

## Acceptance

`sh ess/ui/rounds/round.sh R-03` is on record in `ess/ui/rounds/R-03/`, and against R-02:

- `page_checks.always_visible.misses` on the live climb is 0 (20 in R-02);
- `audit.pages_without_h1` is 0 (4 in R-02);
- no field of the gate decision of a promotion shows a JSON object, held by a check that failed
  before the change;
- no notice of a scenario contains a command's name, held by an expectation that failed before the
  change;
- no metric of `ess/ui/rounds/R-02/round.json` is worse beyond the skill's tolerance;
- each closed finding names a regression asset that failed before the change and passes after it,
  and `ess/ui/rounds/R-03/triage.md` has the table "did the last change sets move their metrics?".

## Provenance

Drafted on 2026-11-26 from `ess/ui/rounds/R-02/triage.md` and the order of that day on UXF-005, given
on the owner's delegation.
