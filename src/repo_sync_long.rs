pub(crate) const REPO_SYNC_LONG: &str = "\
Refresh GWZ member metadata from local git config.

`gwz repo sync` reads already-registered, materialized member repositories and
updates the workspace manifest with their configured git remotes and current
desired ref. A supplied `--root` selects the workspace and does not change the
base for a relative member path; that path remains relative to the invocation
directory. `--target` likewise selects participating repositories without
changing operand path resolution. It does not fetch, push, check out branches,
or rewrite the lock.

A configured remote that differs from the manifest only by URL scheme on
github.com, gitlab.com or bitbucket.org (the mark of a `--url-scheme` clone)
keeps the recorded URL and is reported instead; pass `--force` to record the
configured form.";
