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

# GWZ Log S1.1-B terminal round-two review

## Final-review identity

- **Step:** S1.1-B, terminal round 2 of 2.
- **Date:** 2026-08-30.
- **Mode:** same independent reviewer; remediation-only review against the
  exact filed round-one report above.
- **Amended core candidate:**
  `0baf281c62a369413438427f33eb96c5665901dc`.
- **Sole parent / grading base:**
  `dd31d54439e9244cba876d159383a5fc5e9584b2`.
- **Candidate tree:** `a640517006765d8ce93d386aee0bf6820cdbfec2`.
- **Round-one candidate:**
  `186c8253b2a5ad445c3372648d92434c7b0d0b91`.
- **Filed round-one report commit / this report's parent:**
  `19f1e3d471c70385f1069180b9552ab7cbfa3649`.
- **Current landed gwz-core main used only for integration risk:**
  `2214eace46b72915f76ab28e03e16716ce9d1a60`.

The exact filed report was reread completely before inspecting the amendment.
The amendment is exactly one commit over the dictated base, with subject
`fix(commit): preserve marker identity across root retries`; there is no red
intermediate commit. Review was confined to the two round-one P1s, two P2s,
cheap P3 strengthening, preserved passes/residuals, the hard cap, and
base/integration integrity. Accepted out-of-envelope architectures were not
reopened.

## Terminal verdict

# NO-GO

Severity count:

- P0: 0
- P1: 2
- P2: 1
- P3: 0

Round 2 cures the default-cleanup reproduction, raw pending-YAML discovery,
write-new-before-unlink replacement order, the coupled changed-message test,
the pending-artifact unavailable-HEAD/A-empty cases, root-selected origin/tail
coverage, and marker-disabled root-index coverage. The focused tests are green,
the final production delta is exactly 295/300 gross changed LOC, and the
candidate integrates cleanly with landed main.

Two in-envelope correctness failures nevertheless remain. First, the expected
message is always normalized with libgit2's default prettifier while the shipped
porcelain commit path honors repository `commit.cleanup`; an unchanged honest
retry still splits under normal non-default configuration such as `verbatim`.
Second, the new post-all-members/no-marker inference discards proof failures
before `will_mutate`: after that exact crash cut a changed message can return a
false-success no-op, a changed selection can reuse X, and partial/unavailable
HEAD evidence can no-op or reuse from only the readable members. These outcomes
contradict fail-toward-fresh and are not any named adversarial residual.

The exact atomic-writer cut also commits its stale sibling temp on recovery.
That is lower severity because X itself converges, but the root history gains a
noncanonical `*.yaml.<pid>.<seq>.tmp` artifact. The test named for the cut
constructs a clean no-artifact state and therefore misses the writer state the
round-one gate expressly named.

This is the terminal review. Per the standing 2026-08-30 fallback, L-COA-8
descopes to v2; there is no round 3.

## Terminal findings

### [P1 F6] Canonical retry bytes still disagree with configured porcelain cleanup

Round 2 changes `head_message_equals` to compare raw HEAD bytes with
`canonical_commit_message(expected)` at
`gwz-core/src/workspace_ops/handle_commit.rs:453-457`. The helper at
`gwz-core/src/operation/commit_log/mod.rs:38-42` unconditionally calls
`git2::message_prettify(message, None)`. That exactly cures the round-one
default-whitespace fixture and the strengthened positive test proves the
default subject/body/trailer result.

The shipped writer remains porcelain `git commit -m` at
`gwz-core/src/git/gitbackend/repository.rs:291-305`, expressly so Git
configuration is honored. Its cleanup is controlled by repository
`commit.cleanup`; it is not unconditionally libgit2-default cleanup. A direct
Git 2.52 probe with `commit.cleanup=verbatim` and the strengthened fixture shape
stored the leading blank lines, trailing spaces, repeated blank lines, and tab
verbatim:

```text
$
$
do the work   $
$
$
second line\t$
$
$
GWZ-Commit-ID: 01999999-9999-7999-8999-999999999999$
GWZ-Workspace-ID: ws-probe$
```

