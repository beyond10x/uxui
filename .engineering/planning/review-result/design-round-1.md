---
format: aep.planning-md/3
id: review-result:design-round-1
kind: review-result
status: active
title: Design critic, round 1
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
story:docs-site-first-climb — its acceptance reads a campaign that ends with a ranked frontier, which story:toy-climb is the first to produce, and no depends_on edge runs to story:toy-climb (story:stage-agreement and story:studio-climb carry that edge) — .engineering/planning/story/docs-site-first-climb.md:29, .engineering/planning/story/toy-climb.md:29
story:trial-target — its acceptance has the engine run a campaign on the admin console page, which story:toy-climb is the first to build (roadmap step 6 follows step 5), and no depends_on edge runs to story:toy-climb — .engineering/planning/story/trial-target.md:27, .engineering/planning/story/toy-climb.md:29
story:stock-or-own-decision — its acceptance needs the adjustment costs and the slot web UI case study of held stories 5 and 6, so it cannot be checked until the nouns validate, yet specification-blocker:generator-round-nouns blocks only the epic and the story shows as ready; add a blocks edge from that blocker to this story — .engineering/planning/story/stock-or-own-decision.md:27, .engineering/planning/specification-blocker/generator-round-nouns.md:15
story:ess-ui-needs-per-way — its acceptance lists the stock-way requirements (extension points, theming, per-node overrides) that only held stories 5 and 6 produce, so half of it is blocked while only the own-generator half has an edge; add a blocks edge from specification-blocker:generator-round-nouns, or narrow the acceptance to the own-generator way — .engineering/planning/story/ess-ui-needs-per-way.md:26, .engineering/planning/story/ess-ui-needs-per-way.md:19

What I read: 40 artifacts (8 epics, 26 stories, 3 dependency-blockers and specification-blockers each), every body in full, via `aep plan artifact list`, `relations`, `graph` and `validate` (valid). I walked all 46 edges (26 decomposes, 14 depends_on, 6 blocks) across the whole store, outside the ids handed over as well. The graph is acyclic, and its longest depends_on path is 3 items (hardening, round 3, browser-adapter, docs-site-first-climb), so it does not serialise. I traced 14 acceptance reads to a producer in the set; 10 are recorded by an edge, and the 4 not recorded are the findings above. I did not flag the browser-adapter evidence record against story:ux-static-slice, which proves the evidence contract. The roadmap does not order them, so it is not a defect.

What I could not establish:
- Whether the blocks edges on whole epics (epic:bench, epic:engine-slices, epic:generators-and-themes) are meant to hold the epics' ready stories (story:loop-round-on-studio, story:cost-spike and others). The blockers' bodies say they do not, and the listing shows no inheritance, so I did not raise it.
- Out of my lane: story:round-2 and story:round-3 both edit `ess/` and both depend on the hardening pass. Whether they can run together is for the parallel-safety critic. story:studio-stock-generator, story:own-generator-smallest and story:loop-round-on-studio all touch `ess/ui/studio.ui.yaml`, which is also for the parallel-safety critic.
- Out of my lane: whether the inferred acceptances of the in-flight stories (docs-site, check-tool-internals-research, scaffold-spike) can be checked, and whether the migration covers all the sources, belong to the acceptance and scope critics.

```findings
- file: .engineering/planning/story/docs-site-first-climb.md
  line: 29
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads a campaign that ends with a ranked frontier, which story:toy-climb is the first to produce, and no depends_on edge runs to story:toy-climb (story:stage-agreement and story:studio-climb carry that edge)"
- file: .engineering/planning/story/trial-target.md
  line: 27
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance has the engine run a campaign on the admin console page, which story:toy-climb is the first to build (roadmap step 6 follows step 5), and no depends_on edge runs to story:toy-climb"
- file: .engineering/planning/story/stock-or-own-decision.md
  line: 27
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance needs the adjustment costs and the slot web UI case study of held stories 5 and 6, so it cannot be checked until the nouns validate, yet specification-blocker:generator-round-nouns blocks only the epic and the story shows as ready; add a blocks edge from that blocker to this story"
- file: .engineering/planning/story/ess-ui-needs-per-way.md
  line: 26
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its acceptance lists the stock-way requirements (extension points, theming, per-node overrides) that only held stories 5 and 6 produce, so half of it is blocked while only the own-generator half has an edge; add a blocks edge from specification-blocker:generator-round-nouns, or narrow the acceptance to the own-generator way"
```
