# GWZ Log S1.1/S1.2 peer-blind round-one review

## Verdict

**NO-GO.**

Severity count:

- P0: 0
- P1: 11
- P2: 5
- P3: 1

The nominal member-succeeded/root-failed retry works, and the WAL is normally persisted before the first commit. However, the candidate still permits false fusion, destroys foreign staged evidence, launders unsafe root publications, wedges recoverable crash states, breaks marker-disabled behavior and public backends, and has unresolved path-confinement races. Any one of those P1/P2 findings is release-blocking under L-COA-8.

Review basis was limited to the permitted AGENTS files, canonical [L-COA-8](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-cli/dev-docs/GwzLogRequirements.md:234), the candidate implementation and surrounding production code, and [GwzCommitMarker.md](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/dev-docs/GwzCommitMarker.md). No prohibited plan, prior review, builder report/chat, or audit output was read.

## Findings

### P1-1 — Porcelain hooks can publish extra Git transitions and false-fuse them under X

Evidence:

- Member commit returns the repository’s final HEAD at [handle_commit.rs:1091](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1091), which is checkpointed at lines 1093–1099.
- `CommitAttempt::valid` only requires a changed commit on the same branch at [handle_commit.rs:440](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:440).
- The parent-transition check at lines 1460–1483 runs only during later retry discovery, not before first-invocation boundary/root publication. It also checks only `after^`, not the complete parent vector.
- Root-selected porcelain similarly accepts `backend.commit`’s final HEAD at [handle_commit.rs:1262](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1262) and verifies only candidate bytes afterward.
- Selected members are revalidated before root commit at line 1230, but never again after root hooks and before WAL removal at line 1285.

Reproduction:

1. In a selected member’s `post-commit` hook, disable the hook, create and stage another file, then run a second `git commit`.
2. Invoke `gwz commit` once.
3. The intended X-trailed C1 is followed by C2. The backend reports C2; the WAL, lock and marker claim C2 under X; root publication succeeds although C2 lacks X and has C1, not the durable pre-HEAD, as its parent.

The same construction on the root yields a follow-up C2 accepted as the root result. A root pre/post-commit hook can also commit in a selected member after line 1230; success then leaves live member C2 while the committed lock/marker record C1.

Remedy: use a checked commit primitive tied to the durable expected ref and HEAD and return the actual commit created by that invocation. Verify the complete allowed parent vector and attached ref immediately after each commit and again before root success. Ordinary commits require exactly the durable pre-HEAD as sole parent; if merge-state commits are supported, durably bind their exact allowed parents.

### P1-2 — `commit_marker=false` no longer refreshes the workspace lock

Evidence:

- `lock_for_boundary` starts as the old lock at [handle_commit.rs:1137](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1137).
- Post-member observation and `write_lock` occur only inside the marker-enabled branch at lines 1138–1200.
- The marker-disabled branch at lines 1219–1222 merely synchronizes the unchanged lock.
- This contradicts [GwzCommitMarker.md:378](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/dev-docs/GwzCommitMarker.md:378), especially “Existing lock/root behavior remains unchanged.”

Disposable real-`Git2Backend` reproduction:

```text
candidate cc547a:
member_before=0f503cdd4cf04150fb410e12d689f880d5a2adb1
lock_before=0f503cdd4cf04150fb410e12d689f880d5a2adb1
member_after=11741c4f709590d36b4d60dd503a0f7606675e06
lock_after=0f503cdd4cf04150fb410e12d689f880d5a2adb1
REPRO_EXIT=101
```

The equivalent baseline test updated both member and lock to the new commit and exited 0.

Remedy: retain the baseline post-member observation, lock update and root synchronization independently of marker creation. Extend the disabled-marker regression to assert lock contents, response composition, selected-root behavior, and unrelated root preservation.

### P1-3 — A root hook can publish the private WAL

A root pre-commit hook can run:

```sh
git add -f .gwz/commit/open.yaml
```

