# `gwz repo`

Manage workspace repository members.

```text
gwz repo [OPTIONS] <COMMAND>
```

Implemented subcommands:

```text
gwz repo add [OPTIONS] <repo-path>
gwz repo clone [OPTIONS] <url> [member-path]
gwz repo create [OPTIONS] <member-path>
gwz repo detach [OPTIONS] <member>
gwz repo attach [OPTIONS] <member-id>
gwz repo sync [member-path]
```

## `gwz repo add`

Register an existing local Git repository as a workspace member. GWZ records the
repository as a member; it does not clone a new copy.

```sh
gwz repo add repos/app
```

The argument is a path to an existing local Git repository.

Use `--member-id` and `--source-id` to create a deliberate new designation. A
bare add at a detached member's historical path reactivates that designation
only when exactly one inactive row has non-empty historical commit evidence
and every recorded commit exists in the checkout.

```sh
gwz repo add libs/shared --member-id mem_shared_v2 --source-id src_shared
```

## `gwz repo clone`

Clone a remote Git repository into the current workspace and register it as an
active member.

```sh
gwz repo clone git@github.com:org/shared-lib.git libs/shared
gwz --dry-run repo clone git@github.com:org/shared-lib.git libs/shared
```

The path defaults from the URL. Use an explicit new `--member-id` when placing
a replacement repository at a path retained by an inactive designation.

## `gwz repo create`

Create a new local repository member and register it with the workspace.

```sh
gwz repo create repos/new-service
gwz repo create repos/new-service --member-id mem_service --source-id src_service
```

The argument is the workspace-relative member path. The repository can be pushed
to a remote later.

## `gwz repo detach`

Stop managing a member in the current workspace composition. Detach marks its
manifest row inactive and removes its lock entry, but leaves its checkout,
snapshots, and markers alone.

```sh
gwz repo detach mem_shared
gwz repo detach libs/shared
```

The positional operand selects exactly one active member by id or path and
cannot be combined with global selection flags.

## `gwz repo attach`

Reactivate an inactive designation while preserving its member and source
identity.

```sh
gwz repo attach mem_shared
```

Attach accepts a historical member id, not a path. Every snapshot and marker
commit recorded for that member must exist in the retained checkout. Missing
objects fail with `SourceIdentityMismatch`; GWZ does not fetch them
automatically. When no historical evidence exists, explicit attach proceeds
and emits a warning because the designation was named directly.

## `gwz repo sync`

Refresh GWZ manifest metadata for already-registered, materialized members from
their local Git config. This records local remotes and updates the desired ref
from the current HEAD. It does not fetch, push, check out branches, or rewrite
the lock.

```sh
gwz repo sync gwz-py
```

Use this after adding a Git remote directly:

```sh
git -C gwz-py remote add origin git@github.com:owebeeone/gwz-py.git
gwz repo sync gwz-py
```

This is also the publish-later path after `repo create`: create an empty remote
on the hosting service, add it as `origin`, run `repo sync`, make at least one
commit, then push the selected member. `repo sync` currently leaves its
manifest rewrite unstaged, so stage `gwz.conf` before committing that metadata.

```sh
gwz add gwz.conf
gwz commit -m "Record member origin"
gwz --member mem_gwz_py push
```

## Notes

- Use `repo add` when the Git repository already exists on disk.
- Use `repo clone` to clone and register a new member in an existing workspace.
- Use `repo create` when the workspace should grow a new repository from
  scratch.
- Use `repo detach` and `repo attach` to temporarily remove and later restore
  the same designation.
- Use `repo sync` after changing a member's local Git remotes outside GWZ.
- After adding or creating a member, run `gwz status` to inspect root metadata
  and member state.
- Use `gwz add` for staging file contents; it is not the command for adding a
  repository member.
