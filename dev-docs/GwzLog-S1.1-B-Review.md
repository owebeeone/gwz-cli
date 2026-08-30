# GWZ Log S1.1-B peer-blind round-one review

## Review identity

- **Step:** S1.1-B, round 1 of the fresh two-round cap.
- **Date:** 2026-08-30.
- **Mode:** independent, single-axis, peer-blind review.
- **Core candidate:** `186c8253b2a5ad445c3372648d92434c7b0d0b91`.
- **Sole parent / grading base:** `dd31d54439e9244cba876d159383a5fc5e9584b2`.
- **Candidate tree:** `3aadfd98f6dca3015b81d45f3b8498019228467a`.
- **Current landed gwz-core main used only for integration risk:**
  `2214eace46b72915f76ab28e03e16716ce9d1a60`.
- **Report repository base:** gwz-cli main
  `15625a948a80f3ac18a6c1954503630e6bbf5298`.

The workspace instructions, applicable `AGENTS.md` files, and the mandated
sources were read completely before candidate review, in the dictated order:
`GwzLogPlan.md`, `GwzLogRequirements.md`, `GwzLogAmbiguityRezo.md`,
`GwzLog-S0.1-Review.md`, `AgentQuickStart.md`, and `GwzCommitMarker.md`.
The complete filed old S1.1 round-one/final report and the 2026-08-30 S1.1-B
RE-PLAN block were then read in full.

## Verdict

# NO-GO

Severity count:

- P0: 0
- P1: 2
- P2: 2
- P3: 1

The small candidate is a substantial improvement over abandoned S1.1: the
nominal member-succeeded/root-failed retry reuses X, its P/A/R partition and
`A != empty` guard are structurally sound, changed selection/complement cases
split, marker-disabled member-only selection does not publish the root, the
production delta is 214/300 changed LOC, and no lifecycle/backend/frozen-surface
expansion occurred.

It nevertheless fails two ordinary, in-envelope accident cases. First, it
compares HEAD with the pre-porcelain request string, while shipped `git commit
-m` applies normal message cleanup; an unchanged honest retry with ordinary
whitespace therefore mints Y. Second, discovery discards unreadable pending
evidence before the no-op decision and fresh replacement deletes X before Y is
durable; an I/O failure or crash can then produce a false-success no-op rather
than a safe fresh split. These are L-COA-8 failures, not adversarial-hook or
same-user-surgery defenses excluded by the amended envelope.

Round 2 is available. It must cure the two P1s and make the message/evidence
tests mutation-tight. If round 2 fails, the plan's standing fallback descopes
L-COA-8 to v2; there is no further round.

## Findings

### [P1 F1] Retry compares against pre-cleanup bytes, so ordinary Git message cleanup splits an unchanged retry

`CommitMarkerContext::commit_message` at
`gwz-core/src/workspace_ops/handle_commit.rs:281-293` trims only terminal LF
bytes. `retry_marker` reconstructs that string from the retry request at line
361, and `head_message_equals` at lines 394-400 requires the raw stored message
to be exactly that byte sequence plus one final LF.

The shipped backend does not store that input verbatim. It invokes porcelain
`git commit -m` at `gwz-core/src/git/gitbackend/repository.rs:291-331`, honoring
Git's configured cleanup. With `commit.cleanup` unset, ordinary default cleanup
removes trailing whitespace, trims leading/trailing blank lines, and collapses
consecutive blank lines. That is normal product behavior, not a hook or
same-user attack.

A disposable exact-candidate reproduction changed only the nominal retry
message to:

```text
do the work   \n\n\nsecond line\t
```

The first member commit stored the cleaned body, the root failed as intended,
and the identical retry minted Y. The existing assertion at `g13.rs:386`
failed with different root/member marker ids (test exit 101). A separate direct
default-Git probe showed the same mismatch:

```text
input:  "body with spaces   \n\n\nsecond line\t\n\n<canonical trailers>"
stored: "body with spaces\n\nsecond line\n\n<canonical trailers>\n"
```

