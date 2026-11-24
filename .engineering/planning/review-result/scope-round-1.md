---
format: aep.planning-md/3
id: review-result:scope-round-1
kind: review-result
status: active
title: Scope critic, round 1
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
epic:engine-slices — funnel stage 2 ("project": the top k of stage 1 applied to the real page through the binding table, measured in a browser) and stage 3 ("judge": screenshots side by side, a model's judgement, a person's choice, judgement records) are in the epic's scope ("the funnel of docs/targets.md:9-56") but no story claims either, and specification-blocker:funnel-nouns holds only stage 0; claim them in a story (the natural home is beside story:stage-agreement) or name them as held in the epic's Out of Scope and the blocker — docs/targets.md:44-45 ("| 2 project | the top k of stage 1 → applied to the real page through the binding table, measured in a browser |")
epic:ess-dependencies — roadmap step 2 promises "(e) UX declarations as constructs (proposal from `uxui-declarations/1`)" as an ESS story, but it is not among the 18 gaps of docs/ess-gaps.md and the epic's Out of Scope says UX declarations belong to uxui, so the promise is silently dropped; the epic must record that (e) was dropped and why, or a story must claim it — docs/roadmap.md:29
story:browser-adapter — the acceptance returns only "the declared nodes' geometry and the accessibility tree", while the design promises "boxes, computed styles, accessibility tree, a screenshot" at each viewport and mode, and the screenshot is what stage 3 and the worked example's side-by-side judging read; widen the acceptance or record the narrowing — docs/targets.md:24-25
story:check-tool-internals-research — no sentence of any parent asks for this research (the story says "No source in this tree states it as a plan item"), and epic:engine-slices scope names no research; tie it to a promise in a parent or state in the epic that it is carried in from outside — .engineering/planning/story/check-tool-internals-research.md:22-25
story:docs-site — building a Docusaurus site and "the repository's gate runs that build" is not asked for by docs/targets.md, which describes the site as an existing target; the gate clause is an invented outcome (the story admits "Acceptance: inferred") — .engineering/planning/story/docs-site.md:28-34

What you read: 8 epics, 26 stories, 6 blockers (3 specification-blocker, 3 dependency-blocker) in full, plus the six parents in full (docs/roadmap.md, docs/generator-round.md, docs/ess-gaps.md, docs/targets.md, docs/comparison.md, reviews/2026-11-24-uxsearch-spec-review.md); commands: `aep plan artifact graph`, `aep plan artifact kinds`, `aep plan artifact relations`, `cat` over .engineering/planning/*, and the plugin's scope-critic and rubric files.

Promise count: I extracted about 62 promises from the parents (roadmap 8 steps with 5 sub-gaps plus 4 risks and 4 decisions, generator-round 7 stories, 5 findings and slot classification and the standing question, ess-gaps 18, targets 3 capabilities, 2 adapters, 4 funnel stages, 2 checks, 5 spec changes and the worked example, comparison bench, measures, referee and dogfooding, review 37 findings as one block). I traced 59 to an item or to a drafter-named held noun; the 3 untraced are the 3 parent-side findings above (funnel stages 2-3, roadmap 2e, browser-adapter narrowing, counted as gaps). Everything in the three specification-blocker bodies is a named omission and I did not count it as a gap. The 18 ESS gaps are claimed by story:track-ess-gaps and the two blockers, as intended. The review's R1-R37 are claimed by story:spec-hardening-pass (acceptance covers every finding). No duplicate claims found.

What could not be established:
- Whether F4 (series colours, solved in themes/studio.css) and the roadmap's APCA/WCAG contrast decision need a claiming story or are recorded history. I treated both as decisions or done work, not open promises.
- Whether the consequences of slot's classification ("for uxui themes and templates": G9, F9, the G11 colour) need a story. I treated them as the held slot case study's output.
- The worked example's two targets (page /uxui/ and the hero section): story:docs-site-first-climb covers the page only. I judged this too weak to report.
- Out of my lane: story:stock-or-own-decision depends only on story:own-generator-smallest although its acceptance needs the held adjustment costs (design or dependency lane). The criteria for judging whether the acceptances are checkable are the acceptance critic's.

```findings
- file: docs/targets.md
  line: 44
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "funnel stage 2 (project the top k through the binding table, measured in a browser) and stage 3 (judge: screenshots, model judgement, a person's choice) are in epic:engine-slices scope but no story claims either and specification-blocker:funnel-nouns holds only stage 0; claim them in a story beside story:stage-agreement or name them as held in the epic Out of Scope and the blocker"
- file: docs/roadmap.md
  line: 29
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "roadmap step 2 promises '(e) UX declarations as constructs (proposal from uxui-declarations/1)' as an ESS story, but it is not among the 18 gaps of docs/ess-gaps.md and epic:ess-dependencies Out of Scope says UX declarations belong to uxui, so the promise is silently dropped; the epic must record the drop or a story must claim it"
- file: docs/targets.md
  line: 24
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:browser-adapter acceptance returns only geometry and the accessibility tree, while the design promises boxes, computed styles, accessibility tree and a screenshot per viewport and mode; widen the acceptance or record the narrowing"
- file: .engineering/planning/story/check-tool-internals-research.md
  line: 22
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "no sentence of any parent asks for this research (the story says no source in this tree states it as a plan item) and epic:engine-slices scope names none; tie it to a parent promise or state in the epic that it is carried in from outside"
- file: .engineering/planning/story/docs-site.md
  line: 28
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "building the Docusaurus site and 'the repository's gate runs that build' is not asked for by docs/targets.md, which describes the site as an existing target; the gate clause is an invented outcome (acceptance admitted as inferred)"
```
