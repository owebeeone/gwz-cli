pub(crate) const FETCH_LONG: &str = "\
Contact every selected repository's remote and report what moved.

`gwz fetch` fetches the configured remote of each selected workspace target,
updates that repository's remote-tracking refs, and reports one line per
repository: the tracking ref before and after, how far the current branch is
ahead of and behind it, or `no change`. By default that includes the workspace
root (`@root`) plus configured member repositories, and the selectors are
`push`'s: `--target`, `--member`, `--member-path`, `--all`, `--no-target @root`.

What it does NOT do:
  - It never integrates. No merge, no rebase, no fast-forward, no reset: no
    branch, no HEAD, no index and no working-tree file changes. Use `gwz pull`
    to integrate what a fetch showed you.
  - It never writes workspace artifacts: no lock, no manifest, no boundary
    sync. Because of that it is one of the few network verbs that still runs
    while a merge is open.
  - It never prunes.

It always contacts the remotes. There is no `--check-remotes` and no
`unchanged since the last fetch` short-circuit as there is on `gwz push`: a
fetch that does not connect has answered nothing.

`--dry-run` is the one exception, and it is not git's. `git fetch --dry-run`
contacts the remote and then declines to write the refs; `gwz --dry-run fetch`
contacts no remote at all. It resolves the selection and prints the planned
rows, one per repository it would have contacted, and stops there. So it
answers `which repositories would be contacted` and never `what moved`: the
rows carry no result from any remote.

Exit codes follow `gwz push`: 0 when every selected repository answered,
1 when some answered and some failed (the report is incomplete), and 2 when
the request was refused before any remote was contacted.";
