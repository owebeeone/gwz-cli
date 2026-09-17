# hook

Serve another tool's hooks from this workspace.

A hook command reads the calling tool's JSON payload on standard input, does
the work that tool asked for, and answers on standard output in the shape the
tool documents. Today one family is served: `gwz hook claude-code`.

## `gwz hook claude-code worktree-create`

Reads Claude Code's `WorktreeCreate` payload and prints the path of the working
copy the session should use. One test decides what it makes:

- **A GWZ workspace** gets a local clone of the whole workspace at
  `../<root-dirname>-<name>`, registered in the local clone family. When the
  session started inside a member, the printed path is that member's directory
  inside the lane.
- **Anything else** gets the worktree Claude Code would have made itself: a
  `git worktree` under `.claude/worktrees/<name>` on branch `worktree-<name>`,
  from `origin/<default-branch>` when it resolves and `HEAD` otherwise, with
  the `.worktreeinclude` copy performed for a worktree the hook created.

The contract is narrow on purpose: the hook prints only a path it created or
verified in this invocation, and nothing else ever reaches standard output.
Every failure is one line on standard error, `gwz: <cause>; <remedy>`, with a
non-zero exit and an empty standard output, which makes Claude Code abort the
creation.

### Reuse

An existing lane of the same name is reused only when all of these hold: the
family index has a `ready` row for the name, its recorded path is the
destination, the row belongs to this session, and the lane passes the
completeness check. Every other state is refused, naming the state and its
remedy — an unfinished create, a lane at another path, a lane owned by another
session, a lane that is incomplete.

### Guards

Two guards protect the copy, and neither applies to a reuse:

- Free space on the filesystem holding the destination's parent, against a
  run-time estimate of what the copy costs: a walk of the source for apparent
  size and file count, a block-sharing probe at the destination's parent, and a
  pessimistic share by filesystem. `--min-free-gb` is a floor applied on top of
  the estimate.
- `--max-lanes` (default 8) ready rows in the family.

## `gwz hook claude-code worktree-remove`

Reads the `WorktreeRemove` payload, canonicalises `worktree_path` and
classifies it. A registered git worktree of the project is removed with
`git worktree remove`, never `--force`. A lane root, or a member directory
inside one, is disposed with `gwz local dispose` from the family root, never
`--keep` and never `--force`: a hazard refusal keeps the lane and the session.
A path that no longer exists exits zero. Anything else is refused.

## Options

Every knob is an option on the handler itself, because the desktop app passes
no environment variable of ours.

| Option | Meaning |
| --- | --- |
| `--min-free-gb <gb>` | A floor on free space, on top of the estimate |
| `--max-lanes <n>` | Ready-lane ceiling (default 8) |
| `--wait-secs <secs>` | Deadline for the attempt loop and a removal's lock (default 300) |
| `--base-ref <ref>` | Base for the fallback worktree |
| `--log <path>` | Log location, instead of the fixed one |

## The log

Each hook writes one line per decision: to `<root>/.gwz/claude-hooks.log` in a
workspace and `~/.claude/gwz-lane-hooks.log` otherwise, or wherever `--log`
names. A line carries a timestamp, the event, the name, the session id, the
classification, the path, the outcome and the exit code — never the transcript
path and never the working directory. A location that would appear in
`git status` is not used; the user-level log is used instead and a note says
so. The file is bounded at 1 MB and truncated to its newest half beyond that.