The porcelain commit accepts extra caller/hook paths. Lines 1264–1283 verify planned boundary candidates and marker namespace, but never prove that `.gwz/**` is absent from the commit. Line 1285 then deletes the worktree WAL.

The command reports success with `.gwz/commit/open.yaml` permanently present in root history and a dirty worktree deletion. A checkout can resurrect a valid-looking stale attempt.

Remedy: make the private runtime namespace a forbidden commit path. Prove it absent from parent, index and resulting commit before retirement. If a hook publishes it, retain/quarantine the attempt and use checked rollback or explicit recovery; never report success or blindly unlink it.

### P1-4 — Partial member-only boundary recovery overwrites foreign staged evidence

Evidence:

- Relaxed recovery’s `proves_owned_boundary_write` at [handle_commit.rs:148](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:148) ignores the root index.
- `boundary_worktree_is_owned` at lines 297–317 checks only planned worktree lock/manifest/marker state.
- `stage_marker_boundary` unconditionally stages candidates at [handle_commit.rs:1570](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1570) before verifying them.

Reproduction:

1. Reach an X boundary plan with `root_ready=false`, such as a handled marker-staging failure.
2. Keep X’s exact planned marker bytes in the worktree.
3. Put a different blob for `gwz.conf/markers/X.yaml` in the root index using `git update-index --cacheinfo`.
4. Retry the identical member-only request.

Relaxed ownership accepts X, restages the planned marker, and destroys the foreign index entry. This both loses user evidence and reuses X despite conflicting durable root state.

Remedy: durably bind and classify candidate-path index preimages. Recovery may accept only exact baseline absence or exact planned entries; any third value must refuse without modifying index or worktree. Cover marker, manifest, lock and integrity-sidecar candidates.

### P1-5 — Retry can launder an unsafe root publication

Evidence:

- A root hook that commits X plus a new sibling marker is detected after the first commit at lines 1276–1283, leaving X’s WAL.
- On retry, namespace comparison is rebased on the already-corrupt current HEAD.
- `retire_uncommitted_marker` at [handle_commit.rs:1775](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1775) accepts the committed state solely because HEAD contains X’s expected marker bytes. It does not verify X’s immutable root parent, every boundary candidate, or absence of sibling markers.

The retry therefore retires X, creates Y, succeeds, and leaves the hook-created sibling marker permanently committed.

Remedy: before retiring an already-committed X, verify the entire publication from X’s durable `root_before`: exact parent/ref, message, all planned candidates, and no unowned namespace additions. Unsafe publication must remain a refusal or undergo checked rollback. Extend the existing hook-extra-marker test through the second invocation.

### P1-6 — Ignored orphan markers bypass both preflight and postcondition checks

Evidence:

- `refuse_unowned_new_marker_changes` enumerates only normal backend status at [handle_commit.rs:1626](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1626).
- Normal `Git2Backend::status` uses default options at [repository.rs:155](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/git/gitbackend/repository.rs:155), whose `include_ignored` is false at [types.rs:347](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/git/gitbackend/types.rs:347).
- The final namespace postcheck is additionally gated to root-selected operations at line 1276.

Observed disposable reproduction:

1. Create a foreign `gwz.conf/markers/<foreign>.yaml` absent from root HEAD.
2. Add an exact ignore entry in `.git/info/exclude`.
3. Stage a selected member change and perform a marker-enabled member-only commit.

```text
result_is_ok=true
member_advanced=true
root_advanced=true
foreign_preserved=true
exit=0
```

This contradicts [GwzCommitMarker.md:92](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/dev-docs/GwzCommitMarker.md:92) and lines 354–356. A member hook can also create an orphan after preflight because member-only publication has no postcheck; a root hook can create an ignored unstaged sibling that the current final scanner still misses.

Remedy: physically enumerate the real marker directory, no-follow and without UTF-8 loss, and union that with raw index/status evidence before mutation and after every publication. Ignore rules must not hide reserved-state evidence.

### P1-7 — Crash-surviving atomic-write temps permanently wedge exact recovery

