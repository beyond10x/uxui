---
format: aep.planning-md/3
id: review-result:parallel-safety-round-1
kind: review-result
status: active
title: Parallel-safety critic, round 1
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
story:studio-stock-generator — edits `ess/ui/studio.ui.yaml` and `ess/ui/tests/studio.yaml` (cited), the files story:spec-hardening-pass (active, scope `ess`) already has uncommitted edits on; neither body mentions the other and no depends_on runs between them, so say so on the story (or add an edge recording the shared file, or split the surface) — .engineering/planning/story/studio-stock-generator.md:21-22,28-30 and .engineering/planning/story/spec-hardening-pass.md:30-31
story:loop-round-on-studio — runs a round on the app generated from `ess/ui/studio.ui.yaml` and writes its findings into `ess/ui/` (cited), which story:spec-hardening-pass and story:studio-stock-generator are rewriting; no edge and no mention, so its generated app and its findings go stale or conflict, and the body must name the shared surface (ordering edge or a separate findings location) — .engineering/planning/story/loop-round-on-studio.md:19-21,25-26
story:spec-round-2-scenarios — its acceptance (0 unscenarioed outcomes, every UNMAPPED marker listed in `ess/GAPS.md`) reads the outcome set of `ess/`, which story:spec-round-3-target-adapter changes in the same domain files (`design`, `journeys`, `evaluation`, `promotion`, cited); both depend on the hardening pass, none runs between them, so one wave holds both and round 2 can go red after round 3 lands; the body must say so (ordering edge, or merge/split the surface) — .engineering/planning/story/spec-round-2-scenarios.md:31-32, .engineering/planning/story/spec-round-3-target-adapter.md:29-32
story:loop-process-improvements — both this and story:thin-the-skill land on `plugin/skills/ui-iteration-loop/SKILL.md` (cited, both acceptances) and no edge or sentence says so; the body must name the shared file (ordering edge, or split SKILL.md from `references/`) — .engineering/planning/story/loop-process-improvements.md:26-27, .engineering/planning/story/thin-the-skill.md:28-29
story:ess-ui-needs-per-way — both this and story:track-ess-gaps write `docs/ess-gaps.md` (cited: it lists new requirements there; the other marks closed gaps there) and neither says so; the body must name the file (ordering edge, or one owner of the list) — .engineering/planning/story/ess-ui-needs-per-way.md:28, .engineering/planning/story/track-ess-gaps.md:36-37
story:scaffold-spike — its acceptance reads the documentation landing page that story:docs-site generates from `website/product.json`, both in flight in separate worktrees, and no depends_on edge runs from the reader to the producer; add `depends_on: story:docs-site` (or merge the two) — .engineering/planning/story/scaffold-spike.md:29-31, .engineering/planning/story/docs-site.md:28-29
story:docs-site — its acceptance "the repository's gate runs that build" lands on `Taskfile.yml` and `AGENTS.md` (inferred: body cites no path, the surface is read from `git -C ../uxui-docs-site status --short`), which the worktree `uxui-own-gen` of story:own-generator-smallest also modifies (`Taskfile.yml`, `AGENTS.md`, `Cargo.toml`) and this worktree edits `AGENTS.md` too (themes row); no body says so, so name the shared files or give each in-flight story its own gate entry — .engineering/planning/story/docs-site.md:28-29
story:own-generator-smallest — names no path it writes ("a Rust generator", "a document lists"), so it is unassessed; the worktree `uxui-own-gen` shows `generators/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `docs/own-generator.md` (inferred, from `git status` there, not from the body) and the body must record them — .engineering/planning/story/own-generator-smallest.md:29-31
story:cost-spike — names no surface: its report and Rust program have no path and the tree has no crate (`git ls-files | rg 'Cargo|crates/'` returns nothing), so it is unassessed and must either name its code home or stay out of any concurrent set — .engineering/planning/story/cost-spike.md:27-29 (heading Acceptance; search for a path returned nothing)
story:ux-static-slice — names no surface: `ux-static@1` has no crate, binary or output path and no workspace root exists yet, so it is unassessed and would also collide with story:cost-spike on whichever `Cargo.toml` is created first; name the code home — .engineering/planning/story/ux-static-slice.md:27-29 (heading Acceptance; no path in body)
story:toy-climb — names no surface: the engine code that runs the exhaustive search has no path, so it is unassessed — .engineering/planning/story/toy-climb.md:28-29 (heading Acceptance; no path in body)
story:browser-adapter — names no surface: the adapter has no crate or path, and it is the shared base for story:docs-site-first-climb and story:stage-agreement, so it is unassessed — .engineering/planning/story/browser-adapter.md:29-31 (heading Acceptance; no path in body)
story:stage-agreement — names no surface: where the agreement is computed and recorded (engine code, evidence shape) has no path, so it is unassessed — .engineering/planning/story/stage-agreement.md:27-28 (heading Acceptance; no path in body)
story:scaffold-spike — also names no path for its report or the lifted document, so its surface is unassessed from the body (the worktree `uxui-scaffold` holds `tools/scaffold/`, inferred) — .engineering/planning/story/scaffold-spike.md:29-31 (heading Acceptance; no path in body)

What I read: 8 epics, 26 stories, 6 blockers via `aep plan artifact list`, `graph`, `waves` and every story and epic body; docs/roadmap.md, docs/generator-round.md, docs/targets.md, AGENTS.md; `git ls-files`, `git status --short` and `git worktree list` for this and the five sibling worktrees (read-only). Surfaces: 16 stories cited (14 of them name a path in the body; story:studio-climb and story:own-generator-smallest name only a path they read), 2 inferred (story:own-generator-smallest, story:scaffold-spike, from sibling-worktree status), 3 in other repositories and unplaceable to a path here (story:trial-target, story:migrate-smallest-ui-loop-copy, story:point-old-skill-copies-here; not findings), 5 unplaced (story:cost-spike, story:ux-static-slice, story:toy-climb, story:browser-adapter, story:stage-agreement). Acceptance reads traced to a producer in the set: 19, of which 16 are recorded by a depends_on or blocks edge. `aep plan artifact waves` reports 1 wave, 0 collisions, 25 unassessed, which is the same state, only no scope is recorded.

What I could not establish: the file level of the unplaced engine stories (no code exists to grep); where story:studio-climb and story:docs-site-first-climb write their campaign output; whether the stock-way requirements story:ess-ui-needs-per-way files come from stories 5 and 6 of the round, which are held and not in the store (no producer to check); the same-file claims on `Taskfile.yml` and `AGENTS.md` rest on sibling-worktree status, not on bodies. Out of my lane, not counted: story:stock-or-own-decision's acceptance names `docs/generator-round.md` while story:own-generator-smallest's worktree holds its own untracked copy of that file (design or scope); nothing else.

```findings
- file: .engineering/planning/story/studio-stock-generator.md
  line: 21
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "edits ess/ui/studio.ui.yaml and ess/ui/tests/studio.yaml (cited), which story:spec-hardening-pass (active, scope ess) already has uncommitted edits on; no edge and neither body mentions the other, so the body must name the shared file (ordering edge or split the surface)"
- file: .engineering/planning/story/loop-round-on-studio.md
  line: 25
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "runs a round on the app generated from ess/ui/studio.ui.yaml and writes findings into ess/ui/ (cited), which story:spec-hardening-pass and story:studio-stock-generator are rewriting; no edge and no mention, so name the shared surface (ordering edge or a separate findings location)"
- file: .engineering/planning/story/spec-round-2-scenarios.md
  line: 31
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance (0 unscenarioed outcomes) reads the outcome set of ess/, which story:spec-round-3-target-adapter changes in the same domain files (design, journeys, evaluation, promotion; cited); no edge between them, so one wave holds both and round 2 can go red after round 3; name it (ordering edge or merge/split)"
- file: .engineering/planning/story/loop-process-improvements.md
  line: 26
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "both this and story:thin-the-skill land on plugin/skills/ui-iteration-loop/SKILL.md (cited, both acceptances) and neither says so; name the shared file (ordering edge, or split SKILL.md from references/)"
- file: .engineering/planning/story/ess-ui-needs-per-way.md
  line: 28
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "both this and story:track-ess-gaps write docs/ess-gaps.md (cited) and neither says so; name the file (ordering edge, or one owner of the list)"
- file: .engineering/planning/story/scaffold-spike.md
  line: 29
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance reads the documentation landing page that story:docs-site generates from website/product.json, and no depends_on edge runs from this reader to that producer; add depends_on story:docs-site or merge the two"
- file: .engineering/planning/story/docs-site.md
  line: 28
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance (the repository's gate runs the build) lands on Taskfile.yml and AGENTS.md, which the uxui-own-gen worktree of story:own-generator-smallest also modifies; resting on an inferred surface (sibling-worktree git status, not a body); no body names the shared files"
- file: .engineering/planning/story/own-generator-smallest.md
  line: 29
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no path it writes (a Rust generator, a document); unassessed; the uxui-own-gen worktree shows generators/, Cargo.toml, Cargo.lock, rust-toolchain.toml and docs/own-generator.md (inferred, not from the body) and the body must record them"
- file: .engineering/planning/story/cost-spike.md
  line: 27
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no surface: the report and Rust program have no path and the tree has no crate, so the item is unassessed; name the code home or keep it out of any concurrent set"
- file: .engineering/planning/story/ux-static-slice.md
  line: 27
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no surface: ux-static@1 has no crate, binary or output path, so it is unassessed and would collide with story:cost-spike on the first Cargo.toml created; name the code home"
- file: .engineering/planning/story/toy-climb.md
  line: 28
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no surface: the engine code running the exhaustive search has no path, so it is unassessed"
- file: .engineering/planning/story/browser-adapter.md
  line: 29
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no surface: the adapter has no crate or path and it is the base of story:docs-site-first-climb and story:stage-agreement, so it is unassessed"
- file: .engineering/planning/story/stage-agreement.md
  line: 27
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no surface: where the agreement is computed and recorded (engine code, evidence shape) has no path, so it is unassessed"
- file: .engineering/planning/story/scaffold-spike.md
  line: 29
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "names no path for its report or the lifted document, so the surface is unassessed from the body; the uxui-scaffold worktree holds tools/scaffold/ (inferred)"
```