This violates the byte-exact *canonical* message requirement and L-COA-8's
unchanged honest-retry promise. It cannot be registered as an accepted residual:
no user intervention occurred between attempts.

**Required round-2 cure:** compare against the canonical bytes the shipped
commit path actually stores, including the applicable Git cleanup behavior,
without adding a backend contract or durable state. Pin exact same-request
retry for cleanup-sensitive subject/body whitespace and canonical trailer tail
placement.

### [P1 F2] Pending evidence is filtered before no-op, and replacement deletes X before Y is durable

`pending_marker_ids` at `handle_commit.rs:297-324` is not raw discovery. It
parses and validates each status-visible marker and silently drops any path for
which `read_marker(...).ok()?` fails. `will_mutate` at lines 138-143 then sees
only this already-filtered set. After a member-only root failure, members are
clean and `root_has_changes` is false; if X is missing, transiently unreadable,
or between publications, the retry reports success/no-op instead of minting a
fresh id or surfacing an error. This contradicts the comment at lines 327-328
that unreadable/disagreeing evidence makes the caller mint fresh.

The candidate creates the state itself on a normal split path. It calls
`remove_pending_markers(X)` at line 240 and only then `write_marker(Y)` at line
241. A process crash, I/O error, or atomic-writer interruption between those
operations leaves staged X in the root index but no readable X in the worktree.
The next member-only invocation sees an index-added/worktree-deleted path,
drops it at line 312, and takes the false-success no-op at line 140. With later
new work, the same filtering can instead publish Y alongside unclassified old
evidence.

There is a related uncovered accident cut after the last member commit and
before the canonical marker rename. Member commits finish at lines 191-202,
whereas the marker is not written until lines 217-241; the atomic writer stages
and fsyncs a sibling temp before rename. A crash there can leave X only in the
member HEAD (or leave only the temp), after the member loop has completed. The
next member-only retry can again no-op. The requirements accept a crash
**mid-member-loop** as a split; they do not accept this post-loop publication
cut as a silent success.

These are cooperating-same-user accidents. They are not the old adversarial
WAL/CAS/symlink cases and cannot be placed in that residual register.

**Required round-2 cure:** separate raw pending-path discovery from evidence
qualification so any candidate recovery state suppresses no-op; make evidence
read failure fail toward a fresh split or a typed refusal, never success/no-op;
make Y durable before X is removed; and pin the delete/write, unreadable-marker,
and post-member/pre-marker cuts. Do not introduce a new lifecycle or namespace.

### [P2 F3] The changed-message regression is not mutation-tight

`changed_retry_message_mints_fresh_marker_and_rewrites_pending_artifact` at
`g13.rs:399-464` changes two axes at once: it stages new member work at lines
425-426 and changes the request message at lines 428-429. The complement
mismatch independently forces a fresh id, so the test does not prove that
message bytes participate in A.

In a disposable mutation, the member arm of `completed` treated every known
member as completed and skipped `head_message_equals`. All nine G13 tests still
passed. No retry test elsewhere killed the mutant.

**Required round-2 cure:** change only the request message with identical
selection and no new work; assert X is not reused. Add a body-only/canonical
trailer-placement mutation case so weakening the byte evidence cannot survive.

### [P2 F4] Unavailable evidence and isolated `A = empty` fresh convergence are unpinned

The new `read_head_message` seam at
`gwz-core/src/operation/commit_log/mod.rs:22-28` maps open/head/peel failure to
`None`, and the intended outcome is a bounded fresh split without backend
contract expansion. Every new retry test uses readable real `Git2Backend`
repositories. None proves that unavailable HEAD evidence mints Y, removes X,
commits exactly one marker, and avoids runtime/backend-contract failure.

The existing changed-message/new-work case indirectly kills deletion of the
`completed.is_empty()` guard only because it happens to make `A = empty` and
`R = P`; it does not isolate the unavailable-evidence/A-empty contract.

