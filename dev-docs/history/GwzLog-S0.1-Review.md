# GWZ Log — S0.1 Plan Review

- **Step:** S0.1 (Phase 0 — plan review), per `GwzLogPlan.md` §3.
- **Date:** 2026-08-29.
- **Tier / mode:** single-axis, peer-blind interior review, Opus. No second axis
  (Fable pool empty — see the escalation note under "Verdict").
- **Objects read (in order):**
  1. `gwz-cli/dev-docs/GwzLogRequirements.md` (status RESOLVED 2026-08-29)
  2. `gwz-cli/dev-docs/GwzLogAmbiguityRezo.md` (status RESOLVED 2026-08-29;
     authoritative where the two disagree)
  3. `gwz-cli/dev-docs/GwzLogPlan.md` (the object under review, status DRAFT)
- **Supporting sources read:** `gwz-core/dev-docs/GwzCommitMarker.md`;
  `gwz-cli/dev-docs/AgentQuickStart.md`; `gwz-dev/dev-docs/AgentProcessRules.md`
  and `CurrentProgramCheckpoint.md` (ritual/process spot-checks).
- **Source sanity-checks run (read-only):** `gwz --version` (0.11.1),
  `gwz --help`, `gwz diff -h`; reads of
  `gwz-core/src/workspace_ops/handle_commit.rs`,
  `gwz-core/src/artifact/mod.rs`, `gwz-core/src/diff/{classify,operands}.rs`,
  `gwz-core/src/protocol/generated.rs`, `gwz-cli/src/clirequest/repo.rs`,
  `gwz-cli/src/lib.rs`; read-only `git log` over the workspace root and members;
  read-only inventory of `gwz.conf/markers/`. Nothing was built, mutated,
  committed, pushed or tagged.

---

## Verdict

# NO-GO

The plan cannot be adopted as written. Its foundational milestone (Phase 1)
charters an Opus builder to implement a feature that is **already fully landed
in production** — trailers *and* artifact, across all three channels, in daily
use in this very workspace (316 marker artifacts on disk; the root commit at
`563d352`, dated today, carries `GWZ-Commit-ID`). The plan's authority chain,
the requirements' Coalescing dependency note, and the Rezo's Q-1 annotation all
assert the opposite. See **[P0 F1]**.

The NO-GO is **narrow and bounded**. Phases 2–4 are structurally sound and
survive; the re-plan is confined to Phase 1, the §4 dependency sketch, and
corrections to two upstream documents. A revised plan should be re-reviewed at
S0.1 scope on Phase 1 + §4 only, not re-reviewed whole.

**Finding counts:** P0 × 1, P1 × 2, P2 × 9, P3 × 7 (19 total).

**Escalation note (for the lane owner, not for me to action):** per
`GwzLogPlan.md` §2, any P0/P1/P2 auto-escalates to Fable and PARKS on the
exhausted pool. This report carries twelve such findings. The single P0 is a
matter of verified fact, not judgment — the lane owner may reasonably treat the
Phase-1 correction as a factual reconcile rather than a contested finding
needing a second axis; that call is the lane owner's, and it is recorded here so
it is made explicitly rather than by omission.

---

## Findings

### [P0 F1] Phase 1 re-implements code that has been landed and in production for months

`GwzLogPlan.md` states, in four places, that the commit marker is unbuilt:

- Header authority chain: *"coalescing identity per
  `gwz-core/dev-docs/GwzCommitMarker.md` (status proposed — **Phase 1 implements
  its trailer half**)"*.
- §1.1: *"**The marker's trailer half is IN scope**; the artifact half is NOT
  … **Step S1.1 lands the trailers**"*.
- §3, S1.1: *"Implement `GwzCommitMarker.md`'s trailer half: one lowercase
  UUIDv7 minted per `CommitRequest`; `GWZ-Commit-ID` … appended to EVERY commit
  the operation creates across members and root; the per-operation disable flag"*
  — tagged *"the plan's most scrutinized step — it touches production commit
  semantics for every gwz user"*.
- §3, S1.2: *"`GwzCommitMarker.md` gains a dated annotation: trailer half LANDED
  (cite the commit), artifact half remains proposed"*.

Every one of those clauses is false. The evidence:

| Claim in S1.1 | Where it already exists |
|---|---|
| UUIDv7 minted once per `CommitRequest` | `gwz-core/src/workspace_ops/handle_commit.rs:136` (`gwz_commit_id: new_uuid_v7()?`), generator at `:288-340` |
| Trailers appended to every commit of the operation | `handle_commit.rs:151-154` builds one `commit_message`; `:165` commits members with it, `:220` commits the root with it. Formatter `CommitMarkerContext::commit_message` at `:246-257` emits `GWZ-Commit-ID` / `GWZ-Workspace-ID` / `GWZ-Origin-URL-Hash` |
| Origin-hash rules, no literal URLs | `root_origin_url_hash` at `:276-286` (`sha256:{digest:x}`, `Option` → omitted when absent) |
| The per-operation disable flag | `handle_commit.rs:69` (`request.commit_marker.unwrap_or(true)`); wire slot 4 in `gwz-core/src/protocol/generated.rs:4217-4240`; schema `gwz-core/protocol/gwz.taut.py:1541`; catalogued in `gwz-core/docs/MessageCatalog.md:1267` |
| CLI `--commit-marker` / `--no-commit-marker`, mutually exclusive | `gwz-cli/src/clirequest/repo.rs:150-159` (`conflicts_with = "no_commit_marker"`), lowering at `:294-296` |
| Python API + CLI | `gwz-py/src/gwz/client.py:604`, `gwz-py/src/gwz/cli_mutation.py:181-189`, generated IR at `gwz-py/src/gwz/protocol/generated/gwz.ir.json:6500` |
| The tests S1.1 proposes to write | `gwz-core/src/workspace_ops/tests/g13.rs` — `:110-112` asserts root and member share one `GWZ-Commit-ID`; `:239` `commit_marker_can_be_disabled`; `:196` asserts trailer presence. CLI tri-state at `gwz-cli/src/tests/g01.rs:325`; Python at `gwz-py/src/tests/test_cli_mutation.py:110` |

The **artifact half is landed too** — `artifact::write_marker` is called at
`handle_commit.rs:187-206`; `MarkerArtifact` / `read_marker` / `write_marker` /
`marker_path` / `list_markers` are in `gwz-core/src/artifact/mod.rs:272-411`.
`gwz.conf/markers/` holds **316 marker artifacts**, the newest
(`01a04ba4-8862-7092-ad29-ee8ec4a16c3c.yaml`) written today. The doc is not
merely landed-but-stale — it has been *extended* since (the artifact carries a
`merge:` field, `handle_commit.rs:204`, absent from the doc's schema).

Live evidence in this workspace: root commit `563d352` (2026-08-29) ends with
`GWZ-Commit-ID: 01a04ba4-…`, `GWZ-Workspace-ID: ws_default`,
`GWZ-Origin-URL-Hash: sha256:e35341f4…`. Trailer coverage by repo:
root 316/389, gwz-core 137/348, gwz-cli 32/102, gwz-py 32/68, taut 9/92,
taut-shape 11/18.

Three documents must be corrected, not just the plan:

1. `GwzLogPlan.md` — header authority chain, §1.1, §3 Phase 1, §4 sketch, §5
   item 2.
2. `GwzLogRequirements.md` §Coalescing "Dependency note" (lines 166-172):
   *"today's `gwz commit` writes NO `GWZ-Commit-ID` trailer"* and *"Until
   markers ship, L-COA-1 is exercised by tests only"* — both false. L-COA-1 can
   and should be exercised against real history from day one.
3. `GwzLogAmbiguityRezo.md` Q-1's "Confirmed reading" block (lines 42-45):
   *"NOTE: that design is status *proposed*, so the trailer-writing half becomes
   this project's dependency/companion"* — false. This is the **authoritative**
   document, so the correction needs the lane owner's (or operator's) hand.
   Note that Gianni's own resolution text (line 34) says nothing about markers;
   the error is entirely in the lane owner's annotation, so correcting it does
   not disturb any operator resolution.

Also correct `gwz-core/dev-docs/GwzCommitMarker.md`'s `Status: proposed` line —
it is the root cause of the whole error chain.

**Remedy.** Delete S1.1 and S1.2 as written. Replace Phase 1 with a small
audit-and-reconcile milestone (see **[P2 F5]** for its actual content), and
re-point §4 so Phase 2 does not hang off it.

---

### [P1 F2] The requirements doc is stale against its own "RESOLVED" status; at least seven v0 rows still pose their question instead of stating the answer

`GwzLogRequirements.md`'s header claims *"all thirteen questions … are answered
… the `Q-n` references below now point at final resolutions"* and *"Ready for
implementation planning."* The Rezo was answered, but the **rows themselves were
never edited**. They still read as open questions carrying "provisional"
markers:

