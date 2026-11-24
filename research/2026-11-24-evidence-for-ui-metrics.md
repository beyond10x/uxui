# Evidence for computable UI metrics

Date 2026-11-24. The question: which computable measures of UI and UX quality have evidence that they
predict what people experience (task time, errors, success, perceived usability, aesthetics, trust)?
And which of the 47 metrics in `docs/metrics.md` deserve to rank variants?

This note goes deeper than `research/2026-11-23-external-machine-checkable-ux.md` (the "11-23 note")
and checks its claims (§ 11). Public web only.

Markers:

- **[P]**: read in the primary full text.
- **[A]**: read in the primary abstract or publisher page only.
- **[S]**: secondary source only; not traced to the primary.
- **[Inference]**: our own reading, not a source claim.

Sources are keyed in brackets and listed in § 13.

## 1. Bottom line

1. **Close variants are where every predictor is weakest, and close variants are what the engine
   ranks.**
   - KLM, GOMS and CogTool got the direction of measured differences wrong in 2 of 6 cases on three
     alternative sites [Jor15].
   - Multimodal LLM judges score "only slightly above 50%" when people's preference between two UIs is
     small [Lue25].
   - GPT-4V picks the better UI of a pair 51.58% of the time [Wu24].
   - The best learned saliency model ranks elements in variants of one design at rs = .53 [Byl17].
   - No study found reports rank agreement with people over many close variants for any metric in our
     catalogue. This is the likely reason hermetic metrics did not separate the 28 variants in the
     earlier loop.
2. **There is no single "UX quality" to predict.**
   - Measured with people, time correlates with satisfaction at r = .145, errors with satisfaction at
     .196, and time with errors at .316: Hornbæk & Law's meta-analysis of 73 studies, as quoted in
     [SL09] [P].
   - With task-level ratings, the correlations rise to r .44–.60 [SL09] [P].
   - A metric that predicted task time perfectly would still say little about perceived usability.
     Each metric must name the one outcome it predicts.
3. **Strong evidence, absolute and relative:**
   - Fitts' law for pointing time within one device: r usually > .9 on mean times [SM04].
   - KLM for error-free expert time on a known path: RMS 21% per task, 5–6% over task sequences
     [CMN80] [P].
   - Target size against touch errors [Par06] [Hen11] [Fin17].
   - Body text size against comprehension [Rel16].
   - Word count (concision) against task time and usability [MN97].
   - Grouping and density against visual search time, as a relative measure only [Tul84] [Don91].
4. **Moderate evidence, coarse only.** Computed visual complexity and clutter predict mean ratings
   across very different pages:
   - Reinecke complexity r = .80, in-sample [Rei13].
   - Feature Congestion against perceived clutter of websites r = .92, n = 10 [LR11].
   - They explain 7% of an individual's appeal rating once site and person effects are removed
     [RG14] [P].
5. **Standards consensus without outcome validation:** WCAG contrast thresholds, 24 px target size,
   focus order, keyboard traps, reflow at 320 px. They are legitimate gates, but no study links their
   thresholds to user failure rates. Two controlled studies found that WCAG conformance level did not
   change task success [Pow12] [Vol23].
6. **No evidence:**
   - Ngo-style balance and symmetry, for which human agreement is low (balance r = .053 [ZV16]).
   - Type-scale, spacing-unit, radius and alignment-edge counts.
   - "One primary action per region".
   - Glance cost.
   - Cue churn.
   - Focus-order tau.
   - Layout-shift thresholds: the authors of CLS state that no research informs them [WD20] [P].
7. **Goodhart is a direct threat to this engine.** Picking the best of n variants by a proxy is
   best-of-n optimisation. True quality first rises with n, then falls [GSH23]. Computed, non-learned
   rewards were gamed in generated SVGs (§ 9).

## 2. What "predicts" has to mean for a ranking engine

| level | question | what the literature usually reports |
|---|---|---|
| absolute | does the metric give the right seconds or rating? | RMS or percent error, R² of means |
| across designs | does it order very different designs right? | correlation over many sites or screens |
| among close variants | does it order variants of one design right? | almost never measured (§ 1.1) |
| per person | does it predict one person's experience? | near the human ceiling, which is low (§ 6) |

The engine needs the third row.

- **[Inference] Tie band.** A metric's prediction error gives the difference below which two variants
  count as tied. For KLM that is about 20% per task [CMN80] and about 10% for aggregated tasks
  [Swe13]. Ranking inside that band is noise.
- **Human ceiling.** People agree weakly on close variants:
  - Krippendorff α = .37 among 12 designers [Wu24];
  - α = .308 among 17 designers on 510 pairs of variants [Gao26];
  - one rater against the others r = .34–.60 [Lin06] [P].
  - A metric cannot be validated against close-variant judgements more precisely than that.

## 3. Performance models

### 3.1 KLM

The source is [CMN80] [P].

- **Fit:** RMS error 21% of predicted execution time on single unit tasks, over 10 systems.
  - Methods were prescribed by the experimenters.
  - Tasks with errors or methods other than the prescribed one were dropped; 855 instances (69%)
    remained.
  - On sequences of 4–5 tasks the RMS error was 5% (text editors) and 6% (graphics editors).
- **M = 1.35 s** was fitted on the same data. With M at 1.2 s or 2.0 s the error rises from 21% to 23%.
- **Simpler models are worse** (this matters for `effort.actions` and `effort.keys_keyboard`):

  | model | RMS error |
  |---|---|
  | count of all operators × one constant (0.43 s) | 34% |
  | physical operators + flat mental overhead | 45% |
  | keystrokes only | 49% |

- **Flat P = 1.1 s** overpredicts short moves to large targets [John04].

### 3.2 CogTool and GOMS, and ranking alternatives

- **[John04] [P].** CogTool models were generated automatically and compared with users on 2 designs ×
  2 tasks.
  - Errors: +2%, 0%, −19% and −5%; mean absolute error 6%.
  - Hand-built KLMs: 7–13% by experts, 32% by students, 46% by a novice.
  - Every model ordered the two designs right, but those designs differed about 3.5× in time.
- **[Jor15] [A].** 3 tasks × 3 websites, 20 expert users.
  - "For all tasks and all modeling tools, the model-predicted trend did not correspond to the trend
    in the human performance data."
  - Of 6 significant human differences, the models got 4 directions right and 2 wrong. Humans were
    faster on the interface predicted slower "in one third of the cases."
  - Mean error of the predicted difference: KLM 5.49 s, GOMS 3.98 s, CogTool 3.49 s. The differences
    between the tools were not significant.
- **[Gray93]** (Project Ernestine). This is the best evidence for relative prediction.
  - CPM-GOMS predicted the new workstation 0.63 s slower per call; the field trial of 72,450 calls
    measured 0.65 s slower.
  - The absolute times were underpredicted by about 4.3%, a constant bias.
  - Direction was right in 12 of 15 call categories.
  - A simple keystroke-count estimate predicted the new workstation 18.6% *faster*: "wrong in both
    magnitude and sign". The removed keystrokes sat off the critical path.
  - [Inference] Our serial KLM on π_J has no critical-path model. It can make the same mistake where
    waits overlap with user actions.
