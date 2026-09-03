# Machine Output

GWZ has three script-oriented output modes:

- `--json` renders one structured JSON response.
- `--jsonl` renders newline-delimited JSON records for streaming operation
  consumers.
- `gwz status --porcelain` renders stable path-oriented status text.

`--json` and `--jsonl` are mutually exclusive. `status --porcelain` cannot be
combined with either machine output flag.

## JSON Response

Most commands render a response object:

```json
{
  "kind": "response",
  "meta": {
    "request_id": "req-...",
    "schema_version": "gwz.protocol/v0",
    "action": "Status",
    "aggregate_status": "Ok",
    "operation_id": "op-...",
    "message": null
  },
  "members": [],
  "errors": [],
  "workspace_git_status": null
}
```

Member entries include:

```json
{
  "member_id": "gwz-cli",
  "member_path": "gwz-cli",
  "source_kind": "Git",
  "status": "Ok",
  "error": null,
  "planned": null,
  "state": null,
  "git_status": null,
  "lock_match": null
}
```

Errors use:

```json
{
  "code": "MemberNotFound",
  "message": "unknown member",
  "member_id": null,
  "member_path": null,
  "target_kind": null,
  "detail": null,
  "record_context": null
}
```

Top-level CLI errors in `--json` or `--jsonl` mode keep the same response shape,
with `meta: null`, no members, and one error entry. Per-member failures retain
`member_id`, `member_path`, and `target_kind: "Member"` even when preflight
rejects the whole operation before a normal response exists.

## Commit Log Output

`gwz log` uses a dedicated finite record stream rather than the generic
operation-response envelope. JSON wraps the records with schema
`gwz.log/v0`:

```json
{
  "records": [
    {
      "author": {
        "email": "ada@example.test",
        "name": "Ada",
        "time": { "offset_min": 600, "time": 1788134400 }
      },
      "committer": {
        "email": "ada@example.test",
        "name": "Ada",
        "time": { "offset_min": 600, "time": 1788134400 }
      },
      "members": [
        {
          "hash": "0123456789abcdef0123456789abcdef01234567",
          "member_id": "mem_api",
          "member_path": "services/api",
          "parents": ["89abcdef0123456789abcdef0123456789abcdef"]
        }
      ],
      "provenance": "marker:<uuid-v7>",
      "record": "entry",
      "subject": "Update the shared API"
    },
    {
      "member_id": "mem_web",
      "member_path": "services/web",
      "message": "revision was not found",
      "operand": "topic..HEAD",
      "reason": "revision_unresolved",
      "record": "degradation"
    }
  ],
  "schema": "gwz.log/v0"
}
```

The literal provenance vocabulary is:

- `none` — one uncoalesced commit;
- `heuristic` — compatible unmarked commits coalesced by the conservative
  message, author, and time rule;
- `marker:<uuid-v7>` — commits coalesced by a valid coordinated-commit marker;
- `marker-invalid` — a marker-like claim was present but invalid or malformed;
  the entry is a singleton and never joins a heuristic group.

`body` is present only when `--body` is requested. `lossy` is present only as
`true`, when invalid Git bytes required U+FFFD replacement. Member ordering and
parent hashes are stable and complete.

Degradation `reason` is one of `repository_unreadable`, `repository_missing`,
`unborn`, `revision_unresolved`, `snapshot_entry_missing`,
`lock_entry_missing`, or `unsupported_source_kind`. The optional `operand` and
`message` fields may be null.

JSONL begins with a schema header and then emits the same entry and degradation
objects one per line:

```json
{"record":"header","schema":"gwz.log/v0"}
{"provenance":"heuristic","record":"entry","subject":"Update API and web"}
```

The second line above is abbreviated for readability; real entry records retain
the complete author, committer, member, and timestamp fields shown in the JSON
example. See [`gwz log`](commands/log.md) for limits, ranges, filters, and exit
status behavior.

Durable merge-record compatibility failures populate `record_context` instead
of requiring message parsing:

```json
{
  "merge_id": "merge_example",
  "schema": "gwz.merge-operation/v1",
  "record_schema_version": 1,
  "required_wave": "A1",
  "legacy_mode": null
}
```

