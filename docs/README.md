# GWZ CLI Documentation

This directory is the user-facing documentation set for the `gwz` command line.
It tracks the implemented command surface exposed by `gwz --help`.

Hosted docs URL:
https://github.com/owebeeone/gwz-cli/tree/main/docs

GWZ resolves the workspace from where you run the command, including from inside
member repositories; use `--root <path>` only when you need to override that.
Examples in these docs usually omit it.

## Start Here

- [Install](Install.md): release installers, source install, local development
  runs, and verification.
- [Quick Start](QuickStart.md): create or clone a workspace and run the common
  multi-repository workflow.
- [Concepts](Concepts.md): workspace roots, members, manifests, locks,
  snapshots, tags, selections, remotes, and progress.
- [CLI Reference](CLI.md): generated root and command help from the Clap
  command definitions.
- [Workflows](Workflows.md): task-oriented flows across commands.
- [Repository Member Lifecycle](RepoLifecycle.md): clone, detach, attach,
  evidence-backed re-add, and historical identity verification.
- [Machine Output](MachineOutput.md): `--json`, `--jsonl`, status porcelain,
  listings, and exit codes.
- [Root Workspaces](RootWorkspace.md): using GWZ to manage a GWZ-managed root
  such as `gwz-dev`.
- [Troubleshooting](Troubleshooting.md): common failures and recovery steps.
- [Releases](Releases.md): release installers, checksums, attestations, and
  smoke tests.
- [Agent Bootstrap](AgentBootstrap.md): concise `AGENTS_GWZ.md` guidance for
  agents entering a GWZ-managed repository.

## Command Pages

- [init](commands/init.md): create an empty workspace or initialize from source
  URLs.
- [clone](commands/clone.md): clone a workspace root and materialize members.
- [repo](commands/repo.md): add, clone, create, detach, attach, or sync
  repository members.
- [ls](commands/ls.md): list materialized and configured members.
- [status](commands/status.md): inspect workspace Git state.
- [add](commands/add.md): stage file contents across workspace repositories.
- [commit](commands/commit.md): commit staged changes across members and the
  workspace root.
- [capture](commands/capture.md): record the live worktree state into the lock.
- [snapshot](commands/snapshot.md): record or list workspace snapshots.
- [tag](commands/tag.md): manage real Git tags across the workspace.
- [branch](commands/branch.md): manage local branches across selected members.
- [stash](commands/stash.md): manage coordinated stash bundles across selected
  members.
- [materialize](commands/materialize.md): check out lock, head, snapshot, or tag
  targets.
- [pull](commands/pull.md): move members forward to head or snapshot targets.
- [push](commands/push.md): push member refs to remotes.
- [forall](commands/forall.md): run a command in each selected member.

## Source Of Truth

Terminal help comes from the Clap command definitions in the CLI. The generated
[CLI Reference](CLI.md) is checked against those definitions by `cargo test`.
Regenerate or check it directly with:

```sh
python scripts/generate_cli_reference.py --write
python scripts/generate_cli_reference.py --check
```

When behavior or options are unclear, prefer:

```sh
cargo run -q -p gwz -- --help
cargo run -q -p gwz -- help <command>
```

The command pages add semantics, examples, and recovery notes around that help.
