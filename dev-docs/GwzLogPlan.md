# GWZ Log Implementation Plan

Status: **ADOPTED 2026-08-29** — S0.1 ran both rounds same day: round 1
NO-GO (1 P0 / 2 P1 / 9 P2 / 7 P3, every finding folded), round 2
GO-WITH-CONDITIONS (19/19 round-1 findings CURED; 3 P2 + 5 P3 new, folded
same day; final under the two-round cap) — full trail at §6, report at
`GwzLog-S0.1-Review.md`. Builder launches await the operator's word (and
§5 item 2's ruling for the three parked P2 escalations). Drafted by the
merge-program lane owner (Fable) on the operator's instruction;
**implementation is chartered for non-Fable agents** (Opus builders and
reviewers; Sonnet for mechanical work) under the review loop in §2.

Authority chain: [GwzLogRequirements.md](GwzLogRequirements.md)
(RESOLVED, rows reconciled inline post-round-1) governed by
[GwzLogAmbiguityRezo.md](GwzLogAmbiguityRezo.md) where they disagree.
Coalescing identity: the **landed, in-production commit-marker machinery**
(`GWZ-Commit-ID` trailers + marker artifacts; verified against the tree and
live history at round 1 — `gwz-core/dev-docs/GwzCommitMarker.md`'s
"Status: proposed" line is stale and S1.2 corrects it at source). Repo
split: `gwz-core` owns semantics and structured events, in the new module
home **`gwz-core/src/operation/commit_log/`**, re-exported with minimum
visibility through the existing `operation` seam (the existing
`diff::log_service` is an unrelated subsystem — the diff OUTPUT log — do not
touch it); `gwz-cli`
owns the clap surface (ALL log-specific flags), rendering, and the exit
mapping.

## 1. Scope decisions (made here, reviewable at S0.1 round 2)

1. **The commit marker is LANDED, both halves; Phase 1 is an audit, not a
   build.** Trailers, artifact, wire flag, CLI flags, Python API, and the
   g13 tests all exist and run in production (S0.1 review F1's evidence
   table; independently re-verified by the lane owner: today's root commit
   carries the trailers, 317 artifacts on disk, the writer live in
   `handle_commit.rs`). What is genuinely unbuilt is **retry identity**
   (review F5): a member-succeeded/root-failed retry mints a fresh UUID
   and permanently splits one workspace change across two
   `GWZ-Commit-ID`s — which L-COA-2 forbids the heuristic from healing.
   That defect is Phase 1 (requirement L-COA-8). The marker doc's
   "`gwz log --merged` locates the root by the marker file" sentence is
   superseded — the root's own trailer identifies it — and S1.2 records
   that supersession in the doc.
2. **Trailer-first coalescing, honestly bounded.** `GWZ-Commit-ID` is a
   complete, proven key when `gwz commit` created the commits. The known
   limit (review F4): the root-only-`gwz commit` + plain-`git`-member
   working pattern (the dominant recent pattern in this very workspace)
   yields trailerless member commits that render as singletons;
   **artifact-assisted association** (the marker artifact's
   `members:{commit}` map) is the recorded v2 candidate, not v0.
3. **Heuristic coalescing ships in v0, hardened.** Four conjuncts
   (identical message + identical author + committer window 10 s + author
   window 10 s, different repos only) — the author-date conjunct kills the
   rebase re-stamp false-merge vector (review F6); same-message `forall`
   fan-outs inside the window coalesce BY DESIGN, labeled `heuristic`
   (operator breadth re-confirmation pending in the Rezo; the standing
   spec governs meanwhile). Never across distinct marker values.
4. **The L-COA-4 ↔ L-PRF-1 contract is named**: the bounded coalescing
   window **W = 60 s** (L-COA-7, review F3) — S2.4 builds the grouping
   against it, S2.5 consumes it; out-of-window siblings emit separately
   sharing the provenance key.
5. **`+lock` is its own step with its decision made** (review F8/F10):
   resolution is per-member from the lock's `members:` map; **`@root`
   degrades with a record** (the lock has no root entry) — never an
   error, never silent. Classification is a non-risk (core already routes
   `+` tokens away from paths). `--tagged` keeps diff's refusal to
   combine with `+` operands.
6. **No release in this plan.** Shipping `gwz log` in a vX.Y release is
   the operator's call on the standard three-channel process.