`required_wave` is null for a genuinely unknown schema/version pair. It is
`A1`, `A2`, `A3`, or `A4` for an allocated pair that requires a newer semantic
wave. When the envelope itself is malformed and its header cannot be read,
`record_context` is null.

## Merge JSON

Merge responses use the normal response envelope and populate its `merge`
field. JSON and JSONL expose the complete merge protocol shape, including the
current finalization step:

```json
{
  "merge": {
    "merge_id": "merge_example",
    "state": "Finalizing",
    "open": true,
    "participant_counts": {
      "total": 1,
      "planned": 0,
      "up_to_date": 0,
      "fast_forwarded": 1,
      "merged": 0,
      "conflicted": 0,
      "failed": 0,
      "unattempted": 0,
      "continued": 0,
      "aborted": 0,
      "rolled_back": 0
    },
    "repos": [],
    "operation_drift": [],
    "preservation": null,
    "publication_step": "PublishingCandidate",
    "record": {
      "source_version": "V1",
      "archived": false,
      "terminal_outcome": null,
      "acceptance": {
        "kind": "SupportedPersisted",
        "supported_persisted": {
          "kind": "V1",
          "v1": {
            "operation_baseline_lock_sha256": "sha256...",
            "metadata_base": {
              "source": "OperationBaseline",
              "source_commit": null,
              "manifest_yaml": "...",
              "manifest_sha256": "sha256...",
              "lock_yaml": "...",
              "lock_sha256": "sha256..."
            },
            "lock_yaml": "...",
            "lock_sha256": "sha256...",
            "members": [],
            "root": {
              "kind": "BornAttached",
              "commit": "abc123",
              "symbolic_branch": "main",
              "publication_branch": "main",
              "lock_worktree_sha256": "sha256...",
              "manifest_worktree_sha256": "sha256...",
              "lock_commit_sha256": null,
              "manifest_commit_sha256": null
            }
          }
        },
        "legacy_complete": null,
        "legacy_source": null,
        "legacy_evidence": null,
        "missing_gaps": []
      },
      "recovery": null
    }
  }
}
```

`record` is present on every successful response tied to a durable merge
record: start after creation, open or archived status, continue,
preserve-abort, abort, and id-qualified GC. It is null for dry-run, idle
status, pre-record responses, and unqualified GC. A command failure uses the
top-level error envelope and therefore carries `record_context`, not a partial
`record` projection.

`source_version` is `V0` or `V1`. `archived` is true only when status was read
from immutable archive bytes; only an archived projection has
`terminal_outcome` (`Completed` or `Aborted`). Open v0 records deliberately
have null `acceptance` and `recovery` when their legacy evidence has not been
migrated.

Archive-only status does not inspect repositories. Its `repos` and
`operation_drift` arrays are empty, and the immutable terminal and acceptance
history is reported through `record`.

Acceptance has one of four exclusive shapes:

- `SupportedPersisted` uses `supported_persisted.kind: "V1"` and its complete
  `v1` payload.
- `LegacyComplete` uses `legacy_complete` and `legacy_source` (`Candidate` or
  `BaselineNoPublication`).
- `LegacyUnavailable` uses `legacy_evidence` plus a sorted, nonempty
  `missing_gaps` list. Gap values are `ExactLockBytes`,
  `CompleteMemberAudit`, `AcceptedRootInput`, and `PublicationEvidence`.
- `NotAccepted` has no payload or gaps and applies only to an aborted archive
  that never accepted a workspace.

Accepted member rows use kind `Selected`, `UnselectedPresent`, or `Absent`.
A selected row contains `integration`, `final_checkout`, and `lock_member`; an
unselected-present row contains only `lock_member`; an absent row contains no
payload. Root kind is `BornAttached`, `BornDetached`, or `UnbornAttached`.
Optional fields remain explicit JSON nulls and repeated fields remain arrays,
including when empty.

