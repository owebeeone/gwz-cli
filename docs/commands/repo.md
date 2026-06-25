# `gwz repo`

Manage workspace repository members.

```text
gwz repo [OPTIONS] <COMMAND>
```

Implemented subcommands:

```text
gwz repo add [OPTIONS] <repo-path>
gwz repo create [OPTIONS] <member-path>
gwz repo sync [member-path]
```

## `gwz repo add`

Register an existing local Git repository as a workspace member. GWZ records the
repository as a member; it does not clone a new copy.

```sh
gwz repo add repos/app
```

The argument is a path to an existing local Git repository.

## `gwz repo create`

Create a new local repository member and register it with the workspace.

```sh
gwz repo create repos/new-service
```

The argument is the workspace-relative member path. The repository can be pushed
to a remote later.

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

## Notes

- Use `repo add` when the Git repository already exists on disk.
- Use `repo create` when the workspace should grow a new repository from
  scratch.
- Use `repo sync` after changing a member's local Git remotes outside GWZ.
- After adding or creating a member, run `gwz status` to inspect root metadata
  and member state.
- Use `gwz add` for staging file contents; it is not the command for adding a
  repository member.