7. **Frozen surfaces: no DIFFS under** `gwz-core/src/checked_artifact/`
   or `workspace_ops/merge/v1_lifecycle/`; no census or inventory pin
   moves. **Calling or reading existing core APIs — including the
   artifact readers `+snapshot`/`+lock` need — is always in charter**
   (review F19); only a diff under those trees is out. *(Corrected
   2026-08-29 with the mirror directive: the round-2 text also said "no
   wire pin moves" — an R2-E-era carryover that is WRONG for a new verb.
   A new command adds protocol messages by construction; L-PRO-1 makes
   the growth ADDITIVE-ONLY with existing slots byte-untouched and the
   drift check green in both repos. That is the honest form of the wire
   constraint here.)*
8. **LOC basis, stated once** (review F13): every budget below counts
   handwritten, non-generated LOC **including tests** (regenerated
   protocol artifacts excluded). Budgets are aspirational targets
   (<500), not hard limits.
9. **gwz-py is a full peer client** (operator directive 2026-08-29,
   verbatim: "all gwz-cli work needs to be mirrored as gwz-py work
   too"): every client-surface deliverable lands twice — clap in
   `gwz-cli`, mirror in `gwz-py` (L-PY-1..3) — over the shared protocol
   (L-PRO-1, step S2.0). The feature is not done until the py surface
   ships (the three-channel release rule).
10. **Module-home amendment (operator ruling 2026-08-29).** The engine's
   physical home is `gwz-core/src/operation/commit_log/`, re-exported through
   the existing `operation` seam with the minimum visibility required by
   request dispatch and clients (`pub(crate)` unless implementation proves a
   wider surface is necessary). The originally adopted
   `gwz-core/src/commit_log/` home required either a crate-root export or an
   out-of-tree source mount; the checked-artifact boundary refused those
   probes verbatim as "compiler root manifest changed" (exit 1) and "Rust
   source-loading edge inventory changed" (exit 1). No root-manifest or
   source-loading-edge inventory moves, no pin churn, and no `lib.rs` diff are
   authorized. The F15 substance remains unchanged: commit history has a
   distinct home and never collides with `diff::log_service`; the
   core-owns-semantics split is likewise unchanged. S2.1's single-axis review
   explicitly reviews whether `operation/` is the right existing seam and
   whether visibility stayed minimal.

## 2. Process — the gwz review loop, applied

Every step below runs the program's loop (`AgentProcessRules.md` as
amended + the adopted process optimization), at feature tier:

- **Build**: an Opus builder from a tight brief, in an isolated worktree;
  no push, no tags, no trailers beyond what the step itself implements;
  insurance copies of uncommitted work in scratch.
- **Review**: interior **single-axis peer-blind** (Opus), checklist
  against the step's NAMED requirements rows (every row a step owns is
  cited in its brief below); verdict GO / GO-WITH-CONDITIONS / NO-GO with
  [P0-P3] findings; report filed VERBATIM as
  `gwz-cli/dev-docs/GwzLog-<Step>-Review.md` and committed promptly.
  **Auto-escalation to Fable on any P0/P1/P2 stands as policy; while the
  Fable pool is exhausted, an escalation PARKS the step until the quota
  reset rather than waiving the second axis** — with one recorded carve-out
  exercised at S0.1 round 1: a finding whose substance is *directly
  verifiable fact* (not contested judgment) may be reconciled by the lane
  owner re-executing the evidence, with the ruling recorded in the
  adoption trail. Parked escalations are recorded in the step's review
  file; no step lands with an unserved escalation.
- **Remediation**: two-round cap; round 2 is final; a third architectural
  root means NO-GO and a re-plan. **Review-driven edits to
  `GwzLogRequirements.md` are the lane owner's to execute and record**
  (review F11); the Rezo's operator resolutions are untouchable — the lane
  owner adds dated annotations and fresh `Resolution (Gianni):` lines
  only, and the operator answers there.