- **[Swe13]** (CogTool-Helper, ICSE 2013, not CHI).
  - It compared LibreOffice versions by predicted time with no new human data.
  - It treats differences under 10% as undetectable, citing CogTool's ±10% accuracy.

### 3.3 Fitts' law

- **[SM04] [P].** Correlations are "typically … very high, usually greater than 0.900".
  - ISO 9241-9 conforming mouse throughput: 3.7–4.9 bits/s.
  - Touchpad: 0.99–2.9 bits/s.
  - Isometric joystick: 1.6–2.55 bits/s.
  - The coefficients must be calibrated per device.
- **Real web use** [ZW] [P; venue not checked]: 1,047 users.
  - Mean times r = .98.
  - Single movements: mean absolute deviation 46.4%. Averaging 40 or more movements brings it to
    about 10%.
  - [Inference] A sum of ID over one journey path is a valid relative cost; one move is not
    predictable.
- **Touch** [Bi13] [P]: FFitts law (Fitts plus an absolute finger-error term) fits better.
  - 1D: R² .96 vs .86 for the effective-width model.
  - Error rates were 16.8–17.8% for 2.4–7.2 mm targets.

### 3.4 Hick-Hyman, menus and visual search

- **[Coc07] [P].** Menu selection model.
  - Novice linear search: T = 0.08n + 0.3 s, R² = .99.
  - Expert Hick-Hyman: T = 0.08·log2(n) + 0.24 s, R² = .98.
  - Prediction across menu designs was within 0.8–1.8% of measured means.
  - It also ranked them right: frequency-split menus predicted 21.8% faster, measured 21.5%.
  - It needs item usage frequencies as input.
- **[Liu20] [S].** Choice time in HCI tasks is often better treated as constant; Hick's law is "largely
  irrelevant".
- **[HH11] [P].** EPIC active-vision search model, predictions made a priori.
  - Randomly grouped words: search-time error 6.5%.
  - Semantically grouped words: error 42.6%. People skipped whole groups after one fixation, and the
    model has no semantics.

### 3.5 Information scent

These models need label semantics and goal texts.

- **CogTool-Explorer** [Teo12] [S via citing paper; dissertation P].
  - Explains 63–82% of variance in clicks to success over 108 tasks on 3 layouts.
  - In the dissertation, AutoCWW reached R² .04 on the same measure.
- **SNIF-ACT 2.0** [FP07] [S]: R² .94–.98 for success rate on two sites.
  - The link-click fits come from model tracing, not free simulation.
- [Inference] Scent is computable without I/O: embedding similarity between a goal text and the
  labels on screen, with a frozen local model. It is the only family with evidence that bears on the
  copy axis ("Edit" vs "Edit ticket"). It needs goal texts per journey, which campaigns do not
  declare today.

### 3.6 Clicks, scrolling, below the fold

- **[Por03] [S].** 44 users, 620 tasks, more than 8,000 clicks.
  - No correlation between click count and success.
  - No more quitting at 3 clicks than at 12.
  - [Lau19] [P]: the three-click rule is "not supported by data in any published studies".
- **[Fes18] [P].** Eye tracking, 120 participants.
  - The first screenful gets 57% of viewing time; the first two get 74%; the first three get 81%.
  - In 2010 the first screenful got 80%.
- **[Bak03] [S].** Paging was slower than scrolling for reading, with no difference in comprehension.

## 4. Computational aesthetics and complexity

| measure | best number found | against | granularity | source |
|---|---|---|---|---|
| Ngo et al. 14 layout measures | validation r **unknown** in the primary; "high correlation" per citing papers | ratings | few layouts | [Ngo03] [S] |
| Ngo balance / symmetry / simplicity, independent test | Pearson .053 / .289 / .587 | 15 raters, 10 UIs; raters' kappa .26–.49 | real UIs | [ZV16] [P] |
| Ngo symmetry, balance | dropped: values "did not correspond to our perceptual judgments" | — | — | [RG14] [P] |
| Ngo aesthetic level → search time | 4.68 s (high), 4.87 s (medium), 5.07 s (low); N = 22 | search time | abstract stimuli | [Sal13] [P] |
| Ngo aesthetic level → search time, realistic stimuli | no preference–performance correlation | search time | realistic stimuli | [Sal13] [P] |
| Feature Congestion / Subband Entropy / Edge Density | r = .74 / .75 / .83 with log search time | 19 maps, 6 subjects | maps | [Ros07] [P] |
| Feature Congestion on websites | log perceived clutter r = .92, R² .85 | n = 10 sites, 29 raters | different sites | [LR11] [P] |
| Miniukovich & De Angeli metric sets | up to 51% (aesthetics) and 50% (complexity) of variance on web; 32% on apps | ratings | different sites/apps | [MD14] [MD15] [A] |
| Reinecke complexity / colourfulness models | r = .80 / .88, in-sample, no hold-out | 450 sites | different sites | [Rei13] [P] |
| Same features plus demographics → appeal | marginal R² = .07, conditional R² = .47 | 2.4 M ratings, 430 sites | per person | [RG14] [P] |
| Tullis density, grouping, layout complexity | a priori on 150 new displays: r = .80 (search time), .80 (ratings) | search time, ratings | different displays | [Tul84] [S] |
| Tullis, independent test | predicted vs observed r = .75; absolute 3.4 s predicted vs 15.4 s observed | search time | Shuttle displays | [Don91] [P] |
| Alignment on web pages | "not shown to be a performance-enhancing factor"; uniform density helped | performance | web pages | [Par05] [A] |
| Aesthetics → objective performance | meta-analysis g = 0.12, heterogeneous; 25 studies, N = 3,025 | performance | — | [Thi19] [P] |
| Aesthetics → time | F(2,118) = 0.01, p = .987 | completion time | — | [Tra00] [P] |
| Aesthetics → perceived usability | r = .66 before use, .71 after | perceived usability | — | [Tra00] [P] |
| Design look in credibility | cited in 46.1% of credibility comments | trust (self-report) | — | [Fog03] [S] |
| Appeal at 50 ms vs 500 ms | r = .947 between exposure times; one rater vs the others .34–.60 | ratings | — | [Lin06] [P] |

**Aalto Interface Metrics** [AIM18] [P] is a 4-page service paper with 17 metrics. It reports no
validation of its own: each metric's validity is that of its source paper above.

Readings:

- Complexity, clutter and density track **mean** ratings and search time **across very different
  pages**.
- They do not track individual appeal: 7% of the variance [RG14].
- Their effect is not monotone:
  - perceived clutter grows exponentially, with little effect below Feature Congestion ≈ 9 [LR11];
  - users disliked very simple screens [CM94] [S].
- No study found tests them among close variants of one design.
- Aesthetics changes *perceived* usability and trust, not performance [Tra00] [Thi19].

## 5. Saliency and attention models for GUIs

CC is the correlation between the predicted and the measured fixation-density map. For reference, one
human against the rest reaches CC .52 on natural images [Bor] [S]. No ceiling is reported for UI
datasets.

