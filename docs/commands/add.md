# `gwz add`

Stage file contents across workspace repositories.

```text
gwz add [OPTIONS] [pathspec]...
```

`gwz add` is the multi-repository equivalent of `git add`. Each pathspec is
resolved relative to the current directory, routed to the member or workspace
root repository that owns it, and staged there.

## Arguments And Options

| Item | Meaning |
| --- | --- |
| `[pathspec]...` | Paths to stage, resolved like `git add`. |
| `-A` | Stage all changes across every workspace repository. |
| `--all` | The global target selector (`@all`), not the staging flag. |

## Target Selection

A target selection (`--target`, `--member`, `--path`) scopes where `gwz add`
stages:

- With `-A`, staging is limited to the selected targets — `gwz --member mem_app
  add -A` stages the member and leaves the root index untouched.
- With pathspecs, the selection constrains routing. A pathspec that routes to a
  repository outside the selection is an error and nothing is staged anywhere.
  Repositories reached only by `.` fan-out across a member boundary are skipped
  silently, the same way an unmaterialized fan-out target is.

```sh
# stages in mem_app only
gwz --member mem_app add -A

# error: `docs` is root territory, which the selection excludes
gwz --member mem_app add docs
```

## Examples

Stage one path:

```sh
gwz add gwz-cli/README.md
```

Stage paths in different repositories:

```sh
gwz add gwz-cli/README.md gwz-core/src/lib.rs
```

Stage everything:

```sh
gwz add -A
```

## Notes

- Pair `gwz add` with `gwz commit`.
- Use `gwz repo add` to register an existing repository as a workspace member.
- `--dry-run` plans and validates the staging operation without changing any
  index or `.git/info/exclude`.
