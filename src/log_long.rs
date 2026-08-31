pub(crate) const LOG_LONG: &str = "\
Show local commit history from the workspace root and selected members as one
newest-first stream. Coordinated workspace commits may appear as one entry
attributed to several repositories.

The default compact form is `<date> <member-set> <short-hash> <subject>`.
Small member sets show their workspace-relative member paths; larger sets use
a count such as `[root+5]`. `--full` uses git-style blocks with a complete
member table. `--body` includes commit bodies in full blocks.

Members that cannot contribute are summarized on stderr while surviving
history remains on stdout. `--strict` promotes any such degradation to a
failure. Dates use each commit's recorded offset, and human text is rendered
lossily with terminal control characters sanitized.

Output does not use a pager. `--color=auto` enables ANSI color only when
stdout is a terminal; use `always` or `never` to override.";