The artifact writer creates sibling temps named `<file>.<pid>.<seq>.tmp` at [artifact/mod.rs:473](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/artifact/mod.rs:473) and [artifact/mod.rs:542](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/artifact/mod.rs:542).

A crash after temp fsync but before rename while writing X’s marker or `conf-integrity.yml` leaves a temp under `gwz.conf/markers/`. On restart, the scanner at lines 1620–1649 classifies it as unowned before WAL recovery and refuses every retry. The valid X WAL cannot converge until a user manually deletes GWZ’s own temp.

Remedy: stage temps outside the reserved marker namespace, or durably journal the exact temp name and bytes and recover/retire it before namespace refusal. Add crash injection immediately after temp fsync for marker and integrity-sidecar writes.

### P1-8 — Public/custom `GitBackend` implementations now fail at runtime

Marker-enabled commit now unconditionally depends on optional trait methods whose public defaults return `UnsupportedOperation`, including:

- `read_file_at_commit` at [contract.rs:163](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/git/gitbackend/contract.rs:163)
- `preservation_image` at line 464
- `index_entries_match_candidate_files` at line 555
- `commit_gwz_paths_checked` at line 572
- `changed_paths_between` at line 752

Calls occur during checkpoints, staging verification, scoped publication and postchecks. Existing custom implementations still compile, but default marker-enabled behavior now fails. A partially capable implementation can fail only after a member has committed.

Remedy: introduce an explicit commit-evidence capability/version and prove it nonmutatingly before WAL creation or any Git mutation, or make these methods required/sealed for commit-capable backends. Preserve the marker-disabled legacy path and add contract tests using a backend that inherits every optional default.

### P1-9 — Valid special index states are rejected by default marker commits

Every marker-enabled checkpoint calls `preservation_image` at [handle_commit.rs:553](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:553). Its Git2 implementation rejects any assume-valid, skip-worktree, intent-to-add or extended flag at [preservation_image.rs:575](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/git/gitbackend/preservation_image.rs:575).

Consequences include rejecting valid sparse checkout/index states and unrelated flagged root entries even for member-only requests. Marker-disabled commits do not take this new path, so this is a compatibility regression caused by the default-enabled marker lifecycle.

Minimal reproductions use `git update-index --assume-unchanged`, sparse checkout/skip-worktree, or `git add -N` on a valid unrelated entry, then invoke marker-enabled commit.

Remedy: define a commit-specific checkpoint format that durably hashes and preserves semantic flags and sparse-index state instead of treating them as invalid. Test root/member, `all=false`/`all=true`, assume-valid, skip-worktree, intent-to-add and sparse checkout.

### P1-10 — WAL phase updates and retirement are not compare-and-swap

`write_attempt` blindly atomic-replaces `open.yaml` at [handle_commit.rs:605](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:605); `remove_attempt` blindly unlinks it at line 641.

A member post-commit hook can replace/corrupt the current WAL, after which line 1099 silently overwrites the tamper with the next phase. A root hook can replace it and line 1285 deletes the foreign bytes while returning success.

Remedy: every phase transition must verify and consume the exact prior serialized bytes/state hash using a no-follow, handle-relative CAS protocol. Retirement must accept only the exact final WAL version expected by the publisher. Tamper or replacement must remain untouched and fail closed.

### P1-11 — Path checks remain check-then-use and atomic temps follow symlinks

WAL confinement at [handle_commit.rs:658](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:658) and marker confinement at [handle_commit.rs:1862](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1862) use `symlink_metadata`, release that evidence, then perform pathname reads/writes/removes later.

`stage_durably` uses `File::create` at [artifact/mod.rs:482](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/artifact/mod.rs:482), which follows a precreated temp symlink. Predictable PID/sequence temp names permit an attacker or concurrent writer to redirect/truncate an outside file. Directory swaps between validation and rename/unlink create the same confinement gap.

Remedy: pin real directory handles and use handle-relative operations, `O_NOFOLLOW|O_EXCL` or platform equivalents, then rename/unlink/fsync relative to those handles. Static symlink tests are insufficient; add deterministic parent-swap and temp-symlink race tests.

