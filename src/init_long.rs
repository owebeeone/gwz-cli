pub(crate) const INIT_LONG: &str = "\
Create a workspace or initialize one from source URLs.

A GWZ workspace is a local directory that owns a tracked `gwz.conf/` metadata
directory. `gwz.conf/gwz.yml` describes the workspace and its repository
members. `gwz.conf/gwz.lock.yml` records the exact revisions that make the
workspace reproducible.

Running `gwz init` with no URLs creates an empty workspace at `--root` or the
current directory. Passing one or more URLs creates the workspace and adds those
repositories as initial members, materialized from their heads.

Running `gwz init --update` refreshes root-only GWZ-managed bootstrap files in
an existing workspace, including `AGENTS_GWZ.md`, and ensures that `AGENTS.md`
points agents to it. Existing `AGENTS.md` instructions are preserved. Managed
files are overwritten only when their digest header still matches their body;
use global `--force` to replace a locally edited bootstrap file.

To deliberately accept edited configuration and commit the recovery together,
run `gwz init --update --force --commit`. Both configuration documents must
parse; this accepts their bytes but does not repair incorrect paths or topology.
The commit includes the manifest, existing lock, integrity marker, managed
instructions and agent-reference/settings files changed by this update.
Unrelated staged work is preserved. `--commit` requires `--update`; omit
`--force` when no hand edit needs acceptance. Unchanged updates make no empty
commit, and `--dry-run` changes nothing. Output names the commit and paths.";
