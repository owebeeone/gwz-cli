# Quick Start

This flow shows the normal GWZ lifecycle with the implemented commands.

## Create A Workspace

Create an empty workspace:

```sh
mkdir demo
cd demo
gwz init
```

Initialize a workspace from existing source URLs:

```sh
gwz init git@github.com:org/app.git git@github.com:org/lib.git
```

Put initialized repositories under a prefix:

```sh
gwz init --path repos git@github.com:org/app.git
```

## Clone A Workspace

Clone the root workspace repository and materialize members from the lock:

```sh
gwz clone git@github.com:org/workspace.git work/demo
cd work/demo
gwz status
```

If the root was cloned with plain `git clone`, complete the workspace:

```sh
gwz materialize --lock
```

## Inspect Members

List materialized member paths:

```sh
gwz ls
```

Print workspace-relative paths:

```sh
gwz ls --local
```

Include configured members that are not yet materialized:

```sh
gwz ls --unmaterialized
```

## Add Or Create Members

Register an existing local Git repository:

```sh
gwz repo add repos/local-lib
```

Create a new member repository:

```sh
gwz repo create repos/new-service
```

Check the resulting workspace state:

```sh
gwz status
```

## Stage And Commit

Stage specific paths across the workspace:

```sh
gwz add gwz-cli/README.md gwz-core/src/lib.rs
```

Stage all changes:

```sh
gwz add -A
```

Commit staged changes across members and the workspace root:

```sh
gwz commit -m "Update workspace docs"
```

Stage tracked modifications as part of commit:

```sh
gwz commit -a -m "Refresh generated files"
```

## Snapshot And Restore

Record a named snapshot before risky work:

```sh
gwz snapshot before-refactor
```

List snapshots:

```sh
gwz snapshot --list
```

Return members to a snapshot:

```sh
gwz materialize --snapshot before-refactor
```

## Tags

Create a real Git tag across selected members and the committed workspace root:

```sh
gwz tag v0.3.0
```

Create an annotated tag:

```sh
gwz tag v0.3.0 -m "GWZ v0.3.0"
```

List local tags:

```sh
gwz tag
```

Push one tag to member remotes:

```sh
gwz tag --push v0.3.0
```

Materialize all selected members at the tag:

```sh
gwz materialize --tag v0.3.0
```

## Run A Command In Members

Run a portable argv command in every member:

```sh
gwz forall -- git status --short
```

Run a shell command string:

```sh
gwz forall -c 'printf "%s\n" "$GWZ_MEMBER_PATH"'
```

Run in selected members by id or path:

```sh
gwz forall gwz-cli taut -- cargo test
```

## Pull And Push

Move members forward to repository heads with fast-forward behavior by default:

```sh
gwz pull --head
```

Preview a pull:

```sh
gwz --dry-run pull --head
```

Push member refs:

```sh
gwz push
```

Select a remote:

```sh
gwz --remote origin push
```

## Machine Consumers

Use one JSON response:

```sh
gwz --json status
```

Use newline-delimited JSON records for streaming operation consumers:

```sh
gwz --jsonl pull --head
```

Use stable status porcelain:

```sh
gwz status --porcelain
```