### P2-1 — Exact scoped root publication is unnecessarily split after a crash

For a member-only operation, `commit_gwz_paths_checked` advances the root ref at [handle_commit.rs:1252](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1252). A crash before `remove_attempt` at line 1285 leaves a RootReady X WAL and an exactly published X root commit.

Retry sees a changed root checkpoint, declines X, accepts X’s committed marker during retirement, and mints Y. Yet the backend already exposes `verify_gwz_paths_commit` at [contract.rs:583](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/git/gitbackend/contract.rs:583), which can prove the exact commit from X’s durable parent, candidates and message.

This conflicts with L-COA-8: sameness is provable from durable state, so the operation should retain X. The document’s declared split at [GwzCommitMarker.md:87](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/dev-docs/GwzCommitMarker.md:87) makes the implementation/document mutually consistent but not requirement-compliant.

Remedy: on RootReady scoped recovery, verify current HEAD as the exact X publication, finalize/remove X’s WAL, and report X success. Split only when that verification fails.

### P2-2 — Valid NotEnrolled root-only workspaces enter a permanent RootReady wedge

Missing `conf-integrity.yml` is an accepted NotEnrolled state at [conf_gate.rs:94](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/workspace_bootstrap/conf_gate.rs:94).

For a root-only selected commit:

- `boundary_needs_lock=false`, so the sidecar is not written/staged.
- RootReady is persisted.
- Replay forces `include_lock` because `attempt.selects_root()` at [handle_commit.rs:1540](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1540).
- `marker_boundary_candidates` then unconditionally reads the missing sidecar at lines 1591–1604.

Observed disposable test: two consecutive invocations both returned `IoError`; the WAL remained `root_ready: true`. The asserting regression exited 0.

Remedy: durably represent and verify sidecar absence, or create and journal the deterministic sidecar before RootReady. Add a grandfathered NotEnrolled root-only regression.

### P2-3 — WAL checksum does not bind unknown YAML fields

`CommitAttempt` and nested durable structs at [handle_commit.rs:28](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:28) do not use `deny_unknown_fields`.

`read_attempt` parses YAML at lines 591–602, dropping unknown keys. The checksum is recalculated from the canonical recognized struct at lines 515–524. Appending an unknown top-level or nested key therefore remains valid under the original checksum and is silently erased by a later write.

Remedy: apply `#[serde(deny_unknown_fields)]` to every durable WAL struct or require byte-exact canonical encoding at read time. Add top-level and nested unknown-field tamper regressions.

### P2-4 — Backend and I/O errors are flattened after publication

At [handle_commit.rs:1749](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1749), every error from the final namespace scan is replaced with `GitCommandFailed`, preserving only the message. Lines 1668–1675 likewise map filesystem failures to `GitCommandFailed`.

A custom backend’s `UnsupportedOperation`, or a typed `IoError`, can therefore change code and lose structured details after an irreversible root commit.

Remedy: propagate original `ModelError` codes/details unchanged unless adding contextual fields without replacing them. Add fault tests for UnsupportedOperation, permission-denied, read failure and directory-enumeration failure at each pre/post-publication boundary.

### P2-5 — Safety-critical lifecycle is too monolithic to isolate or audit

The exact candidate delta is 3,541 insertions and 132 deletions:

- `GwzCommitMarker.md`: +201/−38
- `handle_commit.rs`: +1,724/−87, growing from 387 to 2,024 lines
- `g13.rs`: +1,616/−7, growing from 262 to 1,871 lines

WAL serialization, validation, phase transitions, retry classification, boundary planning, filesystem confinement, hook postconditions, namespace recovery and marker retirement are all embedded in one handler file. The confirmed branch-specific regressions—marker-disabled stale lock, partial-boundary index loss, post-publication laundering and NotEnrolled replay—are direct evidence that lifecycle invariants are not localized.

Remedy: extract a typed state-machine/WAL module with explicit phase transitions and CAS ownership; separate filesystem capability handling and checked publication; split the fault matrix into phase-specific tests before another safety review.

