# hook

Serve another tool's hooks for the workspace a session starts in.

A hook command reads the calling tool's JSON payload on standard input, does
the work that tool asked for, and answers on standard output in the shape the
tool documents. Today one family is served: `gwz hook claude-code`, which
carries the two hooks themselves and the `setup` command that installs them.

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
no environment variable of ours. Each leaf carries only the options it obeys:
removal consumes nothing, so none of the creation guards appear on it.

| Option | Meaning | Leaf |
| --- | --- | --- |
| `--min-free-gb <gb>` | A floor on free space, on top of the estimate (default: no floor; only the estimate applies) | create |
| `--max-lanes <n>` | Ready-lane ceiling (default 8) | create |
| `--wait-secs <secs>` | Deadline for the attempt loop, and for a removal's family lock (default 300) | create, remove |
| `--base-ref <ref>` | Base for the fallback worktree (default `origin/<default-branch>`, else `HEAD`) | create |
| `--log <path>` | Log location, instead of the fixed one | create, remove |

`setup` takes the creation options and bakes them into the handler text; the
`WorktreeRemove` handler it writes carries only `--wait-secs` and `--log`.

## The log

Each hook writes one line per decision: to `<root>/.gwz/claude-hooks.log` in a
workspace and `~/.claude/gwz-lane-hooks.log` otherwise, or wherever `--log`
names. A line carries a timestamp, the event, the name, the session id, the
classification, the path, the outcome and the exit code — never the transcript
path and never the working directory. A location that would appear in
`git status` is not used; the user-level log is used instead and a note says
so. The file is bounded at 1 MB and truncated to its newest half beyond that.

## `gwz hook claude-code setup`

Prints the hooks block, and with `--write` merges it into a settings file or
with `--remove` takes it back out. A placement flag is mandatory:

```sh
gwz hook claude-code setup --project                 # print the block
gwz hook claude-code setup --project --local --write # merge into settings.local.json
gwz hook claude-code setup --user --write            # merge into ~/.claude/settings.json
gwz hook claude-code setup --project --local --remove # take it back out
```

```text
Usage: gwz hook claude-code setup <--project [--local] | --user> [--write | --remove] [OPTIONS]
```

The block carries one `WorktreeCreate` handler and one `WorktreeRemove`
handler, each with its own timeout. Inside a workspace the timeouts and the
handlers' `--wait-secs` are computed from the same run-time estimate the create
hook uses — one wait plus one estimated copy plus 60 s for creation, one wait
plus 60 s for removal. Outside a workspace they are the compiled-in defaults.

The default handler is the bare command `gwz hook ...`, resolved through
`PATH`, so a committed project block is machine-independent and identical
everywhere, which is what Claude Code's same-handler dedupe keys on.
`--command PATH` pins an absolute binary instead, for a machine whose desktop
app cannot see `gwz` on its `PATH`.

`setup` also reminds you to install or refresh the agent skill: copy
`skills/gwz/SKILL.md` to `~/.claude/skills/gwz/`.

### Getting a session into a lane

After `--write`, run `claude --worktree <name>` from the workspace root (or
start a background session). The lane appears in `gwz local list` with the
session id as its owner. Integrate it with
`gwz --target @all merge --remote <name>` from the main workspace, then
`gwz local dispose <name>`. See [Claude Code](../ClaudeCode.md) for the whole
lifecycle.

### Writing and removing

`--write` and `--remove` are the lifecycle pair, and are refused together.
Both edit another program's configuration, so each:

- parses the existing file first and refuses one that does not parse, is a
  symbolic link, or is not a regular file;
- writes a temporary file beside the target, fsyncs it, re-parses it and
  renames it over the original;
- changes no byte outside the block it inserts or the entries it removes.

`--write` in addition creates the file when it is absent and does nothing when
the block is already there. `--remove` takes out the two entries this tool
wrote, drops a `WorktreeCreate` or `WorktreeRemove` array or a `hooks` object
left empty behind them, never deletes the file itself, and says so and changes
nothing when the file does not carry the block. A removal after a write leaves
the file byte for byte as the write found it.

### Placements

One placement is the recommendation. A block may sit in both a project's
settings and a user's: Claude Code runs an identical handler once, so identical
text is harmless. Two *differing* handlers both run, in parallel, for the same
name and session; `setup` warns, naming both files, when it finds one. They
stay harmless for creation — the second handler reuses the first's lane and
prints the same path — but a handler that refuses for a reason of its own makes
Claude abort the creation the other completed, and that lane is an orphan whose
inventory is `gwz local list`.

See [Claude Code](../ClaudeCode.md) for the guide.
