---
format: aep.planning-md/3
id: review-result:acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, round 1
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
story:browser-adapter — the acceptance joins four independent outcomes (writes and builds files, renders each viewport and mode, returns geometry and the accessibility tree, and stamps every record with adapter and version) and names no page to run against, so one can pass while another fails: "For a page with a URL, the browser adapter writes a variant's files and builds or serves them, renders … and every evidence record it writes names the adapter and its version." — .engineering/planning/story/browser-adapter.md:29
story:check-tool-internals-research — the acceptance says "per tool" but names no tool, so a note covering one tool and a note covering twenty both satisfy it — .engineering/planning/story/check-tool-internals-research.md:29
story:docs-site — the acceptance joins two independent outcomes, the site builds and the gate runs that build: "`website/` builds the documentation site … and the repository's gate runs that build" — .engineering/planning/story/docs-site.md:28
story:docs-site-first-climb — the acceptance joins two independent outcomes, a campaign ending in a ranked frontier and a chosen variant promoted as a diff with ratchet evidence: "ends with a ranked frontier, and the variant a person chose is promoted as a diff" — .engineering/planning/story/docs-site-first-climb.md:29
story:loop-process-improvements — "name no project of the loop's original author" has no list of project names to search for, so the second clause cannot be checked the same way twice (and it is a second outcome beside the four items held) — .engineering/planning/story/loop-process-improvements.md:26
story:migrate-smallest-ui-loop-copy — the acceptance names no copy and no measure of "smallest" among seven, so nobody can tell which copy must be checked: "The smallest of the organization's UI-loop copies runs its rounds on ratchet and uxui" — .engineering/planning/story/migrate-smallest-ui-loop-copy.md:27
story:own-generator-smallest — the acceptance joins a Rust generator that writes the live climb page with a separate document of re-implemented items, and names no path or location for either — .engineering/planning/story/own-generator-smallest.md:29
story:point-old-skill-copies-here — the acceptance names neither of the "both older copies" nor where they live, so it cannot be checked outside this repository: "Both older copies of the skill outside this repository load or link this repository's `ui-iteration-loop` skill" — .engineering/planning/story/point-old-skill-copies-here.md:24
story:ratchet-presets-and-gates — "is gated by ratchet at merge and at promote" names no observable behaviour (a refusal, an exit status, a record) and no state before the work, so it reads the same with or without the work — .engineering/planning/story/ratchet-presets-and-gates.md:27
story:spec-round-3-target-adapter — one sentence holds five independent specification changes (Target adapter, patch axis value, journey model-or-steps, evaluation key, per-file promotion diff) and its only check, `ess specify validate` exiting 0, does not show any of them is present — .engineering/planning/story/spec-round-3-target-adapter.md:29
story:studio-stock-generator — the acceptance joins three independent outcomes (budget and KPI values as map or list, step table in a declared sort order, `ess ui test` passes) — .engineering/planning/story/studio-stock-generator.md:28
story:themes-light-dark — the acceptance joins three independent outcomes (light and dark mode chosen by `theme-toggle.js`, keyed on `data-uxui-mode`, a generated app writing `data-theme` does not change the mode) — .engineering/planning/story/themes-light-dark.md:30
story:thin-the-skill — "design decisions and taste stay with people" is not observable, so that half of the acceptance is a vote, and it is joined by "and `task check` passes" as a second outcome — .engineering/planning/story/thin-the-skill.md:28
story:track-ess-gaps — the acceptance is a standing rule "For each ESS release …" with no end state, so the story can never be honestly closed, and it joins marking closed with adopting in code — .engineering/planning/story/track-ess-gaps.md:36
story:trial-target — "this repository receives only its generic findings" is a judgement, not an observable, and the admin console repository is not named: "and this repository receives only its generic findings" — .engineering/planning/story/trial-target.md:27