### P3-1 — The regression matrix omits critical lifecycle axes

The G13 fixture is one-member-only at [g13.rs:47](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/tests/g13.rs:47). Missing coverage includes:

- two-member durable prefix and remaining-suffix replay;
- ambiguous cut after a later member commit;
- exact root-ref publication before WAL removal;
- retry after hook-extra-marker failure;
- custom backend default methods;
- special index flags and sparse checkout;
- marker-disabled lock contents;
- ignored orphan markers;
- atomic marker temp crash;
- NotEnrolled root-only replay;
- foreign index entries during pre-RootReady recovery.

## L-COA-8 coverage

| Requirement axis | Verdict | Evidence |
|---|---|---|
| Durable identity/intent before first irreversible commit | Partial pass | New WAL is written and read-verified at lines 1033–1051 before member commit. CAS and no-follow defects weaken durability under concurrent replacement. |
| Exact same-ID retry from durable evidence | Fail | Nominal checkpoint replay works, but hook-created transitions are accepted, partial recovery ignores the index, and exact scoped root publication is split despite available proof. |
| Unproved sameness/new eligible work splits to Y | Partial pass | Ordinary checkpoint mismatch mints Y, but retirement can launder an unsafe committed X and report later success. |
| Member succeeded/root failed converges | Partial pass | The focused single-member handled-failure regression passes. Atomic temp crash, ignored namespace state, NotEnrolled state and foreign index evidence prevent safe convergence. |
| Distinct operations never reuse an ID | Fail | Member/root post-commit hooks can add C2 after intended C1 and have C2 recorded/claimed under X. |
| Missing/corrupt/tampered/torn WAL and marker state fail away from fusion | Fail | Recognized checksum corruption refuses, but unknown fields, non-CAS replacement, ignored orphans, unsafe retirement and temp crash cuts violate the broader property. |
| Marker retirement exact | Fail | Committed-marker retirement checks only X marker bytes, not the complete root publication or sibling namespace. |
| Symlink/path confinement | Fail | Checks are TOCTOU-prone; predictable temps use symlink-following `File::create`. |
| Selection/root/index/worktree preservation | Fail | Normal target resolution remains canonical, but partial recovery destroys foreign staged entries; hooks can alter selected members/private WAL; special index flags are rejected. |
| `all=false`/`all=true`, hooks and boundary witnesses | Partial | The ordinary porcelain flag is still passed through, but hook and boundary postconditions are incomplete. |
| Marker-disabled/no-op compatibility | Fail/Pass | No-op remains compatible in focused tests. Marker-disabled member commit leaves a stale lock. |

Canonical serialization of the recognized typed fields is deterministic, and target construction remains root-first followed by resolved member order at [handle_commit.rs:1979](/Users/owebeeone/limbo/gwz-log-worktrees/s1.1/gwz-core/src/workspace_ops/handle_commit.rs:1979). Those narrow properties do not repair the unknown-field or ownership failures.

## State and crash matrix

| Durable/live state at interruption or retry | Candidate behavior | Verdict |
|---|---|---|
| No WAL, no eligible work | Success/no-op, no marker | Pass |
| New eligible operation | Write/verify X WAL before first commit | Nominal pass |
| Exact completed member checkpoint, root not committed | Resume X and remaining suffix | Nominal pass; multi-member case untested |
| Member ref advanced before durable checkpoint | Split to Y and refresh composition | Acceptable false split |
| Boundary plan recorded, no root write | Resume X | Pass nominally |
| Member-only partial boundary write | Resume X from worktree plan | Fail: root index is ignored and foreign staged evidence is overwritten |
| Root-selected partial boundary | Split to Y | Matches documented porcelain policy |
| RootReady boundary exact | Resume X | Pass nominally |
| Crash during marker/sidecar temp write | Retry rejects GWZ-owned temp forever | Fail |
| Scoped root ref published, WAL not removed | Split to Y although X is exactly provable | Fail |
| Root hook publishes X plus sibling marker | First call fails; retry launders publication into Y | Fail |
| WAL syntactically/checksum corrupt | Refusal | Pass for recognized fields |
| WAL contains unknown YAML field | Accepted under old checksum | Fail |
| WAL replaced between phases | Silently overwritten/deleted | Fail |
| Missing WAL plus visible orphan marker | Refusal | Pass |
| Missing WAL plus ignored orphan marker | Operation succeeds and preserves orphan | Fail |
| RootReady marker/index foreign state | Generally refuses | Pass |
| Successful publication | WAL removed | Nominal pass; removal is not CAS |
| Marker disabled after member commit | Member advances, lock remains old | Fail |