**Required round-2 cure:** add a single-axis unavailable-evidence fixture with
`P = {member, @root}`, `A = empty`, and `R = P`, then assert safe one-marker Y
convergence. Pair it with F2's unreadable-artifact case, which is not an
accepted unavailable-HEAD split because it currently returns false success.

### [P3 F5] Several low-cost matrix edges remain record-only

The focused matrix does not exercise a positive root-selected X reuse, the
three-trailer origin-hash form, an isolated root-only `A = empty` retry, exact
trailer order/uniqueness/tail placement, or preservation of the unrelated root
index entry's staged status in the marker-disabled isolation test. The current
tests establish the main behavior, but these edges are cheap additions while
round 2 is already changing the evidence tests.

## Complete S1.1-B named-row matrix

| Named row / frozen axis | Verdict | Evidence |
|---|---|---|
| L-COA-8 cooperating-same-user envelope | **Fail** | F1 is an unchanged honest retry; F2 is crash/I/O recovery. Neither is adversarial same-user surgery. |
| Read-only P/A/R proof | Pass structurally | `plan` P at lines 362-369; `completed` A at 371-383; remaining P-minus-A and current complement R at 388-391. |
| `A != empty` | Pass structurally; test coupled | Lines 384-386 reject reuse with empty A. The changed-message test kills removal only indirectly; F4. |
| Every completed HEAD carries X | Conditional fail | Exact raw comparison exists, but expected bytes are not Git-canonical; F1. |
| Byte-equal request message + canonical trailers | **Fail** | Default porcelain cleanup changes stored bytes before the comparison; F1. |
| Remaining targets exactly equal plan complement | Pass | Equality at line 391; `new_member_work_breaks_plan_complement...` passes and kills weakening. |
| Unproved sameness fails fresh | **Fail** | Qualified proof failure mints fresh, but unreadable evidence can disappear before `will_mutate` and return no-op; F2. |
| P/A/R A-nonempty nominal X reuse | Pass in simple fixture | `member_succeeded_root_failed_retry_reuses_marker_and_artifact` preserves member HEAD, id, and artifact bytes. |
| Root-failure retry convergence | **Fail overall** | Simple message converges; cleanup-sensitive message and crash/I/O cuts do not. |
| Marker artifact retain when proof matches | Pass in nominal fixture | Exact marker bytes are unchanged and the same path is committed. Same-user artifact editing remains an accepted residual below. |
| Marker artifact replace when proof differs | **Fail crash-safely** | Completed changed-message/selection paths replace X, but X is removed before Y is durable; F2. |
| Changed message splits | Intent present; test not tight | Production exact check exists; test admits a surviving no-message-evidence mutant; F3. |
| Changed selection splits | Pass | `changed_retry_selection_mints_fresh_marker` isolates selection mismatch. |
| Changed complement/new work splits | Pass | `new_member_work_breaks_plan_complement_and_gets_a_fresh_marker` isolates R mismatch. |
| Mid-member-loop crash split | Pass as accepted residual fixture | Two-member interruption test preserves first X, commits remaining work under Y, and records one Y artifact. It does not cover F2's post-loop cut. |
| Marker-disabled member-only root isolation | Pass | Root HEAD unchanged, staged root content uncommitted, root hook tripwire on Unix, lock refreshed. |
| Shipped selection/no-op/hook semantics | Pass except F2 false no-op | Ordinary fan-out, root-only, no-op, and porcelain hook path remain; F2 incorrectly classifies recovery evidence as no work. |
| Custom/non-native HEAD evidence | Partial | No API expansion; HEAD open failure returns `None` and should split. No fixture, and artifact-read unavailability falsely no-ops; F4/F2. |
| No new durable lifecycle / WAL / namespace | Pass | No new file or directory; existing marker artifact only. |
| No GitBackend contract change | Pass | No `src/git/**` or trait diff. The only seam is internal `pub(crate)` raw-message reading. |
| Hard 300 production changed-LOC cap | Pass | 191 additions + 23 deletions = **214/300** gross changed production LOC; 359 changed test LOC excluded. |
| No selection/no-op/hook/marker-disabled expansion | Conditional | No intended expansion; marker-disabled pin passes. F2 is an in-envelope no-op regression. |
| Frozen checked-artifact / merge-lifecycle paths | Pass | No diff under either tree; boundary checker exit 0. |
| Cargo, protocol, generated files, inventories, pins, `lib.rs` | Pass | No touched path; no dependency, schema, wire, census, or root export move. |
| Privacy | Pass | Raw local commit bytes are compared only; no message/URL is logged or exposed. Origin remains hashed. |
| Visibility | Pass | Only `pub(crate) read_head_message` and its `pub(crate)` operation re-export; no external public item. |
| Integration with core main `2214eac` | Pass, low risk | Merge-tree exit 0; only overlapping path is `operation/commit_log/mod.rs`, merged without conflict or symbol collision. |

