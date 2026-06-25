# `gwz snapshot`

Record or list workspace snapshots.

```text
gwz snapshot [OPTIONS] [name]
```

A snapshot captures selected member revisions under a workspace-level name so
the workspace can later be materialized or pulled back to that coordinated
state.

## Arguments And Options

| Item | Meaning |
| --- | --- |
| `[name]` | Snapshot name to record. Omit to list existing snapshots. |
| `--list` | List existing snapshots instead of recording one. |

## Examples

Record a snapshot:

```sh
gwz snapshot before-refactor
```

Record all members explicitly:

```sh
gwz --all snapshot integration-baseline
```

List snapshots:

```sh
gwz snapshot --list
```

Materialize a snapshot:

```sh
gwz materialize --snapshot before-refactor
```

Pull to a snapshot:

```sh
gwz pull --snapshot integration-baseline
```

## Notes

- Use snapshots for reproducible workspace checkpoints that are not necessarily
  release tags.
- Snapshot listings render as human text by default and as a listing object with
  `--json`.
