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

`root`, `origin` and Git's reserved ref names are refused as member names, and
so is a name already recorded in the family. A verbatim copy is refused while
the source has an open coordinated merge: finish or abort it first.
--clean, --bare, -b and --from are reserved and unsupported in this build.";

pub(crate) const LOCAL_LIST_AFTER: &str = "\
Example:
  gwz local list
  gwz local list --json

Output columns: name, kind, state, path. The state column is one word while
the recorded row and the directory agree, and `recorded/observed` when they do
not — `creating/incomplete` for an interrupted create, for instance. A member
that recorded a diagnostic gets a fifth column carrying it.";

pub(crate) const LOCAL_DISPOSE_AFTER: &str = "\
Examples:
  gwz local dispose C
  gwz local dispose C --keep
  gwz local dispose C --force unpreserved-history
  gwz local dispose C --force open-merge,dirty,unpreserved-history

Hazard names: open-merge, dirty, unpreserved-history. They are operands of
this command and are accepted only together with --force; `--force` with no
names is refused, and so is `--keep` together with `--force`.";

pub(crate) const LOCAL_DISBAND_AFTER: &str = "\
Example:
  gwz local disband

Every member directory survives; only the pointers and the index are removed.";
