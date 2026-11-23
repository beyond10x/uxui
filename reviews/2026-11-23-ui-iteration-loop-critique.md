# ui-iteration-loop: flaws of the approach

Analysis, 2026-11-23. Inferred from SKILL.md and its references, not measured.

## A. The metrics measure proxies

1. Goodhart. `reading_load`, `glance_cost`, `actions` are "lower is better" by fiat. An explainer
   page gets better by having *less text*, a form by hiding fields. The loop rewards removing
   content and cramming into the first card.
2. No metric for the actual goal. For an explainer: does the reader understand it afterwards? For a
   tool: did the user make the right decision? Nothing measures comprehension, correctness of the
   content, trust, or aesthetics/hierarchy. A page stating wrong facts passes every gate.
3. axe-core finds only the machine-checkable share of WCAG issues (Deque's own figure: about 57 %
   of issues by volume). 0 axe violations is read as "accessible". No screen-reader run, no zoom
   to 200 %, no font scaling.
4. Headless Chromium with its fonts is the only renderer. Firefox, Safari, OS font settings,
   browser zoom and real DPI are not measured; layout numbers can differ from what people see.
5. Viewports and themes are chosen per adapter. A UI that ships one theme or gates only desktop
   sizes passes while excluding everyone else (here: mobile is "measured-only").

## B. The ratchet locks in local optima

6. Every metric, every scenario, monotone, tolerance 0 on counts. A redesign that is better
   overall but worse on one metric for a few rounds is blocked. The escape is a "decision id";
   either it becomes a rubber stamp, or it is so costly that nobody redesigns.
7. "An improvement must be taken into the baseline in the same merge": a lucky noise reading
   becomes the new floor, and the next normal run fails. Any timing-based metric (layout shift,
   time to act) turns the ratchet flaky.
8. Trade-offs cannot be expressed: one action more for a much clearer screen is always a fail.

## C. The test user is the designer's model of a user

9. The screen-only bot finds controls by accessible role and name, and is told the target. A real
   user does not know the label. The bot tests the accessibility tree, not perception: visually
   confusing but well-labelled UIs pass; visually obvious but unlabelled ones fail.
10. Scenarios, truths, viewports and the bot are written by the same team (here: the same agent
    session) that then fixes the UI. It can pick scenarios the UI already handles. The measuring
    stick is not independent of the thing measured.
11. Anchor scenarios repeated every round get overfitted: the UI learns the test.
12. Hermetic replays cover only states someone thought of; novel failures only show in rounds.

## D. AI judges AI

13. Build agent and both review agents are the same model family. "Fresh context" removes shared
    conversation, not shared bias. Their blind spots correlate.
14. "Confidence 2 when two sources agree" counts two AI reviewers reading the same report and
    screenshots as independent evidence. They are not.
15. A build agent that can edit tests, scenarios, the adapter or the baseline can make the gate
    green without improving the UI. Nothing in the skill guards those files separately.
16. The L5 vision probe is a model judged against a truth; non-deterministic, paid, and scored
    on "where to click", not on understanding.

## E. The statistics do not carry the decisions

17. 3 sessions per round: frequency is 0, 0.33, 0.67 or 1. Metric movements between rounds on n=3
    are noise; "a change that did not move its metric after 2 rounds is reverted" reverts on noise.
18. The triage formula (weights 9/3/1, cost S/M/L guessed, severity set by the reviewer) looks
    objective but is judgement with arithmetic around it. Precision in the report suggests a rigour
    the inputs do not have.
19. "At most 5 changes so each metric move can be attributed": attribution needs isolated changes
    and enough sessions; 5 changes on n=3 cannot be untangled either.

## F. Provenance and fit

20. The skill is reconstructed from a colleague's outputs. The scripts, the review prompts and the
    driver bot are "ours, pending the originals". Its numbers and rules are stated with authority
    whose origin is partly unknown.
21. Built for a live-call assistant panel (no scroll, always-visible order bar, cue truth). Other
    UIs get it through "adapter slots" and "docs-site variants" decided ad hoc; metrics such as
    cue churn are meaningless there and gates like "no scroll" are wrong for documents.
22. Cost: a Playwright + axe + ratchet + round harness is larger than a one-file page (here). For
    small or short-lived UIs the apparatus costs more than the defects it finds, and needs upkeep
    (Playwright/browser version pins, flaky layout assertions).
23. "Every finding becomes a test": the suite only grows. Brittle layout assertions accumulate and
    make every later change slower; insights that cannot be tested (tone, trust, clarity) are
    dropped silently because they have no place to go.

## G. Privacy and safety

24. rrweb recordings of real sessions capture DOM text and input: personal and customer data,
    consent, retention, GDPR. The skill allows only structure-only masking, but L5 sends
    screenshots (with text) to an external model; under company rules customer data must not
    leave.
25. L4/L5 are paid and "operator-only"; the approval is per round, so cost is not bounded per run.

## H. This run specifically (explainer page)

26. The page has no real tasks. The six scenarios were invented in one minute by the session that
    wrote the page. The round will mostly measure keyboard access of the SVG cycle and grid
    overflow on small screens; the numbers will look rigorous but reflect the scenario choice.
27. No human trial, no questionnaire, no L4/L5: only L1–L3 and L7 run. "Applied the skill once"
    means the hermetic half plus AI review.
