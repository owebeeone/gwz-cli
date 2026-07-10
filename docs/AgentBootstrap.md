# Agent Bootstrap

`AGENTS_GWZ.md` is a short bootstrap hint for agents entering a GWZ-managed
repository. It should tell an agent how to install `gwz`, finish materializing a
workspace, inspect status, and find the full docs.

`gwz init` creates this file in new workspace roots, and `gwz init --update`
refreshes it in an existing workspace root. The file is root-only: GWZ does not
write `AGENTS_GWZ.md` into member repositories.

Hosted docs URL for bootstrap files:
https://owebeeone.github.io/gwz-cli/

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

## Generated Template

````md
<!-- gwz-managed-file: sha256=<template-body-sha256> -->

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
- Quick Start: https://owebeeone.github.io/gwz-cli/QuickStart/
- Full documentation: https://owebeeone.github.io/gwz-cli/
````

## Managed-File Updates

GWZ includes a managed-file header with a digest of the Markdown body after the
header and separator line:

```md
<!-- gwz-managed-file: sha256=<template-body-sha256> -->
```

`gwz init --update` overwrites `AGENTS_GWZ.md` automatically only when the
current body still matches the digest in that header. When the header is missing
or the body no longer matches, GWZ refuses to overwrite the file. Use global
`--force` to replace a locally edited bootstrap file:

```sh
gwz --force init --update
```

The update command discovers the workspace root from the current directory, or
uses global `--root <path>` when supplied.

## What Not To Put Here

Do not include detailed workflow guidance, complete command catalogs,
contribution policy, release procedure, or package-specific build instructions.
Those belong in `README.md`, `AGENTS.md`, package docs, or this docs directory.