Filesystem/Git mutations traced were: mutation guard acquisition; optional integrity-sidecar repair; exclude mutation; WAL directory/file creation and phase replacement; member porcelain commits/hooks; post-member lock/sidecar writes; marker write; exact candidate staging; RootReady WAL update; scoped or porcelain root ref publication/hooks; tree/index/worktree/namespace postchecks; stale-marker retirement; and WAL unlink/fsync.

## Compatibility, scope and document verdicts

| Axis | Verdict |
|---|---|
| Exact file scope | Pass: only the three declared files changed |
| Forbidden `checked_artifact`, `merge/v1_lifecycle`, source/inventory/pin/protocol/`gwz.conf` paths | Pass: untouched |
| MarkerArtifact schema includes `merge` | Pass: ordinary marker sets `merge: None` at line 1176 and document schema is additive |
| Root association trailer-first and old marker-file sentence superseded | Pass: document lines 242–248 |
| Landed status/evidence honesty | Generally pass |
| WAL lifecycle description matches implementation | Pass for the intended lifecycle, including its declared post-publication split; that split fails canonical L-COA-8 where scoped proof exists |
| Marker-disabled compatibility statement | Fail |
| Missing-journal/orphan statement | Fail under ignore rules |
| Exact boundary/index preservation statement | Fail during relaxed partial recovery |
| Hook-created sibling refusal statement | Fail on retry laundering and ignored unstaged siblings |
| Complete-state checksum statement | Fail for unknown fields |
| Public/custom backend compatibility | Fail |
| Sparse/special-index compatibility | Fail |
| Architecture/reviewability | Fail |

## Commands and direct exits

Candidate worktree remained clean at exact HEAD `cc547aff7cfd6c60579fa4186d9f72e9457be8be`.

```text
git rev-parse HEAD
exit 0
cc547aff7cfd6c60579fa4186d9f72e9457be8be

git status --short
exit 0
<empty>

git diff --check 2a3297da16a5d3cd814619cb2b3d7d15223640a7..cc547aff7cfd6c60579fa4186d9f72e9457be8be
exit 0

git diff --name-status 2a3297da16a5d3cd814619cb2b3d7d15223640a7..cc547aff7cfd6c60579fa4186d9f72e9457be8be
exit 0
M dev-docs/GwzCommitMarker.md
M src/workspace_ops/handle_commit.rs
M src/workspace_ops/tests/g13.rs

cargo fmt --check
exit 0

cargo clippy --all-targets --all-features -- -D warnings
exit 0

CARGO_TARGET_DIR=/tmp/gwz-s11-review-target-cc547aff \
  cargo test workspace_ops::tests::g13 -- --nocapture
exit 0
9 passed; 0 failed; 1680 filtered out
```

The all-features suite was started in the isolated target and showed no failure before the parent-requested consolidation deadline. It was interrupted after approximately ten minutes, exited 130, and is **not credited as a passing gate**.

Executed disposable real-repository regressions:

- Marker-disabled stale-lock test: candidate exit 101; equivalent baseline test exit 0.
- Ignored foreign marker reproduction: exit 0 while asserting the observed unsafe success and both HEAD advances.
- NotEnrolled root-only wedge reproduction: exit 0 while asserting two consecutive `IoError` results and retained RootReady WAL.

No candidate file, report or source was edited; nothing was committed, pushed, or sent to the builder.
