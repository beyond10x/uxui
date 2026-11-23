# Review-agent briefs

Ours, pending the colleague's originals. The colleague's repo holds only the outputs of the two
reviewers (their shape is kept below); the prompts were asked for and have not arrived. Replace these
briefs when they do.

Start both as fresh background agents after the round command, in one message. Each prompt is: "Read
<skill dir>/references/review-agents.md, section <1|2>, and follow it. The round dir is <dir>." plus
the session's own rules on scratch files and credentials (for example "no work files in a shared
`/tmp`" and "never call the GitLab API with a personal token"). `<skill dir>` is
`${CLAUDE_PLUGIN_ROOT}/skills/ui-iteration-loop`. Both save their file in the round dir before any
person sees the round.

## 1. Heuristic reviewer

Inputs: `report.md` and `shots/`; the round's recordings, replayed with the replay tool; the scenario
cards; the operator notes; `references/heuristics.md`. You may take extra shots; name them
`shots/review-*.png`.

Do:

1. Look at every contact-sheet frame in every theme, and replay each recording around its worst
   moments.
2. Judge each of the 16 heuristics: pass, partial or fail. Every verdict cites evidence: a recording
   file and second, or a screenshot file. No evidence, no verdict: write "not observed".
3. File a finding record (`ux/findings/UXF-nnn.md`, next free number, header fields from SKILL.md § 5)
   for every fail and every partial that costs the user time. Give severity, class, the metric it
   would move, the sessions it affected, and a cost estimate S, M or L.
4. Do not propose fixes beyond one line per finding. Do not read the other reviewer's output.

Write `review-heuristics.md`:

- a verdict line: `<p> pass, <q> partial, <r> fail`;
- a table: heuristic, verdict, evidence;
- "Findings filed": id, title, severity, class, sessions, cost.

## 2. Report reviewer

Inputs: `report.md`, `round.json`, `shots/`, the raw recordings, the operator notes, the round
README, the last round's `report.md`, `trend.md`, the questionnaire results, and the source of the
round and metric tools. Run nothing paid or live.

Do:

1. Recompute the headline numbers yourself from the raw recordings, not from `round.json`. Name every
   number that differs and by how much.
2. Judge whether each metric means what its name says for this round (a session cut short, a missing
   input, a scenario not comparable with the last round).
3. List what the recordings show that the report misses.
4. List defects in the tools that produced the report, as T1..Tn, each with the file and the input
   that shows it.

Write `review-report.md`:

- Verdict: "fit to send" or "not fit to send as is", plus a default action;
- § 1 Are the numbers right and meaningful?
- § 2 Findings the report misses;
- § 3 Tool defects T1..Tn;
- § 4 Caveats to send with the round.

## Driver bot

The colleague's live caller bot (a voice agent playing the counterpart in a trial) is unknown; in the
recorded rounds the operator played both sides. Until its brief arrives, a driver bot here is ours:
deterministic code that plays the scenario card's persona, answers only from the card's facts, and
plays the card's two twists at the planned points. Keep it out of the gates; it only replaces the
person in § 3.1.