## Old S1.1 P1 regrade and accepted-risk register

Every old P1 is classified explicitly. An accepted residual is not a request
for a defense in S1.1-B; any in-envelope regression remains a failure.

| Old row | Regrade under amended envelope | Candidate disposition |
|---|---|---|
| P1-1 porcelain hooks create/amend transitions | `AR-HOOK-REWRITE` — accepted | Hook-created history can forge matching evidence. Out of envelope; ordinary hook execution is unchanged. |
| P1-2 marker-disabled stale lock/root publication | **In-envelope must-pass** | Pass: lock refresh retained; unselected root is not committed and staged root content is not consumed. |
| P1-3 hook publishes private WAL | `AR-HOOK-PRIVATE-PUBLISH` — accepted | Out of envelope and structurally absent: candidate introduces no WAL/private namespace. |
| P1-4 foreign staged/worktree boundary evidence | `AR-HAND-BOUNDARY-SURGERY` — accepted | Same-user third-value index/worktree editing is excluded; no checkpoint/sidecar machinery added. |
| P1-5 unsafe hook publication laundering | `AR-HOOK-HISTORY-SURGERY` — accepted | Hook/history manipulation is excluded; a forged byte-equal HEAD can still fool the proof and is named. |
| P1-6 ignored orphan marker | `AR-HIDDEN-NAMESPACE-SURGERY` — accepted | Same-user marker creation/ignore manipulation is excluded. Normal accidental evidence loss is not; see F2. |
| P1-7 crash-surviving temp/recovery wedge | **In-envelope must-pass** | **Fail:** old retirement machinery is absent, but F2 leaves new post-member and replace-publication crash/no-op gaps. |
| P1-8 public/custom backend compatibility | Must preserve ordinary API; `AR-EVIDENCE-UNAVAILABLE` permits split | No trait change or late unsupported capability. Non-native HEAD proof fails toward split, but lacks a test. Artifact unavailability is F2, not accepted. |
| P1-9 assume-valid/skip-worktree | `AR-INDEX-FLAGS` — accepted | Explicitly out of envelope; candidate adds no preservation rejection. |
| P1-10 WAL CAS/replacement | `AR-LOCAL-BYTE-RACE` — accepted | No WAL exists; adversarial local replacement/races remain excluded. Candidate's own sequential delete/write crash is not adversarial and is F2. |
| P1-11 confinement/non-Unix | Normal publication must-pass; `AR-PATH-SURGERY` accepted | Parent publication path retained; no non-Unix regression introduced. Symlink/parent-swap attack remains excluded. |

Additional named accepted boundaries:

- `AR-SAME-TARGET-CONTENT-CHANGE`: between attempts, a same-user actor may
  add/replace staged content inside an already-remaining target (especially
  `@root`). Target-complement proof can reuse X and publish it. Detecting this
  requires the forbidden pre-failure index/tree witness or lifecycle.
