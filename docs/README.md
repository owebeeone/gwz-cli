# GWZ Documentation

GWZ coordinates multiple ordinary Git repositories as one reproducible,
inspectable workspace. The workspace root records composition and exact state;
member repositories remain normal Git repositories.

[Start with the Quick Start](QuickStart.md) to install `gwz`, create or clone a
workspace, make a cross-repository change, and learn the repository lifecycle.

## Choose A Path

| I want to… | Read… |
| --- | --- |
| Install and use GWZ for the first time | [Quick Start](QuickStart.md) |
| Understand what GWZ adds to Git | [Why GWZ](https://github.com/owebeeone/gwz-core/blob/main/docs/WhyGwz.md) |
| Add, create, detach, attach, or replace a member | [Repository Member Lifecycle](RepoLifecycle.md) |
| Build or change GWZ itself | [Root Workspaces](RootWorkspace.md) |
| Embed the engine or build a remote client | [gwz-core documentation](https://github.com/owebeeone/gwz-core/tree/main/docs) |
| Script the CLI | [Machine Output](MachineOutput.md) and [CLI Reference](CLI.md) |

GWZ resolves the workspace from the current directory, including from inside a
member repository. Use `--root <path>` only to override that discovery.

## Guides

- [Install](Install.md): installers, source installs, and release verification.
- [Concepts](Concepts.md): roots, members, manifests, locks, snapshots,
  selections, remotes, and progress.
- [Workflows](Workflows.md): task-oriented multi-repository recipes.
- [Repository Member Lifecycle](RepoLifecycle.md): clone, create, publish,
  detach, attach, evidence-backed re-add, and replacement.
- [Root Workspaces](RootWorkspace.md): work in a GWZ-managed development root.
- [Troubleshooting](Troubleshooting.md): common failures and recovery.
- [Agent Bootstrap](AgentBootstrap.md): the generated `AGENTS_GWZ.md` hint.
- [Releases](Releases.md): release docs and installer verification.

## Reference

- [CLI Reference](CLI.md): generated root and command help.
- [Machine Output](MachineOutput.md): JSON, JSONL, porcelain, and exit codes.
- [Command pages](commands/status.md): semantics and examples by command.

Terminal help and the generated reference come from the Clap command
definitions. Check generated documentation after changing the command surface:

```sh
python scripts/generate_cli_reference.py --check
```
