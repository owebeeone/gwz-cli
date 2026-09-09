pub(crate) const ADD_LONG: &str = "\
Add an existing local git repository to the workspace.

Use this when a repository already exists on disk and should become a workspace
member. GWZ records the repository as a member; it does not clone a new copy.
The repository path is resolved relative to the directory where `gwz` was
invoked. `--root` selects the workspace and does not change that path base;
`--target` selects participating repositories for workspace operations and does
not change operand resolution. Use `gwz repo create` instead when the member
should be created from scratch.";