- `AR-ARTIFACT-BYTE-TAMPER`: a same-user actor may edit non-proof fields of a
  staged pending artifact and have the retained artifact committed. Byte
  tampering is expressly excluded from the trust envelope.
- The requirement's **mid-member-loop crash split** remains accepted exactly as
  written. It does not silently broaden to the post-loop marker-publication cut
  in F2.

Old P2/P3 residue is also bounded: the scoped-root recovery, NotEnrolled,
unknown-WAL-field, typed-error-flattening, capability-preflight, and monolithic
state-machine findings belonged to abandoned `c6ea636` machinery and have no
analogue in this candidate. The old staged-only orphan/index manipulation is
covered by the accepted same-user surgery rows. The surviving matrix issue is
test incompleteness (F3-F5), not a demand to restore the abandoned defenses.

## Scope, LOC, integration and cleanliness

Exact candidate diff:

```text
8    0   src/operation/commit_log/mod.rs
1    0   src/operation/mod.rs
182  23  src/workspace_ops/handle_commit.rs
356  3   src/workspace_ops/tests/g13.rs
```

Production gross changed LOC is 214, leaving 86 lines under the hard cap.
There are no generated changes. The four paths are ordinary `100644`
modifications; no add/delete/rename/mode/submodule transition exists.

Against current core main, the candidate and main are siblings over exact base
`dd31d544`. Main adds S2.2/S2.2-B. Their changed-path intersection with this
candidate is only `src/operation/commit_log/mod.rs`. `git merge-tree
--write-tree 2214eac 186c825` exited 0 and produced tree
`e36441fe32684d353892b8502b87544bb72c3ff8`; the helper lands cleanly among
main's refactored imports and no symbol collision exists. Landing still owns a
post-merge build/test.

The candidate worktree stayed exactly at `186c8253...`, and both candidate and
landed-main gwz-core worktrees were clean after review. No core file, ref, or
commit was changed.

## Commands and direct exits

```text
git show -s --format='%H%n%P%n%T' 186c8253...
exit 0
candidate 186c8253b2a5ad445c3372648d92434c7b0d0b91
parent    dd31d54439e9244cba876d159383a5fc5e9584b2
tree      3aadfd98f6dca3015b81d45f3b8498019228467a

git diff --check dd31d544...186c8253
exit 0

cargo fmt --all -- --check
exit 0

python3 scripts/checks/check_checked_artifact_boundaries.py
exit 0
checked-artifact boundary: ok (15 visible entries, 5 classified modules)

CLIPPY_CONF_DIR="$PWD" cargo clippy --locked --all-targets -- -D warnings
exit 0

cargo test --locked workspace_ops::tests::g13 -- --nocapture
exit 0
9 passed; 0 failed

cargo test --locked operation::commit_log -- --nocapture
exit 0
35 passed; 0 failed

git merge-tree --write-tree 2214eac... 186c825...
exit 0
e36441fe32684d353892b8502b87544bb72c3ff8

git status --short --branch   # candidate core, final
exit 0
## codex/gwz-log-s1-1-b
```

The canonical 15-minute full suite was deliberately not repeated; landing owns
it and the builder's full evidence remains canonical. Disposable reproduction
and mutation copies were restored/removed; they are not credited as clean-HEAD
gates.

## Round-2 gates

Round 2 must, at minimum:

1. cure F1 with a cleanup-sensitive exact same-request retry and canonical
   trailer-tail assertion;
2. cure F2 so raw pending evidence always suppresses no-op, fresh publication
   is crash-safe without a new lifecycle, and unreadable/delete-write/
   post-member-pre-marker cuts converge safely or refuse explicitly;
3. kill the no-member-message-evidence mutant from F3;
4. pin unavailable HEAD evidence and isolated A-empty fresh convergence from
   F4; and
5. keep production gross changed LOC at or below 300 and every frozen boundary
   green.

No amendment may add a WAL, durable sidecar/lifecycle, namespace, backend
contract, or selection/no-op/hook/marker-disabled behavior. Round 2 is final.