| Row | Text as it stands | Resolved value (Rezo) |
|---|---|---|
| L-DEP-1 | *"there is a default depth limit … (limit shape — global vs per-member — **and its value** → Q-2)"* | global cap, N=50, lifted on any range/filter |
| L-ORD-2 | *"merged newest-first by timestamp (committer vs author timestamp → Q-4)"* | committer date |
| L-OUT-1 | *"(default format → Q-9)"* | compact one-line default |
| L-OUT-2 | *"whether it or a full block rendering is the default → Q-9"* | compact default, `--full` for blocks |
| L-OUT-3 | *"SHOULD exist (v0 inclusion → Q-9)"* | grouped mode **deferred to v2** |
| L-JSN-1 | *"Body inclusion → Q-12"* | subject only; `--body` opt-in |
| L-RNG-4 | *"SHOULD exist (spelling and v0-vs-v2 placement → Q-10)"* | `+lock` pseudo-operand, v0 |
| L-SEL-2/3, L-TOL-2, L-FIL-1, L-OUT-4/5, L-EXIT-1 | carry stale *"(provisional, → Q-n)"* / *"→ Q-n"* markers | endorsed as written |

This is not cosmetic. `GwzLogPlan.md` §2 defines the review method as
*"checklist against the step's **named requirements rows**"*, and S4.1's
traceability sweep binds *"every v0 `MUST` … with a named test"*. Rows that say
"→ Q-2" instead of "N=50" give the reviewer and the sweep no fixed target, and a
builder working from the requirements alone (which the requirements'
"Orientation for the implementing agent" section instructs them to do) cannot
build L-DEP-1, L-ORD-2, L-OUT-2 or L-JSN-1 at all.

**Remedy.** Reconcile the rows to the Rezo — fold each resolved value into the
row text, keep the `Q-n` reference as provenance, drop every "provisional".
Then add the grouped rendering (L-OUT-3) to the requirements' "Deferred (v2
candidates)" list, which currently lists only `--graph` *under* the grouped
rendering, not the rendering itself.

---

### [P1 F3] L-COA-4 (order by *latest* sibling timestamp) and L-PRF-1 (bounded-memory streaming k-way merge) cannot both hold; S2.3 and S2.4 own the two halves with no contract between them

L-COA-4: *"A coalesced entry's ordering timestamp is the LATEST committer
timestamp among its siblings."* L-PRF-1: *"The interleave MUST stream (k-way
merge over per-repo cursors), **not load entire histories into memory**."*

A pure k-way merge over per-repo cursors emits the newest cursor head and never
looks ahead. To emit a coalesced entry at its *latest*-sibling timestamp, the
merge must already know all siblings — but a sibling may sit arbitrarily deep in
another repo's cursor (rebase, amend, or simply a member committed hours later
under the same marker). The implementer has exactly three options and the
requirements name none of them:

1. Buffer until every cursor passes the entry's window → violates L-PRF-1.
2. Emit at first sighting and suppress later siblings → violates L-COA-4 (the
   entry lands at the *first*, i.e. newest-encountered, sibling's position, and
   the member set is incomplete at emit time, which also breaks L-JSN-1's
   `members[]` completeness).
3. A **bounded reorder window** — the only workable answer, and the one nobody
   has specified.

The plan makes this worse by splitting the two rows across steps: S2.3 owns
L-COA-4 (*"ordering timestamp = latest sibling committer date"*), S2.4 owns
L-PRF-1 (*"K-way streaming merge (L-PRF-1, bounded memory)"*). Two separate
Opus builders, sequential, with no named interface. Whichever builds second
discovers the contradiction with the first's code already landed.

Note this is *not* a problem for L-ORD-1 in the uncoalesced case — a k-way merge
over cursors preserves each cursor's own order by construction regardless of
timestamp monotonicity, so `--no-coalesce` is safe. The tension is specific to
coalescing.

**Remedy.** Before S2.3 launches, the plan must name the bounded window (a
natural choice: max(the L-COA-2 10 s heuristic window, a marker lookahead of K
entries per cursor), with entries outside it emitted un-coalesced and flagged),
and state the resulting L-COA-4 relaxation explicitly. Consider making the
window a requirements row so S4.1 can bind a test to it.

---

### [P2 F4] §1.1's trailer/artifact split is *mechanically* sound but the framing omits the case where the artifact is the only record — roughly half this workspace's member commits

**On the narrow question the mandate asks:** are the halves separable? **Yes.**
`handle_commit.rs:151-165, 220` proves that when `gwz commit` creates the
commits, members *and* root receive the identical trailer, so `GWZ-Commit-ID`
alone is a complete and *proven* coalescing key for L-COA-1. The marker doc's
Goals list makes the artifact load-bearing for *workspace-state restore and
inspection* (Goal 2) — which `gwz log` does not need. The one sentence in
`GwzCommitMarker.md` that cuts the other way is in §Marker Artifact: *"Future
`gwz log --merged` finds the root commit by locating the commit that contains
the marker file"* — a mechanism made redundant by the root's own trailer. §1.1
should cite and dismiss that sentence rather than leave it standing unaddressed;
a reader checking the split against the doc will hit it and stop.

**What §1.1 does not say, and should.** The artifact carries `committed_targets`
and a `members: {…, commit: <sha>}` map. In this workspace's *actual* recent
practice, member commits are made with plain `git`, not `gwz commit`, while the
root is committed through `gwz commit`. The newest marker
(`01a04ba4-…yaml`) shows this exactly: `committed_targets: ['@root']`, while
`members:` records `mem_gwz_core: commit: 8e18403…` and
`mem_gwz_cli: commit: 11bca66…`. Those member commits carry **no trailer**
(verified: `gwz-core` `8e18403`'s message ends at "acceptance record", no
`GWZ-Commit-ID`), and their messages differ completely from the root's, so the
L-COA-2 heuristic will not rescue them either.

Scale: 211 of 348 gwz-core commits, 70 of 102 gwz-cli commits, and 168 of 316
markers are in this shape. Trailer-only coalescing renders every one of those as
a singleton. This does **not** invalidate the feature — 148 markers committed
more than one target and will coalesce beautifully — but the plan should say so
honestly rather than presenting the trailer as universally sufficient, and
should record "artifact-assisted association for gwz-native-but-trailerless
member commits" as an explicit v2 candidate rather than leaving it invisible.

---

### [P2 F5] S1.1's one genuinely-unbuilt clause — "idempotence under partial-commit retries" — names a real defect in the landed code, and is the actual content Phase 1 should have

S1.1 asks for *"idempotence under partial-commit retries reviewed explicitly."*
The landed implementation is **not** idempotent, and the failure mode lands
squarely on L-COA-1:

- `handle_commit.rs:136` mints a fresh UUIDv7 on **every** invocation.
- `preflight_marker_path` (`:260-274`) *errors* if a marker with that id already
  exists — so it guards against id reuse, not against retry.
- Members are committed first (`:158-167`), the root last (`:219-221`).

Therefore: if the member loop succeeds and the root commit fails, a retry mints
a **new** `gwz_commit_id`. The already-committed members keep the old trailer;
the retry's members (those still dirty) and the root get the new one. The
siblings of one logical workspace change end up under **two different
`GWZ-Commit-ID` values** — and L-COA-2 forbids the heuristic from repairing it
(*"it MUST never merge across different `GWZ-Commit-ID` values"*), so the split
is permanent and un-healable.

This is the real Phase 1. Recommended replacement milestone: (a) correct
`GwzCommitMarker.md`'s status and reconcile its schema to the landed
`MarkerArtifact` (incl. `merge:`); (b) audit and, if confirmed, fix retry
identity (candidates: derive the id deterministically from the operation, or
persist-and-resume a pending id); (c) add the regression test. That is a
genuinely small, core-owned step and it keeps the plan's stated shape — a
foundational Phase 1 that makes the coalescing identity trustworthy — without
rewriting working code.

---

### [P2 F6] The heuristic's false-merge surface is materially understated — rebase is the case nobody has considered

§1.2 defends L-COA-2 with one guard: *"It never merges across distinct marker
values."* That guard is real but narrow. L-COA-2's own conjunction
(byte-identical full message + identical author name+email + committer stamps
within 10 s + different repos) leaves these open:

1. **Rebase re-stamping — the sharp one.** A rebase rewrites committer
   timestamps to *now*. `gwz forall -- git rebase origin/main` across N members
   re-stamps many commits into the same few seconds. Two historically unrelated
   commits in different members that happen to share a message ("wip", "fmt",
   "clippy", "Update deps") will land inside one 10 s window and merge. Rebase
   *destroys the timestamp diversity the heuristic depends on*, and it destroys
   it exactly across members, which is the axis the heuristic keys on. The
   requirements acknowledge rebases only for ordering (*"rebases backdate"*,
   §Unified stream) — never for coalescing.
