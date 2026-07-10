pub(crate) const REPO_CLONE_AFTER: &str = "\
Examples:
  gwz repo clone git@github.com:org/shared-lib.git
  gwz repo clone git@github.com:org/shared-lib.git libs/shared
  gwz --dry-run repo clone git@github.com:org/shared-lib.git libs/shared
  gwz repo clone git@github.com:org/replacement.git libs/shared --member-id mem_replacement";
