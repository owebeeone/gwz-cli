pub(crate) const REPO_CLONE_LONG: &str = "\
Clone a Git repository into the current workspace and register it as an active
member.

The optional member path controls where the checkout is created. Without it,
GWZ derives the path from the repository URL. `--member-id` and `--source-id`
override the derived designation and logical source identities. Dry-run plans
the clone without creating the checkout or changing workspace artifacts.";