- **Landing**: the lane owner (or the operator's designated lander) lands
  from a pristine overlay with per-repo gates green and DIRECT exit codes
  (`cargo fmt --all -- --check`; `cargo check --all-targets`;
  `CLIPPY_CONF_DIR="$PWD" cargo clippy --all-targets -- -D warnings`; the
  repo's test suite; gwz-core landings also run the boundary checker —
  expected untouched-green per §1.7). **Any landing whose diff touches
  `gwz-core/src/workspace_ops/` additionally runs the real-workspace
  battery** (the J-7 ritual as a landing duty, review F11 — S1.1 is
  exactly the step that needs it). **gwz-cli landings that move command
  help/docs surface regenerate `docs/CLI.md` and run the docs gate**
  (precedent: the `--no-ff` unhide). Multi-commit trains are per-commit
  green or land SQUASHED citing the reviewed shas (ritual 7). **gwz-py
  landings** run its python test suite and, whenever the schema moved
  anywhere in the train, the protocol drift/regen check in BOTH repos
  (the checks its own `scripts/release.py` runs are the reference set).
  Worktrees removed at each lane's close.
- **Traceability**: every v0 row the plan implements — **MUST or SHOULD**
  (review F17) — lands with a named test; S4.1 executes the sweep.
- **Tier economy**: builders and reviewers Opus; mechanical work Sonnet;
  Fable only for parked escalations after reset.

## 3. Phases

Foundational-first; parallelism is permitted where marked (independent
agents MAY pick up parallel steps, never must). Phase 1 and Phase 2 are
**fully parallel lanes** (review F9): nothing in Phase 2 depends on S1.1.

### Phase 0 — adoption (milestone: the plan is reviewed and adopted)

- **S0.1 — plan review.** Round 1 EXECUTED 2026-08-29: NO-GO
  (`GwzLog-S0.1-Review.md`), all findings folded (§6). Round 2: the same
  reviewer re-verdicts the remediated set — emphasis on the re-planned
  Phase 1, the §4 sketch, and the requirements-row reconciliation —
  appended to the same report file. GO gates Phases 1 and 2.

### Phase 1 — marker trust (milestone: one workspace change ⇒ one `GWZ-Commit-ID`, provably)

*(gwz-core lane, parallel with Phase 2.)*

- **S1.1 — retry-identity audit and fix** *(gwz-core,
  `workspace_ops/handle_commit.rs` + the marker artifact lifecycle;
  **~350-400 LOC incl. tests** — re-budgeted at round 2 (F22): the
  round-1 "small step" call was the reviewer's own under-estimate;
  deterministic id derivation has no stable input across a partial retry,
  so the realistic shape is durable pending state; owns **L-COA-8**)*.
  Reproduce review F5's split (members commit before root; a retry mints
  a fresh UUIDv7; `preflight_marker_path` guards id reuse, not retry);
  then implement **L-COA-8's retry-sameness predicate**: durable
  per-operation state recorded BEFORE the first commit (the marker
  artifact's own pending-then-finalized lifecycle is the natural
  candidate), sameness proven from that state or a NEW id minted —
  failing toward splitting, never fusing (a false fuse is strictly worse
  than a false split). Regression test drives the
  member-succeeded/root-failed retry to ONE id; a second test proves two
  genuinely different back-to-back operations get two ids even when the
  first left pending state. Real-workspace battery at this landing (§2).
  This step touches production commit semantics: its review is the
  phase's scrutiny point.