`message_prettify(..., None)` removes that whitespace. The same request on an
honest retry therefore cannot put the already-committed member in A and mints
Y rather than retaining X. `commit.cleanup=strip` plus comment-shaped message
content supplies the inverse configuration-sensitive class. Repository Git
configuration is ordinary cooperating-user product behavior, not a hook
rewrite, hand-Git intervention between attempts, or local-state tamper.

Round-one F1 required the canonical bytes the **shipped commit path actually
stores, including applicable Git cleanup behavior**. The default-only helper
and test do not meet that gate.

### [P1 F7] Post-loop recovery cannot preserve fail-fresh across message, selection, or unavailable evidence

`recover_post_loop_marker` at
`gwz-core/src/workspace_ops/handle_commit.rs:336-381` reconstructs X only from
the **current** selected members whose readable HEAD message matches the
**current** request. A mismatching or unreadable HEAD is silently skipped at
lines 348-365. The caller at lines 131-147 invokes this only when pending YAML
and member work are both empty, then feeds only a successful reconstruction
into `will_mutate`.

Consequently the exact post-all-members/pre-marker crash state has this formal
path table (A and B are members already committed under X; no `X.yaml` exists):

| Retry state | Candidate execution | Required disposition |
|---|---|---|
| Same message, same A+B selection, both HEADs readable | Reconstruct and reuse X | Reuse X; pass |
| Changed message only | Both HEAD messages mismatch, reconstruction is `None`, `will_mutate` is false, return success/no-op | Mint fresh Y |
| Changed selection A+B to A | Scan only A, reconstruct X, publish an artifact whose current plan omits B, and reuse X | Mint fresh Y |
| Same selection, A readable at X and B unavailable | Skip B and reuse X from A alone | Fail fresh or refuse explicitly |
| Same selection, sole completed HEAD unavailable | Reconstruct nothing and return success/no-op | Fail fresh or refuse explicitly |

Changing only root-selected versus root-unselected has the same selection-loss
problem because the recovery helper receives no original root-selection plan.
Scanning only current selection also means the emitted `committed_targets` and
`members` maps can omit a sibling that already carries X.

These are not attacks. They are ordinary retries after the exact accident cut
round one required round 2 to cure. The read-only mechanism may use trailer
HEADs and the marker where present, but it must still fail toward splitting
when sameness is unproved. Here proof failure is erased before the no-op gate,
repeating the semantic core of round-one F2. The new changed-message and
unavailable-evidence tests exercise the pending-`X.yaml` path, so they do not
detect this no-artifact branch.

### [P2 F8] The actual atomic-write crash artifact is staged and committed during X recovery

The atomic marker writer stages and fsyncs a sibling
`<X>.yaml.<pid>.<seq>.tmp` before rename at
`gwz-core/src/artifact/mod.rs:471-503,542-551`. A crash in that interval is the
precise post-member/pre-marker cut called out in round-one F2. On retry,
`pending_marker_ids` accepts only exact `.yaml` names at
`handle_commit.rs:320-333`, so it ignores the stale temp and enters post-loop
recovery. Writing canonical `X.yaml` does not remove a prior-process temp.
Finally `sync_workspace_boundary` stages the whole `gwz.conf` directory through
`stage_workspace_git_metadata` at
`gwz-core/src/workspace_ops/stage_workspace_git_metadata.rs:7-13`; only
`gwz.conf/.tmp/` is excluded, not sibling temps in `gwz.conf/markers/`.

The root commit therefore contains canonical `X.yaml` **and** the stale
writer-temp path. Marker listing ignores the latter, so assertions based only
on `list_markers().len() == 1` do not catch it. The round-two
`post_member_pre_marker_cut_recovers_head_identity` fixture manually commits a
member and writes the lock, but creates no staged writer temp and does not
assert the complete root tree. Identity X converges, so this is P2 rather than
another P1, but accident recovery does not converge to the canonical durable
artifact set and publishes process/sequence implementation detail into history.

## Round-one remediation disposition

