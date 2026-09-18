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

## What `materialized` means

`materialized` is observed, not claimed. A member is materialized when the
workspace lock records it *and* its directory exists.

Those two can disagree. `gwz clone` of a workspace skips a private member whose
access the server refuses — by design, quietly: it removes the directory,
reports no row and says nothing. It does not rewrite the lock (a clone
materializes the committed lock), so the lock goes on recording that member as
materialized while nothing is on disk.

Such a member is still listed — hiding it is what made the discrepancy
invisible — but it reports `materialized: false` and a `note` saying why:

```text
$ gwz ls --local
repos/app
repos/secret	(private, skipped)
```

The note is the one thing that interrupts the plain path list, and only for a
line whose path does not exist, so a script that consumed `gwz ls` was already
getting a path it could not use there. `private, skipped` is the quiet-clone
case; a member recorded by the lock and missing for any other reason (a
directory removed by hand) reads `recorded in the lock but absent on disk`.
Members the lock never materialized are unchanged: omitted unless
`--unmaterialized`, and carrying no note.

The note is human text. Read `materialized` to decide anything.

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
      "materialized": true,
      "note": null
    },
    {
      "id": "mem_secret",
      "path": "repos/secret",
      "abspath": "/work/gwz-dev/repos/secret",
      "materialized": false,
      "note": "private, skipped"
    }
  ]
}
```

`note` is `null` on every ordinary row.
