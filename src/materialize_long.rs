pub(crate) const MATERIALIZE_LONG: &str = "\
Materialize workspace members to an explicit target.

Materialization makes the local repositories match a workspace target. It is not
raw `git pull`; GWZ plans the workspace operation first and applies the selected
target across members. With no target flag, `gwz materialize` uses the workspace
lock. Use `--head`, `--snapshot`, `--tag`, or `--switch` for a different
target.

`--url-scheme <manifest|ssh|https>` (or `GWZ_URL_SCHEME`) chooses the URL form
for known-host repositories this run clones; members already checked out keep
their remotes. Without it, a preference recorded by an earlier run in
`.gwz/url-scheme.yml` applies, then the manifest as written.";
