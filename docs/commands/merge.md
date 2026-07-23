# `gwz merge`

Merge one source ref into the current branch of each selected member.

```text
gwz merge <source> [--dry-run]
gwz merge --status
gwz merge --continue
gwz merge --abort
```

The source name is resolved independently in every selected repository. With
no selection, all active members participate and the workspace root does not.
Use the normal global `--target` and `--no-target` options to select members.

Merge across all active members, or inspect the plan without mutation:

```sh
gwz merge feature/refactor
gwz merge feature/refactor --dry-run
gwz merge --status
```

Select members and request JSON Lines output:

```sh
gwz merge feature/refactor --target mem_app --target mem_docs --jsonl
```

## Current behavior

- The command exposes start, `--dry-run`, read-only `--status`, coordinated
  `--continue`, and safe coordinated `--abort`.
- Explicit `--target @root`, `--partial`, `--force`, and reserved forms return
  typed core errors; they are never silently weakened or ignored.
- A conflict remains in that member's ordinary Git merge state. Resolve it in
  the member repository, stage the resolution with `gwz add`, and run
  `gwz merge --continue`. Use `gwz merge --abort` to roll back the entire
  coordinated operation. Do not run raw `git merge --abort` as a substitute.
- `gwz merge --status` is strictly read-only. It reports the publication step,
  recorded and live participant commits, conflicts, drift, and whether each
  participant is eligible for continue or abort.
- Abort preflights every participant before changing any of them. It rejects
  the whole abort if post-merge work makes any rollback unsafe. If finalization
  already created root composition evidence, abort also verifies and rolls
  back that evidence.
- Other members may already have changed, but the accepted workspace lock
  remains the exact pre-merge baseline while the coordinated operation is
  open. Status compares every recorded result with live Git state.
- If an unexpected failure halts the batch, the durable operation record keeps
  earlier exact outcomes, identifies the failed member, and marks later members
  unattempted. The accepted lock is not partially advanced.
- Before each participant Git mutation, GWZ durably records the exact pending
  action. After interruption, status reports whether that action is not
  started, in its expected conflict state, completed exactly, or ambiguous.
  Only exact states are adopted automatically.
- True merge commits use the message
  `Merge '<source>' into '<target-branch>'` with `GWZ-Merge-ID` and
  `GWZ-Operation-ID` trailers.
- After every participant succeeds, GWZ publishes the updated lock and merge
  marker in one checked root composition commit. Interrupted finalization stays
  open and `gwz merge --continue` resumes it without creating a second evidence
  commit.
- Source and target must share history. GWZ rejects unrelated histories for
  both this command and `pull --sync merge`; it does not implicitly enable
  Git's `--allow-unrelated-histories` behavior.
- Human, JSON, and JSONL results identify the action as `merge` and include
  every participant's source, target branch, recorded/live outcome, conflict
  paths, recovery eligibility, pending-action reconciliation, and structured
  drift. They also include the current publication step.

Preservation (`--abort --preserve`), strategy flags, custom merge messages,
record GC, and explicit workspace-root participation are not yet available.
These forms remain hidden and return typed unsupported errors if submitted
directly.

The generated command reference shows global options on every command. Merge
rejects unrelated operation policies supplied explicitly: `--sync`,
`--remote`, `--jobs`, `--max-per-host`, and `--progress-interval`. It also
rejects the reserved `--partial` and `--force` policies. Core diagnostics name
the option that must be removed.

`gwz branch --merge <source>` remains as a deprecated compatibility spelling.
It constructs the same first-class merge request and does not invoke the old
branch-merge protocol operation.