| Round-one finding / requested gate | Terminal disposition | Evidence |
|---|---|---|
| P1 F1: cleanup-sensitive byte-exact retry | **Partial / fail** | Default cleanup, root-selected shape, origin trailer, uniqueness and exact tail pass. Configured porcelain cleanup still disagrees with the fixed libgit2 prettifier; F6. |
| P1 F2: raw evidence suppresses no-op | Pass for status-visible exact `.yaml` paths | Raw filename discovery precedes qualification. Unreadable and index-A/worktree-D paths both force fresh publication. |
| P1 F2: publish Y before unlink X | Pass by code order | `write_marker(fresh)` is line 259; old-path removal is line 260. A crash with multiple visible YAMLs is re-read as multiple pending ids and causes another fresh publication before unlink. |
| P1 F2: post-all-member/pre-marker cut | **Partial / fail** | Exact simple retry reconstructs X, but fail-fresh message/selection/unavailable cases are F7 and the real writer temp is F8. |
| P2 F3: changed-message mutation tightness | Pass | The amended test changes only the message. With no new work, weakening member-message participation would retain X and fail the Y assertion. |
| P2 F4: unavailable HEAD and isolated A-empty convergence | Pass in pending-artifact fixtures; fail in post-loop branch | The two new fixtures converge to exactly one Y for pending X. F7 shows the new no-artifact branch still no-ops/reuses on unavailable evidence. |
| P3 F5: root-selected/origin/tail/index strengthening | Pass | Positive retry is root-selected and cleanup-sensitive, pins the origin trailer and byte-exact unique tail; root-only A-empty and marker-disabled unrelated index-A status are pinned. |

## Complete terminal S1.1-B named-row matrix

