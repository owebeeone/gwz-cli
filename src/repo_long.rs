pub(crate) const REPO_LONG: &str = "\
Manage repository members inside a workspace.

Repository commands bring member repositories into a workspace and manage their
manifest metadata. Clone or create a new member, add an existing checkout,
detach a designation from the current composition, attach an inactive
designation, or sync metadata from local git config. `--root` selects the
workspace; it does not make relative repository operands relative to that root.
Relative operands stay relative to the directory where `gwz` was invoked, and
`--target` limits participating repositories without changing that path base.
Use top-level commands such as `gwz status`, `gwz pull`, and `gwz push` for
workspace-wide operations.";