| model / study | data | result | source |
|---|---|---|---|
| Itti-Koch, BMS, GBVS on mobile UIs | 30 people, 193 UIs | CC .082, .131, .580 | [Lei20] [P] |
| SAM retrained on mobile UI data | same | CC .834 | [Lei20] [P] |
| UMSI++ (trained on UEyes) | 62 people, 1,980 UIs, test 108 | CC .833, AUC .905 | [Jia23] [P] |
| UMSI pretrained / GBVS on UEyes | same | CC .431 / .314 | [Jia23] [P] |
| Cross UI type (train on one, test on another) | same | AUC drops by about .05–.09; "could not truly be generalized to other UI types" | [Jia23] [P] |
| Webpage saliency | 149 pages, 11 subjects | CC .40 (.45 with position bias) | [SZ14] [P] |
| Importance in graphic designs | clicks; 264 variants of 11 designs | element-rank rs = .53 among variants | [Byl17] [P] |
| UMSI on annotation data | importance annotations, not eye tracking | CC .915 (GDI), .875 (Imp1k) | [Fos20] [P] |

Readings:

- Bottom-up saliency models fail on UIs (CC .08–.58).
- UI-trained models reach about .83 within the UI type they were trained on.
- The only close-variant figure is rs = .53.
- [Inference] A frozen UI saliency model can run without I/O and estimate whether a required node or
  primary action draws attention. It is a soft score at best, valid only for UI types like its
  training data.

## 6. Accessibility: automated checks against audits and users

### 6.1 Coverage of automated tools

- **Deque 2021** [Deq21] [P]: 13,000+ pages, 2,000+ first audits, 294,958 issues.
  - axe-core found 57.38% of the issues by volume.
  - Only 16 of 50 WCAG 2.1 AA success criteria (SC) had any automated finding.
  - Contrast (1.4.3) is 30.08% of all issues; 83.11% of those were found automatically.
  - 4.1.1 Parsing, obsolete in WCAG 2.2, is 11.69%.
  - Recomputed from the report's table (our arithmetic): 53.0% without 4.1.1; 37.5% without 4.1.1 and
    1.4.3.
  - SC with 0% automated findings: 2.4.3 Focus Order, 2.4.7 Focus Visible, 1.4.11 Non-text Contrast,
    1.4.10 Reflow, 2.1.2 Keyboard Trap.
- **[Vig13] [A]:** 6 tools.
  - At most 50% of SC were covered.
  - Completeness 14–38%.
  - Correctness: 93–96% for the less complete tools, 66–71% for the more complete ones.
- **UK GDS 2017** [GDS17] [P]: 143 barriers on one artificial page, 10 tools.
  - Best tool: 37% (errors and warnings) or 41% (including manual prompts).
  - All tools combined: 71%; 29% were missed by every tool.
- **Manual audits are noisy too** [Bra10] [A]: in about 50% of SC, experts fell short of 80%
  agreement.
  - Experts: 20% false positives, 32% of true problems missed.
  - Pooling two experts finds at most 76% of true problems [Bra12] [A].

### 6.2 Do violations predict what users hit?

- **[Pow12] [P].** 32 blind users, 16 sites, 1,383 problems.
  - WCAG 2.0 SC covered 50.4% of the problems users hit.
  - Conformance level did not reduce problems: F = 1.107, n.s.
  - The **number of distinct SC violated** correlated with problems per site per user: r = .54. The
    **number of violation instances** did not.
  - The largest uncovered categories: "content found where not expected" (99), "not found where
    expected" (88).
- **[Vol23] [P].** Randomised: 66 visually impaired and 65 sighted users, one shop at "no conformance"
  vs AA.
  - Completion: F(1,127) = 0.17, p = .683. Impaired users completed 66.7% at no conformance vs 73.9%
    at AA.
  - No effect on time or UX.
- **[Sch17] [A].** Higher conformance gave better performance and ratings for both groups. Effect
  sizes unknown.
- **[Pet07] [A].** The problem sets of blind and sighted users overlap by about 15%.
- **Prevalence, WebAIM Million 2026** [WAM26] [P]: 95.9% of home pages had detected failures, 56.1
  per page. Low contrast on 83.9% of pages.

### 6.3 Thresholds

- **Contrast.**
  - WCAG's 4.5:1 is ISO's 3:1 × 1.5, for the contrast sensitivity lost at 20/40 acuity [W3C-143]
    [P]. It has no reading-performance validation.
  - For normal vision, reading rate fell by less than half for a tenfold reduction in contrast
    [Leg87] [A].
  - Positive polarity is better for proofreading at equal contrast: F(1,165) = 9.92, η² = .06
    [Pie13] [P]. WCAG 2's symmetric ratio cannot express this.
  - **APCA:** no peer-reviewed study with human readers validating it against WCAG 2 was found. We
    found only formula-against-formula comparisons.
- **Target size.**
  - WCAG 2.5.8 chose 24 px "after reviewing the average target size of various high-traffic sites"
    [W3C-258] [P]. No user study is cited.
  - Errors could not be told apart above 9.6 mm (discrete targets) or 7.7 mm (serial targets)
    [Par06] [P].
  - Error rate exceeded 40% under 8 mm in 120 M touches [Hen11] [P].
  - Users with motor impairment: 42.1% errors at 6 mm, 7.0% at 18 mm [Fin17] [P].
  - [Inference] At about 150–160 CSS px per inch on a phone, 24 CSS px is about 4 mm: below every
    empirical touch threshold.
- **Reflow.** Usability was higher and nausea lower with word-wrap than with a magnifier that needs
  horizontal scrolling; comprehension did not differ [Hal17] [A]. No task-time cost at 320 px was
  found.
- **Focus order and keyboard traps.** No empirical study linking them to measured user failure was
  found. The "67% abandon after a trap" figure that circulates is unsourced.

## 7. Readability and copy

- **Concision.** [MN97] [P]: 51 users, 5 versions of one site, between subjects.
  - Composite usability: concise text (about half the words) +58%, scannable +47%, objective +27%,
    combined +124%.
  - Raw task time: 359 s (control), 209 s (concise), 149 s (combined).
  - Some table values in the report are inconsistent.
- **How much is read** [Wei08] [P], [Nie08] [P]: 25 users, 45,237 page views.
  - Time on page ≈ 25 s + 4.4 s per 100 words, for pages of 30–1,250 words.
  - About 20% of the words are read.
  - Lower-literacy users read every word [Sum05] [P].
- **Reading speed** [Bry19] [A]: 190 studies, 18,573 participants. Silent reading 238 wpm (non-fiction),
  260 (fiction).
  - This describes reading every word. For scanning web users the marginal cost per word is about
    5× lower [Wei08].
- **Readability formulas** (Flesch and similar) were built for schoolbooks and ignore layout and
  lists [Red00] [A]. NLP models beat them at predicting comprehension [Cro19] [A]. No validation for
  UI text was found.
- **Error messages** [Shn82] [P]: specific messages gave 28% better repair scores. "?"-only messages
  produced 10.7 errors vs 6.1 with brief messages.
- **Text size** [Rel16] [A + author summary]: 104 readers.
  - Comprehension was significantly lower at 10 and 12 pt.
  - Readability improved up to 18 pt (24 px).
  - Line spacing had a small effect.
- **Plain language with low-literacy users** [Sum05] [P]: success rose from 46% to 82%. Content and
  navigation changed together.

## 8. Learned predictors and model judges