| Named row / frozen axis | Terminal verdict | Evidence |
|---|---|---|
| L-COA-8 cooperating-same-user accident envelope | **Fail** | F6 is ordinary repository configuration; F7 and F8 are crash/honest-retry paths. |
| Read-only P/A/R proof | Pass with pending artifact; **fail after post-loop cut** | Pending proof still forms P, A, P-minus-A and R. Without YAML, recovery substitutes current selection for the missing durable plan and erases mismatches; F7. |
| `A != empty` | Pass in `retry_marker`; **fail-to-fresh incomplete** | Empty A returns `None`, but only raw pending YAML keeps `will_mutate` true. Post-loop empty evidence returns false-success no-op; F7. |
| Every completed HEAD carries X | Pass for pending proof; **fail for partial post-loop evidence** | `retry_marker` checks every P member. Recovery skips unavailable selected HEADs and can reuse from a readable subset; F7. |
| Byte-equal request message + canonical trailers | **Fail across shipped configurations** | Default cleanup and exact tail pass; configured cleanup is F6. |
| Remaining targets exactly equal plan complement | Pass with pending artifact; **unproved post-loop** | Equality remains exact in `retry_marker`. Recovery has no original selected/committed plan and can reuse changed selection; F7. |
| Unproved sameness fails fresh | **Fail** | Raw pending YAML now fresh-splits safely; no-artifact mismatch/unavailability may no-op or partially reuse; F7. |
| Nominal member-succeeded/root-failed X reuse | Pass for default cleanup | Same id, member HEAD, artifact bytes, root-selected commit, origin and exact canonical tail are pinned. F6 is the configuration qualification. |
| Root-failure retry convergence | **Fail overall** | Pending/default cases converge; F6 and F7 remain ordinary nonconvergence/wrong-disposition cases. |
| Marker artifact retain when proof matches | Pass in pinned pending fixture | Exact X bytes remain unchanged and are committed. `AR-ARTIFACT-BYTE-TAMPER` remains accepted. |
| Marker artifact replace when proof differs | Pass for pending YAML and normal execution | Fresh YAML is written before old YAMLs are unlinked; unreadable/deleted paths converge. F7 covers absent-YAML proof failure, and F8 covers temp residue. |
| Raw/unreadable/index-only-deleted pending evidence | Pass | Each status-visible exact YAML path suppresses no-op and fresh-splits to one canonical YAML. |
| Fresh-Y publish/unlink crash order | Pass structurally, with F8 qualification | New YAML publication precedes unlink. A retry with multiple pending YAML ids fresh-splits again; sibling writer temps are not converged. |
| Multiple pending YAML convergence | Pass structurally | More than one raw id makes `retry_marker` fail fresh; write-new-before-remove-old converges the next uninterrupted attempt to one YAML. |
| Changed message, single-axis mutation kill | Pass for pending path; **fail after post-loop cut** | The amended fixture kills the round-one mutant. F7's no-YAML path false-no-ops. |
| Changed selection splits | Pass for pending path; **fail after post-loop cut** | Stored `selected_targets` rejects pending X. Recovery scans only current selection; F7. |
| Changed complement/new member work splits | Pass | Nonempty `members_to_commit` disables post-loop reuse; the isolated complement fixture remains green. |
| Post-all-members/pre-marker cut | **Fail overall** | Simple exact retry reuses X and refreshes the lock. Cross-axis fail-fresh cases are F7; stale temp publication is F8. |
| Mid-member-loop crash split | Pass as accepted residual | Remaining member work disables post-loop recovery, so the existing X/Y split fixture remains green. |
| Unavailable/custom HEAD evidence | **Fail in new recovery branch** | Pending-artifact fixture fresh-splits without backend expansion. Post-loop `None` is skipped and can no-op/reuse; F7. |
| Isolated A-empty fresh convergence | Pass with pending X; **fail without YAML** | Root-only and unavailable pending fixtures commit exactly one Y. Post-loop changed-message/unavailable A-empty no-ops; F7. |
| Marker-disabled member-only root isolation | Pass | Root HEAD is unchanged; unrelated root index entry remains `A`; root hook tripwire and lock refresh are preserved. |
| Ordinary fan-out/root-only/no-op/hook behavior | Pass except recovery false no-op | Existing ordinary tests remain green and no GitBackend/hook path changed. F7 creates a recovery-only false no-op. |
| No new durable lifecycle / WAL / namespace | Pass structurally; runtime pollution F8 | No designed file/schema/lifecycle was added. Accident recovery can nevertheless commit an existing atomic temp path. |
| No GitBackend contract change | Pass | No `src/git/**` delta and no trait method. New visibility is internal only. |
| No public API / backend-selection expansion | Pass | `MarkerIdentity` is only `pub(super)` and the three helper re-exports are only `pub(crate)`. |
| Hard 300 production changed-LOC cap | Pass | Base-to-round-2 production gross is `+267/-28 = 295/300`; test LOC are excluded. |
| Frozen checked-artifact / merge lifecycle | Pass | No changed path under either surface; boundary checker passes. |
| Cargo/protocol/generated/inventory/pin/lib root boundaries | Pass | No relevant file changed; no dependency, schema, wire, generated, census or external export move. |
| Privacy | Pass for message/URL handling; F8 qualification | Raw messages stay local and origin stays hashed. F8 publishes a PID/sequence temp pathname and duplicate intended marker bytes. |
| Visibility | Pass | No externally public item. Internal coalescer parsing is reused without widening the crate API. |
| Integration with landed core main `2214eac` | Pass, low conflict risk | Merge base is exact parent; synthetic merge exits 0 and yields tree `00c5a05a2ae480a7732f3df7d595c1821f37d976`. |

## Old S1.1 P1 terminal regrade and accepted-risk preservation