When a durable record is in `RecoveryRequired`, `record.recovery` reports its
literal `origin_state`, record-derived `base_phase`, the current
`next_action: "ReportRecoveryRequired"`, and the action a resume would take in
`resume_action`. This projection is diagnostic: status does not reconcile or
rewrite the record.

Repository rows include their target, source, branch, before/resulting/live
commits, lifecycle state, prediction, conflicts, eligibility flags, structured
participant drift, an optional structured error, and an optional
`pending_action`. A pending action contains its `kind`, reconciliation `state`
(`NotStarted`, `ExpectedConflict`, `CompletedExactly`, or `Ambiguous`), and a
guidance message. Merge errors use the same seven-field shape as envelope
errors, including `target_kind` and `record_context`. Operation drift entries
contain `kind` and `message`.
Preservation entries contain `target_id`, `path`, `backup_ref`,
`backup_commit`, `stash_id`, and `stash_object_id`.

Preservation remains null until that later feature is available. Publication
steps are populated while finalization is open and end at `Complete`. GWZ is
pre-1.0, so strict consumers must tolerate additive keys while continuing to
validate the keys they understand.

Participant drift distinguishes advanced, rewound, and diverged heads, missing
recorded objects or repositories, exact native-merge mismatches, and foreign
integration/sequencer state. Status carries member context and expected/live
evidence for these cases rather than returning a memberless backend error.
An ambiguous pending action also emits the dedicated
`PendingActionAmbiguous` drift kind and blocks both continue and abort until a
fresh exact classification succeeds.

`MergeOperationState` includes the append-only `Idle` value used by the
read-only merge-status lifecycle when no coordinated merge is open. An idle
response has no merge id, participants, or drift and does not fabricate a
completed operation. Its `record` field is null.

### `crash_recovery`

`merge.crash_recovery` reports the start-time crash-recovery decision — whether
the workspace volume can prove the durable filesystem identity GWZ needs to
reconstruct an interrupted start. It is the machine truth for that decision;
consumers never need to parse the human warning off stderr.

```json
{
  "merge": {
    "crash_recovery": {
      "supported": false,
      "filesystem": "btrfs",
      "gap": "NoDurableIdentity"
    }
  }
}
```

- `supported` — `true` when the volume proved identity and the merge records
  its artifacts as before; `false` when the merge proceeds without that
  recording. A `false` decision is not a failure: the response is still a
  success, and the merge ran.
- `filesystem` — the name of the filesystem behind the decision, or absent when
  it cannot be named.
- `gap` — why identity could not be proved, and absent when `supported` is
  true. One of `NoDurableIdentity`, `RemoteFilesystem`, `VolatileFilesystem`.

The whole object is **absent** on any response that made no such decision:
`--abort`, `--status`, `--gc`, dry run, and every start that writes a v0 record
(ordinary and `--ff-only` merges). Present means a decision was made, not that
crash recovery is available; read `supported` for that.

The Rust and Python driver tests compare semantic JSON values with the single
canonical fixture at
`gwz-core/protocol/fixtures/cli_parity/merge_response.json`. Driver development
checkouts therefore retain the usual sibling `gwz-core` layout; both drivers
already require that checkout through their development path dependency. The
fixture is test-only and is not read by an installed driver at runtime.
It includes both an envelope error and an error-bearing failed repository row,
so cross-driver parity covers the complete structured error sub-shape.

## JSONL Stream

`--jsonl` streams event records as an operation runs, then the final render path
prints the response object. Event records have this shape:

```json
{
  "kind": "event",
  "operation_id": "op-...",
  "request_id": "req-...",
  "sequence": 1,
  "timestamp_ms": 0,
  "event_kind": "MemberProgress",
  "severity": "Info",
  "member_id": "gwz-cli",
  "member_path": "gwz-cli",
  "message": null,
  "member": null,
  "error": null,
  "attribution": null,
  "target_kind": "Member",
  "merge_state": null,
  "merge_member": null,
  "artifact_path": null,
  "progress": {
    "phase": "Receiving",
    "received_objects": 10,
    "total_objects": 20,
    "received_bytes": 1024,
    "indexed_deltas": null,
    "total_deltas": null
  }
}
```

