pub(crate) const LOCAL_AFTER: &str = "\
Examples:
  gwz local list
  gwz local dispose C --keep
  gwz local dispose C --force open-merge,dirty,unpreserved-history
  gwz local disband

Create a family member with `gwz clone --local --name <name> [dest]`.";

pub(crate) const LOCAL_LIST_AFTER: &str = "\
Example:
  gwz local list

Output columns: name, kind, state, path.";

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
