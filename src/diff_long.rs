pub(crate) const DIFF_LONG: &str = "\
Show changes across the GWZ workspace as one unified, workspace-relative diff.

`gwz diff` behaves like `git diff` over the whole workspace: it diffs the root
repository and each active member and renders one patch with workspace-relative
paths (e.g. `a/gwz-core/src/lib.rs`), root first, then members in manifest
order.

Revisions, ranges (`A..B`, `A...B`), and `+snapshot` ids are passed to the core
untouched and classified per repository; put literal pathspecs after `--`.

Comparison forms:
  gwz diff                     index vs worktree
  gwz diff --cached [<commit>] HEAD (or <commit>) vs index
  gwz diff <commit>            <commit> vs worktree
  gwz diff <a> <b>             tree vs tree
  gwz diff <a>..<b>            tree vs tree
  gwz diff <a>...<b>           merge-base(a,b) vs b
  gwz diff +<snapshot>         a captured snapshot vs the worktree

Patch output is paged on a terminal (honoring $GIT_PAGER/$PAGER, then `less`);
piped or machine output is written directly. `--exit-code` exits 1 when there
are differences; `--quiet` suppresses output and implies `--exit-code`.";
