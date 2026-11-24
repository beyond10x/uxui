---
format: aep.planning-md/3
id: review-result:design-round-2
kind: review-result
status: active
title: Design critic, round 2
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
story:browser-adapter — its acceptance runs the adapter on the documentation landing page, which story:docs-site is the one to produce, and no depends_on edge runs from it to story:docs-site (only story:docs-site-first-climb and story:scaffold-spike carry that edge); add depends_on story:docs-site — .engineering/planning/story/browser-adapter.md:37, .engineering/planning/story/docs-site.md:40
story:stage-agreement — its acceptance ("a campaign that runs both stage 1 and stage 2") reads stage 2, the projection through the binding table, which no story in the set builds and specification-blocker:funnel-nouns holds, yet that blocker blocks only epic:engine-slices, so the story shows as ready; add a blocks edge from specification-blocker:funnel-nouns to this story (the fix round 1 applied to story:stock-or-own-decision), or narrow the acceptance to what stage 1 plus the browser adapter can show — .engineering/planning/story/stage-agreement.md:33, .engineering/planning/specification-blocker/funnel-nouns.md:22

What I read: all 44 artifacts in full (8 epics, 26 stories, 3 dependency-blockers, 3 specification-blockers, 4 review-results), via `aep plan artifact list`, `relations`, `graph` and `validate` (valid). I walked all 56 declared non-review edges (26 decomposes, 22 depends_on, 8 blocks) across the whole store, not only the ids handed over. The graph is acyclic. The longest depends_on path is spec-hardening-pass, spec-round-2-scenarios, spec-round-3-target-adapter, browser-adapter, docs-site-first-climb (5 items). The set does not serialise: the chains that exist carry a stated reason (shared `ess/domains/` files, shared SKILL.md). The edges that run through toy-climb also fan out, so there is no queue. I traced about 17 acceptance reads to a producer in the set; 15 are recorded by an edge or a blocker, and the 2 that are not are the findings above. All four round-1 findings are closed: toy-climb edges on docs-site-first-climb and trial-target, and blocks edges on stock-or-own-decision and ess-ui-needs-per-way. No split abstraction or horizontal slice found. The stories are vertical, and the spec rounds 2 and 3 are cut by outcome set and not by layer.

What I could not establish (none of it set the verdict):
- Whether story:ratchet-presets-and-gates should depend on story:ux-static-slice. Its acceptance reads "a promotion whose metric...", which a hand-built evidence record in ratchet's draft shape could supply, so I did not raise it. The same goes for story:migrate-smallest-ui-loop-copy ("a round on ratchet and uxui"): which uxui piece it needs is not stated.
- Whether the engine stories need story:spec-round-2-scenarios. The roadmap says round 2 "unblocks generating anything from the spec" (`docs/roadmap.md:25`), but nothing says the engine is generated from the spec and the cost spike is the first code, so I did not raise it.
- The blocks edges that sit on whole epics (epic:bench, epic:engine-slices, epic:generators-and-themes) do not hold those epics' ready stories, per the blockers' own bodies. Stating that in the blocker bodies is enough.
- Out of my lane (parallel-safety): story:ess-ui-needs-per-way and story:track-ess-gaps both edit `docs/ess-gaps.md`, and the former adds gaps beyond the 18 the latter's acceptance counts. The inferred acceptances of the in-flight stories belong to the acceptance critic, and coverage belongs to the scope critic.

```findings
- file: .engineering/planning/story/browser-adapter.md
  line: 37
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance runs the adapter on the documentation landing page, which story:docs-site produces (docs-site.md:40), and no depends_on edge runs from it to story:docs-site; add depends_on story:docs-site"
- file: .engineering/planning/story/stage-agreement.md
  line: 33
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads stage 2 (projection through the binding table), which no story builds and specification-blocker:funnel-nouns holds (funnel-nouns.md:22), yet that blocker blocks only epic:engine-slices so the story shows as ready; add a blocks edge from specification-blocker:funnel-nouns to this story, or narrow the acceptance to stage 1 plus the browser adapter"
```