| Old row | Terminal regrade under amended envelope | Round-two disposition |
|---|---|---|
| P1-1 porcelain hooks create/amend transitions | `AR-HOOK-REWRITE` — accepted | Preserved out of envelope; ordinary hook execution remains shipped behavior. |
| P1-2 marker-disabled stale lock/root publication | **In-envelope must-pass** | Pass, including unrelated root index-A preservation and no root hook/commit. |
| P1-3 hook publishes private WAL | `AR-HOOK-PRIVATE-PUBLISH` — accepted | Preserved and structurally absent; no WAL/private namespace. |
| P1-4 foreign staged/worktree boundary evidence | `AR-HAND-BOUNDARY-SURGERY` — accepted | Preserved; no checkpoint/third-value defense added. |
| P1-5 unsafe hook publication laundering | `AR-HOOK-HISTORY-SURGERY` — accepted | Preserved; forged/re-written matching history remains excluded. |
| P1-6 ignored orphan marker | `AR-HIDDEN-NAMESPACE-SURGERY` — accepted | Preserved for intentional same-user hide/ignore surgery. Ordinary absent/unreadable evidence is still in envelope and fails in F7. |
| P1-7 crash-surviving temp/recovery wedge | **In-envelope must-pass** | **Fail:** false no-op/partial reuse after the post-loop cut is F7; tracked stale temp is F8. YAML replace order itself is cured. |
| P1-8 public/custom backend compatibility | Must preserve API; `AR-EVIDENCE-UNAVAILABLE` permits split | API preservation passes. Pending evidence splits, but post-loop unavailable evidence no-ops or partial-reuses rather than splitting/refusing; F7. |
| P1-9 assume-valid/skip-worktree | `AR-INDEX-FLAGS` — accepted | Preserved out of envelope; no flag defense added. |
| P1-10 WAL CAS/replacement | `AR-LOCAL-BYTE-RACE` — accepted | Preserved; no WAL. Candidate-owned YAML replacement order is new-before-old and passes. |
| P1-11 confinement/non-Unix | Normal publication must-pass; `AR-PATH-SURGERY` accepted | No path-confinement/non-Unix regression in the delta. Intentional parent/symlink surgery remains excluded. |

The additional round-one register remains exact and unchanged:

- `AR-SAME-TARGET-CONTENT-CHANGE`: same-user staged-content change inside an
  already-remaining target may fuse because the forbidden pre-failure
  index/tree witness is absent.
- `AR-ARTIFACT-BYTE-TAMPER`: same-user edits to non-proof fields of retained
  pending YAML may be committed.
- The documented **mid-member-loop crash split** remains accepted.

F6-F8 do not demand defenses against any of those residuals. Configured cleanup
exists before both attempts; F7 changes request axes or loses evidence without
local-state surgery; F8 is produced by the shipped atomic writer itself.

## Delta, integration, gates and cleanliness

Exact base-to-round-2 diff:

```text
2    2   src/operation/commit_log/coalesce.rs
22   0   src/operation/commit_log/mod.rs
1    0   src/operation/mod.rs
242  26  src/workspace_ops/handle_commit.rs
564  3   src/workspace_ops/tests/g13.rs
```

Production gross changed LOC is exactly 295. All paths are existing `100644`
files; there is no add/delete/rename/mode/submodule transition and no generated
change.

Exact round-one-to-round-two remediation delta:

```text
2    2   src/operation/commit_log/coalesce.rs
14   0   src/operation/commit_log/mod.rs
1    1   src/operation/mod.rs
95   38  src/workspace_ops/handle_commit.rs
227  19  src/workspace_ops/tests/g13.rs
```

The candidate and landed main remain siblings over exact base `dd31d544`.
`git merge-tree --write-tree 2214eac... 0baf281...` exits 0 with tree
`00c5a05a2ae480a7732f3df7d595c1821f37d976`. Landing would still own a
post-merge build/test, but there is no textual or structural merge blocker.

Focused/formal review evidence:

```text
git diff --check dd31d544...0baf281c
exit 0

cargo fmt --all -- --check
exit 0

python3 scripts/checks/check_checked_artifact_boundaries.py
exit 0
checked-artifact boundary: ok (15 visible entries, 5 classified modules)

cargo clippy -p gwz-core --lib -- -D warnings
exit 0

cargo test -p gwz-core --lib workspace_ops::tests::g13 -- --nocapture
exit 0
14 passed; 0 failed

cargo test -p gwz-core --lib operation::commit_log -- --nocapture
exit 0
35 passed; 0 failed

git merge-tree --write-tree 2214eac... 0baf281c...
exit 0
00c5a05a2ae480a7732f3df7d595c1821f37d976
```

The formal post-loop table above is direct path evaluation of lines 131-147
and 336-381, not a claim based on an adversarial mutation. The configured-Git
probe used a disposable repository and made no core change. The canonical
15-minute full suite was deliberately not repeated; landing owns it and the
builder's full evidence remains canonical.

The core review worktree remained exactly at `0baf281c...` and clean throughout.
No core file, ref, index or commit was changed. The report worktree was clean at
the required parent `19f1e3d...` before this report-only commit.
