pub(crate) const CLONE_LONG: &str = "\
Clone a GWZ workspace from a URL.

`gwz clone <url>` is the one-shot form of `git clone <url>` followed by
`gwz materialize --lock`. It clones the workspace root repository (the one that
owns the tracked `gwz.conf/` directory) into a target directory, verifies it is
a GWZ workspace, then materializes every member: missing member repositories are
cloned and checked out at the commits recorded in `gwz.conf/gwz.lock.yml`.

If the target directory is omitted, it is derived from the URL.

Manifests may record member remotes in the ssh form (`git@github.com:...`) or
the https form. `--url-scheme https` (or `GWZ_URL_SCHEME=https`) clones every
github.com, gitlab.com and bitbucket.org repository over https instead, for a
reader without SSH keys; `--url-scheme ssh` asks for the ssh form. The default,
`manifest`, uses each URL as written, so a contributor with SSH keys needs
nothing. The choice is remembered in `.gwz/url-scheme.yml` for later
`gwz materialize` runs in that workspace.";
