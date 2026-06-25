# `gwz init`

Create a workspace or initialize one from source URLs.

```text
gwz init [OPTIONS] [url]...
```

With no URLs, `gwz init` creates an empty GWZ workspace. With one or more URLs,
it creates the workspace, adds those repositories as initial members, and
materializes them from their heads.

GWZ writes workspace metadata under `gwz.conf/`, including the manifest
`gwz.conf/gwz.yml` and lock `gwz.conf/gwz.lock.yml`.

## Options

| Option | Meaning |
| --- | --- |
| `--path <path-prefix>` | Workspace-relative prefix for initialized source repositories. Defaults to an empty prefix. |

Global options such as `--dry-run`, `--partial`, `--force`, `--sync`,
`--remote`, `--jobs`, `--max-per-host`, `--json`, and `--jsonl` are also
available.

## Examples

Create an empty workspace:

```sh
gwz init
```

Initialize from two sources:

```sh
gwz init git@github.com:org/app.git git@github.com:org/lib.git
```

Place initialized repositories under a prefix:

```sh
gwz init --path repos git@github.com:org/app.git
```

## Notes

- `--path` affects paths assigned to source repositories during initialization.
- If you already have a GWZ root repository cloned, use
  `gwz materialize --lock` instead of `gwz init`.
- Use `gwz repo add` to register an existing local repository after workspace
  creation.