| predictor | number | ground truth | close variants? | source |
|---|---|---|---|---|
| Tappability CNN | precision 90.2%, recall 87.0%; the `clickable` attribute alone 89.9% / 79.6% | 5 workers; full agreement on 58%, κ .52 | — | [SL19] [P] |
| Tappability, pixels only | AUC .903 | same kind | — | [Sch22] [P] |
| Webthetics | r = .85 with mean rating on 98 test sites | crowd ratings (as in [RG14]) | no | [Dou19] [P] |
| UIClip | 75.12% pair accuracy | 12 designers, α .37 | partly | [Wu24] [P] |
| GPT-4V / LLaVA on the same pairs | 51.58% (refused about 10%) / 54.59% | same | — | [Wu24] [P] |
| GUI-usability agents, fine-tuned | AUC .607 (base agents at or below chance) | 17 designers, 510 variant pairs, α .308 | yes | [Gao26] [P] |
| MLLM judges, 30 UIs, 9 factors | 59.6–60.0% pair accuracy; "only slightly above 50%" when human preference differs little; about 90% for large gaps | crowd | yes | [Lue25] [P] |
| GPT-4o on perturbed designs | r = .52–.77 with humans; "cannot distinguish small details" | 60 raters | no | [Har24] [P] |
| Web dev judge | best 70.34% vs human–human 84.56%; prefers the second position 15.8% | experts | — | [WDJ26] [P] |
| GPT-4 heuristic evaluation | F1 .466 vs .478 for one expert | experts | — | [Dua24] [P] |
| GPT-4 heuristic evaluation, across rounds | accuracy fell with each round of design improvement | experts | — | [Dua24] [P] |
| GPT-4o usability issues | found 21.2% of the experts' issues; 24.3% false positives | experts | — | [Gue25] [P] |
| UICrit few-shot critique | quality .31 → .48 (human critiques .75) | 6 experts, only 6 UIs | — | [Dua24b] [P] |
| UICrit screen ratings vs store ratings | r = .007–.023 | app-store ratings | — | [Dua24b] [P] |
| Web page metrics (WebTango) | good vs not good: 67% overall; in-sample | expert award scores | no | [ISH01] [P] |
| Web page metrics, what they explain | about 10% of variance in overall score | same | no | [ISH01] [P] |

General LLM-judge biases [Zhe23] [P]:

- Position consistency on near-identical answers: GPT-4 65.0%, Claude-v1 23.8%.
- Self-preference: GPT-4 +10% win rate for its own answers, Claude-v1 +25%.

A model recognises its own output 73.5% of the time, and self-preference grows with that recognition
[Pan24] [P].

No model predicting SUS or UMUX from a UI alone was found.

## 9. Simulated users

- **[TUI26] [P]** (Agent-KLM): 84 tasks, 15 terminal UIs, 6 people.
  - Success: humans 76.4%, agent 82.9%.
  - Per-task completion correlation ρ = .35, 95% CI [−.03, .57].
  - Operation cost (keystrokes) correlated at ρ = .77; interpretation cost at ρ = .62.
- **[SCW25] [P]** (synthetic cognitive walkthrough).
  - Completion: GPT-4 100%, Gemini-2.5-pro 97.2%, humans 88.2%.
  - Humans found 10 failure points; all model runs together found 3.
- **[Lu25] [P]:** 31,865 shop sessions.
  - Best prompt-only next-action accuracy: 11.86%.
  - Fine-tuned 7B model: 17.26%.
- **[AAB26] [P]** (Agent A/B): one case study.
  - Agent purchases 414 vs 404, p = .03. The table prints 403, an inconsistency.
  - "Aligned directionally" with a human test, whose numbers are not reported.
  - Humans took 15.96 actions per session, agents 6.05.
- **SimUser** [Xia24] [A]: feedback similarity "from 35.7% to 100%" by group and category. The
  70% / 80% claim of the 11-23 note was not found in the abstract and is unverified (§ 11).
- **UXAgent** [Lu25b] [P]: no comparison of agent and human success. The authors say agents are "not
  meant to replace human participants."
- **Benchmarks:**
  - WebArena: best agent 14.41% vs humans 78.24% [Zho23] [P].
  - OSWorld: 12.24% vs 72.36% [Xie24] [P].

Reading:

- Agents match aggregate success and disagree task by task.
- They over-succeed and find fewer problems.
- Their **interaction costs** track people better than their **success** does. That favours computed
  KLM-style costs over agent success as a metric.

## 10. Goodhart in UI and experimentation practice

- **Best-of-n** [GSH23] [P]. True reward against proxy optimisation follows R(d) = d(α − βd), with
  d = √KL and KL = log n − (n−1)/n for best-of-n. It rises, peaks, then falls.
  - The fit holds up to n = 1,000 and was validated to n = 60,000.
  - Choosing the best of 3,456 variants by fixed metrics is this setting [Inference].
