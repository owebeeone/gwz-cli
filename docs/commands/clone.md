# `gwz clone`

Clone a GWZ workspace root repository from a URL and materialize its members.

```text
gwz clone <url> [directory]
```

`gwz clone` is the one-shot form of cloning the root repository and then running
`gwz materialize --lock`. The root repository is the repository that owns the
tracked `gwz.conf/` directory.

## Arguments

| Argument | Meaning |
| --- | --- |
| `<url>` | Git URL of the workspace root repository. |
| `[directory]` | Target directory. Defaults to a directory name derived from the URL. |

## Examples

Clone into a derived directory:

```sh
gwz clone git@github.com:org/workspace.git
```

Clone into an explicit directory:

```sh
gwz clone git@github.com:org/workspace.git work/demo
```

Complete a workspace after plain `git clone`:

```sh
gwz materialize --lock
```

## Notes

- `gwz clone` verifies that the cloned root is a GWZ workspace.
- Missing member repositories are cloned and checked out at lock revisions.
- `--dry-run` is rejected for `gwz clone`.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--remote`, `--progress-interval`, and `--ssh-timeout`.
- A second working copy of a workspace on the same machine is a different
  command: [`gwz local clone`](local.md). `gwz clone` takes a URL and nothing
  else.

## Private members

A member marked `private: true` is cloned when access is available. If the
remote refuses access (including an authentication refusal or a remote not-found
response), workspace clone silently skips that member. Other members still
clone, and the private member's manifest and lock entries remain for a later
`gwz materialize --lock` retry. Skipped clones produce no member progress,
response row or transport diagnostic.

Unmarked members retain normal failure behavior. A private marker does not hide
local disk errors, corrupt repositories, checkout errors or workspace-root clone
failures. It also does not change repository visibility: the member name and URL
remain readable in the manifest.

Set or clear the marker through [repo sync](repo.md#gwz-repo-sync):

```sh
gwz repo sync gwz-core-evidence --private
gwz repo sync gwz-core-evidence --public
```
