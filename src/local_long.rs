pub(crate) const LOCAL_LONG: &str = "\
Create, inspect and retire the local clone family of this workspace.

A local clone is a second working copy of the whole workspace on the same
machine, made with `gwz local clone <name> [dest]`. The family index lives on
the workspace root; every clone carries a pointer back to it, so these commands
work from any ready member of the family.

`gwz local clone` creates a member. `gwz local list` reports the family.
`gwz local dispose` removes one member — its tree, or only its registration
with `--keep`. `gwz local disband` retires the family itself and leaves every
directory in place.";

pub(crate) const LOCAL_CLONE_LONG: &str = "\
Create a local clone of this workspace as a new family member.

Copies the current workspace into a second working copy on this machine and
registers it, by name, in the local clone family. Integrate work from the
receiving workspace with `gwz merge --remote <name>`. Family pull and push are not supported
by this build. There is no URL and no network. The destination defaults to
`../<root-dirname>-<name>`.

Only verbatim cloning is supported: it copies the source tree as it sits,
dirt and build directories included. The parser accepts --clean, --bare,
-b and --from, but this build refuses those forms before copying.
Keep the source quiet for the whole invocation.";

pub(crate) const LOCAL_LIST_LONG: &str = "\
List the local clone family recorded on the workspace root.

Reads the index through this workspace's family pointer and reports every
member: its name, kind (checkout or bare), state, and path — the root first,
then every member in name order.

The state column carries both what the index recorded (creating, ready,
disposing) and what was observed on disk (ready, incomplete,
interrupted_disposal, missing, pointer_removed, mismatched, malformed,
unobserved). They are shown as one word while they agree and as
`recorded/observed` when they do not, so an interrupted create or an
interrupted disposal is visible without a second command. Any diagnostic the
index recorded for a member is shown beside its row.

The listing performs no repair and takes no lock; --json and --jsonl carry
every field of every row.";

pub(crate) const LOCAL_DISPOSE_LONG: &str = "\
Dispose of one local family member.

By default this deletes the member's directory, and it refuses to do so while
any hazard is detected: an open coordinated merge, uncommitted or untracked
work, or history that is not verifiably preserved in another surviving family
member. A clean working tree is not preservation proof, and network-only
preservation is not certified.

Each reported hazard must be named explicitly with `--force` before the
deletion proceeds. That is an operator loss waiver, not crash recovery: an
incomplete or interrupted member is retained rather than force-deleted.

`--keep` is the non-destructive alternative: it removes only the pointer and
the index row, so the tree, its open merge and its history stay on disk and
remain usable as an ordinary workspace.

The workspace root is never disposed, and neither is the member you are
standing in.";

pub(crate) const LOCAL_DISBAND_LONG: &str = "\
Retire the local clone family.

Removes the family pointers and the index only. Every member directory stays
exactly where it is and remains an ordinary GWZ workspace. Family names stop
resolving afterwards, so `--remote <name>` on pull, push and merge falls back
to ordinary Git remote resolution.

Disband may be repeated after an error; it never routes a remaining row through
directory deletion.";