- **Computed rewards get gamed too.** In RL-trained SVG generation (NeurIPS 2025, arXiv id not
  recorded) [S via the agent's reading]:
  - a 1×1 viewBox inflated the image reward;
  - a length reward collapsed the output;
  - `<text>` rendered the prompt.
  - Learned image rewards: optimising incompressibility collapsed into noise, and a VLM reward was
    fooled by writing the answer as text [Bla24] [P]. Tuning on an aesthetic score collapsed diversity
    [Cla23] [P].
- **A/B practice.**
  - Higher ad load raised short-term revenue; long-term revenue was "essentially zero". Users learned
    to ignore ads with a half-life of about 60 days [HOT15] [P].
  - A ranking bug in a search engine *raised* queries per user by more than 10% and revenue per user
    by more than 30% [Koh12] [P].
  - It is "very easy to increase CTR on some element of the page at the expense of other elements"
    [Dmi17] [P].
- **Dark patterns** [LS21] [P]: n = 1,963.
  - Acceptance: 11.3% (control), 25.8% (mild), 41.9% (aggressive).
  - Mild patterns caused no measurable backlash.
  - A metric of conversion or of fewer steps rewards them.
- **Confirmations wear out.** Without countermeasures, habituation cut the share of people who noticed
  a change in a dialog threefold [BL14] [A]. A just-in-time, specific cue eliminated postcompletion
  errors; non-specific cues did not [CB08] [A].
- **Taxonomy:** regressional, extremal, causal and adversarial Goodhart [MG18].
  - Extremal Goodhart is the engine's case: the search drives metrics into ranges where the
    metric–outcome relation was never measured [Inference].

## 11. Checks of the 11-23 note

| claim | check |
|---|---|
| KLM RMS 21% | confirmed [CMN80] [P]. Add: 5–6% over task sequences; 34–49% for count-only models. |
| H = 0.4 s secondary only | not re-checked here. |
| CogTool about 10% (John 2004) | [John04] [P] reports 6% mean absolute error on 4 cells. The "±10%" is how [Swe13] (ICSE 2013) cites it. |
| Groningen: 5.49 / 3.98 / 3.49 s | confirmed; Jorritsma et al., Procedia Manufacturing 2015 [Jor15]. Add: wrong direction in 2 of 6. |
| Fitts mouse 3.7–4.9 bits/s | confirmed [SM04] [P]. |
| Deque 57.38% | confirmed [Deq21] [P]. Add: 30% of issues are contrast; 37.5% without contrast and parsing (our recomputation). |
| GDS: 142 barriers, best 40%, worst 13%, combined 71% | 143 barriers; best 37% or 41% depending on counting; combined 71% [GDS17] [P]. "Worst 13%" not re-checked. |
| Brysbaert 238 wpm | confirmed [Bry19] [A]; 18,573 participants. |
| Duan CHI 2024: F1 like one expert | confirmed (.466 vs .478) [Dua24]. Add: accuracy fell as designs improved. |
| arXiv 2507.02306: 73% / 77% | confirmed. |
| arXiv 2506.16345: 24.3% false positives | confirmed. Add: it found 21.2% of the experts' issues. |
| WebArena 14.41% vs 78.24% | confirmed. |
| SimUser 70% scenarios, 80% issues | **unverified**: the abstract says 35.7% to 100% [Xia24]. |
| Agent-KLM: similar success, different tasks | confirmed. Per-task ρ = .35 with a CI crossing 0 [TUI26]. |
| UXAgent arXiv 2502.12561 | that ID is a different UXAgent paper (a framework). The CHI EA 2025 system paper read here is arXiv 2504.09407 [Lu25b]. |
| CLS "good ≤ 0.1" | the threshold source states "we are not aware of research that can directly inform the thresholds for this metric". 0.1 rests on internal page evaluation; 0.25 on the CrUX distribution [WD20] [P]. |
| concept.md § 4: KLM ranks close designs badly | confirmed [Jor15]. |

## 12. The catalogue judged

Evidence grades:

- **strong:** replicated against measured user outcomes.
- **moderate:** one or a few studies, or across-design only.
- **weak:** indirect or mixed.
- **consensus:** a standard with no outcome validation.
- **spec:** a correctness check of declared truth. It needs no predictive evidence: a failure is a
  defect by definition, while a pass predicts nothing about people.
- **none:** no evidence found.

Roles:

- **gate:** a hard rule.
- **soft:** a score that ranks only outside its tie band.
- **diag:** shown, never ranked.
- **drop:** remove from the catalogue.

### 12.1 Per metric

| metric | evidence | basis | recommendation |
|---|---|---|---|
| `completion.journey_completes` | spec | A walker that knows the target is a lower bound: a pass does not predict human success (per-task ρ .35 [TUI26]; agents over-succeed [SCW25]). | gate (keep) |
| `completion.keyboard_parity` | spec + consensus | WCAG 2.1.1; no outcome study found. | gate |
| `completion.miss` | spec | correctness of declared required nodes | gate |
| `completion.false_complete` | spec, weak support | Postcompletion errors are removed by a just-in-time, specific cue [CB08]; a premature completion signal is the opposite. | gate |
| `completion.refusal_visible` | spec, weak support | Specific error messages help repair [Shn82]; no study of the message's location. | gate |
| `completion.content_true` | spec | correctness | gate |
| `effort.actions` | weak | Count-only models: RMS 34–49% [CMN80]; clicks do not correlate with success [Por03]. | soft, tie-breaker only |
| `effort.actions_min` | n/a | diagnostic by design | diag |
| `effort.klm_time` | strong (absolute, error-free experts); weak (close variants) | 21% per task, 5–6% over sequences [CMN80]; direction wrong 1 in 3 [Jor15]; serial sums miss parallel slack [Gray93] | soft, with a tie band of 20% per journey [Inference from CMN80]. Replace flat P by per-environment Fitts (below). |
| `effort.fitts_id` | strong (relative, within one device) | r > .9 on means [SM04]; single moves ±46% [ZW]; touch needs FFitts [Bi13] | soft. Compute per environment; for touch add the finger-error term. |
| `effort.keys_keyboard` | moderate | keystroke-only model RMS 49% [CMN80]; agent–human keystroke cost ρ .77 [TUI26] | soft, tie-breaker |
| `effort.wait_virtual` | moderate (mechanism), numbers not gathered here | response-time research: immediate response for control tasks [DM11] [A] | soft |
| `attention.always_visible` | spec | declared requirement | gate |
| `attention.offscreen_needed` | moderate | 57% of viewing time on the first screenful, 74% on two [Fes18]; but paging is slower than scrolling [Bak03] [S] | soft |
| `attention.glance_cost` | none | no formal model (11-23 note); the 1–3 levels are invented | diag |
| `attention.visible_required` | spec-like | declared requirement; no outcome study | soft (or gate where declared) |
| `attention.regions_in_view` | moderate (as Tullis grouping) | Number and size of groups were the strongest search-time predictors; a priori r .80, relative only [Tul84] [Don91] | promote to soft as a Tullis-style grouping measure; the direction is non-monotone, so report a band |
| `layout.overflow` | consensus + moderate | WCAG 1.4.10; wrap beats horizontal scroll for low vision [Hal17] | gate |
| `layout.clipping` | spec | correctness of shown text | gate where not declared truncatable (now K) |
| `layout.shift` | weak | mechanism plausible; the thresholds have no published user research [WD20] | soft, no absolute threshold claimed |
| `layout.shift_after_input` | none | — | diag |
| `layout.density` | moderate (relative, non-monotone) | Tullis overall and local density [Tul84]; uniform density helped [Par05]; medium complexity preferred [CM94] | diag or soft with a band, never "lower is better" |
| `load.reading_words` | moderate | concision +58% [MN97]; 4.4 s per 100 words [Wei08] | soft; counterweight `completion.content_true` stays mandatory |
| `load.reading_time` | weak as a time model | 238 wpm is full reading [Bry19]; UI users scan (about 20% read) [Wei08] | diag; if kept, use 25 s + 4.4 s per 100 words per page [Wei08], labelled as the web-page fit |
| `a11y.act_failed` | consensus; weak link to outcomes | Conformance did not change success [Pow12] [Vol23]; distinct SC violated r .54 with problems [Pow12] | gate on failures. As a KPI, count distinct rules, not instances [Pow12]. |
| `a11y.act_cant_tell` | n/a | ACT outcome vocabulary | diag (insufficient) |
| `a11y.contrast_min` (WCAG 2.x) | consensus; low-vision support | threshold derivation [W3C-143]; normal readers are robust to contrast [Leg87] | gate |
| APCA contrast (kept beside WCAG 2.x by owner decision) | none from human readers | no peer-reviewed reader study found; it models polarity, which matters for proofreading [Pie13], but its values are unvalidated | soft or diag beside the WCAG gate |
| `a11y.name_missing` | consensus; weak support | unlabelled forms are a top frustration of blind users [Laz07] [A] | gate |
| `a11y.keyboard_unreachable` | consensus | no outcome study found; logically necessary for keyboard-only users | gate |
| `a11y.focus_order` (Kendall tau) | none | no outcome study; the formula is our own | diag (demote from K) |
| `a11y.focus_obscured` | consensus | WCAG 2.4.11; no outcome study | gate |
| `a11y.focus_trap` | consensus | WCAG 2.1.2; no outcome study; 0% found by axe [Deq21], so computing it from the model is a real gain | gate |
| `a11y.target_size` (24 px) | consensus; the threshold is weak | 24 px came from a site survey [W3C-258]; empirical touch thresholds are 7.7–9.6 mm [Par06], 18 mm for motor impairment [Fin17] | gate at 24 px. Add a soft touch-size score in mm per environment (missing list). |
| `consistency.state_vocabulary` | weak | physical inconsistency raised errors, conceptual inconsistency had no effect [OS00] [S] | soft |
| `consistency.command_label` | weak | same | soft |
| `consistency.primary_actions` | none | eye-tracking practitioner study: link vs button gave small gains, placement mattered more [LW] [S] | house rule: keep as a declared design rule if wanted, labelled "no evidence"; not a KPI |
| `consistency.pattern` | weak | [OS00] [S] | diag |
| `steering.cue_accuracy` | moderate (mechanism) | just-in-time, specific cues remove postcompletion errors; non-specific cues do not [CB08] | soft |
| `steering.cue_churn` | none | — | diag |
| `steering.cue_leads` | none (as computed) | a modelled stand-in with no validation | diag |
| `proxy.emphasis_per_region` | none | — | diag |
| `proxy.state_colour_share` | none for this formula | colourfulness predicts mean first-impression ratings [Rei13]; preferences vary by group [RG14] | diag |
| `proxy.urgency_levels` | none found (alarm-design literature not searched) | — | diag |
| `proxy.type_scale` | none | — | diag |
| `proxy.spacing_units` | none | — | diag |
| `proxy.alignment_edges` | none; one negative result | alignment "not shown to be a performance-enhancing factor" [Par05] | diag |
| `proxy.radii` | none | — | diag, or drop |

Summary of changes:

- Promote `layout.clipping` to gate and `attention.regions_in_view` to soft.
- Demote `a11y.focus_order`, `steering.cue_churn` and `steering.cue_leads` to diag.
- Demote `attention.glance_cost` from K to diag.
- Treat `consistency.primary_actions` as a declared house rule, not an evidence-based gate.
- Add tie bands to every soft score.
- Drop nothing that is diag already. `proxy.radii` is the weakest candidate.

### 12.2 Missing, with evidence

| candidate | evidence | form |
|---|---|---|
| Minimum body text size per environment | 10–12 pt hurt comprehension; best at 18 pt [Rel16]; R-01's reviewer found text under 14 px that nothing measured | gate at a floor (the value is a policy choice), soft above |
| Touch target size in mm (environment px/mm) | 9.2–9.6 mm [Par06]; errors over 40% under 8 mm [Hen11]; 18 mm for motor impairment [Fin17] | soft in touch environments, beside the 24 px gate |
| Pointing time by Fitts / FFitts per environment instead of flat P | [SM04] [Bi13]; P = 1.1 s overpredicts [John04] | inside `effort.klm_time@2` |
| Menu and choice-list search cost | 0.08 s per item (novice), log2 for experts; ranked menu designs right [Coc07] | soft; needs an expertise assumption and, for experts, item frequencies |
| Grouping and density (Tullis) | a priori r .80 on search time and ratings; relative only [Tul84] [Don91] | soft with a band |
| Visual clutter or complexity (edge density, Feature Congestion, quadtree leaves) | r .83 with search time [Ros07]; r .92 with perceived clutter on sites [LR11]; .80 [Rei13] | guard against extremes, not a target. Needs a raster or a box-level approximation [Inference]. |
| Distinct a11y rules violated (instead of instance counts) | r .54 with blind users' problems [Pow12] | KPI form of `a11y.act_failed` |
| Information scent of labels for a declared goal | CogTool-Explorer R² .63–.82 [Teo12] | soft; needs goal texts and a frozen local embedding model |
| Error-message specificity at refusals | [Shn82] | spec-like check: the refusal names the field and the fix (needs a declared reason) |
| Confirm count per journey | habituation cuts attention threefold [BL14] | counterweight to the confirm design rule |
| UI-trained saliency of required nodes and the primary action | CC .83 within type [Jia23] [Lei20]; rs .53 among variants [Byl17] | soft at most; only for UI types like the training data |

### 12.3 What the evidence says about ranking thousands of variants

- Rank only on gates plus soft scores with tie bands. Inside a band, call it a tie and leave the
  choice to people. Concretely: 20% of KLM time per journey [CMN80], and the dispersion of single
  Fitts moves [ZW].
- Keep the frontier Pareto. Do not collapse to a weighted sum: the outcomes are weakly correlated
  [SL09], so one sum hides trade-offs that people weigh differently.
- Expect best-of-n over-optimisation [GSH23]. Report how far frontier variants are from the base
  document, and prefer the smallest change within a tie.
- Aesthetic and complexity measures are guards against extremes, not targets [Thi19] [RG14] [LR11].
- Model judges and agent walkers do not rank close variants [Lue25] [Wu24] [TUI26]. They propose
  findings for people.

## 13. Sources

Performance and models

- [CMN80] Card, Moran, Newell. The Keystroke-Level Model for User Performance Time with Interactive
  Systems. CACM 23(7):396–410, 1980. doi:10.1145/358886.358895
- [John04] John, Prevas, Salvucci, Koedinger. Predictive Human Performance Modeling Made Easy. CHI
  2004. doi:10.1145/985692.985750
- [Swe13] Swearngin, Cohen, John, Bellamy. Human Performance Regression Testing. ICSE 2013.
  doi:10.1109/ICSE.2013.6606561
- [Jor15] Jorritsma, Haga, Cnossen, Dierckx, Oudkerk, van Ooijen. Predicting human performance
  differences on multiple interface alternatives: KLM, GOMS and CogTool are unreliable. Procedia
  Manufacturing 3:3725–3731, 2015. doi:10.1016/j.promfg.2015.07.806
- [Gray93] Gray, John, Atwood. Project Ernestine: Validating a GOMS Analysis for Predicting and
  Explaining Real-World Task Performance. Human-Computer Interaction 8(3), 1993.
  doi:10.1207/s15327051hci0803_3
- [SM04] Soukoreff, MacKenzie. Towards a standard for pointing device evaluation. IJHCS 61:751–789,
  2004. doi:10.1016/j.ijhcs.2004.09.001
- [ZW] Zheng, Wang. Exploring Fitts' Law in Web Browsing. Venue not checked (possibly WWW 2015).
- [Bi13] Bi, Li, Zhai. FFitts Law. CHI 2013. doi:10.1145/2470654.2466180
- [Coc07] Cockburn, Gutwin, Greenberg. A Predictive Model of Menu Performance. CHI 2007.
  doi:10.1145/1240624.1240723
- [Liu20] Liu, Gori, Rioul, Beaudouin-Lafon, Guiard. How Relevant is Hick's Law for HCI? CHI 2020.
  doi:10.1145/3313831.3376878
- [HH11] Halverson, Hornof. A Computational Model of "Active Vision" for Visual Search in HCI. HCI
  26(4):285–314, 2011. doi:10.1080/07370024.2011.625237
- [Teo12] Teo, John, Blackmon. CogTool-Explorer. CHI 2012. doi:10.1145/2207676.2208414. Also Teo's
  dissertation, CMU-HCII-11-102.
- [FP07] Fu, Pirolli. SNIF-ACT. Human-Computer Interaction 22(4):355–412, 2007.
- [Por03] Porter. Testing the Three-Click Rule. UIE, 2003-04-16 (secondary; URL not checked).
- [Lau19] Laubheimer. The 3-Click Rule for Navigation Is False. NN/g, 2019.
  https://www.nngroup.com/articles/3-click-rule/
- [Fes18] Fessenden. Scrolling and Attention. NN/g, 2018.
  https://www.nngroup.com/articles/scrolling-and-attention/
- [Bak03] Baker. Usability News 5.1, 2003 (secondary).
- [DM11] Dabrowski, Munson. 40 years of searching for the best computer system response time.
  Interacting with Computers 23(5):555–564, 2011. doi:10.1016/j.intcom.2011.05.008
- [CB08] Chung, Byrne. Cue effectiveness in mitigating postcompletion errors in a routine procedural
  task. IJHCS 66(4):217–232, 2008. doi:10.1016/j.ijhcs.2007.09.001
- [BL14] Bravo-Lillo et al. Harder to Ignore? Revisiting Pop-Up Fatigue and Approaches to Prevent It.
  SOUPS 2014. https://www.usenix.org/conference/soups2014/proceedings/presentation/bravo-lillo

Usability construct

- [SL09] Sauro, Lewis. Correlations among prototypical usability metrics. CHI 2009.
  doi:10.1145/1518701.1518947. It quotes Hornbæk & Law, Meta-analysis of correlations among usability
  measures, CHI 2007, doi:10.1145/1240624.1240722.

Aesthetics, complexity, saliency

- [Ngo03] Ngo, Teo, Byrne. Modelling interface aesthetics. Information Sciences 152:25–46, 2003.
  doi:10.1016/S0020-0255(02)00404-8
- [ZV16] Zen, Vanderdonckt. Assessing User Interface Aesthetics based on the Inter-subjectivity of
  Judgment. BCS HCI 2016. doi:10.14236/ewic/HCI2016.25
- [Sal13] Salimun. PhD thesis, University of Glasgow, 2013. https://theses.gla.ac.uk/4256/ ; NordiCHI
  2010, doi:10.1145/1868914.1868963
- [Ros07] Rosenholtz, Li, Nakano. Measuring visual clutter. Journal of Vision 7(2):17, 2007.
  doi:10.1167/7.2.17
- [LR11] Lafleur, Rummel. Mensch & Computer 2011, pp. 101–109.
  https://dl.gi.de/handle/20.500.12116/7917
- [MD14] Miniukovich, De Angeli. Quantification of interface visual complexity. AVI 2014.
  doi:10.1145/2598153.2598173
- [MD15] Miniukovich, De Angeli. Computation of interface aesthetics. CHI 2015.
  doi:10.1145/2702123.2702575
- [Rei13] Reinecke et al. Predicting users' first impressions of website aesthetics with a
  quantification of perceived visual complexity and colorfulness. CHI 2013.
  doi:10.1145/2470654.2481281
- [RG14] Reinecke, Gajos. Quantifying visual preferences around the world. CHI 2014.
  doi:10.1145/2556288.2557052
- [Lin06] Lindgaard et al. Attention web designers: You have 50 milliseconds to make a good first
  impression! Behaviour & IT 25(2):115–126, 2006. doi:10.1080/01449290500330448
- [Tul84] Tullis. Predicting the usability of alphanumeric displays. PhD dissertation, Rice University,
  1984 (secondary). Review: Human Factors 25:657–682, 1983.
- [Don91] Donner et al. NASA TM-104742, 1991.
- [Par05] Parush et al. Human Factors 47(1):141–157, 2005.
- [CM94] Comber, Maltby. OZCHI 1994 (secondary).
- [Thi19] Thielsch et al. Meta-analysis on aesthetics and performance. Mensch und Computer 2019.
  doi:10.1145/3340764.3340794
- [Tra00] Tractinsky, Katz, Ikar. What is beautiful is usable. Interacting with Computers 13(2):127–145,
  2000. doi:10.1016/S0953-5438(00)00031-X
- [Fog03] Fogg et al. How do users evaluate the credibility of Web sites? DUX 2003 (secondary).
- [AIM18] Oulasvirta et al. Aalto Interface Metrics (AIM). UIST 2018 Adjunct.
  doi:10.1145/3266037.3266087
- [SZ14] Shen, Zhao. Webpage Saliency. ECCV 2014. doi:10.1007/978-3-319-10584-0_3
- [Byl17] Bylinskii et al. Learning Visual Importance for Graphic Designs and Data Visualizations. UIST
  2017. arXiv:1708.02660
- [Fos20] Fosco et al. Predicting Visual Importance Across Graphic Design Types. UIST 2020.
  doi:10.1145/3379337.3415825
- [Lei20] Leiva et al. Understanding Visual Saliency in Mobile User Interfaces. MobileHCI 2020.
  arXiv:2101.09176
- [Jia23] Jiang et al. UEyes. CHI 2023. doi:10.1145/3544548.3581096
- [Bor] Borji et al., MIT saliency benchmark figures (secondary).

Accessibility

- [Deq21] Deque. The Automated Accessibility Coverage Report, 2021.
  https://accessibility.deque.com/hubfs/Accessibility-Coverage-Report.pdf
- [Vig13] Vigo, Brown, Conway. Benchmarking web accessibility evaluation tools. W4A 2013.
  doi:10.1145/2461121.2461124
- [GDS17] UK Government Digital Service. What we found when we tested tools on the world's least
  accessible webpage. 2017.
  https://accessibility.blog.gov.uk/2017/02/24/what-we-found-when-we-tested-tools-on-the-worlds-least-accessible-webpage/
- [Bra10] Brajnik, Yesilada, Harper. Testability and validity of WCAG 2.0: the expertise effect.
  ASSETS 2010. doi:10.1145/1878803.1878813
- [Bra12] Brajnik, Yesilada, Harper. Is accessibility conformance an elusive property? ACM TACCESS
  4(2), 2012. doi:10.1145/2141943.2141946
- [Pow12] Power, Freire, Petrie, Swallow. Guidelines are only half of the story. CHI 2012.
  doi:10.1145/2207676.2207736
- [Vol23] Vollenwyder et al. How compliance with web accessibility standards shapes the experiences of
  users with and without disabilities. IJHCS 170:102956, 2023. doi:10.1016/j.ijhcs.2022.102956
- [Sch17] Schmutz, Sonderegger, Sauer. Human Factors 59(6):956–972, 2017.
  doi:10.1177/0018720817708397
- [Pet07] Petrie, Kheir. CHI 2007. doi:10.1145/1240624.1240688
- [Laz07] Lazar et al. What frustrates screen reader users on the web. IJHCI 22(3):247–269, 2007.
  doi:10.1080/10447310709336964
- [WAM26] WebAIM. The WebAIM Million, 2026. https://webaim.org/projects/million/
- [W3C-143] W3C. Understanding SC 1.4.3.
  https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- [W3C-258] W3C. Understanding SC 2.5.8.
  https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- [Leg87] Legge, Rubin, Luebker. Psychophysics of reading V. Vision Research 27(7):1165–1177, 1987.
- [Pie13] Piepenbrock et al. Positive display polarity is advantageous. Ergonomics 56(7):1116–1124,
  2013. doi:10.1080/00140139.2013.790485
- [Par06] Parhi, Karlson, Bederson. Target size study for one-handed thumb use. MobileHCI 2006.
  doi:10.1145/1152215.1152260
- [Hen11] Henze, Rukzio, Boll. 100,000,000 taps. MobileHCI 2011. doi:10.1145/2037373.2037395
- [Fin17] Findlater et al. CHI 2017. doi:10.1145/3025453.3025603
- [Hal17] Hallett, Dick, Jewett, Vu. AHFE 2017, AISC 607:665–674. doi:10.1007/978-3-319-60492-3_63

Copy and reading

- [MN97] Morkes, Nielsen. Concise, SCANNABLE, and Objective. NN/g, 1997.
  https://www.nngroup.com/articles/concise-scannable-and-objective-how-to-write-for-the-web/
- [Wei08] Weinreich et al. Not quite the average. ACM TWEB 2(1), 2008. doi:10.1145/1326561.1326566
- [Nie08] Nielsen. How Little Do Users Read? NN/g, 2008.
  https://www.nngroup.com/articles/how-little-do-users-read/
- [Sum05] Summers, Summers. Proc. ASIS&T 42, 2005. doi:10.1002/meet.1450420179
- [Bry19] Brysbaert. How many words do we read per minute? Journal of Memory and Language 109:104047,
  2019. doi:10.1016/j.jml.2019.104047
- [Red00] Redish. Readability formulas have even more limitations than Klare discusses. ACM J. Comput.
  Doc. 24(3), 2000. doi:10.1145/344599.344637
- [Cro19] Crossley, Skalicky, Dascalu. Journal of Research in Reading 42, 2019.
  doi:10.1111/1467-9817.12283
- [Shn82] Shneiderman. System message design. CACM 25(9), 1982. doi:10.1145/358628.358639
- [Rel16] Rello, Pielot, Marcos. Make It Big! CHI 2016. doi:10.1145/2858036.2858204
- [OS00] Ozok, Salvendy. Measuring consistency of web page design and its effects on performance and
  satisfaction. Ergonomics 43(4):443–460, 2000 (findings secondary).
- [LW] Wroblewski. Primary & Secondary Actions in Web Forms.
  https://www.lukew.com/ff/571/primary-secondary-actions-in-web-forms (practitioner study, secondary)

Learned predictors, judges, simulated users

- [SL19] Swearngin, Li. Modeling Mobile Interface Tappability. CHI 2019. doi:10.1145/3290605.3300305
- [Sch22] Schoop et al. CHI 2022. doi:10.1145/3491102.3517497
- [Dou19] Dou et al. Webthetics. IJHCS 124:56–66, 2019. doi:10.1016/j.ijhcs.2018.11.006
- [Wu24] Wu et al. UIClip. UIST 2024. doi:10.1145/3654777.3676408
- [Dua24b] Duan et al. UICrit. UIST 2024. arXiv:2407.08850
- [Gao26] Gao et al. Training Computer Use Agents to Assess the Usability of Graphical User Interfaces.
  arXiv:2604.26020, 2026.
- [Lue25] Luera et al. MLLM as a UI Judge. arXiv:2510.08783, 2025.
- [Har24] Haraguchi et al. Can GPTs Evaluate Graphic Design Based on Design Principles? SIGGRAPH Asia
  2024. arXiv:2410.08885
- [WDJ26] WebDevJudge. ICLR 2026. arXiv:2510.18560
- [Dua24] Duan, Warner, Li, Hartmann. CHI 2024. doi:10.1145/3613904.3642782
- [Gue25] Guerino et al. arXiv:2506.16345, 2025.
- [Zhe23] Zheng et al. Judging LLM-as-a-Judge. NeurIPS 2023. arXiv:2306.05685
- [Pan24] Panickssery, Bowman, Feng. LLM Evaluators Recognize and Favor Their Own Generations.
  arXiv:2404.13076
- [IHS01] Ivory, Hearst. The state of the art in automating usability evaluation of user interfaces.
  ACM Computing Surveys 33(4), 2001. doi:10.1145/503112.503114
- [ISH01] Ivory, Sinha, Hearst. Empirically validated web page design metrics. CHI 2001.
  doi:10.1145/365024.365035
- [TUI26] Beyond Task Completion: Measuring Interaction Cost in Terminal User Interfaces.
  arXiv:2610.05047, 2026.
- [SCW25] Synthetic cognitive walkthrough. arXiv:2512.03568, 2025.
- [Lu25] Lu et al. Can LLM Agents Simulate Multi-Turn Human Behavior? arXiv:2503.20749, 2025.
- [Lu25b] Lu et al. UXAgent: A System for Simulating Usability Testing of Web Design with LLM Agents.
  CHI EA 2025. arXiv:2504.09407
- [AAB26] Agent A/B. CHI EA 2026. arXiv:2504.09723
- [Xia24] Xiang et al. SimUser. CHI 2024. doi:10.1145/3613904.3642481
- [Zho23] Zhou et al. WebArena. arXiv:2307.13854
- [Xie24] Xie et al. OSWorld. arXiv:2404.07972

Goodhart

- [GSH23] Gao, Schulman, Hilton. Scaling Laws for Reward Model Overoptimization. ICML 2023.
  arXiv:2210.10760
- [Bla24] Black et al. Training Diffusion Models with Reinforcement Learning. ICLR 2024.
  arXiv:2305.13301
- [Cla23] Clark et al. Directly Fine-Tuning Diffusion Models on Differentiable Rewards.
  arXiv:2309.17400
- [HOT15] Hohnhold, O'Brien, Tang. Focusing on the Long-term. KDD 2015. doi:10.1145/2783258.2788583
- [Koh12] Kohavi et al. Trustworthy online controlled experiments: five puzzling outcomes explained.
  KDD 2012. doi:10.1145/2339530.2339653
- [Dmi17] Dmitriev et al. A Dirty Dozen. KDD 2017. doi:10.1145/3097983.3098024
- [LS21] Luguri, Strahilevitz. Shining a Light on Dark Patterns. Journal of Legal Analysis
  13(1):43–109, 2021. doi:10.1093/jla/laaa006
- [Mat19] Mathur et al. Dark Patterns at Scale. CSCW 2019. doi:10.1145/3359183 (1,818 instances on
  about 11K shopping sites)
- [MG18] Manheim, Garrabrant. Categorizing Variants of Goodhart's Law. arXiv:1803.04585
- [WD20] web.dev. How the Core Web Vitals metrics thresholds were defined.
  https://web.dev/articles/defining-core-web-vitals-thresholds

## 14. Open question

Does any computed metric order **close variants of one design** the way people do?

- No study found measures rank agreement over many such variants for KLM, Fitts, complexity, saliency
  or model judges. The few direction counts are 4 of 6 [Jor15] and 12 of 15 [Gray93], plus the
  near-chance results of model judges [Lue25] [Wu24].
- The engine's tie bands are therefore inferred from absolute error, not measured.
- [Inference] Settling it needs a small human calibration set: pairs of frontier variants per campaign,
  timed tasks and pairwise preference, scored as rank agreement per KPI. The result decides which
  soft scores may rank and how wide their bands are.