- **S1.2 — `GwzCommitMarker.md` reconciled at source** *(docs;
  ~40 lines; rides S1.1's train)*. The stale `Status: proposed` line
  corrected with a dated note (landed; evidence cites); the artifact
  schema reconciled to the landed `MarkerArtifact` (including the
  `merge:` field the implementation grew); the "`gwz log --merged`
  locates the root by marker file" sentence marked superseded by the
  root trailer (§1.1); the S1.1 retry-identity fix recorded.

### Phase 2 — the core log engine (milestone: `commit_log` emits the coalesced, tolerant, streamed entry stream)

*(Engine steps in `gwz-core/src/operation/commit_log/`; engine parameters only — no
clap flags in this phase, review F7.)*

- **S2.0 — the protocol surface** *(gwz-core + gwz-py regen; ~250 LOC
  handwritten (schema + dispatch stub + drift-check assertions;
  regenerated artifacts excluded); after S0.1, PARALLEL with S2.1; owns
  **L-PRO-1**)*. Define the log request/response/entry/degradation
  messages in `gwz-core/protocol/gwz.taut.py` following the
  `StatusRequest`/`DiffRequest` shapes (streamed-output precedent:
  `diff_output`); regenerate BOTH repos' artifacts; wire the core
  dispatch through the existing `operation` seam to a
  `src/operation/commit_log/` stub the Phase-2 engine steps fill; protocol drift
  check green in both repos. ADDITIVE ONLY — existing messages and
  slots byte-untouched, asserted in the step's tests. Message-shape
  choices (streaming vs paged response, degradation record form) are
  this step's reviewable decisions.
- **S2.1 — selection + per-repo cursors** *(~400 LOC; after S0.1; owns
  **L-SEL-2, L-RNG-2, L-RNG-5, L-ORD-1, L-TOL-1, L-TOL-3, L-TOL-4,
  L-TOL-5, L-TOL-6**)*. Read-only repo opening for the default selection
  (`@root` + members); no-operand default = each repo's `HEAD` history,
  detached included; per-repo newest-first cursors preserving each
  repository's own `git log` default order; degradation records for
  unreadable/unborn cases; NO conf gate, NO network, NO mutation lock;
  structured entry/degradation events through the message-oriented API in
  `src/operation/commit_log/`, re-exported with minimum visibility through the
  existing `operation` seam per §1.10.
- **S2.2 — operands and narrowing** *(~350 LOC; after S2.1; owns
  **L-RNG-1** (incl. its pathspec clause — the round-1 "L-PTH" citation
  was dangling, review F14), **L-RNG-3, L-SEL-3, L-TOL-2**)*. Wire `gwz
  diff`'s operand classifier; `+snapshot` per-member resolution with the
  root-degrades-with-record divergence L-RNG-3 records; pathspecs after
  `--`; `--tagged` narrowing with its existing `+`-operand refusal;
  per-member resolve-or-degrade with the strictness escalation semantics
  (the `--strict` FLAG itself is S3.1's).
- **S2.3 — the `+lock` pseudo-operand** *(~150 LOC; after S2.2; owns
  **L-RNG-4**)*. Per-member resolution from `gwz.lock.yml`'s `members:`
  map; `@root` degrades with a record per §1.5's decision. The project's
  only new range surface, deliberately its own small step. Rider (round-2
  F27, record-only): `--tagged`'s existing refusal wording says "snapshot
  operands" — generalize or reuse deliberately when `+lock` hits that
  path, so the message does not mis-name the operand.
- **S2.4 — coalescing** *(~450 LOC; after S2.1, PARALLEL with S2.2; owns
  **L-COA-1, L-COA-2, L-COA-3 (engine semantics), L-COA-4, L-COA-6,
  L-COA-9**)*.
  Marker-keyed grouping; the four-conjunct heuristic (incl. F24's rule:
  marked commits never join heuristic groups); provenance tags; the
  no-coalesce engine switch. S2.4 exposes a **group-assembly API built
  against the L-COA-7 window contract** — it holds no cross-cursor state
  itself; the window buffer is S2.5's (round-2 F21's ownership fix).
  Fixtures: real-trailer siblings (the machinery ships — use it);
  MUST-merge-labeled `forall` fan-out; MUST-NOT-merge: same message
  different author, outside either window, distinct markers with
  identical messages, a marked commit against a matching unmarked group,
  and the **rebase re-stamp** case (review F6 — committer dates collapsed
  to now, author dates distinct).
- **S2.5 — streaming merge + depth** *(~350 LOC; after S2.2 + S2.4; owns
  **L-COA-7, L-ORD-2, L-DEP-1, L-PRF-1, L-PRF-2**)*. K-way merge OWNING
  the L-COA-7 window: the reorder buffer and the emission hold live here
  (group closed when every live cursor passes newest-sibling − W; memory
  O(repos × entries within W)), driving S2.4's group-assembly API;
  newest-first by committer date with the deterministic tiebreak; the
  global cap 50 with explicit-`n`/no-limit/lift-on-range-or-filter
  semantics; `--jobs` concurrency with ordering unaffected.
- **S2.6 — filters + aggregate status** *(~250 LOC; after S2.5; owns
  **L-FIL-1, L-COA-5**)*. The six-filter passthrough applied per-repo
  pre-merge with entry-level semantics; the per-member outcome aggregate
  that S3.1's exit mapping consumes (the EXIT mapping itself is
  L-EXIT-1, owned by S3.1 — review F7's misassignment corrected).

### Phase 3 — the CLI surface (milestone: `gwz log` ships end to end)

*(Re-sliced at round 2 — F23/F20: the single surface step had re-grown
F10's bundling at exactly the budget ceiling.)*

- **S3.1 — subcommand, flags, exit mapping** *(gwz-cli; ~300 LOC; after
  S2.6; owns **L-SEL-1, L-EXIT-1**; CO-OWNS the flag surface of five
  core-owned rows, named for its reviewer's checklist: `-n`/`--no-limit`
  (L-DEP-1), the six filters (L-FIL-1), `--strict` (L-TOL-2),
  `--no-coalesce` (L-COA-3), `--body` (L-JSN-1))*. Clap surface with the
  standard global selectors AND every log-specific flag in one place
  (review F7's option (b)), plus `--color`; request lowering to the core
  engine; exit mapping through the existing `exit_code_for_response`
  seam. Renders nothing beyond plumbing-level output.
- **S3.2 — human rendering + docs** *(gwz-cli; ~300 LOC; after S3.1; owns
  **L-OUT-1, L-OUT-2, L-OUT-4, L-OUT-5**)*. Compact default line with
  the member-set rendering; `--full` blocks with the member table; stderr
  degradation summary; color on tty; no pager per L-OUT-5's standing
  resolution (the Rezo re-confirmation may flip this — check before
  landing). Command docs + help text land here; the docs gate runs at
  this landing.
- **S3.3 — machine output** *(gwz-cli; ~250 LOC; PARALLEL with S3.2 after
  S3.1; owns **L-JSN-1, L-JSN-2**)*. `--json`/`--jsonl`: uniform
  `members[]` shape, provenance, degraded-member records, `--body`
  semantics in machine records.
- **S3.5 — gwz-py CLI mirror + API** *(gwz-py; ~450 LOC; after S3.1 +
  S2.6; owns **L-PY-1, L-PY-2**)*. The `cli_log`-family mirror: same
  operands, flags, defaults, degradation reporting, exit semantics as
  S3.1's surface, lowered through S2.0's protocol messages; the
  `client.py` API (`log`/`log_output`-shaped, per the `diff` precedent);
  py tests mirroring the flag tri-states. Py command docs/help ride
  here.
- **S3.6 — gwz-py rendering + machine output** *(gwz-py; ~350 LOC; after
  S3.2 + S3.3 + S3.5; owns **L-PY-3**)*. Human rendering via the
  `cli_render` pattern (semantic parity MUST, byte parity SHOULD);
  `--json`/`--jsonl` byte-compatible with gwz-cli's records; parity
  assertions against captured gwz-cli output where cheap.
- **S3.4 — the end-to-end battery + real-workspace run** *(tests only;
  ~300 LOC; after S3.2 + S3.3 + S3.5 + S3.6 — numbered before the py
  steps for history, dependency-ordered after them)*. Batteries drive
  BOTH clients over the same multi-repo
  fixtures: trailer coalescing, heuristic arms (incl. the rebase
  MUST-NOT and marked-vs-heuristic-group MUST-NOT), unborn root, detached
  member, degrade + `--strict`, `+snapshot`/`+lock` ranges (root
  degradation visible), `--tagged`, the cap and its lifts, exit codes,
  `--no-coalesce`. Plus the real-workspace arm: actual `gwz init` +
  `gwz commit` fixtures asserting a coordinated commit renders as one
  entry with the right member set (trailers ship today, so this arm runs
  unconditionally; S1.1's retry-regression stays in S1.1's own tests —
  no coupling here).

### Phase 4 — settle (milestone: the feature is accepted)

- **S4.1 — traceability sweep + settle review** *(docs + review)*. The
  requirements-to-tests table over every v0 row the plan implements,
  MUST or SHOULD (review F17); full gates both repos; a single-axis
  settle review over the whole delta (Opus — no Fable dual: no frozen
  surface, no amendment; §2's escalation path covers surprises); verdict
  filed verbatim; adoption record appended here. Release decision handed
  to the operator per §1.6.

## 4. Step dependency sketch

```text
S0.1 ──┬── S1.1 ── S1.2                     (Phase 1: marker-trust lane)
       ├── S2.0 ─────────────────┐          (protocol surface; ∥ S2.1)
       └── S2.1 ──┬── S2.2 ── S2.3
                  │           │
                  └── S2.4 ───┴── S2.5 ── S2.6 ── S3.1 ──┬── S3.2 ──┬── S3.4 ── S4.1
                                   ▲                     ├── S3.3 ──┤
                                   └── (S2.0 feeds the   └── S3.5 ── S3.6
                                        request wiring       (gwz-py mirror;
                                        from S2.2 on)          feeds S3.4)
```

(S2.0 runs parallel with S2.1; request-facing steps from S2.2 onward
consume its messages. S3.4's battery closes over all four client steps —
S3.2, S3.3, S3.5, S3.6.)

Phase 1 feeds nothing downstream (the trailer machinery already ships;
S1.1 hardens it). A Phase-1 park does not block anything else. S2.2 and
S2.4 are the parallel pair off S2.1; S2.3 is the small decision step off
S2.2.

## 5. Open items

1. **Operator re-confirmations pending in the Rezo** (standing
   resolutions govern meanwhile): heuristic breadth (Q-1 note — coalesce
   `forall` fan-outs?) and pager (Q-11 note — the premise was wrong;
   `gwz diff` pages; keep no-pager or flip to diff-parity?). S2.4 and
   S3.2 respectively check these before landing.
2. **The proportionality carve-out (round-2 F25) — operator's ruling
   requested.** §2's escalation rule parks any P0/P1/P2 on the empty
   Fable pool; round 2's three P2s had remedies that were one-sentence
   lane-owner document edits (all executed same day), yet a literal
   reading parks S1.1/S2.4/S3.1's launches anyway. Proposed second
   carve-out, NOT self-adopted (the reviewer rightly flagged that §2
   carve-outs are program-level moves a feature plan should not keep
   minting for itself): *a P2 whose complete remedy is a document edit
   the lane owner is already authorized to make, executed and recorded
   in the adoption trail, is served without a Fable pass.* One line from
   the operator adopts or rejects it; until then, the three round-2 P2
   escalations stand PARKED as §2 requires — practically moot unless
   builders launch before the quota reset or the ruling.
3. **Release vehicle** — operator's call (§1.6).
4. **v2 candidates recorded in the requirements**: artifact-assisted
   association for trailerless member commits; grouped rendering;
   `--format`; pickaxe; `--follow`.

## 6. Adoption trail

- **S0.1 round 1 (2026-08-29): NO-GO** — 1 P0, 2 P1, 9 P2, 7 P3
  (`GwzLog-S0.1-Review.md`, filed verbatim). Headline: the round-1 plan's
  Phase 1 chartered a rebuild of the already-landed commit-marker
  machinery, off `GwzCommitMarker.md`'s stale status line.
- **Escalation ruling (lane owner, recorded per §2):** the P0 (F1) is
  verified fact, not contested judgment — the lane owner re-executed the
  evidence directly (root commit `563d352`'s trailers; 317 artifacts in
  `gwz.conf/markers/`; the writer at `handle_commit.rs`) and reconciled
  it without a Fable second axis; the reviewer's report itself invited
  exactly this ruling and it is taken explicitly, not by omission. The
  P1/P2 findings' remedies are all document edits, executed 2026-08-29:
  requirements rows reconciled inline with resolved values (F2);
  L-COA-7's bounded window minted (F3); the artifact coverage limit and
  v2 candidate recorded (F4); Phase 1 re-planned around retry identity
  with L-COA-8 minted (F5); the heuristic hardened with the author-date
  conjunct and the rebase fixture (F6); all flags moved to S3.1 and
  L-EXIT-1 reassigned (F7); `+lock`'s root-degrade decision made and
  L-RNG-4 rewritten (F8); the dependency sketch redrawn parallel and the
  moot fallback clause deleted (F9); S2.2/S2.4 split into
  S2.2/S2.3/S2.4/S2.5/S2.6 (F10); the three §2 process gaps closed
  (F11); the pager premise corrected with an operator re-ask (F12); the
  LOC basis stated (F13); the dangling "L-PTH" citation fixed (F14); the
  `commit_log/` module home named (F15); L-OUT-3 marked deferred and
  listed (F16); the sweep scope widened to implemented SHOULDs (F17);
  the Q-1 breadth re-ask filed (F18); the charter wording fixed to
  diffs-not-reads (F19). Coverage: the three unowned rows are owned
  (L-RNG-2, L-ORD-1 → S2.1; L-OUT-3 → deferred), the double-owned row
  single-owned (L-COA-5 → S2.6), the misassigned row moved (L-EXIT-1 →
  S3.1), and every implied row is now named in its owning step.
- **S0.1 round 2 (2026-08-29): GO-WITH-CONDITIONS — final under the
  two-round cap.** All 19 round-1 findings CURED (0 partial, 0 not
  cured); coverage matrix clean (36/36 rows named-owned, zero unowned/
  double-owned/misassigned/dangling). The reviewer empirically validated
  the two quantitative cures on real history: W = 60 s carries ~60×
  headroom (153 multi-repo marker groups, max spread 1 s), and the
  author-date conjunct regressed zero true merges over 573 unmarked
  commits. Eight new findings (3 P2, 5 P3), folded same day: [F22]
  L-COA-8's retry-sameness predicate defined (durable pre-commit state;
  fails toward splitting) and S1.1 re-budgeted ~350-400 (the round-1
  "small step" sizing was the reviewer's own under-call, owned in the
  report); [F21] L-COA-7 given MUST force and moved to S2.5 (the merge
  owns the window buffer; S2.4 exposes group assembly against the
  contract); [F23/F20] Phase 3 re-sliced S3.1-S3.4 with the five
  co-owned flag rows named; [F24] marked commits never join heuristic
  groups (L-COA-2); [F25] the proportionality carve-out proposed to the
  operator, NOT self-adopted (§5 item 2) — the three P2 escalations
  stand parked per §2 meanwhile; [F26] the `GwzCommitMarker.md` stale
  status line corrected at source SAME DAY as a standalone docs-only
  commit (decoupled from the parkable Phase-1 lane; S1.2 keeps the
  fuller schema reconcile); [F27] the `--tagged` wording rider recorded
  at S2.3. **The plan is ADOPTED as of this record**, with builder
  launches gated on the operator's word (and, for S1.1/S2.4/S3.1, on
  the F25 ruling or the quota reset per §2).
- **Operator scope directive, 2026-08-29 (post-adoption amendment;
  verbatim: "all gwz-cli work needs to be mirrored as gwz-py work
  too").** Folded same day by the lane owner: gwz-py made a full peer
  client (§1.9; requirements L-PY-1..3 + Orientation precedents); the
  protocol surface made an explicit foundational step (S2.0, owning the
  new L-PRO-1) — which also corrected §1.7's "no wire pin moves"
  carryover to the honest additive-only form; Phase 3 gained S3.5
  (py CLI mirror + API) and S3.6 (py rendering + machine parity); the
  S3.4 battery now drives BOTH clients and closes over all four client
  steps; §2 gained the gwz-py landing gates. The amendment is
  operator-chartered scope, not a reviewable-finding fold — its new
  steps get their scrutiny in their own single-axis reviews (S2.0's
  message-shape decisions are that step's named review points).
- **Module-home amendment, 2026-08-29 (operator ruling).** S2.1's two
  boundary probes refused the adopted top-level home as "compiler root
  manifest changed" (exit 1) and "Rust source-loading edge inventory changed"
  (exit 1). The engine therefore moves to
  `gwz-core/src/operation/commit_log/`, minimally re-exported through the
  existing `operation` seam; no inventory, pin or `lib.rs` change is allowed.
  F15's distinct-home/no-`diff::log_service`-collision substance and the
  core-owns-semantics split are preserved unchanged. S2.1 review owns the
  placement and visibility check.
- **S2.4 re-charter, 2026-08-29 (lane-owner-dictated amendment, S2.4
  terminal NO-GO).** The terminal two-round NO-GO is accepted as recorded;
  a bare third round remains refused. Because its blocking root includes a
  lane-owner specification gap around marker validity and invalid-marker
  disposition, one amended-specification remediation and review round is
  authorized with the same reviewer. Its scope is only the RFC variant-nibble
  validation added to L-COA-1; L-COA-9's broad-exclusion/strict-keying
  classification and `marker-invalid` singleton provenance added to
  L-COA-6; and the dictated wrong-variant, mangled-separator, valid-v7, and
  two-invalid-identical-heuristic-exclusion regressions. The review checklist
  is these amended rows plus the S2.4 round-2 findings, nothing wider. If the
  re-chartered round returns NO-GO, S2.4 is dead as chartered: freeze the
  lane, file the report, and return it to the operator for re-planning. No
  fourth round exists under any framing. S1.1 and S2.2 remain frozen until
  this round completes or terminates.