What I read: all 26 stories (whole bodies) via `aep plan artifact list --kind story`, `aep plan artifact lifecycle story` and `aep plan artifact show <id>` for each; checked in the tree that `presets/web-ux.ratchet.yaml`, `ess/GAPS.md`, `themes/studio.css`, `themes/theme-toggle.js` and `plugin/skills/ui-iteration-loop/SKILL.md` exist, and that `website/` does not (consistent with docs-site being in flight elsewhere). Citations are the acceptance sentence lines (heading line plus 2).
Approved without finding (acceptance observable, one outcome): cost-spike, ess-ui-needs-per-way, loop-round-on-studio, scaffold-spike, spec-hardening-pass, spec-round-2-scenarios, stage-agreement, stock-or-own-decision, studio-climb, toy-climb, ux-static-slice.

What I could not establish: I did not run `ess specify validate --path ess --format json`, so I did not confirm that it reports an "unscenarioed outcomes" count (spec-round-2-scenarios) or that `ess ui check` exists for scaffold-spike. The "fixed in ess/" test of spec-hardening-pass is per-finding judgement, which I left alone as the review gives each finding its own text. Out of my lane: the docs-site acceptance has the gate run a docs build, which may touch the tests-stay-fast budget (design); the three stories that cite held nouns (ess-ui-needs-per-way, stock-or-own-decision, studio-stock-generator) depend on blockers (coverage/design).

```findings
- file: .engineering/planning/story/browser-adapter.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins four independent outcomes (writes and builds files, renders each viewport and mode, returns geometry and the accessibility tree, and stamps every record with adapter and version) and names no page to run against"
- file: .engineering/planning/story/check-tool-internals-research.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance says per tool but names no tool, so a note covering one tool and a note covering twenty both satisfy it"
- file: .engineering/planning/story/docs-site.md
  line: 28
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins two independent outcomes: the site builds, and the repository's gate runs that build"
- file: .engineering/planning/story/docs-site-first-climb.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins two independent outcomes: a campaign ending in a ranked frontier, and the chosen variant promoted as a diff with ratchet evidence"
- file: .engineering/planning/story/loop-process-improvements.md
  line: 26
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'name no project of the loop's original author' has no list of project names to search for, so the clause cannot be checked the same way twice, and it is a second outcome beside the four items held"
- file: .engineering/planning/story/migrate-smallest-ui-loop-copy.md
  line: 27
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names no copy and no measure of smallest among seven, so nobody can tell which copy must be checked"
- file: .engineering/planning/story/own-generator-smallest.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins a Rust generator that writes the live climb page with a separate document of re-implemented items, and names no path or location for either"
- file: .engineering/planning/story/point-old-skill-copies-here.md
  line: 24
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names neither of the 'both older copies' nor where they live, so it cannot be checked outside this repository"
- file: .engineering/planning/story/ratchet-presets-and-gates.md
  line: 27
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "'is gated by ratchet at merge and at promote' names no observable behaviour (a refusal, an exit status, a record) and no state before the work, so it reads the same with or without the work"
- file: .engineering/planning/story/spec-round-3-target-adapter.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "one sentence holds five independent specification changes and its only check, ess specify validate exiting 0, does not show any of them is present"
- file: .engineering/planning/story/studio-stock-generator.md
  line: 28
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins three independent outcomes: budget and KPI values as map or list, step table in a declared sort order, and ess ui test passing"
- file: .engineering/planning/story/themes-light-dark.md
  line: 30
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins three independent outcomes: light and dark mode chosen by theme-toggle.js, keyed on data-uxui-mode, and unchanged when a generated app writes data-theme"
- file: .engineering/planning/story/thin-the-skill.md
  line: 28
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "'design decisions and taste stay with people' is not observable, so that half of the acceptance is a vote, and it is joined by 'task check passes' as a second outcome"
- file: .engineering/planning/story/track-ess-gaps.md
  line: 36
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance is a standing rule ('For each ESS release ...') with no end state, so the story can never be honestly closed, and it joins marking a gap closed with adopting it in code"
- file: .engineering/planning/story/trial-target.md
  line: 27
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "'this repository receives only its generic findings' is a judgement, not an observable, and the admin console repository is not named"
```
