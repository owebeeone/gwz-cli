pub(crate) const CLONE_AFTER: &str = "\
Examples:
  gwz clone git@github.com:org/workspace.git
  gwz clone git@github.com:org/workspace.git work/demo
  gwz clone --url-scheme https git@github.com:org/workspace.git
  GWZ_URL_SCHEME=https gwz clone https://github.com/org/workspace.git

If you already ran a plain `git clone` on a workspace root, run
`gwz materialize --lock` inside it to complete the clone instead.";
