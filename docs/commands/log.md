# `gwz log`

Show the selected workspace repositories as one local, newest-first commit
stream. The default selection is the workspace root plus all active members;
the standard `--target`, `--no-target`, `--member`, and `--all` selectors
narrow it in the usual way.

```sh
gwz log
gwz log -n 20
gwz log main..topic
gwz log +release..HEAD -- src
gwz --target mem_api log --since 2026-08-01T00:00:00Z
```

With no explicit depth or filter, the global result is capped at 50 entries
after filtering and workspace coalescing. `-n <n>` sets an explicit global
cap; `-n 0` and `--no-limit` disable it. An explicit revision range or a
`--since`/`--until` bound automatically lifts only the default cap so core can
search the complete requested history. An explicit `-n` remains authoritative.

Snapshot operands begin with `+`. In range position, `+lock..HEAD` resolves
each selected repository's accepted workspace-lock revision. For compatibility
with released snapshot ids, bare `+lock` is still an ordinary snapshot named
`lock`; it does not mean the lock pseudo-endpoint outside a range. Put Git
pathspecs after `--` so they cannot be mistaken for revision operands.

The compact default is one line per workspace-level change:

```text
2026-08-31 10:42:17 +1000 [., members/api] 4b8c1a73d995 Add log rendering
```

Sets of at most three repositories show their complete workspace-relative
paths. Larger sets use a count such as `[root+5]`. Coordinated commits may
coalesce into one entry, while `--no-coalesce` shows raw per-repository
commits. Use `--full` for git-style blocks with a complete member table, and
add `--body` when commit bodies are needed.

Machine modes are selected with `gwz --json log ...` or
`gwz --jsonl log ...`. They use the dedicated `gwz.log/v0` record schema, not
the generic operation-response envelope. Entry provenance is one of `none`,
`heuristic`, `marker:<uuid-v7>`, or `marker-invalid`; the last token means a
marker-like claim was present but unusable, so that commit is deliberately a
singleton. Degradation records carry stable reason tokens. See
[Machine Output](../MachineOutput.md#commit-log-output) for the complete record
shape.

Filters are evaluated by the shared core engine before histories are merged:
`--since`, `--until`, `--author`, `--grep`, `--no-merges`, and
`--first-parent`. Rust regular-expression syntax is used for author and
message filters. Dates accept RFC3339/ISO-8601 forms or `@epoch-seconds`.

If a selected repository cannot contribute to the requested history, its
degradation is summarized on stderr while surviving entries remain on stdout.
Benign degradations keep exit 0; unreadable selected repositories produce exit
1. `--strict` promotes any degradation to exit 1. Invalid requests and
unreadable workspaces use exit 2.

Human text is rendered lossily when Git contains invalid UTF-8, and control
characters are sanitized before reaching a terminal. Dates retain the
commit's recorded UTC offset and lines are never width-truncated.

`gwz log` deliberately does not use a pager. Color defaults to
`--color=auto`, which enables ANSI styling only when stdout is a terminal;
`--color=always` and `--color=never` override it. See `gwz help log` or the
generated [CLI Reference](../CLI.md#gwz-log) for the complete flag surface.