2. **Release trains.** This program's own three-channel process (core → cli →
   py) produces `chore(release): gwz-core 0.11.1` and siblings, same author,
   seconds apart, different repos. Those are three deliberately-separate
   operations; they will coalesce.
3. **Scripted fan-outs.** `gwz forall -- git commit -m …` and `gwz forall -- git
   cherry-pick <sha>` complete well inside 10 s with byte-identical messages and
   (for cherry-pick) preserved author identity. Arguably *desirable* coalescing —
   but the plan should say which of these it intends, because L-COA-6 will label
   them `heuristic` and consumers need to know what that means.
4. **`commit --amend` twins** are safe: L-COA-2 requires *different repos*, and
   an amend replaces in place. Worth stating so, since the requirements never
   do.

None of these is fatal — the heuristic is opt-outable (L-COA-3) and provenance
is machine-visible (L-COA-6). But §1.2's "honestly framed?" test fails on
rebase. **Remedy:** name the rebase hazard in §1.2 and in L-COA-2's rationale;
add a MUST-NOT-merge fixture for it to S2.3's list (which today covers only
"same message different author", "outside the window", "distinct markers");
and consider narrowing the heuristic with a cheap extra conjunct (e.g. identical
*author* timestamp, which rebase preserves and independent commits will not
share).

---

### [P2 F7] Steps assign CLI-owned surface to gwz-core steps; the clap flags for most of the feature are unowned

`GwzLogPlan.md`'s header states the split: *"`gwz-core` owns semantics …
`gwz-cli` owns the clap surface and rendering."* The step briefs violate it:

- **L-EXIT-1** is assigned to S2.4 (*gwz-core*): *"exit-code mapping 0/1/2 with
  `--strict` promotion (L-EXIT-1, Q-13)"*. Verified: the mapping lives in
  **gwz-cli** — `exit_code_for_response` re-exported at
  `gwz-cli/src/globalargs.rs:12`, and `std::process::exit` at
  `gwz-cli/src/lib.rs:140-194`. Core produces `AggregateStatus`; only the CLI
  turns it into a process exit code.
- **Flags owned by core steps:** `--no-coalesce` (S2.3), `--strict` (S2.2),
  `-n`/`--no-limit` and the `--since`/`--until`/`--author`/`--grep`/
  `--no-merges`/`--first-parent` set (S2.4), `--body` (S3.2 — correct),
  `--color` (S3.1 — correct).
- **S3.1's brief covers only** *"Clap surface with the standard global
  selectors"* plus rendering. It never says it lands the log-specific flags.

Result: a builder taking S2.4 cannot land `-n` (no clap surface exists yet), and
a builder taking S3.1 will read its brief and land only the global selectors.
The flags fall in the gap.

**Remedy.** Either (a) state that each core step lands its semantics plus the
corresponding clap flag in gwz-cli (making S2.2/S2.3/S2.4 cross-repo steps, with
the §2 landing discipline applied to both repos), or (b) move all flag
definitions into S3.1 and expand its brief and budget accordingly. Reassign
L-EXIT-1 to S3.1 (or split: aggregate-status semantics S2.4, exit mapping S3.1)
either way.

---

### [P2 F8] `+lock` cannot resolve for `@root`, and the plan never says what happens to the root under a lock-relative range

`LockArtifact` (`gwz-core/src/artifact/mod.rs:159-164`) has exactly four fields:
`schema`, `workspace_id`, `manifest_schema`, `members`. **There is no root
entry** — confirmed against `gwz.conf/gwz.lock.yml`, which goes straight from
`manifest_schema:` to `members:`.

Since L-SEL-2 puts `@root` in the default selection, `gwz log +lock..` degrades
`@root` on *every* invocation — the workspace's own history, which Q-3
deliberately made first-class, silently vanishes from the one range that is
supposed to answer "what moved since the recorded state".

Two adjacent facts the plan should also absorb, both good news for §1.3's
feasibility worry and both needing a decision:

- **Classification is not the problem.** `is_never_path_operand`
  (`gwz-core/src/diff/operands.rs:173-175`) returns true for *any* `+`-prefixed
  token, so `+lock` already classifies as a revision without touching
  `classify.rs`. §1.3's fallback to `--since-lock` is unlikely to be needed —
  the risk is in *resolution*, not classification.
- **`gwz diff` already omits the root under `+` operands** (`operands.rs:75-79`:
  *"A snapshot operand narrows the candidate set (root is omitted; snapshotless
  members are …)"*). That is a usable precedent — but adopting it means
  `+lock`/`+snapshot` ranges silently drop `@root`, which contradicts L-SEL-2's
  spirit and is not stated in L-RNG-3 either.
- **`--tagged` rejects `+` operands** (`operands.rs:114`: *"--tagged does not
  accept GWZ snapshot operands"*). S2.2 owns both L-SEL-3 and L-RNG-3/4 and does
  not mention the exclusion.

**Remedy.** Decide and record: does `+lock` degrade `@root` (with a degradation
record per L-TOL-2/L-JSN-2), or does it resolve the root against something —
and if so, what? Add the answer to L-RNG-4 before S2.2 launches.

---

### [P2 F9] §4's dependency sketch invents a dependency that does not exist, serializing the project and defeating its own stated parallelism

The sketch is:

```text
S0.1 ── S1.1 ── S1.2
          │
          └── S2.1 ──┬── S2.2 ──┐
```

This makes **all of Phase 2 depend on S1.1**. The plan's own parenthetical
contradicts it: *"S2.3 needs S1.1's trailers **only for fixtures**"*. S2.1
(selection, repo opening, per-repo cursors, degradation records) has no
relationship to commit markers whatsoever. Under §3's charter that *"independent
agents MAY pick up parallel steps"* and the standing rule to minimise cross-step
coupling, S2.1 should branch directly off S0.1 in parallel with Phase 1. As
drawn, a Phase-1 park (which §2 makes likely, since any P0/P1/P2 parks) freezes
the entire project.

The fallback clause has two further defects:

- *"S2.3 proceeds on hand-built trailer fixtures and **re-verifies against the
  landed trailers before S4.1**"* — the re-verify point is set later than the
  real cross-check. S3.3's real-workspace battery, which runs actual
  `gwz init` + `gwz commit`, *is* the verification; it sits before S4.1 already.
  Naming S4.1 adds a redundant gate and, worse, implies the S3.3 battery is not
  the check.
- If Phase 1 is still parked when S3.3 runs, S3.3's marker assertion (*"asserting
  a coordinated commit renders as one entry"*) **cannot run** — but the plan
  does not say the battery's marker arm parks with it. (Note: with F1 corrected,
  this whole branch is moot, because trailers already ship and S3.3's real
  workspace will produce them unconditionally. The clause should simply be
  deleted rather than repaired.)

---

### [P2 F10] S2.4 bundles four goals and S2.2 bundles five-plus-a-decision; neither is one goal under ~500 LOC

Per the standing "one step = one goal, aspirational < 500 LOC" rule:

- **S2.4 (~300 LOC)** carries four independent goals, each with its own
  requirement family: the k-way streaming merge with `--jobs` concurrency
  (L-PRF-1/2), the depth cap and its lift rules (L-DEP-1), the six-filter
  passthrough with entry-level semantics (L-FIL-1 + L-COA-5), and exit-code
  mapping (L-EXIT-1). The merge alone — streaming, bounded-memory, concurrent,
  order-stable, and now (per **F3**) carrying a bounded coalescing window — is a
  full step. 300 LOC for all four is not credible; a realistic figure is 600–800.
- **S2.2 (~350 LOC)** carries classifier wiring, `+snapshot` per-member
  resolution, pathspec routing, `--tagged`, resolve-or-degrade with `--strict`,
  *and* `+lock` — which §1.3 and §5 both flag as a **reviewable architectural
  decision** with a fallback. A step containing a reviewable decision should not
  also contain five mechanical goals; the decision gets rubber-stamped under
  budget pressure.

**Remedy.** Split S2.4 into "merge + depth" and "filters + exits"; lift `+lock`
out of S2.2 into its own small step (it is the only new range surface in the
project and it has an open design question per **F8**).

---

### [P2 F11] §2 process gaps: no real-workspace battery duty at the Phase-1 landing, no docs-gate duty where the marker CLI surface moves, no owner for review-driven requirements edits

§2 is otherwise a faithful and proportionate instantiation of the program loop
at feature tier — single-axis Opus interior reviews, two-round cap with round 2
final, verbatim reports filed beside the plan, ritual-7 per-commit-green or
squash-citing-reviewed-shas landings, direct exit codes on all four gates,
worktree removal at lane close, insurance copies, escalations PARK rather than
waive. That is the right shape and it is not over-heavy for a feature lane. The
gaps:

