# `gwz ls`

List workspace members.

```text
gwz ls [OPTIONS]
```

Human output is one path per line and has no header, so it can be used in simple
scripts.

## Options

| Option | Meaning |
| --- | --- |
| `--local` | Print workspace-relative paths instead of absolute paths. |
| `--unmaterialized` | Include configured members that are not materialized. |

## Examples

Print absolute paths for materialized members:

```sh
gwz ls
```

Print workspace-relative paths:

```sh
gwz ls --local
```

Include configured members that are not checked out locally:

```sh
gwz ls --unmaterialized
```

Use JSON for member ids and paths:

```sh
gwz --json ls
```

## Machine Shape

`gwz --json ls` renders:

```json
{
  "kind": "members",
  "entries": [
    {
      "id": "gwz-cli",
      "path": "gwz-cli",
      "abspath": "/work/gwz-dev/gwz-cli",
      "materialized": true
    }
  ]
}
```
