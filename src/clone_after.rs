pub(crate) const CLONE_AFTER: &str = "\
Examples:
  gwz clone git@github.com:org/workspace.git
  gwz clone git@github.com:org/workspace.git work/demo
  gwz clone --local --name A ../gwz-dev-A
  gwz clone --local --clean -b lane/agent-17 --name C ../gwz-dev-C
  gwz clone --local --clean --from A --name B ../gwz-dev-B
  gwz clone --local --bare --name hub ../gwz-dev-hub

If you already ran a plain `git clone` on a workspace root, run
`gwz materialize --lock` inside it to complete the clone instead.

Local clones are listed and retired with `gwz local`.";
