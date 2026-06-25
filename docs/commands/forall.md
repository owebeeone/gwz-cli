# `gwz forall`

Run a command in each selected member.

```text
gwz forall [OPTIONS] [projects]... [-- <cmd>...]
```

`gwz forall` resolves materialized members, then runs the requested command in
each member directory. Positional `projects` match member ids or member paths.
When no projects are supplied, the command runs in all materialized members.

## Arguments And Options

| Item | Meaning |
| --- | --- |
| `[projects]...` | Members to run in by id or path. Empty means all. |
| `[cmd]...` | Command and args, run directly without a shell. Use after `--`. |
| `-c`, `--command-string <string>` | Run a shell command string. |
| `--no-banner` | Suppress the per-member banner. |

## Examples

Run direct argv:

```sh
gwz forall -- git status --short
```

Run a shell command string:

```sh
gwz forall -c 'printf "%s\n" "$GWZ_MEMBER_PATH"'
```

Run in selected members:

```sh
gwz forall gwz-cli taut -- cargo test
```

Suppress banners:

```sh
gwz forall --no-banner -- git rev-parse --short HEAD
```

Continue after a member command fails:

```sh
gwz --partial forall -- cargo test
```

Use member path substitution in argv mode:

```sh
gwz forall -- printf '%s\n' '{@}'
```

## Environment

Each child process receives:

| Variable | Meaning |
| --- | --- |
| `GWZ_MEMBER_ID` | Member id. |
| `GWZ_MEMBER_PATH` | Workspace-relative member path. |
| `GWZ_MEMBER_ABSPATH` | Absolute member path. |
| `GWZ_ROOT` | Absolute workspace root. |

## Output And Failure

- Child stdio is inherited and streams live.
- Unless `--no-banner` is supplied, a `=== <path> ===` banner is written to
  stderr before each member command.
- The command stops on the first failing member unless global `--partial` is
  supplied.
- `--json` and `--jsonl` are rejected for `forall`; child output is not wrapped
  in machine records.
