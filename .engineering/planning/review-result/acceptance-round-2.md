---
format: aep.planning-md/3
id: review-result:acceptance-round-2
kind: review-result
status: active
title: Acceptance critic, round 2
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
revision: 1
---
needs-revision
story:spec-round-2-scenarios — the acceptance joins two independent outcomes with "and" (0 unscenarioed outcomes, and every UNMAPPED marker left in `ess/` listed in `ess/GAPS.md`), so one can pass while the other fails, and the first clause's `--format json` count was not shown to exist — .engineering/planning/story/spec-round-2-scenarios.md:40
story:own-generator-smallest — the acceptance only requires a document that "lists ... next to the command that writes that page", which is true whether or not the command runs and writes a page, so the Outcome's "one working page" is never observed; name the command's run and the page it writes — .engineering/planning/story/own-generator-smallest.md:45
story:stage-agreement — "Every campaign that runs both stage 1 and stage 2 records ... the agreement" names no campaign to run and no measure of agreement, so any value passes and no finite check closes the "every" — .engineering/planning/story/stage-agreement.md:33
story:loop-process-improvements — the acceptance names four items by label only ("decision template, reviewer sections, the A/B decision order and the survey rule") with no content test, and reviewer sections already exist in `plugin/skills/ui-iteration-loop/references/review-agents.md:14,37`, so that item reads the same before and after the work — .engineering/planning/story/loop-process-improvements.md:30

Read: all 26 stories (whole bodies) by shell glob over `.engineering/planning/story/*.md` and the round-1 acceptance review-result. I read every id the `aep plan artifact list --kind story` printed. I used `aep plan artifact lifecycle story`. I checked the tree: `ess/GAPS.md`, `docs/ess-gaps.md` (18 gaps counted: 1-17 plus 8a), `ess/ui/tests/studio.yaml` and the skill references. I ran `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml`: it passes today (5 passed). It has no steps that read the budget, KPI or sort order, so `story:studio-stock-generator` names a real transition and is not a finding. I did not run `aep plan artifact show` per id; the shell glob reads the same files.
Not established: `ess specify validate --path ess --format json` timed out at 100 s, so I did not confirm that it reports an unscenarioed-outcome count (`story:spec-round-2-scenarios`). I did not confirm that `ess ui check` exists (`story:scaffold-spike`). I did not check that every finding R1-R37 of the review gets its disposition; `ess/GAPS.md` has no R-ids yet, so that story's transition is open and checkable. The "fixed in ess/" test in `story:spec-hardening-pass` is per-finding judgement, and I left it alone. I wrote a scratch file under the session directory to hold the validate output and removed it.
Out of my lane: `story:docs-site` and the held-noun stories (`story:ess-ui-needs-per-way`, `story:stock-or-own-decision`, `story:studio-stock-generator`) depend on blockers (design and scope). The overlap of `story:thin-the-skill` and `story:loop-round-on-studio` (findings recorded in two places) is out of my lane too.

```findings
- file: .engineering/planning/story/spec-round-2-scenarios.md
  line: 40
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins two independent outcomes with and (0 unscenarioed outcomes; every UNMAPPED marker left in ess/ listed in ess/GAPS.md), so one can pass while the other fails"
- file: .engineering/planning/story/own-generator-smallest.md
  line: 45
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance only requires a document listing the re-implemented items next to the command that writes the page, which holds whether or not the command runs and writes a page, so the Outcome's one working page is never observed"
- file: .engineering/planning/story/stage-agreement.md
  line: 33
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "every campaign that runs both stages records the agreement names no campaign to run and no measure of agreement, so any value passes and the every cannot be closed by a finite check"
- file: .engineering/planning/story/loop-process-improvements.md
  line: 30
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names four items by label only with no content test, and reviewer sections already exist in plugin/skills/ui-iteration-loop/references/review-agents.md, so that item reads the same before and after the work"
```
