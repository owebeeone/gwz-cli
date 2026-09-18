# GwzClaudeIntegrationPlan.md — SAFETY-AXIS REVIEW, ROUND 4

**Review object:** WORKING-TREE `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `32bc9c74d831308515e31ab55e88129c5087219224dfeda11231b24e12788bcf`, 860 lines; with `gwz-core/dev-docs/GwzLaneCleanFixes.md` §3.6 (R20–R22), sha256 `ad8b23cde45b42f12939986a06e250e16fc79e077e5d9070c8ebf7440f9a166b`, in scope as the mechanism the plan rests on.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7` (object modified, review docs untracked); gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8` (`GwzLaneCleanFixes.md` modified). Tuple verified identical at start and at end of the review; nothing moved.
**Date:** 2026-09-17 **Axis:** SAFETY — what the text permits to go wrong: degraded and mixed-version paths, irreversible steps, disclosure, stuck states, the honesty of "harmless".

**Verdict: NO-GO** — 0 P0, 0 P1, **4 P2**, 4 P3. No NEW ARCHITECTURAL root cause: the R20/R21 mechanism is adequate in shape; every finding is a defect in how the plan *sequences and scopes* it. **Pre-commit: I will return GO on a bounded revision** that (a) adds the `creating`-row-owned-by-this-session case to D6's table as a wait rather than a refusal, (b) re-evaluates the guards after the wait, (c) gives the remove hook a `Busy` class and a stated wait policy, and (d) makes R20 state its index-format compatibility. No new evidence is needed for any of them.

---

## 0. Evidence base

Text read in full: the object (860 lines) and `GwzLaneCleanFixes.md` §3.6 (lines 107–132). Prior rounds consulted for closed-finding regression only: `-ReviewSafety-3.md` (GO, P3-1 lock-nesting — now moot, the hook takes no lock) and its closure table for round-3 P2-2 (the orphan case), whose cure explicitly rested on "the reuse rule is evaluated first, **under the family lock**". That premise is gone in this revision; finding P2-1 below is that closure re-opening by a different door.

Code facts verified read-only in gwz-core (nothing built, nothing mutated):

- `crates/family-store/src/format.rs:45-53,61-72` — `IndexFile` and `RowFile` are `#[serde(deny_unknown_fields)]`; `format.rs:123-128` — `decode_index` refuses any `schema:` not byte-equal to `INDEX_SCHEMA` (`crates/family-model/src/lib.rs:47`, `gwz.local-family/v1`). The module header states the intent: an unknown field or a foreign schema "refuse as malformed rather than being accepted or upgraded".
- `crates/workspace-install/src/request.rs:44-60` — the reserved row is written `MemberState::Creating` *before* the copy; `src/local_clone/create.rs:216-218` takes `try_lock` and `:272-277` runs `install` while still holding that session. So the `creating` row is visible, lock-free, for the entire multi-minute copy.
- `crates/family-store-contract/src/lib.rs:283` and `crates/family-store/src/lib.rs:244` — `read_view` is genuinely lock-free; `crates/family-store/src/lib.rs:6` — index writes are same-directory temp+rename with checked flushes, so the plan's lock-free read cannot tear. **This part of the plan's section 1 is accurate.**
- `src/local_clone/dispose.rs:140,153` and `:317,324` — dispose does `read_view` then `try_lock`; no wait exists today and the plan asks for none.
- `src/local_clone/errors.rs:101` maps `StoreError::Busy` to `ErrorCode::OpenOperation`; `:167` maps `InstallRefusal::SourceOpenMerge` to the *same* code.
- `src/local_clone/create.rs:220-228` — under the lock, create regenerates the root's managed `.git/info/exclude` block; i.e. a hook invocation writes into the root repository's git metadata.
- Environment: `gwz 1.0.13`; `git check-ignore -v .gwz/local-family.yml` → `.git/info/exclude:4:/.gwz/` (untracked, per-clone, gwz-managed — the plan's statement is correct); `df -g .` → 37 GB available against a 65 GB workspace.

## 1. Findings

### [P2-1] The reuse table treats a `creating` row as terminal and is evaluated *before* the wait, so the R21 wait covers only the millisecond window it was not written for

**Location.** Object D6:320-322 ("or before calling it when a row already exists, the hook reads the family index … and decides reuse"), D6:332 ("a `creating/incomplete` row: refuse; retire it through S3.4"), S1.1:487-492 (the same order in execution sequence); D5:284-288 asserts the opposite outcome.

**Root cause.** The pre-clone index read and the reuse table are placed ahead of the only thing that waits (the clone's `--wait`), and the table's `creating` row carries no owner qualifier — so the state that *means* "the other handler is mid-copy, wait for it" is classified as the state that means "a dead create left wreckage, refuse".

**Violated invariant.** D5:280-288: differing handlers for one `name`/`session_id` are harmless because "the second handler's clone waits behind the first's".

**State sequence the text permits.** Two differing handlers (project block + a user block pinned with `--command`, the case D5 declares harmless and S4.1 documents) fire in parallel for `name=api`, one `session_id`.
1. t=0 both processes spawn. A reaches `local clone` first, takes the lock, writes the `creating` row (`request.rs:51`), begins a 65 GB copy.
2. t≈0.2 s B does its pre-clone lock-free read (D6:320). It sees a **`creating`** row.
3. D6:332 applies: refuse, non-zero, nothing on stdout. B never calls clone, so `--wait` is never reached.
4. Claude aborts the creation (section 1:48 — any non-zero exit aborts).
5. A completes minutes later: a full lane, `ready`, owned by a session that no longer exists.

The interleaving the plan designs for (B sees *no* row and waits) occupies only the few milliseconds between B's process spawn and A's reservation; the interleaving that refuses occupies the whole copy. **The failure path is the dominant one, not the edge case.** The plan's own acceptance test at S1.1:522-526 — "two creations with the same `name` and `session_id` run concurrently (the second waiting behind the first's lock) yielding one lane and one path … one lane, two zero exits, one path, no abort" — cannot pass against the table as written, which is independent confirmation that the sequencing is wrong rather than merely unclear.

**Impact.** Every differing-handler creation normally ends in an aborted session plus an orphan lane of a 65 GB-class workspace that, before R0, retires only through the S3.4 waiver procedure (`--force dirty,unpreserved-history`) after the operator's own comparison against the family. The retry is a *new* session, so D6:339 ("owned by another session: refuse, naming that session") makes the slug unusable too. D5's "harmless" claim is not honest under this interleaving, and round-3 P2-2 — closed on a lock premise this revision deleted — is re-opened.

**Required correction.** Add to D6's table: *a `creating` row whose owner equals this `session_id`* → call `local clone --owner … --wait <wait-secs>` anyway; the clone blocks on the lock, R21's mandated reread then reports the name as held, and the hook re-evaluates the table on the settled row (`ready`+mine+complete → reuse; still `creating` → refuse). Equivalently: state that a row existing at pre-check suppresses the clone call *only* for terminal classifications, and enumerate which are terminal. Keep the unowned/foreign-owner `creating` row terminal.

**Closure test.** S1.1's concurrency test must start handler B **after** A's `creating` row is observable in the index (not merely "concurrently"), and assert one lane, two zero exits, one path, no abort. A second variant with A's clone killed mid-copy must assert B refuses fail-closed.

---

### [P2-2] The two guards are evaluated before an unbounded wait and never re-evaluated, so R21 converts a formerly fail-closed collision into a breach of the ready-row ceiling

**Location.** Object D6:305-312 (the two guards and "The guards apply only to an invocation that will create; the reuse rule is evaluated first"); S1.1:487-492 (guards, then clone with `--wait`).

**Root cause.** Guard evaluation is a lock-free pre-check whose result is consumed after an arbitrary blocking wait, with no re-check at the moment the create actually proceeds.

**Violated invariant.** D6:305-310: creation refuses when the family already holds `--max-lanes` `ready` rows, or free space is below `--min-free-gb`. These are the plan's only defence against the disk-full class of incident cited at section 1:147-149.

**State sequence the text permits.** Two sessions in gwz-dev create **different** names (a background session and a chip — S2.1/S2.2 both do this routinely). Family holds 7 `ready` rows, `--max-lanes` 8.
1. Both hooks read the index lock-free: 7 ready. Both pass both guards. Both pass the free-space floor (37 GB measured today).
2. A's clone takes the lock and copies. B's clone waits on `--wait` — which is exactly what R21 newly makes survivable; in gwz 1.0.13 B would have failed `Busy` here, fail-closed, with the ceiling intact.
3. A finishes: 8 ready. B acquires, rereads (R21 — the name is *not* held, it is a different name), creates: **9 ready rows**. The ceiling is exceeded, silently, with no refusal and no log line saying so.

The same holds for `--min-free-gb` with N concurrent creates all admitted against one pre-wait measurement.

**Impact.** The ceiling is the only bound on lane sprawl and therefore on the orphan accumulation P2-1 and D5 both produce; the floor is the only guard against repeating the 2026-09-11 disk-full incident. Both are breachable by exactly the concurrency this revision exists to enable, and the breach is invisible in D10's log because no decision was refused. Bounded (+N−1) but real, and not disclosed anywhere in the text.

**Required correction.** State that the guards are re-evaluated after the wait succeeds and before the copy begins — either by passing them into the clone call, or by the hook re-reading and re-checking once the clone reports it is about to create. If the operator prefers to accept the breach, D6 must say so explicitly with the bound, rather than stating an unqualified ceiling.

**Closure test.** Two concurrent creates of *different* names with the family at `--max-lanes` − 1 and the second carrying `--wait`: exactly one new lane, the second refused by the ceiling after its wait, nothing copied for it, the refusal logged.

---

### [P2-3] The remove hook has no `Busy` class and no wait policy, so a routine overlap either mis-reports a lock as a hazard or leaves a kept session pointing at a deleted lane

**Location.** Object D8:394-409 (the remove hook's decision procedure), D3:255-259 ("A refusal (unpreserved history, or dirt) …"), S1.1:497-505 (remove tests: no Busy case), D5:288-289 ("the hook whose `worktree_path` is already gone exits zero"). `--wait` appears nowhere on the remove path (grep of the object: every occurrence is on the create side). R21:122-125 requires it on *every* family command.

**Root cause.** The plan adopts R21 for `local clone` only; `local dispose` keeps 1.0.13's immediate `Busy` (`src/local_clone/dispose.rs:153,324`), and no refusal class is defined for it.

**Violated invariant.** D8:406-409 — the log line "distinguishes a refusal by the hook's own classification from a hazard refusal by gwz, so … a removal that failed for a reason D3 does not intend, can be seen for what it is."

**State sequences the text permits.**
*(a) Single handler.* Session X exits with removal while session Y's create holds the family lock for a multi-minute copy — the interleaving S1.3 and S2.2 deliberately exercise. Dispose returns `Busy`, which gwz maps to `ErrorCode::OpenOperation` (`errors.rs:101`), **the same code as an open coordinated merge** (`errors.rs:167`). The hook exits non-zero. A refusal that is neither hook-classification nor hazard is reported as the one class D8 leaves for it, and the operator's documented response to a refused removal (S3.4) is the `--force dirty,unpreserved-history` waiver procedure — applied on a false premise to a lane that was merely lock-blocked and would have disposed cleanly a minute later.
*(b) Two handlers.* Both classify the path as a lane; A's dispose takes the lock, B's returns `Busy` immediately while the directory still exists. Per section 1:59-61 B's non-zero exit with the directory present means **the removal fails and the session is kept** — while A's dispose completes and deletes the lane. The session is retained pointing at a path that no longer exists. D5:288-289 anticipates only the already-gone case, which is the narrow window; the wide window is "present at classification, gone by dispose".

**Impact.** A transient, self-clearing condition is presented to the operator as a permanent hazard whose documented remedy is an irreversible force-waiver retirement; and the two-handler race produces a retained session with no worktree, which no step in the plan inventories.

**Required correction.** Give the remove hook the same `--wait <secs>` treatment as create (R21 already mandates it), sized inside the remove hook's own timeout; add a third refusal class, "the family was busy", to D8's log distinction and to D10's stderr remedy line, with "retry; another family command holds the lock" as its remedy; and state the classify→dispose race outcome explicitly (dispose reporting the name absent must exit zero, not "anything else is refused").

**Closure test.** A remove against a family whose lock a stub holds past the deadline: refused, classified `busy`, logged as such, lane intact. A second remove whose target row disappears between classification and dispose: exit zero, logged.

---

### [P2-4] R20 adds a field to a frozen, `deny_unknown_fields` index format without saying what an older gwz does, and the plan's mixed-version row assumes a coexistence the format forbids

**Location.** `gwz-core/dev-docs/GwzLaneCleanFixes.md:115-121` (R20; silent on the schema version and on readers); object D6:337-339 ("a `ready` row at the destination with no owner (a lane made by hand **or by an older gwz**): refuse, fail-closed") and section 1:139-144.

**Root cause.** R20 specifies a new persisted row field and its consumer, but not its format-compatibility contract, against a store whose documented and enforced policy is that any unknown field or non-matching `schema:` is malformed.

**Violated invariant.** `crates/family-store/src/format.rs:5-7` and `:45-72,123-128`: `RowFile` is `deny_unknown_fields` and `decode_index` requires `schema == "gwz.local-family/v1"` exactly.

**State sequence the text permits.** A dogfood machine runs the R20 build, creates one lane; the row now carries `owner`. The operator then runs any older gwz in that workspace — a rollback after a bad release, the pinned 1.0.13 the acceptance runbook still names, or a second clone whose installed binary predates the release S1.4 waits for. `decode_index` fails on the unknown field: **not that row, the whole index**. `gwz local list`, `gwz local clone`, and `gwz local dispose` all refuse `ManifestInvalid` for every lane in the family. The retirement procedure S3.4 depends on exactly those commands, so the recovery path for the orphans P2-1 and D5 create is itself blocked until the newer binary is reinstalled or the YAML is hand-edited. Bumping the schema to `/v2` instead does not help an older reader (`wrong_schema` refuses identically) and is equally unstated.

**Impact.** A one-way, workspace-wide compatibility trap on a file the plan makes hooks write unattended and often, whose only stated fallback ("a row with no token … reports none", R20:120-121) describes a direction the format cannot reach. Recoverable by reinstalling gwz, hence P2 not P1.

**Required correction.** R20 must state the compatibility contract: whether `owner` is added under a bumped `schema:` (and then what an older reader's refusal message must say and how the operator downgrades), or whether `RowFile` relaxes to forward-compatible decoding for this field. The plan's D6:337-339 row and section 1:139-144 must then reflect whichever is chosen.

**Closure test.** R22's scope extended: an index written by an owner-aware build is read by a decoder without the field, and the outcome is the one the requirement states — with the message naming the version remedy.

---

### [P3-1] D10's "the log is the only file the hooks write inside a workspace" is false, and the ignore-check scope rests on it

**Location.** Object D10:440. **Root cause.** The sentence accounts for the hook's own writes and, parenthetically, gwz's index — but not the rest of what the in-process create writes. `src/local_clone/create.rs:220-228` regenerates the root repository's managed `.git/info/exclude` block under the lock; the lock file `.gwz/local-family.lock` is also created. **Impact.** The claim is the justification for scoping D10's ignore check to the log alone, and it tells a reader auditing the hook that a creation touches nothing else in the main checkout — while in fact a hook invocation mutates the root repository's git metadata and can change what `git status` shows on a workspace's first lane. S1.1:543-545's assertion that `git status --porcelain` is unchanged after a creation is therefore measuring a weaker property than the sentence claims. **Correction.** Rewrite the sentence to name every file a create writes at the root, and say which of them the ignore check governs. **Closure test.** The S1.1 fixture asserts the exclude block's before/after content explicitly rather than only `git status`.

### [P3-2] The owner token is a Claude `session_id` that `gwz local list` prints into evidence notes the plan commits to a public repository

**Location.** R20:117-119 ("MUST be reported by `gwz local list` (human and `--json`)"); object S1.3:604-607 and S3.4:678-680, which make `gwz local list` the inventory and commit probe notes to `gwz-cli/dev-docs/`. **Root cause.** A new caller-identity field is introduced into the output of the one command the plan's steps transcribe into committed documents, with no redaction rule. **Impact.** Not a credential and not committed by default (`.gwz/` is excluded — verified), but gwz-dev is public and the operator's own standing practice is to redact machine and account identifiers from committed gwz-dev docs; session ids link a lane to a local transcript. **Correction.** One line in S1.3/S3.4: owner tokens are redacted from committed probe notes and inventories. **Closure test.** None needed; a stated rule suffices.

### [P3-3] The fallback's stated justification for sharing one worktree does not cover the hazard sharing creates

**Location.** Object D8:368-376 — "a plain worktree is cheap and recreatable, and `git worktree remove` without `--force` protects a dirty tree, so the plan accepts that two sessions naming the same slug share it". **Root cause.** The justification addresses dirtiness; the hazard of sharing is removal of a **clean** shared worktree while a second session is still working in it, which a `--force`-less remove does not prevent. **Impact.** Session 1 exits with removal, `git worktree remove` succeeds on a clean tree, and session 2's working directory vanishes mid-run. In practice Claude's `git worktree lock` on a running agent's worktree (section 1:83-85) probably blocks it — but the plan neither claims that protection nor tests it, and U10 leaves the lock's behaviour open. **Correction.** Either state the lock as the protection and record it in S1.3's fallback probe, or have the remove hook decline a fallback worktree it did not create in this session's lifetime. **Closure test.** S1.1's fallback removal test with a second live holder of the same worktree.

### [P3-4] A hook killed at Claude's timeout leaves a `creating` row and no log line, and the plan never enumerates that residue

**Location.** Object S1.2:552-553 (the create timeout comes from S1.0), D6:313-317 (`--wait-secs` "sized from S1.0 to fit inside Claude's hook timeout"), D10:436-437 ("one line per decision"). **Root cause.** The wait budget is sized against a quiet-machine measurement (S1.0), while S1.3:588-590 deliberately repeats creation *under* a concurrent `cargo build`; nothing states what a timeout kill leaves behind. **Impact.** The hook is killed mid-copy: the OS releases the flock, but the `creating` row and partial files persist (`request.rs:51`, section 1:109-110), the name is thereafter terminal for every session under D6:332, and D10's per-decision log has no entry for the killed invocation, so the residue is discoverable only through `gwz local list`. Fail-closed and bounded, but undocumented. **Correction.** One sentence in D6 or D10: a timeout kill leaves a `creating` row and its files, `gwz local list` is the inventory, S3.4 the retirement, and no log line is written. **Closure test.** S1.1 kills a create mid-copy and asserts the next attempt with the same name and session refuses fail-closed with nothing on stdout.

## 2. Invariant analysis

What survived a genuine attack and should not be re-litigated:

- **The lock facts in section 1:113-119 are accurate.** `try_lock`, `flock(LOCK_EX|LOCK_NB)`, immediate `Busy`, `local list` lock-free — all confirmed against `crates/family-store/src/lib.rs:244-264` and `crates/family-store-contract/src/lib.rs:283`. The round-3 architectural correction has been absorbed honestly, not papered over.
- **Lock-free reuse reads cannot tear.** Index writes are same-directory temp+rename with checked flushes (`crates/family-store/src/lib.rs:6`), so the reuse decision always reads a whole index. The plan asserts this only by analogy to `local list`; the analogy holds.
- **The hook takes no lock**, so round-3's P3-1 (a hook holding the lock across a clone that takes it) is genuinely moot, not merely unmentioned.
- **The reuse table is fail-closed in every direction that matters**: unowned row, foreign owner, path mismatch, incomplete lane, unrelated directory. The no-owner row (D6:337-339) is the correct mixed-version answer — its only problem is P2-4, that older gwz cannot reach the state the row describes.
- **The remove hook's resolution discipline holds**: canonicalise first, resolve through `.gwz/family-root`, require exactly one matching `ready` row, explicit `--root`, cwd at the family root (which `dispose.rs:202-206,226` genuinely needs — the containment check is lexical against the canonical cwd), never `--force`/`--keep`. I could not construct a wrong-lane deletion from the text. The basename-collision test at S1.1:530-532 is the right one.
- **D3's fail-closed removal posture is sound** for the hazard classes it names; my objection (P2-3) is that a fourth class exists, not that these three are mishandled.
- **The settings writer (S1.2:555-571)** — parse-first, refuse non-regular targets and unparseable files, temp+fsync+re-parse+rename, no byte outside the block — is adequate for editing another program's configuration, and the interrupted-write test is present. The differing-handler *warning* rather than refusal is defensible on its own terms; what makes the differing-handler case unsafe is P2-1, not the warning.
- **D10's log** (bounded 1 MB, no `transcript_path`, no `cwd`, ignore-location check with fallback to the user log) discloses nothing sensitive and dirties no workspace. P3-1 is about the scope sentence, not the log.
- **S1.4's ordering choice** — committing the project block before R0 lands, with the one-line alternative (`settings.local.json`) named and the decision handed to the operator — is adequately framed: the consequence (routine lane creation for every clone while retirement is still the L1 waiver procedure) is stated in the same paragraph as the choice. Judged as adequacy, it passes; I take no position on the preference.

## 3. Risks and next action

The mechanism is right; the plan mis-sequences it. Three of the four P2s share one shape — a decision taken from a lock-free read *before* an unbounded wait and never revisited after it — and the cure in each case is to say what is re-evaluated on the far side of the wait. That is a bounded edit to D6, D8 and D10; none of it requires new evidence, a new probe, or a change to R20/R21's shape, which is why I am not classifying any finding as architectural.

The single most consequential risk in the current text is **P2-1**: it makes the plan's stated happy path the rare interleaving and its enumerated refusal the common one, so the first real differing-handler run will produce an aborted session and a 65 GB orphan that costs the operator the full L1 waiver procedure — and S1.1's own concurrency test, as written, will fail on the first run. **P2-4** is the one that could bite outside this plan entirely: it is a format change to a file every gwz workspace carries, and it currently has no stated compatibility contract at all.

Next action: one remediation patch covering P2-1 through P2-4 and the four P3s, then a re-verdict. The correction surface is D6's table and its ordering sentence, D8's remove-hook classification, D10's two sentences, S1.1's test list, and one added paragraph in `GwzLaneCleanFixes.md` R20. I will return GO on that revision if each correction lands as specified.
