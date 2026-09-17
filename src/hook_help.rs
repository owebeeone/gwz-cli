pub(crate) const HOOK_LONG: &str = "\
Serve another tool's hooks from this workspace.

A hook command reads the calling tool's JSON payload on standard input, does
the work the tool asked for, and answers on standard output in the shape that
tool documents. Hooks are meant to be run by the tool, not by hand; running one
by hand is still useful for a probe, and every hook reads its input the same
way whoever calls it.

Today one family is served: `gwz hook claude-code`.";

pub(crate) const HOOK_CLAUDE_CODE_LONG: &str = "\
Serve Claude Code's WorktreeCreate and WorktreeRemove hooks.

Claude Code creates a git worktree of the project when a session asks for an
isolated working copy. In a GWZ workspace the members are separate
repositories, git-ignored by the root, so such a worktree has no members and
nothing builds in it. These two commands replace that default: a GWZ workspace
gets a local clone of the whole workspace (`gwz local clone`), and any other
project gets the plain git worktree Claude Code would have made.

worktree-create prints only a path it created or verified in this invocation.
worktree-remove deletes only the lane or worktree that `worktree_path`
canonically names. Nothing else is ever printed, and nothing else is ever
deleted. Every failure is one line on standard error, `gwz: <cause>; <remedy>`,
and a non-zero exit with nothing on standard output.

Write the settings block with `gwz claude-code setup`.";

pub(crate) const HOOK_CLAUDE_CODE_AFTER: &str = "\
Examples:
  gwz claude-code setup --project --local --write
  echo '{\"name\":\"fix-123\",\"session_id\":\"abc\"}' | gwz hook claude-code worktree-create

Each hook logs one line per decision: to `<root>/.gwz/claude-hooks.log` in a
workspace, and to `~/.claude/gwz-lane-hooks.log` otherwise, or wherever --log
names. A location that would appear in `git status` is never used.";

pub(crate) const HOOK_CREATE_LONG: &str = "\
Create or verify the working copy Claude Code asked for, and print its path.

Reads Claude Code's WorktreeCreate payload on standard input and takes `name`
and `session_id` from it. The name is validated as a slug of [A-Za-z0-9._-]
that is not a name GWZ reserves.

In a GWZ workspace the result is a local clone of the workspace at
`../<root-dirname>-<name>`, created in process, and the printed path is that
lane — or, when the session started inside a member, that member's directory
inside the lane. An existing lane of the same name is reused only when it is
ready, sits at that destination, belongs to this session and passes the
completeness check; every other state is refused with its own remedy.

Two guards protect the copy: free space against a run-time estimate of what
the copy costs (a walk of the source, a block-sharing probe at the
destination's parent, and a pessimistic share by filesystem), with
--min-free-gb as a floor on top of it; and a ceiling on how many ready lanes
the family may hold. Neither applies to a reuse, which consumes nothing.

Outside a workspace the hook reproduces Claude Code's own default: a worktree
under `.claude/worktrees/<name>` on branch `worktree-<name>`, from
`origin/<default-branch>` when it resolves and `HEAD` otherwise, with the
`.worktreeinclude` copy performed for a worktree this invocation created.";

pub(crate) const HOOK_REMOVE_LONG: &str = "\
Retire the lane or worktree that `worktree_path` names.

Reads Claude Code's WorktreeRemove payload on standard input, canonicalises
`worktree_path` and classifies it. A registered git worktree of the project is
removed with `git worktree remove`, never with --force, so a dirty or locked
worktree keeps the session. A lane root, or a member directory inside one, is
resolved to its family and disposed with `gwz local dispose`, never with
--keep and never with --force: a hazard refusal keeps the lane and the session,
which is the safe outcome. A path that no longer exists exits zero. Anything
else is refused.";

pub(crate) const CLAUDE_CODE_LONG: &str = "\
Set this machine up to run GWZ's Claude Code hooks.

`gwz claude-code setup` prints the hooks block, and with --write merges it
into `.claude/settings.json` of the workspace root (--project), its
`settings.local.json` (--project --local), or `~/.claude/settings.json`
(--user).";

pub(crate) const CLAUDE_CODE_SETUP_LONG: &str = "\
Print the Claude Code hooks block, and with --write merge it into a settings
file.

The block carries one WorktreeCreate handler and one WorktreeRemove handler,
each with its own timeout. Inside a workspace the timeouts and the handlers'
--wait-secs are computed from the same run-time estimate the create hook uses;
outside one they are the compiled-in defaults.

The default handler is the bare command `gwz hook ...`, resolved through PATH,
so a committed project block is machine-independent and identical everywhere,
which is what Claude Code's same-handler dedupe keys on. --command pins an
absolute binary instead, for a machine whose desktop app cannot see `gwz` on
its PATH.

--write edits another program's configuration, so it parses the existing file
first and refuses one that does not parse or is not a regular file; writes a
temporary file beside the target, fsyncs it, re-parses it and renames it over
the original; changes no byte outside the inserted block; creates the file when
it is absent; and does nothing when the block is already there.";

pub(crate) const CLAUDE_CODE_SETUP_AFTER: &str = "\
Examples:
  gwz claude-code setup --project
  gwz claude-code setup --project --local --write
  gwz claude-code setup --user --write --command /usr/local/bin/gwz

One placement is the recommendation. Two placements are harmless when the
handler text is identical, because Claude Code runs an identical handler once;
two differing handlers both run, and a refusal by either orphans the lane the
other created.";
