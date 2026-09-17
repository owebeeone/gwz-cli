pub(crate) const LOCAL_AFTER: &str = "\
Examples:
  gwz local clone A ../gwz-dev-A
  gwz local list
  gwz local dispose C --keep
  gwz local dispose C --force open-merge,dirty,unpreserved-history
  gwz local disband";

pub(crate) const LOCAL_CLONE_AFTER: &str = "\
Examples:
  gwz local clone A ../gwz-dev-A
  gwz local clone A --owner claude-code:session_7 --wait 120

`root`, `origin` and Git's reserved ref names are refused as member names, and
so is a name already recorded in the family. A verbatim copy is refused while
the source has an open coordinated merge: finish or abort it first.
--clean, --bare, -b and --from are reserved and unsupported in this build.

--owner <token> takes up to 128 bytes of [A-Za-z0-9._:-] and is recorded on
the row, reported, and never interpreted. --wait <secs> retries a busy family
lock until the deadline. The first index write by a gwz that supports --owner
makes the family index format 2, which gwz 1.0.14 and later read.";

pub(crate) const LOCAL_LIST_AFTER: &str = "\
Example:
  gwz local list
  gwz local list --json

Output columns: name, kind, state, path, plus owner when any member records
one and last_error when any member carries one. The state column is one word while
the recorded row and the directory agree, and `recorded/observed` when they do
not — `creating/incomplete` for an interrupted create, for instance. A member
that recorded a diagnostic gets a fifth column carrying it.";

pub(crate) const LOCAL_DISPOSE_AFTER: &str = "\
Examples:
  gwz local dispose C
  gwz local dispose C --keep
  gwz local dispose C --keep --wait 60
  gwz local dispose C --force unpreserved-history
  gwz local dispose C --force open-merge,dirty,unpreserved-history

Hazard names: open-merge, dirty, unpreserved-history. They are operands of
this command and are accepted only together with --force; `--force` with no
names is refused, and so is `--keep` together with `--force`.";

pub(crate) const LOCAL_DISBAND_AFTER: &str = "\
Example:
  gwz local disband

Every member directory survives; only the pointers and the index are removed.";
