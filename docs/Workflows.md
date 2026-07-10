# Workflows

## Create A Workspace From Existing Repositories

```sh
mkdir demo
cd demo
gwz init git@github.com:org/app.git git@github.com:org/lib.git
gwz status
gwz snapshot initial
```

Use `--path <prefix>` during init when member repositories should live under a
directory such as `repos/`.

## Clone And Complete A Workspace

```sh
gwz clone git@github.com:org/workspace.git work/demo
cd work/demo
gwz status
```

If the root was cloned with Git:

```sh
gwz materialize --lock
gwz status
```

## Grow A Workspace

Create a new member:

```sh
gwz repo create repos/new-service
gwz status
gwz add -A
gwz commit -m "Add new service"
```

Register an existing local repository:

```sh
gwz repo add repos/local-lib
gwz status
gwz add -A
gwz commit -m "Add local lib member"
```

Clone a new member directly into the workspace:

```sh
gwz repo clone git@github.com:org/shared-lib.git repos/shared-lib
gwz status
```

## Detach And Reattach A Member

Temporarily remove a member from the active composition while retaining its
checkout and historical identity:

```sh
gwz repo detach mem_shared
# Later, after restoring the checkout at its recorded path:
gwz repo attach mem_shared
```

Attach verifies every commit recorded for the member in snapshots and markers.
GWZ does not fetch missing history. If verification reports
`SourceIdentityMismatch`, fetch enough history into the retained checkout and
retry attach.

Bare add is a convenience when the detached checkout is still present:

```sh
gwz repo add libs/shared
```

It reactivates the old row only when exactly one inactive row at that path has
non-empty, fully verified commit evidence. Use explicit `repo attach
mem_shared` when no evidence was recorded or when the path has ambiguous
history.

To replace the member with a different repository, keep the old row inactive
and create a new designation:

```sh
gwz repo detach mem_shared
mv libs/shared ../shared-old
gwz repo clone git@github.com:org/replacement.git libs/shared \
  --member-id mem_replacement
```

See [Repository Member Lifecycle](RepoLifecycle.md) for the complete identity,
warning, and recovery contract.

## Make A Multi-Repository Change

```sh
gwz status
gwz snapshot before-change
gwz forall -- cargo test
gwz add -A
gwz commit -m "Update shared API"
gwz status
```

If the change must be backed out to the snapshot:

```sh
gwz materialize --snapshot before-change
```

## Prepare A Release Tag

```sh
gwz status
gwz forall -- cargo test
gwz snapshot release-candidate
gwz tag v0.3.0 -m "GWZ v0.3.0"
gwz tag --push v0.3.0
```

Verify the tag checkout:

```sh
gwz materialize --tag v0.3.0
gwz status
```

## Pull A Workspace Forward

Preview first:

```sh
gwz --dry-run pull --head
```

Then update:

```sh
gwz pull --head
gwz status
```

If a remote host is slow or overloaded:

```sh
gwz --jobs 20 --max-per-host 4 --ssh-timeout 10 pull --head
```

## Run Maintenance In Members

Run a direct command:

```sh
gwz forall -- git status --short
```

Run a shell command that uses GWZ environment variables:

```sh
gwz forall -c 'echo "$GWZ_MEMBER_PATH"; git rev-parse --short HEAD'
```

Continue through failures and report the failed members at the end:

```sh
gwz --partial forall -- cargo test
```

## Script Member Paths

Human `gwz ls` output is one path per line:

```sh
gwz ls --local
```

For structured member metadata:

```sh
gwz --json ls
```

For stable status path output:

```sh
gwz status --porcelain
```
