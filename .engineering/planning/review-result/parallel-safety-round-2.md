---
format: aep.planning-md/3
id: review-result:parallel-safety-round-2
kind: review-result
status: active
title: Parallel-safety critic, round 2
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
story:browser-adapter — its acceptance runs on the documentation landing page that story:docs-site builds from `website/product.json` (cited), and no depends_on edge runs from it to story:docs-site (its only edge is spec-round-3-target-adapter), so waves orders them only by the Cargo.toml chain; add `depends_on: story:docs-site` (or merge) — .engineering/planning/story/browser-adapter.md:37, .engineering/planning/story/docs-site.md:40
story:cost-spike — its body says the root `Cargo.toml` is being introduced by in-flight story:own-generator-smallest (cited), yet no edge orders them and `aep plan artifact waves` puts cost-spike in wave 1 and own-generator-smallest in wave 2, the reverse of the order the body states; add `depends_on: story:own-generator-smallest` recording the shared `Cargo.toml` as its reason (or move the crate off the root file) — .engineering/planning/story/cost-spike.md:28-30, .engineering/planning/story/own-generator-smallest.md:40-41
story:themes-light-dark — it edits the `themes/` row of `AGENTS.md` (uncommitted in this worktree: `git diff -- AGENTS.md` shows the row) but its scope lists only `themes`, so `AGENTS.md` is a third story on a file that story:docs-site and story:own-generator-smallest each name as shared with only each other; add `AGENTS.md` to its scope and name the sharing (ordering edge, or one owner of `AGENTS.md`) — .engineering/planning/story/themes-light-dark.md:40, .engineering/planning/story/docs-site.md:35-36
story:docs-site-first-climb — its acceptance names `website/product.json` and the theme CSS (cited) but the story records no scope, so `aep plan artifact waves` lists it unassessed; record `website/product.json` (and the CSS path) as scope — .engineering/planning/story/docs-site-first-climb.md:31
story:studio-climb — its acceptance runs a campaign on `ess/ui/studio.ui.yaml` (cited) but it records no scope (waves: unassessed) and no depends_on edge runs to story:studio-stock-generator or story:spec-hardening-pass, which rewrite that document; add the scope and the edge (or name the read) — .engineering/planning/story/studio-climb.md:27, .engineering/planning/story/studio-stock-generator.md:32-33
story:own-generator-smallest — its acceptance reads `ess/ui/studio.ui.yaml` (cited) to list the ESS semantics it re-implements, a file story:spec-hardening-pass (active) is editing, and neither its scope nor its body names that read; name it as a read surface and say whether the list is taken before or after the hardening pass — .engineering/planning/story/own-generator-smallest.md:45-47, .engineering/planning/story/spec-hardening-pass.md:30-31

What I read: 8 epics, 26 stories, 6 blockers and the round-1 review-results via `aep plan artifact list`, `graph`, `waves`, `validate` (valid) and every story body with its `scope` and relations; `git status --short` and `git diff -- AGENTS.md` here and `git status --short` in the five sibling worktrees. Round 1 is fixed for: studio-stock-generator and loop-round-on-studio vs spec-hardening-pass (bodies say so, edges exist), spec-round-2 vs round-3 (edge and sentence), loop-process-improvements vs thin-the-skill (edge and sentence), ess-ui-needs-per-way vs track-ess-gaps (both bodies state the shared file and that each touches only its own lines; not a defect), scaffold-spike reading docs-site (edge added), docs-site vs own-generator-smallest (both bodies name `Taskfile.yml` and `AGENTS.md`). Surfaces: 16 stories cited, 5 inferred (`Cargo.toml` stand-in), 5 unplaced. Acceptance reads traced to a producer in the set: 26, of which 21 are recorded by a depends_on edge.

What I could not establish: where the engine code lives (story:cost-spike, ux-static-slice, toy-climb, browser-adapter, stage-agreement record `Cargo.toml` as an inferred, admitted placeholder, so waves serializes them; I do not count that as a finding, but their real surface is unknown); the surface of story:trial-target, story:migrate-smallest-ui-loop-copy and story:point-old-skill-copies-here (other repositories, not findings). Not findings, noted: story:own-generator-smallest's scope omits `Cargo.lock`, `rust-toolchain.toml`, `docs/change-list.md` and `docs/generator-round.md`, which its worktree holds (`Cargo.lock` follows the `Cargo.toml` serialization; `docs/generator-round.md` is ordered by the depends_on from story:stock-or-own-decision); story:docs-site's scope omits `.github/` and `tools/docs.sh`, which no other story in the set touches; waves puts in-flight story:own-generator-smallest in wave 2 after in-flight docs-site although both are already running (the plan states the shared files, and the operator decides). Out of my lane: whether `Cargo.toml` as a stand-in surface is the right way to record an unknown code home (design or scope).

```findings
- file: .engineering/planning/story/browser-adapter.md
  line: 37
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance runs on the documentation landing page that story:docs-site builds from website/product.json (cited), and no depends_on edge runs from this reader to story:docs-site; add depends_on story:docs-site or merge"
- file: .engineering/planning/story/cost-spike.md
  line: 28
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "body says the root Cargo.toml is being introduced by in-flight story:own-generator-smallest (cited) but no edge orders them and waves places cost-spike in wave 1 before own-generator-smallest in wave 2; add an ordering edge recording the shared Cargo.toml, or move the crate off the root file"
- file: .engineering/planning/story/themes-light-dark.md
  line: 40
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "edits the themes row of AGENTS.md (uncommitted here, cited) but scope lists only themes, so AGENTS.md is a third story on a file story:docs-site and story:own-generator-smallest name as shared with only each other; add AGENTS.md to scope and name the sharing"
- file: .engineering/planning/story/docs-site-first-climb.md
  line: 31
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance names website/product.json and the theme CSS (cited) but no scope is recorded, so waves lists the story unassessed; record the paths as scope"
- file: .engineering/planning/story/studio-climb.md
  line: 27
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance runs a campaign on ess/ui/studio.ui.yaml (cited) with no scope recorded (waves: unassessed) and no depends_on edge to story:studio-stock-generator or story:spec-hardening-pass, which rewrite that document; add the scope and the edge or name the read"
- file: .engineering/planning/story/own-generator-smallest.md
  line: 45
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance reads ess/ui/studio.ui.yaml (cited), which story:spec-hardening-pass (active) is editing, and neither scope nor body names that read; name it and say whether the list is taken before or after the hardening pass"
```