Merge JSONL uses the same event envelope. Each invocation emits operation
start/finish events. Actionable participants emit member start/finish events;
`MemberFinished` carries the durable merge participant outcome in
`merge_member`. Verified operation-record and evidence writes emit
`ArtifactWritten` with `artifact_path`. Lifecycle transitions carry
`merge_state`. Participant outcome and state-change events are emitted only
after their corresponding durable write succeeds. A merge start on a volume
that cannot support crash recovery also emits one `Diagnostic` event with
`severity: "Warn"`, no member, and the warning text in `message`; the same
decision is in the response's `crash_recovery` object, which is the field to
read rather than the event.

After successful finalization verification, the stream reports the composition
evidence in this order:

1. `git:@root/<commit>` for the checked root evidence commit;
2. `gwz.conf/markers/<id>.yaml` for the merge marker;
3. `gwz.conf/gwz.lock.yml` for the accepted lock; and
4. `.git/info/exclude` for the local workspace boundary.

These events describe verified publication. Recovery may report them again
when it re-verifies a publication whose prior process stopped before terminal
completion.

Both drivers emit merge events as they occur rather than buffering them until
the operation finishes. After `OperationFinished`, the stream contains exactly
one final `kind: "response"` object. A failed invocation retains any events
already emitted and ends with one structured error response. Event-stream
completion is not published until that final response, or its structured
failure, is available to the driver.

The Rust and Python event serializers compare against the shared
`gwz-core/protocol/fixtures/cli_parity/merge_event.json` fixture. This pins the
merge-member outcome and artifact fields to the same JSONL shape in both
drivers.

Progress event frequency is controlled by:

```sh
gwz --progress-interval 250 --jsonl pull --head
```

Use `--progress-interval 0` to emit every update.

## Listings

Read-only listing commands render listing objects with `--json` or `--jsonl`.

`gwz --json ls`:

```json
{
  "kind": "members",
  "entries": [
    {
      "id": "gwz-cli",
      "path": "gwz-cli",
      "abspath": "/work/gwz-dev/gwz-cli",
      "materialized": true
    }
  ]
}
```

`gwz --json tag --list`:

```json
{
  "kind": "tags",
  "entries": [
    {
      "name": "v0.9.0",
      "members": 3
    }
  ]
}
```

`gwz --json snapshot --list`:

```json
{
  "kind": "snapshots",
  "entries": [
    {
      "name": "before-refactor",
      "created_at": "2026-06-25T00:00:00Z",
      "created_by": "user",
      "members": 3
    }
  ]
}
```

## Status JSON

`gwz --json status` includes `workspace_git_status`:

```json
{
  "clean": false,
  "root_status": {
    "branch": "main",
    "detached": false,
    "head": "abc123",
    "staged": 0,
    "unstaged": 1,
    "untracked": 0,
    "dirty": true,
    "unborn": false
  },
  "root_file_changes": [],
  "file_changes": [],
  "branches": [],
  "branch_groups": [],
  "branch_differences": []
}
```

File change entries use `repo_path`, `workspace_path`, `index_status`,
`worktree_status`, and `original_repo_path`.

## Status Porcelain

Use porcelain for stable path-oriented status text:

```sh
gwz status --porcelain
```

Output is similar to Git status porcelain, with workspace-relative paths:

```text
 M gwz-cli/docs/README.md
?? gwz-cli/docs/commands/init.md
```

If no file changes are available but members have non-OK status, porcelain
prints `!! <member-path>` lines.

## Forall

`gwz forall` rejects `--json` and `--jsonl`. It inherits child process stdio and
streams child output directly, so GWZ does not wrap that output in machine
records.

For machine-readable member selection, combine `gwz --json ls` with external
tooling rather than `forall --json`.

## Exit Codes

GWZ maps aggregate status to process exit codes:

| Aggregate status | Exit code |
| --- | --- |
| `Accepted`, `Ok`, `Noop`, `Dirty` | `0` |
| `Partial`, `Failed`, `Conflicted` | `1` |
| `Rejected` | `2` |

Argument parsing and top-level CLI construction errors also exit non-zero.
