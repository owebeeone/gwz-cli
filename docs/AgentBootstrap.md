# Agent Bootstrap

`AGENTS_GWZ.md` is a short bootstrap hint for agents entering a GWZ-managed
repository. It should tell an agent how to install `gwz`, finish materializing a
workspace, inspect status, and find the full docs.

This v0.3.0 documentation page describes the intended bootstrap content and how
to use it when present. The current CLI help in this workspace does not expose
an implemented `gwz init --update` command, so automatic creation, update, and
overwrite behavior are not documented as available CLI behavior here.

Hosted docs URL for bootstrap files:
https://github.com/owebeeone/gwz-cli/tree/main/docs

## Intended File Scope

An `AGENTS_GWZ.md` file belongs at the workspace root repository. It is not a
member-repository policy file and should not try to replace local `AGENTS.md`
instructions.

Keep it deliberately brief:

- One sentence that the repository is managed by GWZ.
- Install commands or links.
- The command to clone a workspace when it is not cloned yet.
- The command to materialize members when the root repository already exists.
- A status check.
- Links to `gwz --help` and the hosted docs.

## Suggested Template

````md
# GWZ Workspace

This repository is managed by GWZ, a multi-repository workspace tool.

Install `gwz` from the latest release:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Or install from source:

```sh
cargo install --git https://github.com/owebeeone/gwz-cli
```

If the workspace is not cloned yet:

```sh
gwz clone <workspace-git-url> [directory]
```

If this root repository is already cloned:

```sh
gwz materialize --lock
gwz status
```

Docs:

- `gwz --help`
- https://github.com/owebeeone/gwz-cli/tree/main/docs
````

## Managed-File Updates

A future managed bootstrap writer should be safe around local edits. A robust
scheme is to include a managed-file header with a digest of the Markdown body
after the header and separator line, then overwrite automatically only when the
current body still matches that digest.

Example header shape:

```html
<!-- gwz-managed-file: sha256=<template-body-sha256> -->
```

When the header is missing or the body no longer matches the recorded digest,
an updater should skip the file or require an explicit force flag.

## What Not To Put Here

Do not include detailed workflow guidance, complete command catalogs,
contribution policy, release procedure, or package-specific build instructions.
Those belong in `README.md`, `AGENTS.md`, package docs, or this docs directory.