1. **Real-workspace battery duty is scoped to one step, not to landings.** The
   J-7 ritual appears only inside S3.3 (*"the J-7 lesson applied from birth"*).
   The program treats real-workspace batteries as a *landing* duty (see
   `CurrentProgramCheckpoint.md:2405`, "REAL-WORKSPACE BATTERIES ALL GREEN at
   afbc25d (the J-7 ritual…)"). The step that most needs one is the Phase-1
   commit-semantics step — a change to `handle_commit.rs` that passes unit tests
   but leaves the root worktree dirty or the marker pending would only show up
   against a real workspace. §2's Landing bullet should carry the duty for any
   step touching `workspace_ops/`, not just S3.3.
2. **Docs gate is conditioned on "when docs move" but no step is told the marker
   CLI surface moves them.** §2 says *"gwz-cli landings run the docs gate when
   docs move"* and S3.1 correctly claims it. Any Phase-1 successor that touches
   `--commit-marker` help text must regenerate `docs/CLI.md` (precedent: gwz-cli
   `3000916`, *"A1: unhide `--no-ff` in the merge CLI; regenerate docs/CLI.md"*).
3. **No owner for requirements-doc corrections.** This review produces findings
   (**F2**, **F6**, **F8**) whose remedy is an edit to `GwzLogRequirements.md`,
   and one (**F1**) whose remedy edits the *authoritative* Rezo. §2's
   Remediation bullet covers code remediation only. State who may edit the
   requirements, and that Rezo edits require the operator.

---

### [P2 F12] Q-11's pager resolution rests on a factually false premise about house behavior

Q-11 states: *"Other gwz commands write straight through (no pager) and this
suits composition."* That is wrong for the nearest-neighbour command. `gwz diff`
**pages by default**: `gwz diff -h` lists `--no-pager` ("Do not pipe human patch
output through a pager"), and the implementation is real —
`gwz-cli/src/pager.rs`, consumed at `gwz-cli/src/diff_exec.rs:30, 150-161`
(`PagerDecision::Pager`, `pager::resolve_pager_command`, `PatchSink`), with a
documented pager-quit path.

Gianni answered *"yes - no pager"* on that premise. The resolution is
authoritative and I am not second-guessing it — but the operator was told the
house behavior was one thing when it is the other, and the consequence is that
`gwz log` will be the only long-output gwz read command that does **not** page,
diverging from the command whose grammar it is explicitly modelled on.

**Remedy.** Surface the correction to the operator before S3.1 launches and get
a one-line re-confirmation; whichever way it goes, record the divergence (or its
removal) in L-OUT-5. Cheap to do now, expensive after the surface ships.

---

### [P3 F13] LOC budget basis is inconsistent, so no budget is checkable

S1.1 reads *"~250 LOC + tests"*; every other step gives a bare figure (S2.1
~400, S2.2 ~350, S2.3 ~350, S2.4 ~300, S3.1 ~400, S3.2 ~250, S3.3 ~300 —
"tests only"). Whether the Phase 2/3 numbers include their tests is undecidable,
and it is a 2× difference. S3.1's figure additionally has to absorb command docs
and help text. State the basis once in §2 (the program's own convention excludes
generated output from handwritten LOC — `AgentProcessRules.md:1361`) and restate
each step against it.

*(Incidental: S1.1's ~250 was, as it happens, a fair estimate — the marker code
in `handle_commit.rs` is roughly 160 lines plus the artifact helpers. The budget
was sound; the step was simply already done.)*

---

### [P3 F14] S2.2 cites "L-PTH rows" — no such rows exist

S2.2: *"pathspecs after `--` (**L-PTH rows**)"*. `GwzLogRequirements.md` has no
`L-PTH-*` family. Pathspec handling is inside **L-RNG-1** (*"pathspecs come
after `--` and a leading `+` after `--` is a path"*). Either cite L-RNG-1 or add
the family; as written, the coverage matrix has a dangling reference and the
S2.2 reviewer has nothing to check against.

---

### [P3 F15] Name collision: `log` is already taken inside gwz-core

`gwz-core/src/diff/log_service.rs` (392 lines) defines `DiffLog`,
`DiffLogRegistry` and the taut-shape mailbox pump — the diff **output** log,
nothing to do with commit history — and `gwz-core/src/diff/tests/t_log.rs` (473
lines) tests it. The plan never names a module home for the new engine. A
builder creating `gwz-core/src/log/` or adding `log` symbols to `diff/` will
collide with, or be confused by, an unrelated subsystem. Name the home (e.g.
`gwz-core/src/commit_log/`) in §1 and note the existing `diff::log_service` is
unrelated.

---

### [P3 F16] L-OUT-3 (grouped rendering) is deferred by Q-9 but is neither owned by a step nor listed as deferred

Q-9's resolution defers grouped mode to v2 (*"grouped mode deferred to v2 (`gwz
forall -- git log` covers the gap meanwhile)"*). L-OUT-3 still reads *"SHOULD
exist (v0 inclusion → Q-9)"*, the requirements' "Deferred (v2 candidates)" list
mentions only `--graph` *under* the grouped rendering, and no plan step claims
it. It is the one row that is genuinely orphaned in both documents. Fold into
**F2**'s reconcile pass.

---

### [P3 F17] S4.1's traceability sweep covers "every v0 MUST", leaving the SHOULD rows the plan actually builds untraced

S4.1 and §2 both scope traceability to *"every v0 `MUST`"*. But the plan builds
two `SHOULD` rows: **L-RNG-4** (`+lock`, S2.2 — a headline v0 feature per Q-10)
and **L-FIL-1** (the filter passthrough set, S2.4 — six user-visible flags).
Neither would appear in the sweep. Extend S4.1 to "every v0 row the plan
implements, MUST or SHOULD", or promote those two rows to MUST in the reconcile
pass since the operator chose them for v0.

---

### [P3 F18] Q-1's narrowing from "the same commit" to "one coordinated `gwz commit` operation" is a lane-owner interpretation worth one operator confirmation

Gianni's answer (Rezo line 34) is: *"In essence 'gwz log' should merge log
entries where they came from the same commit. The output should be clear on
which log entries applies to which set of member repos."* The lane owner's
"Confirmed reading" narrows "the same commit" to sibling commits of one
coordinated `gwz commit`. That reading is well-argued, clearly marked as the
lane owner's, and almost certainly right.

But note that the requirements then straddle *both* readings without saying so:
L-COA-1 implements the narrow reading (one operation, proven by marker), while
L-COA-2's heuristic implements a broader one (any commits sharing message +
author + a 10 s window — which catches `forall` fan-outs and cherry-picks that
were *not* one operation). Given **F6**, the operator may want to say which he
meant. One line in the Rezo settles it and it changes what S2.3 builds.

**On fidelity generally:** the second half of Gianni's sentence — *"clear on
which log entries applies to which set of member repos"* — is faithfully and
well encoded (L-OUT-1 quotes it verbatim; the member-set entry model in
§Concepts, L-JSN-1's uniform `members[]` array including the singleton case, and
L-COA-5's narrowing semantics all follow from it). Q-2 through Q-13 are
substantively faithful; their defect is staleness of form, not of content
(**F2**). The one outright fidelity break is the marker-status claim in Q-1's
annotation (**F1**).

---

### [P3 F19] §1.5's "frozen surfaces untouched" is sound for gwz log, but the charter's wording bans reads it will need

**On the mandate's question — is it plausible gwz log needs nothing under
`checked_artifact/` or `v1_lifecycle/`? Yes, entirely.** `gwz log` opens repos,
walks revisions, and groups entries. `checked_artifact/` is the catalog /
admission / capability machinery and `workspace_ops/merge/v1_lifecycle/` is the
merge lifecycle; neither has any bearing on a read-only history walk. The
verified facts back this: `gwz log` will not gate on conf integrity (L-TOL-6),
takes no mutation lock, and reuses `diff::classify` — none of which reaches
those trees.

The wording is the issue. §1.5 says *"A step whose diff **touches** those trees
is out of charter by construction."* But S2.2 must **read** lock and snapshot
artifacts to resolve `+snapshot` (L-RNG-3) and `+lock` (L-RNG-4). Those live in
`gwz-core/src/artifact/` — *not* under either frozen tree, so the charter holds
as written. Still, since `artifact/mod.rs` is adjacent to the conf-integrity
system and a `+lock` accessor may not exist yet, spell out that *calling* an
existing artifact API is always in charter and only a **diff** under
`checked_artifact/` or `v1_lifecycle/` is out. One sentence prevents a
false-positive charter stop mid-step.

---

## Coverage matrix

Every requirement row in `GwzLogRequirements.md`, mapped to the plan step that
owns it. "Named" = the step brief cites the row id. "Implied" = the step brief
describes the behavior without citing the row (buildable, but invisible to §2's
"checklist against the step's **named** requirements rows" method and to S4.1's
sweep).

| Row | Owner | Basis | Status |
|---|---|---|---|
| L-SEL-1 | S3.1 | *"Clap surface with the standard global selectors"* | implied |
| L-SEL-2 | S2.1 | *"the standard selection (`@root` + members, Q-3)"* | implied |
| L-SEL-3 | S2.2 | *"`--tagged` (L-SEL-3)"* | **named** |
| L-RNG-1 | S2.2 | *"operand classifier (L-RNG-1)"* | **named** |
| **L-RNG-2** | **—** | no step mentions the no-operand default (HEAD history, detached included) | **UNOWNED** |
| L-RNG-3 | S2.2 | *"`+snapshot` per-member resolution (L-RNG-3)"* | **named** |
| L-RNG-4 | S2.2 | *"the `+lock` pseudo-operand (L-RNG-4, Q-10)"* | **named** (see F8, F10) |
| L-RNG-5 | S2.1 | *"NO network (L-RNG-5)"* | **named** |
| L-COA-1 | S2.3 | *"The L-COA-1..6 block"* | **named** |
| L-COA-2 | S2.3 | as above | **named** |
| L-COA-3 | S2.3 | as above | **named** — but `--no-coalesce` is a clap flag in a core step (F7) |
| L-COA-4 | S2.3 | *"ordering timestamp = latest sibling committer date"* | **named** — conflicts with L-PRF-1 (F3) |
| **L-COA-5** | **S2.3 + S2.4** | S2.3 *"L-COA-1..6 block"*; S2.4 *"entry-level semantics per L-COA-5"* | **DOUBLE-OWNED** |
| L-COA-6 | S2.3 | *"provenance tags"* | **named** |
| L-TOL-1 | S2.1 | *"(L-TOL-1/3/4/5)"* | **named** |
| L-TOL-2 | S2.2 | *"per-member resolve-or-degrade (L-TOL-2)"* | **named** |
| L-TOL-3 | S2.1 | *"(L-TOL-1/3/4/5)"* | **named** |
| L-TOL-4 | S2.1 | as above | **named** |
| L-TOL-5 | S2.1 | as above | **named** |
| L-TOL-6 | S2.1 | *"NO conf-integrity gate (L-TOL-6)"* | **named** |
| **L-ORD-1** | **—** | S2.1's *"newest-first per-repo revwalk"* does not settle the sort (git's default commit-date order vs `--topo-order`; the requirements' §Unified stream says *"topological, newest first"*, which conflates the two) | **UNOWNED** |
| L-ORD-2 | S2.4 | *"K-way streaming merge"* | implied |
| L-DEP-1 | S2.4 | *"the global default cap 50 with `-n`/`--no-limit` and lift-on-range/filter (Q-2)"* | implied (cited by Q, not row) |
| L-FIL-1 | S2.4 | *"the filter passthrough set applied per-repo pre-merge … (Q-5)"* | implied (cited by Q, not row) |
| L-OUT-1 | S3.1 | *"the set rendering of L-OUT-1"* | **named** |
| L-OUT-2 | S3.1 | *"compact default line `<date> <member-set> <short-hash> <subject>`"* | implied |
| **L-OUT-3** | **—** | deferred to v2 by Q-9; no step owns it, and the requirements' Deferred list omits it | **UNOWNED** (see F16) |
| L-OUT-4 | S3.1 | *"stderr degradation summary (L-OUT-4)"* | **named** |
| L-OUT-5 | S3.1 | *"color on tty only, no pager (Q-11)"* | implied (see F12) |
| L-JSN-1 | S3.2 | *"per L-JSN-1/2"* | **named** |
| L-JSN-2 | S3.2 | as above | **named** |
| L-EXIT-1 | S2.4 | *"exit-code mapping 0/1/2 with `--strict` promotion (L-EXIT-1, Q-13)"* | **named but MISASSIGNED** — mapping lives in gwz-cli (F7) |
| L-PRF-1 | S2.4 | *"(L-PRF-1, bounded memory)"* | **named** — conflicts with L-COA-4 (F3) |
| L-PRF-2 | S2.4 | *"concurrency under `--jobs` without ordering effects, L-PRF-2"* | **named** |
| *"L-PTH rows"* | S2.2 | cited by S2.2 | **DANGLING — no such family** (F14) |

**Summary.**

- **Unowned rows (3):** L-RNG-2, L-ORD-1, L-OUT-3.
- **Double-owned rows (1):** L-COA-5 (S2.3 and S2.4).
- **Misassigned rows (1):** L-EXIT-1 (core step, CLI-owned surface).
- **Dangling citation (1):** "L-PTH rows".
- **Named vs implied:** 24 named, 8 implied. Given that §2's review method and
  S4.1's sweep both key on *named* rows, every implied row should be made
  explicit in its step brief.
- **Steps owning no requirement row (5):** S0.1 (this review), **S1.1**, S1.2,
  S3.3, S4.1. Four of those are legitimately non-implementation steps (review,
  docs, tests, sweep). **S1.1 owning no row is the tell** — the plan's
  self-declared "most scrutinized step" is not traceable to a single requirement
  it satisfies, which is consistent with **F1**'s finding that it has no work to
  do. Its only genuine content (retry idempotence, **F5**) is likewise
  untraceable, because no requirement row covers marker identity stability —
  arguably a missing row for `GwzCommitMarker.md`'s own requirements, not this
  project's.

---

## Dispositions — checklist items 2 through 6

2. **§1 scope decisions.** *Not sound as a set.* (a) The trailer/artifact split
   is mechanically correct and the halves *are* separable for L-COA-1 — but the
   premise that either half is unbuilt is false (**P0 F1**), the doc's own
   `gwz log --merged` sentence goes unaddressed, and the artifact's role for the
   ~half of member commits made outside `gwz commit` is unframed (**F4**).
   (b) Heuristic-in-v0 is the right call but dishonestly framed: rebase
   re-stamping, release trains and `forall` fan-outs are unmentioned (**F6**).
   (c) `+lock` framing is over-worried about classification (which already works
   — `is_never_path_operand`) and under-worried about resolution, which cannot
   serve `@root` at all (**F8**). (d) "Frozen surfaces untouched" is genuinely
   plausible — `gwz log` needs nothing under `checked_artifact/` or
   `v1_lifecycle/` — but the "touches" wording would false-positive on the
   artifact reads `+snapshot`/`+lock` require (**F19**).

3. **LOC budgets and step boundaries.** *Not sound.* S1.1's ~250 was a fair
   estimate for work already done (**F1**, **F13**); S2.4 bundles four goals and
   S2.2 five plus a reviewable decision (**F10**); the budget basis is
   inconsistent and therefore uncheckable (**F13**); and CLI-owned surface is
   scattered across core steps (**F7**). On the mandate's specific question:
   idempotence-under-retry *is* a bigger surface than budgeted — it is a real
   defect in landed code and should be Phase 1's entire content (**F5**).

4. **Process instantiation (§2).** *Faithful and proportionate, with three
   gaps.* The loop is correctly applied at feature tier — single-axis Opus
   interior reviews, two-round cap, verbatim reports beside the plan, ritual-7
   landings, four direct-exit-code gates, worktree hygiene, escalations PARK
   rather than waive while the Fable pool is empty. Nothing is over-heavy for a
   feature lane. Missing: real-workspace battery as a *landing* duty rather than
   one step's content, docs-gate duty where the marker CLI surface moves, and an
   owner for review-driven requirements/Rezo edits (**F11**).

5. **Dependency sketch (§4).** *Incorrect and not parallel-friendly.* It invents
   a Phase-1 dependency for S2.1 that the plan's own prose denies, serializing
   everything behind a step that (per §2) parks on any P0/P1/P2 (**F9**). The
   hand-built-fixture fallback is sound in principle but sets its re-verify point
   later than the real check (S3.3's real-workspace battery) and does not park
   that battery's marker arm alongside Phase 1 — and with **F1** corrected the
   entire clause is moot and should be deleted, not repaired.

6. **Reconciliation fidelity.** *Substantively faithful, formally stale, with one
   outright break.* Q-1's sharpening is well encoded — the member-set entry
   model, L-OUT-1's verbatim quote of *"clear on which log entries applies to
   which set of member repos"*, L-JSN-1's uniform `members[]` shape, and
   L-COA-5's narrowing semantics all follow correctly from the operator's
   sentence; the narrowing of "the same commit" to "one `gwz commit` operation"
   is a defensible, clearly-marked lane-owner reading that deserves one operator
   confirmation given **F6** (**F18**). Q-2..Q-13 are endorsed-as-proposed and
   substantively carried, but the requirement **rows were never edited** to state
   their resolved values — at least seven remain unbuildable as written despite
   the doc's RESOLVED banner (**P1 F2**), and L-OUT-3 is orphaned by Q-9's
   deferral (**F16**). Q-11's resolution was given on a false premise about house
   pager behavior (**F12**). The one outright break is Q-1's annotation asserting
   `GwzCommitMarker.md` is unbuilt (**P0 F1**) — an error in the *authoritative*
   document, requiring the lane owner or operator to correct.

---

## Single most important condition

**Delete Phase 1 as written.** Correct `GwzCommitMarker.md`'s status line, the
requirements' Coalescing dependency note, and the Rezo's Q-1 annotation to
record that the marker — trailers *and* artifact — landed and is in production;
then re-plan Phase 1 as the small audit it should be, centred on the retry
identity defect in `handle_commit.rs` (**F5**), and re-point §4 so Phase 2
branches directly off S0.1.

---
---

# Round 2

- **Step:** S0.1 round 2 — re-verdict of the remediated set. Final under the
  two-round cap (`GwzLogPlan.md` §2, Remediation).
- **Date:** 2026-08-29. Same reviewer, same single-axis Opus mode, peer-blind.
- **Objects re-read in full:** `GwzLogPlan.md` (round-2 rewrite),
  `GwzLogRequirements.md` (rows reconciled inline), `GwzLogAmbiguityRezo.md`
  (post-review addendum, two dated notes).
- **Verification performed beyond re-reading** (read-only; nothing built,
  mutated, committed or tagged):
  - Rebuilt the coverage matrix from scratch against the resliced steps.
  - **Measured the new W = 60 s window against real marker groups** — every
    `GWZ-Commit-ID` across root + 7 members, grouped, committer-date spread
    computed. See "Empirical verification" below.
  - **Executed the old and new heuristics over real unmarked history** (573
    unmarked commits, all cross-repo pairs) to test whether the author-date
    conjunct regresses any true merge. See below.
  - Re-checked `gwz-core/src/diff/operands.rs:106-118` for the `--tagged` × `+`
    refusal that L-SEL-3 now asserts.
  - Re-confirmed the F5 defect is unmitigated in the tree (no pending-id or
    resume mechanism anywhere in `handle_commit.rs`).

## Verdict

# GO-WITH-CONDITIONS

The fold is thorough, accurate and — where I could test it — **empirically
correct**. All nineteen round-1 findings are cured; none is partially cured or
untouched. The coverage matrix is now clean for the first time: every one of the
36 requirement rows is owned by exactly one named step, with no unowned rows, no
double-ownership, no misassignment and no dangling citations. The P0 is properly
reconciled in all three documents, and the escalation ruling is recorded
explicitly rather than by omission, as round 1 asked.

Round 2 raises **8 new findings (3 P2, 5 P3)**, none architectural. Every P2's
entire remedy is a document edit the lane owner is already authorized to make
under §2, and each gates exactly one step's launch. The plan is adoptable.

**Round-1 dispositions:** 19 CURED / 0 PARTIALLY CURED / 0 NOT CURED.
**New findings:** P0 × 0, P1 × 0, P2 × 3, P3 × 5.

### Conditions (each discharged by a plan/requirements edit before that step's brief is issued)

1. **Before S1.1 launches** — require the step to define the "is this a retry of
   the *same* operation?" predicate, and re-budget it. **[P2 F22]**
2. **Before S2.4 launches** — say where the L-COA-7 window buffer lives (S2.4 or
   S2.5), and give L-COA-7 a modal verb. **[P2 F21]**
3. **Before S3.1 launches** — split it or raise its budget honestly, and name the
   flag-surface rows it co-owns. **[P2 F23, P3 F20]**

The five P3s are record-only and need not gate anything.

---

## Empirical verification of the two substantive cures

These are the two places where the fold made a *quantitative* claim, so I tested
both rather than reading them.

### W = 60 s (L-COA-7, curing F3) — VALIDATED, with 60× headroom

Grouped every `GWZ-Commit-ID` across the root and all seven members (546
trailer-carrying commits, 153 groups spanning more than one repository) and
measured each group's committer-date spread:

| spread | groups |
|---|---|
| ≤ 10 s | **153 (all)** |
| 11–60 s | 0 |
| 61–300 s | 0 |
| > 300 s | 0 |

The observed **maximum spread is 1 second** (most are 0). My concern going in was
the opposite of what the data shows — that a slow 20-member `gwz commit` could
straddle the window and split a *proven* marker group, silently relaxing
L-COA-1's MUST. On this workspace's real history that cannot happen: W = 60 s is
60× the worst observed case. The choice is sound and the L-COA-7 relaxation will
effectively never fire for marker groups, which is exactly the right shape.

### The author-timestamp conjunct (L-COA-2, curing F6) — SAFE, and correctly fixture-backed

Ran both specifications over the 573 unmarked commits, comparing every
cross-repo pair sharing a byte-identical message and author identity:

| | pairs merged |
|---|---|
| round-1 3-conjunct spec | 130 |
| round-2 4-conjunct spec | 130 |
| **true merges regressed by the new conjunct** | **0** |

Every surviving pair is a genuine `gwz commit` fan-out (committer-date delta 0 s,
author-date delta 0 s, root + members sharing one subject — e.g. `Full gwz diff
impl.` across root/gwz-core/gwz-cli/gwz-py). So the conjunct costs nothing on
real data: **zero false negatives**.

It also refuses nothing on this corpus (130 − 130 = 0), because the rebase hazard
it was minted for has not yet occurred here. That is fine — it is prophylactic —
but it means the conjunct's *effectiveness* rests entirely on a synthetic
fixture. S2.4's brief already requires exactly that fixture (*"the **rebase
re-stamp** case … committer dates collapsed to now, author dates distinct"*), so
the cure is complete rather than merely asserted. Good.

I also re-derived the reasoning independently: rebase and cherry-pick both
*preserve* author date while re-stamping committer date, so the conjunct
discriminates precisely where round-1 F6 said it must, and L-COA-2's
"same-repo commits NEVER merge" clause does make `--amend` twins safe by
construction, as claimed.

### L-SEL-3's `--tagged` × `+` claim — ACCURATE

`parse_tagged_comparison` (`gwz-core/src/diff/operands.rs:106-118`) rejects any
endpoint that is not `Endpoint::Revision`. Since `Endpoint::classify` marks every
leading-`+` token as a snapshot reference, `+lock` hits the same rejection as
`+snapshot`. L-SEL-3's "same refusal" is structurally true, not merely hoped for.
(One wording nit follows as **F27**.)

---

## Round-1 finding dispositions

| # | Sev | Finding | Disposition | Where cured |
|---|---|---|---|---|
| F1 | P0 | Phase 1 re-implements landed marker machinery | **CURED** | Plan header + §1.1 + Phase 1 rewritten as audit + §6 ruling; Requirements Concepts + L-COA-1; Rezo Q-1 dated correction. All three docs. (Root cause in `GwzCommitMarker.md` deferred to S1.2 — see **F26**.) |
| F2 | P1 | Requirements rows stale against RESOLVED banner | **CURED** | Every row now states its resolved value with `Q-n` as provenance; no "provisional" survives; the only remaining `→` is L-COA-5's cross-reference to L-JSN-1, which is correct usage. |
| F3 | P1 | L-COA-4 ↔ L-PRF-1 unsatisfiable | **CURED** | L-COA-7 minted (W = 60 s, closure rule, out-of-window siblings share the provenance key); L-COA-4 scoped "within the L-COA-7 window"; L-PRF-1 restated against it. Empirically validated above. Ownership seam → **F21**. |
| F4 | P2 | Artifact coverage limit unframed | **CURED** | Requirements Concepts states the trailerless-member limit explicitly; the superseded `--merged` sentence is called out; "artifact-assisted association" recorded under Deferred; plan §1.2 carries it. |
| F5 | P2 | Retry-identity defect is the real Phase 1 | **CURED** | Phase 1 rebuilt around it; L-COA-8 minted; S1.1 owns it with a regression test and real-workspace battery duty. Scope/hazard residual → **F22**. |
| F6 | P2 | Heuristic false-merge surface understated (rebase) | **CURED** | L-COA-2 hardened to four conjuncts with the rebase rationale in-row; fan-out semantics declared; rebase MUST-NOT fixture named in S2.4. Empirically verified safe above. |
| F7 | P2 | CLI surface assigned to core steps | **CURED** | All log-specific flags → S3.1 (option (b)); L-EXIT-1 reassigned to S3.1 and its row now names the `exit_code_for_response` seam; Phase 2 heading says "engine parameters only — no clap flags". Residuals → **F20**, **F23**. |
| F8 | P2 | `+lock` cannot resolve `@root` | **CURED** | L-RNG-4 rewritten (per-member from `members:`; `@root` degrades with a record; classification named a non-risk); §1.5 records the decision; S2.3 owns it. L-RNG-3 additionally gained the parallel `+snapshot` root divergence — more than I asked for, and correct. |
| F9 | P2 | §4 invents a Phase-1 dependency | **CURED** | Sketch redrawn with Phase 1 as an independent lane; *"Phase 1 feeds nothing downstream … A Phase-1 park does not block anything else"*; the moot fallback clause deleted rather than repaired. |
| F10 | P2 | S2.4/S2.2 bundle multiple goals | **CURED** | Resliced into S2.1/S2.2/S2.3/S2.4/S2.5/S2.6, one goal each, `+lock` lifted to its own decision step as asked. (The same principle now bites S3.1 as a consequence of F7's fix → **F23**.) |
| F11 | P2 | Three §2 process gaps | **CURED** | Real-workspace battery is now a landing duty on any `workspace_ops/` diff; docs-gate duty on CLI-surface moves with the `--no-ff` precedent cited; requirements-edit ownership stated with an annotation-only rule for the Rezo. All three, precisely. |
| F12 | P2 | Q-11 pager premise false | **CURED** | Rezo Q-11 carries the dated premise correction plus a fresh `Resolution (Gianni):` line; L-OUT-5 records the caveat and that no-pager governs meanwhile; S3.1 checks before landing; §5 tracks it. |
| F13 | P3 | LOC basis inconsistent | **CURED** | §1.8 states it once: handwritten non-generated LOC including tests, aspirational not hard. |
| F14 | P3 | Dangling "L-PTH rows" citation | **CURED** | S2.2 owns L-RNG-1 *"incl. its pathspec clause"* and names the round-1 dangling citation. |
| F15 | P3 | `log` name collision in core | **CURED** | `gwz-core/src/commit_log/` named in the plan header, the Phase 2 heading and the requirements' Orientation, each with an explicit "do not touch `diff::log_service`" warning. |
| F16 | P3 | L-OUT-3 orphaned | **CURED** | Marked `(v2 — DEFERRED)` in-row and listed first under Deferred, with Q-8's `--graph` folded beneath it. |
| F17 | P3 | Sweep scope excludes implemented SHOULDs | **CURED** | §2 Traceability, S4.1 and the requirements' Conventions all now read MUST-or-SHOULD. (In practice the reconciliation promoted the SHOULDs to MUST, so this is belt-and-braces — no harm.) |
| F18 | P3 | Q-1 breadth narrowing unconfirmed | **CURED** | Rezo Q-1 carries the breadth re-ask with a fresh `Resolution (Gianni):` line; L-COA-2 declares the intended consequence so the standing behavior is unambiguous while it is pending. |
| F19 | P3 | Charter wording bans needed reads | **CURED** | §1.7 now reads *"no DIFFS under"* and states that calling or reading existing core APIs is always in charter. |

---

## New findings

### [P3 F20 — folded into condition 3] S3.1 lands seven flags whose requirement rows it does not own

S3.1's ownership list is L-SEL-1, L-OUT-1, L-OUT-2, L-OUT-4, L-OUT-5, L-EXIT-1.
Its brief, however, lands `-n`/`--no-limit` (L-DEP-1, owned by S2.5), the six
filters (L-FIL-1, S2.6), `--strict` (L-TOL-2, S2.2), `--no-coalesce` (L-COA-3,
S2.4) and `--body` (L-JSN-1, S3.2).

The engine/flag split itself is correct and is stated in-brief for L-COA-3
(*"engine semantics"*) and L-TOL-2 (*"the `--strict` FLAG itself is S3.1's"*) —
that is a clean resolution of F7. The residual is purely a review-checklist gap:
§2 defines review as *"checklist against the step's NAMED requirements rows"*, so
no reviewer's checklist covers the seven flags S3.1 actually ships. A reviewer
checking S2.4 will not see `--no-coalesce` (it is not in S2.4's surface) and a
reviewer checking S3.1 will not see L-COA-3.

*Severity note:* filed P3 in substance, but it is folded into condition 3 because
the fix is one line in the same edit that resolves **F23**.

**Remedy.** Add to S3.1: *"co-owns (flag surface only): L-DEP-1, L-FIL-1,
L-TOL-2, L-COA-3, L-JSN-1."*

### [P2 F21] L-COA-7 assigns merge-emission behavior to S2.4, which has no merge state — and the row has no modal verb

L-COA-7 is owned by **S2.4** (coalescing). Its normative text describes the
**merge**: *"The streaming merge holds emission only within a bounded reorder
window … a group is closed when **every live cursor has advanced past** (group's
newest committer timestamp − W) … bounded memory is O(selected repos × entries
within W)"*.

"Every live cursor's position" is k-way-merge state, and the k-way merge is
**S2.5** — a later, separate step and a separate builder. §1.4 gestures at the
seam (*"S2.4 builds the grouping against it, S2.5 consumes it"*) and S2.5's brief
claims the memory bound (*"K-way merge consuming the L-COA-7 window (memory
O(repos × entries within W))"*), so the same behavior is described from both
sides without either being told to *implement the buffer*.

Two readings, both defensible, and nothing chooses between them:

- **(a)** S2.4 owns a windowed grouping buffer that takes cursor positions as
  inputs and emits closed groups; S2.5 merely feeds it. L-COA-7 is then
  satisfiable inside S2.4 and S2.5's memory claim is inherited.
- **(b)** S2.4 owns only the grouping *predicate* (same marker / four-conjunct
  match); S2.5 owns the buffer and closure. L-COA-7 is then **not satisfiable by
  its owning step**, and S2.4's reviewer will checklist a row the step
  structurally cannot meet.

Under (b) — which is the more natural reading of "coalescing" vs "merge" as step
names — S2.4's review fails a row through no fault of its builder, and S2.5's
builder discovers a missing component after S2.4 has landed. This is exactly the
class of two-builder seam that round-1 F3 was about; the *contract value* (W) is
now named, but the *interface* is not.

Secondary, same row: **L-COA-7 carries no `MUST`/`SHOULD`/`MAY`**. It is the
newest and most load-bearing normative row in the document (it is the F3
contract, and it explicitly relaxes a MUST), yet under
`GWZRequirements.md`'s convention — which the requirements' own Conventions
section adopts — a row without a modal verb is not a testable requirement. S4.1's
sweep keys on exactly that. (L-COA-3 and L-FIL-1 share the defect but are
inherited from round 1 and lower-stakes.)

**Remedy.** One sentence in §1.4 or S2.4's brief naming which step owns the
window buffer — reading (a) is the better design, since it keeps all coalescing
state in one place and lets S2.5 stay a pure merge. Then give L-COA-7 a `MUST`.

### [P2 F22] S1.1 needs a "same operation?" predicate that neither document requires, and 200 LOC is optimistic — my round-1 estimate was the optimistic one

L-COA-8 requires: *"One workspace-level change MUST carry ONE `GWZ-Commit-ID`
across partial-commit retries."* S1.1 offers the builder two candidate shapes,
both taken verbatim from my round-1 report: derive the id deterministically, or
persist-and-resume a pending id.

**I under-called this in round 1** — I described it as *"a genuinely small,
core-owned step"* and the ~200 LOC budget was set on that characterisation. On
closer reading both shapes are harder than that:

- *Deterministic derivation* has no stable input. A retry after a partial commit
  sees a **different tree state** (some members already committed), so deriving
  from message + selection + tree hashes yields a different id — precisely in the
  case that matters. The only genuinely stable input is the id already written
  into an already-committed sibling, which means reading trailers back, which is
  a lookup, not a derivation.
- *Persist-and-resume* is durable state: a pending-id file, its cleanup on
  success, its behavior under concurrent operations, and its staleness policy.
  (`durable_fs.rs` exists, so the machinery is there — but it is still a feature,
  not a patch.)

More important than the budget: **both shapes need a predicate neither document
states — "is this invocation a retry of the same operation, or a new one?"** If
the answer is guessed wrong in the permissive direction, the fix *creates* the
inverse defect: a user who abandons a failed commit, changes the content, and
commits again would have two genuinely different workspace changes fused under
one `GWZ-Commit-ID` — an unrecoverable false merge in L-COA-1, which is strictly
worse than the split L-COA-8 exists to prevent. The step must be told to define
and test that predicate, including its expiry, not left to infer it.

**Remedy.** Add to L-COA-8 (or S1.1's brief) that the fix MUST define when two
invocations are the same operation, MUST NOT fuse distinct operations, and MUST
carry a regression test for *both* directions; re-budget S1.1 to ~350–400 LOC
including tests; and consider making the shape choice an explicit reviewable
in-step decision the way `+lock`'s was in §1.5.

### [P2 F23] S3.1 is F10's principle recurring in Phase 3 — three goals at the budget ceiling before docs

S3.1 is budgeted *~500 LOC including tests* (§1.8's basis) and carries: the clap
subcommand and standard global selectors, **eleven-plus log-specific flags**, the
compact default rendering with L-OUT-1's member-set rules (≤3 inline, count form
beyond), the `--full` block rendering with a member table, the stderr degradation
summary, color handling, the exit-code mapping, **and** command docs plus help
text — with the docs gate at its landing.

That is three or four goals sitting exactly at the aspirational ceiling before a
line of test code. Phase 2 was resliced from two fat steps into six clean ones in
response to F10; Phase 3 was not, and F7's option (b) made S3.1 fatter than it
was in round 1. The step most likely to blow its budget is now also the one
carrying the user-visible surface.

This is a *consequence* of correctly fixing F7, not a regression — but it should
be resliced by the same standard.

**Remedy.** Split into S3.1a (subcommand + all flags + exit mapping + docs) and
S3.1b (human rendering: compact, `--full`, member-set forms, degradation
summary, color), both after S2.6, parallel with each other and with S3.2 —
or keep one step and raise the budget honestly to ~800 with a note that it
exceeds the aspirational target by design.

### [P3 F24] L-COA-2 does not say whether a *marked* commit may join a *heuristic* group

L-COA-2 governs *"Unmarked commits … MUST coalesce under a conservative
heuristic"* and forbids merging across different `GWZ-Commit-ID` values. It does
not say whether a **marked** commit and an **unmarked** commit that satisfy all
four conjuncts may join. Both answers are reachable from the text, and L-COA-6's
provenance vocabulary (`marker:<uuid>` | `heuristic` | `none`) has no value for a
mixed group, so S2.4 and S3.2 would have to invent one.

The case is not hypothetical: it is exactly the F4 pattern (root committed via
`gwz commit`, members via plain `git`) whenever the messages happen to match.
Worth deciding deliberately — permitting it would partially close the F4 coverage
gap; forbidding it keeps `marker:` provenance meaning "proven, whole group".

**Remedy.** One clause in L-COA-2, and if mixed groups are permitted, a
provenance value for them in L-COA-6.

### [P3 F25] §2's new escalation carve-out is a program-level amendment made inside a feature plan, and it has no proportionality valve

§2 now reads: *"with one recorded carve-out exercised at S0.1 round 1: a finding
whose substance is **directly verifiable fact** (not contested judgment) may be
reconciled by the lane owner re-executing the evidence, with the ruling recorded
in the adoption trail."*

The carve-out is narrow, well-drafted, correctly recorded in §6, and my own
round-1 report invited exactly this ruling — I have no objection to how it was
exercised. Two observations for the record:

1. It amends a **program-level** rule (*"no step lands with an unserved
   escalation"*) inside a **feature-lane** plan. Program rules amended in feature
   documents become precedent by accretion. It should be ratified where the rule
   lives, or at minimum surfaced to the operator, rather than inherited silently
   by the next feature plan that copies this §2.
2. It covers verifiable *fact* but not **trivially-remediable judgment**. Applied
   literally, this round's three P2s escalate and park S1.1, S2.4 and S3.1 — and
   because S2.5/S2.6/S3.x sit downstream of S2.4, that parks nearly the whole
   project on findings whose entire remedy is a one-sentence document edit the
   lane owner is *already* authorized to make under the same §2's Remediation
   clause. A second carve-out — "a P2 whose complete remedy is a
   lane-owner-authorized document edit is discharged by making the edit and
   recording it" — would restore proportionality without weakening the policy for
   findings that need a real second axis.

### [P3 F26] The stale line that caused the P0 is the one thing still uncorrected, and its fix is gated behind a parkable lane

`gwz-core/dev-docs/GwzCommitMarker.md` still says `Status: proposed`. Deferring
the edit to S1.2 to avoid a dangling modified file in a member repo is sound repo
hygiene and I agree with the call. But S1.2 *"rides S1.1's train"*, S1.1 is the
project's most scrutinized step, and Phase 1 is explicitly a parkable lane — so
the document that produced a P0 stays wrong for as long as Phase 1 is parked, and
anyone reading it in the meantime gets the same false premise.

Mitigation is genuinely in place (plan, requirements and Rezo all now say the
line is stale), so this is low risk. But the correction is one line and does not
depend on S1.1's outcome.

**Remedy.** Decouple the status-line correction from the rest of S1.2 so it can
land on its own train whenever convenient; leave the schema reconciliation and
the retry-fix record riding S1.1.

### [P3 F27] `--tagged +lock` will report "does not accept GWZ snapshot operands"

L-SEL-3 says the `+`-operand refusal carries over *"the same refusal, same
wording"*. Structurally correct (**verified** — `parse_tagged_comparison` rejects
any non-`Revision` endpoint, and `+lock` classifies as a snapshot reference). But
the inherited message names snapshots, so `gwz log --tagged +lock..` tells the
user their `+lock` is a snapshot operand it is not.

Reusing the existing refusal is right; reusing its exact wording is a small,
cheap-to-avoid confusion. **Remedy:** either broaden the message to "`+`-prefixed
operands (`+snapshot`, `+lock`)" — a one-string change in
`gwz-core/src/diff/operands.rs`, which also improves `gwz diff` — or note in
L-SEL-3 that the wording is knowingly imprecise for `+lock`.

---

## Coverage matrix — round 2 (clean)

Every requirement row, its owning step, and the step brief that names it. All
ownership is now **explicit** (round 1 had 8 implied rows; there are none).

| Row | Owner | Row | Owner |
|---|---|---|---|
| L-SEL-1 | S3.1 | L-TOL-1 | S2.1 |
| L-SEL-2 | S2.1 | L-TOL-2 | S2.2 *(flag: S3.1)* |
| L-SEL-3 | S2.2 | L-TOL-3 | S2.1 |
| L-RNG-1 | S2.2 | L-TOL-4 | S2.1 |
| L-RNG-2 | S2.1 | L-TOL-5 | S2.1 |
| L-RNG-3 | S2.2 | L-TOL-6 | S2.1 |
| L-RNG-4 | S2.3 | L-ORD-1 | S2.1 |
| L-RNG-5 | S2.1 | L-ORD-2 | S2.5 |
| L-COA-1 | S2.4 | L-DEP-1 | S2.5 *(flag: S3.1)* |
| L-COA-2 | S2.4 | L-FIL-1 | S2.6 *(flags: S3.1)* |
| L-COA-3 | S2.4 *(flag: S3.1)* | L-OUT-1 | S3.1 |
| L-COA-4 | S2.4 | L-OUT-2 | S3.1 |
| L-COA-5 | S2.6 | L-OUT-3 | *(v2 — deferred, listed)* |
| L-COA-6 | S2.4 | L-OUT-4 | S3.1 |
| L-COA-7 | S2.4 *(seam: F21)* | L-OUT-5 | S3.1 |
| L-COA-8 | S1.1 | L-JSN-1 | S3.2 *(`--body` flag: S3.1)* |
| L-EXIT-1 | S3.1 | L-JSN-2 | S3.2 |
| L-PRF-1 | S2.5 | L-PRF-2 | S2.5 |

- **Unowned rows: 0** (round 1: 3). **Double-owned: 0** (round 1: 1).
  **Misassigned: 0** (round 1: 1). **Dangling citations: 0** (round 1: 1).
  **Implied rather than named: 0** (round 1: 8).
- **Steps owning no row:** S0.1 (this review), S1.2 (docs), S3.3 (tests), S4.1
  (sweep) — all legitimately non-implementation. S1.1 now owns L-COA-8, closing
  round 1's "the most scrutinized step is traceable to nothing" tell.
- **Flag/engine splits** (marked *(flag: S3.1)*) are deliberate and stated
  in-brief; the only gap is that S3.1's ownership line does not list them —
  **F20**.

---

## Escalation classification (lane owner's to action, not mine)

Per `GwzLogPlan.md` §2, the three P2s auto-escalate. None is a verified-fact
finding, so round 1's carve-out does not apply to them. All three are, however,
findings whose complete remedy is a document edit §2 already places in the lane
owner's hands — which is the proportionality gap **F25** describes. Applied
literally the policy parks S1.1, S2.4 and S3.1, and via S2.4 most of the
downstream chain; applied to its evident purpose (getting a second axis onto
judgments that warrant one) the three conditions are discharged by making the
edits and recording them in §6.

I classify honestly and leave the ruling where it belongs. Round 2 is final under
the two-round cap: these are conditions on step launch, not a request for a third
review round.

