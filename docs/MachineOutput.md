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
  "detail": null
}
```

Top-level CLI errors in `--json` or `--jsonl` mode keep the same response shape,
with `meta: null`, no members, and one error entry.

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
      "name": "v0.3.0",
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
