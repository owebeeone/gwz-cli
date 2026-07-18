# `gwz merge`

Merge one source ref into the current branch of each selected member.

```text
gwz merge <source> [--dry-run]
```

The source name is resolved independently in every selected repository. With
no selection, all active members participate and the workspace root does not.
Use the normal global `--target` and `--no-target` options to select members.

Merge across all active members, or inspect the plan without mutation:

```sh
gwz merge feature/refactor
gwz merge feature/refactor --dry-run
```

Select members and request JSON Lines output:

```sh
gwz merge feature/refactor --target mem_app --target mem_docs --jsonl
```

## Current behavior

- Starting a merge and using `--dry-run` are the only released forms. Continue,
  coordinated abort, status, preservation, and strategy flags are not yet
  available.
- Explicit `--target @root`, `--partial`, `--force`, and reserved forms return
  typed core errors; they are never silently weakened or ignored.
- A conflict remains in that member's ordinary Git merge state. Resolve or
  abort it with Git commands in the member repository.
- Other members may already have changed. GWZ does not yet provide coordinated
  continue or rollback, and the workspace lock reflects clean member outcomes.
- If an unexpected failure halts the batch, the lock still records earlier
  outcomes that GWZ verified clean; the failed member is reported and later
  members remain unattempted.
- True merge commits use the message
  `Merge <source> into <target-branch>` without quoting or GWZ operation
  trailers.
- Source and target must share history. GWZ rejects unrelated histories for
  both this command and `pull --sync merge`; it does not implicitly enable
  Git's `--allow-unrelated-histories` behavior.
- Human, JSON, and JSONL results identify the action as `merge` and include
  every participant's source, target branch, outcome, and conflict paths.

The generated command reference shows global options on every command. Merge
rejects unrelated operation policies supplied explicitly: `--sync`,
`--remote`, `--jobs`, `--max-per-host`, and `--progress-interval`. It also
rejects the reserved `--partial` and `--force` policies. Core diagnostics name
the option that must be removed.

`gwz branch --merge <source>` remains as a deprecated compatibility spelling.
It constructs the same first-class merge request and does not invoke the old
branch-merge protocol operation.
