---
format: aep.planning-md/3
id: review-result:scope-round-2
kind: review-result
status: active
title: Scope critic, round 2
relations:
- reviews: story:browser-adapter
- reviews: story:check-tool-internals-research
- reviews: story:cost-spike
- reviews: story:docs-site
- reviews: story:docs-site-first-climb
- reviews: story:ess-ui-needs-per-way
- reviews: story:loop-process-improvements
- reviews: story:loop-round-on-studio
- reviews: story:migrate-smallest-ui-loop-copy
- reviews: story:own-generator-smallest
- reviews: story:point-old-skill-copies-here
- reviews: story:ratchet-presets-and-gates
- reviews: story:scaffold-spike
- reviews: story:spec-hardening-pass
- reviews: story:spec-round-2-scenarios
- reviews: story:spec-round-3-target-adapter
- reviews: story:stage-agreement
- reviews: story:stock-or-own-decision
- reviews: story:studio-climb
- reviews: story:studio-stock-generator
- reviews: story:themes-light-dark
- reviews: story:thin-the-skill
- reviews: story:toy-climb
- reviews: story:track-ess-gaps
- reviews: story:trial-target
- reviews: story:ux-static-slice
- reviews: epic:bench
- reviews: epic:engine-slices
- reviews: epic:ess-dependencies
- reviews: epic:generators-and-themes
- reviews: epic:ratchet-integration
- reviews: epic:skill
- reviews: epic:specification
- reviews: epic:targets
revision: 1
---
needs-revision
epic:engine-slices — "Counter: tie bands, the Pareto frontier, prefer the smallest change, and a small human calibration set per campaign before a ranking is trusted" is promised as the roadmap's counter to its first risk, and no epic Risks section, story or blocker carries it (grep for "calibration" in .engineering/planning finds nothing); carry it in this epic's Risks or Scope, or name it held — docs/roadmap.md:59-60
story:docs-site-first-climb — "the chosen variant becomes a diff to `website/product.json` and the theme CSS, a pull request, and ratchet evidence; the site's next build is gated on the promoted values" is promised, but the acceptance and epic:targets Done When claim only the diff plus evidence, so the pull request and the gating of the site's next build are dropped with nothing recording it — docs/targets.md:74-75
story:docs-site-first-climb — "Target: page `/uxui/` (scope page), and the hero section (scope section)" names two targets and the story claims the page only, with the hero-section target neither claimed nor recorded as dropped — docs/targets.md:60
epic:generators-and-themes — the slot classification assigns "Styling and layout, for uxui themes and templates, not ESS: G9 (column widths, one-line cells, class hooks), its F9 (label beside value), the colour per value of G11" to uxui, and no item, held row or Out of Scope line claims it (themes-light-dark covers light and dark modes only, and the blocker's slot case study row lists four buckets, none of them a uxui theme or template); claim it or name it held in this epic — docs/generator-round.md:87-89
specification-blocker:bench-nouns — the head-to-head row narrows the referee to the browser measurement on held-out journeys and omits the referee's second half, "A blind pairwise preference by people (and, labelled as such, by a review agent) between the two proposals and the original, recorded as judgement records", so the held record says less than the parent; add it to row 4 — docs/comparison.md:50-51

What I read: 8 epics, 26 stories, 6 blockers (3 specification-blocker, 3 dependency-blocker) and review-result:scope-round-1 in full, plus all six parents in full (docs/roadmap.md, docs/generator-round.md, docs/ess-gaps.md, docs/targets.md, docs/comparison.md, reviews/2026-11-24-uxsearch-spec-review.md) and AGENTS.md:20-50. I read the parents before the set. Commands: `aep plan artifact graph`, `aep plan artifact validate` (44 artifacts, valid), `cat` and `grep` over `.engineering/planning/*` and `docs/*`, plus the plugin's scope-critic and rubric files. Promise count: about 66 extracted (roadmap: 8 steps, 5 sub-gaps (a)-(e), 4 risks plus counters, 4 decisions; generator-round: 7 stories, F1-F5, slot classification, standing question; ess-gaps: 18; targets: 3 capabilities, 2 adapters, 4 funnel stages, 2 checks, 4 spec changes, the worked example; comparison: bench, 6 measures, referee, evidence, dogfooding; review: R1-R37 and the ten hardening targets as one block). About 61 traced to an item or a drafter-named held noun; the 5 above are untraced.

Round 1 findings: all five landed.
- Funnel stages 2 and 3 are now rows in specification-blocker:funnel-nouns and in epic:engine-slices Out of Scope.
- Roadmap 2(e) is named as dropped, with the reason, in epic:ess-dependencies Out of Scope.
- story:browser-adapter now claims box, computed styles, accessibility tree and screenshot, naming the adapter.
- story:check-tool-internals-research is disclosed in epic:engine-slices as carried in from outside. The story is still not traceable to a roadmap sentence, but it traces to AGENTS.md:30-33 and the round 1 remedy allowed this, so I did not repeat it.
- The invented gate clause is gone from story:docs-site.

No two items claim one outcome. The scaffold spike versus funnel stage 0 overlap is disclosed in story:scaffold-spike Out of Scope. Stage agreement versus the layout-model rank correlation are two separate promises in the parents.

What I could not establish:
- Citation drift, out of my lane (design or acceptance). Epics cite docs/roadmap.md:56, 58-59, 60 and 30,57 for the layout-model risk, the hermetic-metrics risk, the older-copies risk and the exhaustive-search risk. In the working copy those sit at lines 61, 63-64, 65 and 62, and line 56 is the first risk. Affected: epic:engine-slices, epic:specification, epic:bench, epic:skill, story:toy-climb, story:point-old-skill-copies-here. Re-cite them.
- Whether the roadmap's APCA/WCAG contrast decision and F4 (series colours, solved in `themes/studio.css`) need a claiming story. I treated both as recorded decisions or done work, as in round 1.
- Out of my lane (design): story:stage-agreement needs funnel stage 2, which is held in specification-blocker:funnel-nouns, yet it depends only on story:browser-adapter and story:toy-climb. story:ess-ui-needs-per-way and story:stock-or-own-decision are drafted and blocked by a specification-blocker, although the brief says stories without an ESS home were not drafted.

```findings
- file: docs/roadmap.md
  line: 59
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the roadmap's counter to its first risk, 'tie bands, the Pareto frontier, prefer the smallest change, and a small human calibration set per campaign before a ranking is trusted', is claimed by no epic Risks section, story or blocker; carry it in epic:engine-slices or name it held"
- file: docs/targets.md
  line: 74
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the worked example's promotion promises a diff, a pull request and 'the site's next build is gated on the promoted values'; story:docs-site-first-climb and epic:targets Done When claim only the diff and ratchet evidence, so the pull request and the build gating are dropped silently"
- file: docs/targets.md
  line: 60
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the worked example names two targets, page /uxui/ and the hero section (scope section); story:docs-site-first-climb claims the page only and does not record the hero-section target as dropped"
- file: docs/generator-round.md
  line: 87
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the slot classification assigns styling and layout (G9 column widths, one-line cells, class hooks; F9 label beside value; colour per value of G11) to uxui themes and templates, and no item, held row or Out of Scope line in epic:generators-and-themes claims it"
- file: docs/comparison.md
  line: 50
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "specification-blocker:bench-nouns head-to-head row names only the browser-measurement half of the referee and omits the blind pairwise preference by people (and a labelled review agent) recorded as judgement records"
```
