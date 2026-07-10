# Workflows

The [Quick Start](QuickStart.md) owns the beginner create/clone path. Use
[Repository Member Lifecycle](RepoLifecycle.md) for member creation,
publish-later setup, clone/add distinctions, detach/attach verification, and
replacement. The recipes below start with an existing workspace.

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
gwz tag v0.9.0 -m "GWZ v0.9.0"
gwz tag --push v0.9.0
```

Verify the tag checkout:

```sh
gwz materialize --tag v0.9.0
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
