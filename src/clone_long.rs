pub(crate) const CLONE_LONG: &str = "\
Clone a GWZ workspace, from a URL or from this machine.

`gwz clone <url>` is the one-shot form of `git clone <url>` followed by
`gwz materialize --lock`. It clones the workspace root repository (the one that
owns the tracked `gwz.conf/` directory) into a target directory, verifies it is
a GWZ workspace, then materializes every member: missing member repositories are
cloned and checked out at the commits recorded in `gwz.conf/gwz.lock.yml`.

If the target directory is omitted, it is derived from the URL.

`gwz clone --local --name <name> [dest]` instead copies the current workspace
into a second working copy on this machine and registers it in the local clone
family, so the two can exchange work by name. There is no URL: the single
positional is the destination, which defaults to `../<root-dirname>-<name>`.
The default mode copies the source tree as it sits; `--clean` takes the frozen
state without worktree dirt, and `--bare` makes a share point of bare member
repositories. Keep the source quiet for the whole invocation.";
