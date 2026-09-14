pub(crate) const PUSH_LONG: &str = "\
Push workspace target refs to configured remotes.

`gwz push` applies one push request across selected workspace targets. By
default that includes the workspace root (`@root`) plus configured member
repositories. Use `--remote` to choose a remote name and selectors such as
`--target`, `--member`, `--member-path`, `--all`, and `--no-target @root` to
control which targets participate.

Publication:
  - Root dependencies are proven by this operation's own reads or accepted
    pushes, never by remote-tracking refs.
  - By default, repositories unchanged since the last fetch or push are not
    checked for changes or pushed; human output counts them in one summary
    line, and `--verbose` shows each reason. A push that contacts the root
    still reads each dependency.
  - `--check-remotes` reads every selected remote and every root dependency,
    pushes repositories whose remote lacks their branch's commit, and proves
    a selected root even when it has nothing to push.";
