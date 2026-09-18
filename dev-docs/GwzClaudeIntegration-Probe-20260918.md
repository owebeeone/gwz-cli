# GWZ and Claude Code hooks: local adoption and the first probe

Date: 2026-09-18. Plan: `GwzClaudeIntegrationPlan.md`, step **S1.3**. Guide:
`gwz-cli/docs/ClaudeCode.md`. Command reference: `gwz-cli/docs/commands/hook.md`.
Workspace: `/Users/owebeeone/limbo/gwz-dev` (the real one; S1.3 is the step that
adopts it). Binary: `gwz 1.0.14` at `~/.cargo/bin/gwz`. Claude Code
`2.1.247`, macOS 25.6.0, APFS.

Status: **the hook family works end to end on gwz-dev for creation, reuse,
owner refusal, hazard refusal on removal, merge-back and retirement, and for
the whole plain-repository fallback. Four defects and three
documentation mismatches are recorded in §11 and §12 for fold-back into S1.1
and S1.2.** Two probes could not run: the isolation check (U2) and any
lane-side Claude behaviour, because a non-interactive `claude -p` launched from
inside this session cannot authenticate (§5). S1.3 is therefore **partially
complete**: everything the hooks themselves do is evidenced; everything that
needs a live authenticated Claude session in a lane is not, and is listed in
§13 as what still has to run before S1.4.

Owner tokens (session ids) are redacted to `<session>` throughout, except where
one invocation's refusal has to be shown naming *another* invocation's owner, in
which case both are `<session-A>` and `<session-B>`. The scratchpad path that
carries this session's own id is written `<scratch>`.

## 1. Method

Every command below was run from a shell, with `date +%s.%N` around it for wall
time; exit codes, stdout and stderr were captured to separate files. Nothing
was built, in gwz-dev or in the lane (28 GiB free at the start). No source file
in gwz-dev was edited: the only writes were the settings block of §2, the lane
the hook made, the hook's own log, and this note.

Two other lanes (`cleanup-p1b`, `docs-p4`) were being created and used by other
agents throughout. That contention is visible in the timings and is called out
where it matters; it is representative of the real dogfood case, not a defect.

## 2. Step 1: `setup --project --local`

Print only, from the gwz-dev root:

```sh
gwz hook claude-code setup --project --local
```

```json
{
  "hooks": {
    "WorktreeCreate": [
      {
        "hooks": [
          {
            "command": "gwz hook claude-code worktree-create --wait-secs 132",
            "timeout": 324,
            "type": "command"
          }
        ]
      }
    ],
    "WorktreeRemove": [
      {
        "hooks": [
          {
            "command": "gwz hook claude-code worktree-remove --wait-secs 132",
            "timeout": 192,
            "type": "command"
          }
        ]
      }
    ]
  }
}
```

stderr:

```text
gwz: not written; pass --write to merge it
gwz: install or refresh the agent skill too: copy `skills/gwz/SKILL.md` to `~/.claude/skills/gwz/`
```

Exit 0, **9.19 s** — the run-time cost estimate (D6's source walk and reflink
probe) is what the nine seconds buy. `--wait-secs 132`, create timeout 324 s,
remove timeout 192 s: consistent with S1.2's rule (one wait plus one estimated
copy plus 60 s; one wait plus 60 s), given an estimated copy of ~132 s. The
measured first copy was 207 s under three-way contention (§3), so the computed
timeout was *smaller* than the copy actually took here; see F5 in §12.

No differing-handler warning: `~/.claude/settings.json` carries no `hooks` key
(checked; it has only `attribution`, `skipWorkflowUsageWarning`, `theme`), so
this machine has one placement.

`--write` printed the same block and:

```text
gwz: merged WorktreeCreate and WorktreeRemove into /Users/owebeeone/limbo/gwz-dev/.claude/settings.local.json
```

Exit 0, **9.56 s**. `.claude/settings.local.json` afterwards, verbatim:

```json
{
"hooks": {
"WorktreeRemove": [{"hooks":[{"command":"gwz hook claude-code worktree-remove --wait-secs 132","timeout":192,"type":"command"}]}],"WorktreeCreate": [{"hooks":[{"command":"gwz hook claude-code worktree-create --wait-secs 132","timeout":324,"type":"command"}]}]},
  "permissions": {
    "allow": [
      "Bash(find ~/.cargo/registry/src -type d -name \"git2-0.21.0\" 2>/dev/null | head -5)",
      "Bash(gwz --version)"
    ]
  }
}
```

It parses (`json.load` round-trips; keys `hooks`, `permissions`), the bytes
outside the block are untouched, and `.claude/settings.json` was not opened.
The block's *formatting* is the defect F1 of §11: the merge writes one
near-minified line, unindented, with the two events in the opposite order to the
printed block. It is a settings file a human reads and a reviewer diffs, and
S1.4 will put the same writer against the committed `.claude/settings.json`.

`git status --porcelain` at the root: empty.

## 3. Step 2: the create hook by hand

```sh
echo '{"session_id":"<session-A>","name":"probe-20260918","cwd":"/Users/owebeeone/limbo/gwz-dev","hook_event_name":"WorktreeCreate","transcript_path":"/tmp/x"}' \
  | CLAUDE_PROJECT_DIR=/Users/owebeeone/limbo/gwz-dev gwz hook claude-code worktree-create
```

| run | payload | exit | wall | stdout | stderr |
|---|---|---|---|---|---|
| 1 | `<session-A>` | 0 | **207.42 s** | `/Users/owebeeone/limbo/gwz-dev-probe-20260918` (one line, no trailing noise) | empty |
| 2 | `<session-A>`, identical | 0 | **0.0196 s** | the same single line | empty |
| 3 | `<session-B>`, same name | 1 | **0.0176 s** | **0 bytes** | one line, below |

Run 3's stderr, verbatim:

```text
gwz: the lane `probe-20260918` belongs to session <session-A>; choose another worktree name
```

The refusal names the owning session, as D6 requires, and the remedy is
"choose another worktree name". D1's stdout contract held in all three runs:
exactly the path, or nothing at all.

`gwz local list` after run 1 (owner column, other agents' lanes included):

```text
root            checkout  ready  -           /Users/owebeeone/limbo/gwz-dev
cleanup-p1b     checkout  ready  -           /Users/owebeeone/limbo/gwz-dev-cleanup-p1b
docs-p4         checkout  ready  -           /Users/owebeeone/limbo/gwz-dev-docs-p4
probe-20260918  checkout  ready  <session-A> /Users/owebeeone/limbo/gwz-dev-probe-20260918
```

The owner column carries the session id and only the hook-made lane has one:
R20 works, and `local list` is the inventory the plan leans on.

**207 s against an estimated 132 s.** `cleanup-p1b` was in state
`creating/incomplete` when run 1 started and `docs-p4` was created during it;
creations serialise on the family lock, so most of the 207 s was queueing, not
copying. That is exactly the case S1.2's timeout has to survive, and it did not
(F5).

**The 20 ms reuse (F2).** D6 and `hook.md` both say a reuse is allowed only
when "the lane's repositories pass the same completeness check a fresh create
reports". 19.6 ms is not a nine-repository completeness check; the fresh
create's own inventory walk takes seconds (§2's estimate alone took 9 s). Either
the check is cheaper than the documents imply or it is not being run on the
reuse path. It needs a look in S1.1, and a fixture that makes a lane incomplete
and asserts the reuse refuses.

## 4. Step 3: inside the lane

`gwz status` from the lane root: `On branch main`, **2.26 s**, exit 0.
`gwz ls` listed all eight members at their lane paths. Both work.

Every member and the root were on `main` and clean, and **no `worktree-*`
branch was created anywhere** — D9 confirmed by observation.

Lane-local state the copy carried (O2):

- `.gwz/` in the lane is *regenerated*, not copied: it holds `family-root`
  (schema `gwz.family-root/v1`, the family id, `root_path` pointing at
  `/Users/owebeeone/limbo/gwz-dev`), `local-clone-allocation`,
  `local-clone-copy.yml` (233 KB) and an empty `stash/`. The root's
  `local-family.yml`, `local-family.lock`, `claude-hooks.log`,
  `catalog-final/`, `checked-artifacts/`, `locks/` and `merge/` did **not**
  travel. The hook's own log therefore does not follow the lane, which is
  what D10 wants.
- **`.claude/settings.local.json` travelled, byte-identical, hook block and
  all.** A session in the lane would have inherited a `WorktreeCreate` handler
  pointed at the lane, and — more sharply — that copied file is itself a
  disposal hazard (§6 names it explicitly). So is `.claude/.cc-writes/`.
- There is **no `.gwz/url-scheme.yml`** in gwz-dev today, so O2's named example
  could not be observed. O2's real answer from this probe is different and
  larger: what should not travel is `.claude/`, not `.gwz/`.
- The root's managed `.git/info/exclude` block was intact in the lane and
  `git status` was empty in the root and in all eight members, before and
  after.
- Size: `du -sh` reports 105 GB apparent (38 GB of it `target/`), while the
  volume's free space went from 28 GiB to 27 GiB across the whole probe
  including two other agents' lanes. APFS `clonefile` sharing is doing what
  D6's 94% assumes. No exact per-lane delta was isolated — three lanes were
  being created concurrently — so this is corroboration, not a measurement;
  S3.2 still owes the number.

Edit, stage, revert (no commit was made):

```sh
echo "probe scratch" > gwz-cli/PROBE_SCRATCH.txt
gwz status      # ?? gwz-cli/PROBE_SCRATCH.txt ; "gwz-cli: uncommitted work differs from the locked commit"
gwz add gwz-cli/PROBE_SCRATCH.txt   # status: Ok, exit 0
gwz status      # A  gwz-cli/PROBE_SCRATCH.txt
git -C gwz-cli status --short       # A  PROBE_SCRATCH.txt
```

`gwz add` from the lane root stages into the member, `gwz status` reports it and
the lock comparison, and a `gwz commit -m ...` would have had a staged tree to
consume. (`gwz commit` has no `--dry-run`, so "would work" is evidenced by the
staged state, not by a rehearsal.) The file was unstaged and deleted; `gwz
status` and `git -C gwz-cli status --porcelain` were empty again.

## 5. Step 4: a real Claude session

```sh
cd /Users/owebeeone/limbo/gwz-dev
claude --worktree probe-20260918 -p "run: gwz status; then run: git -C gwz-core status --short; report both outputs verbatim"
```

Exit 1 after **0.354 s**, stdout empty, stderr verbatim:

```text
Error creating worktree: WorktreeCreate hook failed: gwz hook claude-code worktree-create --wait-secs 132: gwz: the lane `probe-20260918` belongs to session <session-A>; choose another worktree name
```

What this establishes, and it is a lot:

- **`--worktree` combines with `-p`.** The CLI accepted it; nothing printed a
  restriction. `claude --help` documents `-w, --worktree [name]`.
- **The hook fires, from the project settings block written in step 1, with the
  handler text exactly as written (`--wait-secs 132` is echoed).**
- **The hook's stderr line reaches the user verbatim**, wrapped as
  `Error creating worktree: WorktreeCreate hook failed: <handler>: <our line>`.
  That is the shape S4.1's table should quote.
- **A non-zero create hook aborts the session**, as D8 assumes.
- The hook runs **before** authentication (§5's second run proves the ordering:
  there the hook succeeded and the session then died at 401).

What it does not establish: the session never started, because the lane was
owned by the hand-minted `<session-A>` of step 2 and a real session mints its
own id. **Ordering defect in the step itself, not in gwz:** step 2's hand
invocation and step 4's real session cannot share a lane name, because D6's
reuse rule is keyed on `session_id`. Whoever repeats S1.3 must use two names,
or run step 4 first.

A second attempt to get a live session — `claude --worktree probe-iso -p ...`
in the throwaway plain repository of §8 — created the worktree and then failed:

```text
Failed to authenticate. API Error: 401 OAuth access token has expired. Re-authenticate to continue.
```

A `claude -p` launched from inside a Claude Code session does not inherit usable
credentials. **Nothing that requires a live authenticated session was
observable in this probe**: U2 (the isolation check on `gwz` and `git`), the
"session's working directory is the lane" confirmation, U3, U10 for a running
worktree, and the build-in-the-lane and build-during-copy measurements S1.3 also
asks for. §13 lists them.

## 6. Step 5: the remove hook by hand (U6)

```sh
echo '{"worktree_path":"/Users/owebeeone/limbo/gwz-dev-probe-20260918","hook_event_name":"WorktreeRemove"}' \
  | gwz hook claude-code worktree-remove
```

Exit **1**, wall **7.58 s**, stdout **0 bytes**. `gwz local list` unchanged: the
lane and its row survived, which is D3's intended outcome. stderr was a single
line of **3,556 bytes**; its head and its tail:

```text
gwz: local dispose `probe-20260918` at /Users/owebeeone/limbo/gwz-dev-probe-20260918: unwaived hazard(s): `@root` <dirty>: ignored user data (ignored does not mean disposable) (.claude/.cc-writes/), ignored user data (ignored does not mean disposable), text content (.claude/settings.local.json), ignored user data (ignored does not mean disposable) (.cursor/), ignored user data (ignored does not mean disposable) (.pytest_cache/), ...
```

```text
... `mem_taut_shape_rs` <dirty>: ignored user data (ignored does not mean disposable) (target/); name each accepted loss with --force <hazard,...> to delete, or --keep to detach and retain every file; nothing was removed; effects: []; integrate it first: `gwz merge --remote probe-20260918` from the main workspace, then `gwz local dispose probe-20260918`
```

The whole hazard list, by repository: `@root` (20 entries, "and 3 more"),
`mem_gwz_cli` (one `__pycache__/` plus **4 native stash entries**),
`mem_gwz_core` (11 entries plus **2 native stash entries**), `mem_gwz_py` (13),
`mem_taut` (10), `mem_taut_shape_rs` (`target/`). Every hazard is `dirty`;
none is `open-merge` or `unpreserved-history`.

**U6 is answered.** The message does name the remedy, and names it as the
plan's two-command sequence. It is also the defect F3 of §11: 3.5 KB on one
line is not "one stderr line of the form `gwz: <cause>; <the one command that
resolves it>`", it is the dispose command's full report with the hook's remedy
stapled on. Claude will surface it wrapped in
`Error removing worktree: ...`, so this is what a user gets in their terminal.

Two hazard entries deserve their own line, because they are *ours*:
`.claude/.cc-writes/` and `.claude/settings.local.json` — the latter being the
hook block this step wrote. Local adoption makes its own lanes dirtier. It
changes nothing here, because `.venv/`, `bazel-out`, `target/` and six stash
entries already make gwz-dev's lanes unconditionally dirty, but it is a real
answer to O5: **sessions are not the only thing writing Claude state into a
lane; the adoption method writes some before any session runs.**

## 7. Step 6: the log, and D10

`<root>/.gwz/claude-hooks.log` gained five lines (5,660 bytes total). With the
`message=` field truncated for readability, and session ids redacted:

```text
ts=1789701035668 event=worktree-create name=probe-20260918 session=<session-A> class=lane path=/Users/owebeeone/limbo/gwz-dev-probe-20260918 outcome=created exit=0
ts=1789701041330 event=worktree-create name=probe-20260918 session=<session-A> class=lane path=/Users/owebeeone/limbo/gwz-dev-probe-20260918 outcome=reused exit=0
ts=1789701047808 event=worktree-create name=probe-20260918 session=<session-B> class=refused-by-hook path=- outcome=refused exit=1 message="gwz: the lane `probe-20260918` belongs to session <session-A>; choose another worktree name"
ts=1789701114618 event=worktree-create name=probe-20260918 session=<session-C> class=refused-by-hook path=- outcome=refused exit=1 message="gwz: the lane ... choose another worktree name"
ts=1789701164515 event=worktree-remove name=gwz-dev-probe-20260918 session=- class=refused-by-hazard path=- outcome=refused exit=1 message="gwz: local dispose `probe-20260918` ... [4,716 bytes]"
```

`<session-C>` is the id Claude itself minted for step 4's aborted session; it is
in the log, which is how we know the hook ran with real Claude input.

Confirmed for D10:

- `class=` carries the three classes the plan names: `lane`,
  `refused-by-hook`, `refused-by-hazard`, and (§8) `fallback-worktree`.
- No `transcript_path` and no `cwd` in any line, though both were in the
  payload.
- The log location is ignored: `git check-ignore -v` answers
  `.git/info/exclude:4:/.gwz/`.
- **`git status --porcelain` at the root and in all eight members was empty
  before, during and after every step of this probe.** D10's central claim
  holds.

Three log defects, F4 in §11: the remove line's `name=` is the *directory
basename* `gwz-dev-probe-20260918`, not the lane name `probe-20260918` the
create lines use; `path=-` on the remove refusal although `worktree_path` was
known and canonicalised; and one 4.7 KB line, which at the 1 MB bound means the
log holds ~200 removals' worth of history and truncation cuts mid-story.

## 8. Step 7: merge-back and retirement

```sh
gwz --target @all merge --remote probe-20260918
```

Exit 0, **2.48 s**. A clean no-op, as expected for a lane with no new commits:
`status: Ok`, `state: completed`, `terminal outcome: completed`, `acceptance:
supported-persisted`, `publication: complete`, `participants: total 9;
up-to-date 9`, and for every one of the nine `no changes transferred` with
`recorded ... before X; result X` equal. (The root's `source` was the commit
this session started at and its `recorded before`/`result` a later one: another
agent committed to the root mid-probe. Still `up-to-date`, nothing transferred.)
`git status` at the root stayed empty.

Then, exactly as S3.4 directs:

| command | exit | wall | result |
|---|---|---|---|
| `gwz local dispose probe-20260918` | 1 | 0.012 s | `gwz: OpenOperation: local dispose: local family: family lock /Users/owebeeone/limbo/gwz-dev/.gwz/local-family.lock is held by another operation` |
| `gwz local dispose probe-20260918 --wait 600` | 1 | 26.82 s | `gwz: UnwaivedHazard: ...` — 4,470 bytes, the same hazard list as §6, **`dirty` only** |
| `gwz local dispose probe-20260918 --force dirty,unpreserved-history --wait 600` | 0 | 29.92 s | deleted |

The first row is worth keeping: with other lanes in flight the plain
`gwz local dispose` of S3.4 fails instantly on the family lock, and the
procedure needs `--wait`. S3.4 should say so.

**The step's question — was `unpreserved-history` still needed at 1.0.14?
No.** After the merge, the refusal named `dirty` on six repositories and
nothing else; `unpreserved-history` did not appear for any of the nine. The
identical-copy witness in 1.0.14 does its job, and the S3.4 recipe can drop
that waiver name for a merged lane. It was passed anyway, because the step
said to, and the success line echoes the flags rather than the waivers actually
applied:

```text
status: Ok
deleted local clone `probe-20260918`: /Users/owebeeone/limbo/gwz-dev-probe-20260918 removed, its row removed; forced past: dirty, unpreserved-history
```

Reporting `forced past: unpreserved-history` for a hazard that was never raised
is F6 of §11 — minor, but it is the line an operator pastes into an audit trail.

Afterwards the directory is gone and `gwz local list` reads:

```text
root         checkout  ready  /Users/owebeeone/limbo/gwz-dev
cleanup-p1b  checkout  ready  /Users/owebeeone/limbo/gwz-dev-cleanup-p1b
```

Only the root and one other agent's lane. `docs-p4` had been disposed by its
own owner while this step ran; nothing here touched it.

## 9. Step 8: the fallback, in a plain repository

Repository at `<scratch>/plain-repo`: `git init -b main`, one commit of
`README.md`, `.gitignore` (`secrets.env`) and `.worktreeinclude` (`secrets.env`),
plus an untracked-and-ignored `secrets.env` containing `TOKEN=abc`. No remote,
so the base is `HEAD` (O3's default path, second branch).

| what | exit | wall | result |
|---|---|---|---|
| create, `<session-D>` | 0 | **0.072 s** | prints `<scratch>/plain-repo/.claude/worktrees/probe-plain`; `git worktree list` registers it on `[worktree-probe-plain]`; the worktree holds `README.md`, `.gitignore`, `.worktreeinclude` and **`secrets.env` (10 bytes, `TOKEN=abc`)** — the `.worktreeinclude` copy ran |
| create again, `<session-D>` | 0 | 0.039 s | same path; **spurious stderr**, see below |
| create again, `<session-E>` (different session) | 0 | — | same path, no owner check — D8's "no session check, unlike a lane", confirmed |
| remove `probe-plain2` (clean) | 0 | — | worktree gone from `git worktree list` |
| remove `probe-plain` **dirty** | **1** | — | refusal, below; worktree kept |
| remove `probe-plain` after `git checkout --` | 0 | — | removed; `.claude/worktrees/` left empty |
| remove a path that no longer exists | 0 | — | silent, logged `outcome=absent` |

The dirty refusal, verbatim (paths shortened):

```text
gwz: git worktree remove refused <scratch>/plain-repo/.claude/worktrees/probe-plain: fatal: '<scratch>/plain-repo/.claude/worktrees/probe-plain' contains modified or untracked files, use --force to delete it; commit or discard the changes in that worktree, then remove it by hand
```

D8 holds: never `--force`, a dirty worktree keeps the session, and the remedy is
named. Note the clean removal succeeded *with `secrets.env` still present* —
git tolerates ignored files — so an included secret is deleted with the
worktree, not preserved.

Two things the fallback got wrong, and they are the same bug (**F7**, the most
substantive finding of this probe):

1. On reuse the hook printed
   `gwz: .claude/worktrees/probe-plain/secrets.env is listed in .worktreeinclude and missing from the reused worktree`
   while `secrets.env` was sitting in that worktree. Removing the real file
   changed the message to `gwz: secrets.env is listed ...`. So the warning that
   fired was about a *different* file: the copy inside the worktree, seen as an
   untracked file of the project root at path
   `.claude/worktrees/probe-plain/secrets.env`, matched by the unanchored
   gitignore-style pattern `secrets.env`, and then looked for at
   `<worktree>/.claude/worktrees/probe-plain/secrets.env`, which does not exist.
2. The same enumeration on the *create* path copies those files in. Creating a
   second worktree while the first existed produced:

   ```text
   .claude/worktrees/probe-plain2/secrets.env
   .claude/worktrees/probe-plain2/.claude/worktrees/probe-plain/secrets.env
   ```

   The new worktree contains a copy of the other worktree's copy. With N
   worktrees this nests, and every worktree's included files leak into every
   other worktree. For `.worktreeinclude` patterns that name credentials — the
   documented use — that is a real exposure, not just clutter.

The fix is one line of scope: exclude `<root>/.claude/worktrees/` (and, for
symmetry, the destination itself) from the untracked-file enumeration, and match
the pattern against the project-root-relative path it was written for.

**U10, fallback case: no lock was held.** `.git/worktrees/probe-plain/` and
`.git/worktrees/probe-iso/` contained `HEAD`, `ORIG_HEAD`, `commondir`,
`gitdir`, `index`, `logs/`, `refs/` and **no `locked` file**, for the hand-made
worktrees and for the one `claude --worktree probe-iso` made before it died at
401. As the step anticipated: no Claude session ran to completion, so nothing
took the lock. Whether Claude locks a running agent's worktree — and what
`git worktree remove`'s refusal on a locked worktree reads like, which is the
protection D8 leans on — remains unobserved.

Also observed: `git worktree remove` leaves the `worktree-<name>` branch behind.
Both were deleted by hand afterwards. The plan's "they accumulate until removed
by hand" understates it by one noun: worktrees *and branches*.

The fallback wrote to `~/.claude/gwz-lane-hooks.log`, never into the repository,
and classified correctly as `class=fallback-worktree` — except for the
"path no longer exists" case, which logged
`class=refused-by-hook ... outcome=absent exit=0`. A zero-exit non-refusal
filed under a refusal class is F4's third sibling.

## 10. Step 9: taking the block back out

```sh
gwz hook claude-code setup --project --local --remove
```

Exit 0, **8.73 s**. It printed the block again (on stdout, which is odd for a
removal but harmless) and then:

```text
gwz: removed WorktreeCreate and WorktreeRemove from /Users/owebeeone/limbo/gwz-dev/.claude/settings.local.json
```

`diff` against the copy taken before step 1: **no output**. md5 before and
after: `bb5fdf6869587a943be4f7c65f3e291e` both. Byte-identical, as `hook.md`
promises. `.claude/settings.json` was never touched (verified by content), no
user-level settings were touched, and `git status --porcelain` at the root is
empty.

## 11. Defects, for fold-back into S1.1 and S1.2

- **F1 (S1.2, cosmetic but committed).** `setup --write` merges into an
  existing file as one near-minified, unindented line, and in the opposite
  event order to the block it prints. S1.4 commits this writer's output to
  `.claude/settings.json`. It should emit the same pretty text it prints, in a
  stable event order, and preserve the file's indentation. A fresh file (the
  plain repo, §9) *is* written pretty, so only the merge path is affected.
- **F2 (S1.1).** The reuse path returns in 20 ms, which is not consistent with
  running "the same completeness check a fresh create reports" (D6, `hook.md`).
  Either the documents overstate the check or the check is skipped. Needs a
  fixture that corrupts a lane member and asserts the reuse refuses.
- **F3 (S1.1, D10).** The hazard refusal is 3.5 KB on one stderr line — the
  dispose report in full, with the remedy appended. D10 specifies one line of
  the form `gwz: <cause>; <the one command that resolves it>`. Claude wraps it
  and puts it in the user's terminal. It should be summarised (n hazards across
  m repositories, the class, the remedy) with the full report left to
  `gwz local dispose`, and the log line should carry the summary, not the
  4.7 KB message.
- **F4 (S1.1, D10).** Three log-field faults: the remove line's `name=` is the
  destination's *basename* (`gwz-dev-probe-20260918`) where the create lines
  carry the lane name; `path=-` on a remove refusal whose path was known;
  and `class=refused-by-hook` on the exit-0 `outcome=absent` case, which is not
  a refusal at all.
- **F5 (S1.2).** The computed create timeout (324 s, from a 132 s estimated
  copy) was only 1.6x the copy that actually happened (207 s), and the copy was
  slow because the family lock was held by other lanes — precisely the case
  `--wait-secs` exists for. The estimate models copy time and ignores queueing.
  On a busier machine, or with `--max-lanes` sessions starting together,
  Claude's timeout will kill a copy mid-flight and leave a `creating` row (which
  D6 already describes as needing S3.4). The wait should have a floor, or the
  timeout a multiple.
- **F6 (gwz-core, minor).** `local dispose --force a,b` reports
  `forced past: a, b` — the flags given, not the hazards that were actually
  raised and waived. It should report what it waived.
- **F7 (S1.1, the substantive one).** The fallback's `.worktreeinclude`
  handling does not exclude `<root>/.claude/worktrees/` from the project root's
  untracked-file enumeration. Consequences: a spurious "listed in
  .worktreeinclude and missing" warning on every reuse once the file exists,
  and — on create — copies of *other worktrees'* included files into the new
  worktree, nesting one directory deeper per worktree. Since the documented use
  of `.worktreeinclude` is exactly the untracked credential file, this leaks
  each worktree's secrets into the next. Fix the enumeration's scope and match
  patterns against the project-root-relative path.

## 12. What the plan and the guide should change

Not edited here, per the step. For whoever holds the pen:

- **`GwzClaudeIntegrationPlan.md` S1.3** should say that the hand-run create
  probe and the `claude --worktree` probe must use **different names**, because
  D6's reuse rule is keyed on `session_id` and a real session mints its own
  (§5). As written, step 4 of the step cannot succeed after step 2.
- **S3.4** should carry `--wait <secs>` on the `gwz local dispose` line: with
  any other family operation in flight the bare command fails in 12 ms on the
  family lock (§8).
- **S3.4** should drop `unpreserved-history` from the recommended waiver for a
  **merged** lane at gwz 1.0.14: it is no longer raised (§8). Keep it for an
  unmerged one until measured.
- **Section 2, U6** can be closed: §6 has the text and it names the remedy.
- **Section 2, U11** stays closed; nothing here disturbed it.
- **Open item O2** should be rewritten. `.gwz/url-scheme.yml` does not exist in
  gwz-dev and `.gwz/` is regenerated lane-local anyway. The state that
  travels and should not is **`.claude/`** — `settings.local.json` (the hook
  block itself, under local adoption) and `.cc-writes/` — and both become
  `dirty` hazards at disposal (§4, §6). That also answers half of **O5** early:
  Claude state is inside the lane from the moment it is created, before any
  session writes anything.
- **D6/`hook.md`** should be reconciled with F2 (what the reuse actually
  checks), **D10/`hook.md`** with F3 and F4 (line length and field
  correctness), and **D8/`hook.md`** with F7.
- **`docs/ClaudeCode.md` and `docs/commands/hook.md`** both say "Outside a
  workspace they are the compiled-in defaults" (300 s wait, 600 s timeouts).
  They are not: `setup --project --local --write` in the plain repository of §9
  produced `--wait-secs 1`, `timeout 62` and `timeout 61` — the run-time
  estimate, applied to a three-file repository. Either the documents are wrong
  or the estimate should be gated on being in a workspace. A 1 s wait in a
  large non-GWZ repository is a hair trigger.
- **`docs/ClaudeCode.md`** can now quote §5's wrapper text
  (`Error creating worktree: WorktreeCreate hook failed: <handler>: <line>`) as
  observed, and can state as observed that `--worktree` works with `-p`, that
  the create hook runs before authentication, and that a non-zero create hook
  aborts the session with the hook's own line intact. It should also say that
  `git worktree remove` leaves the `worktree-<name>` **branch** behind, not only
  the worktree.
- The guide's status line still says "Requires a gwz that carries the
  `gwz hook` family (the release after 1.0.13); the installed 1.0.13 does not."
  1.0.14 is installed and carries it.

## 13. What S1.3 still owes, and why

Everything below needs a **live, authenticated Claude session**, which a
`claude -p` launched from inside a Claude Code session cannot be (§5: 401,
expired OAuth token). It has to be run by the operator from their own terminal,
with the block re-written by step 1, against a lane name no hand invocation has
claimed:

- **U2**, the isolation check on `gwz` and on `git -C <member>` inside a lane,
  with the exact refusal text if it refuses. Unobserved. This is the item most
  likely to change the design, because `gwz` runs git internally.
- Confirmation that the session's working directory **is** the lane, and that
  Claude accepts a **member directory inside a lane** as a session root (D8's
  first bullet, the "started inside a member" case). Unobserved.
- **U3** (whether an interactive session's `EnterWorktree` fires the hook),
  **U4** (whether Claude checks for an existing directory before calling the
  hook), **U5**, **U9**: unobserved, all needing a live session or the desktop
  app. **U1** and **U7**'s Claude-side half likewise.
- **U8** (remove hook exits zero with the directory still present) was not
  provoked; the hook never exited zero with the path still there.
- **U10** for a *running* worktree: unobserved (§9 records only that no lock
  exists when no session runs).
- `cargo build -p gwz` in a lane: cost in time and disk. Not run (no builds,
  per the step's own constraint here and 28 GiB free).
- Creation while a `cargo build` runs in the source, and D10's cargo warning:
  not run.
- Creation with a plain git worktree present under `<root>/.claude/worktrees/`
  and D2's warning: not run. (The directory does not exist in gwz-dev, which is
  the state S4.1 will recommend.)
- `gwz` hidden from `PATH`: not run.
- **O4**, whether Claude's own outside-any-repository check is confused by the
  empty `~/.git`: not run. The gwz side is unaffected — every hook invocation
  here resolved correctly.

## 14. Cleanup

- Lane `probe-20260918`: merged (no-op) and disposed; directory gone,
  index row gone (§8). `gwz local list` shows the root and `cleanup-p1b`, an
  agent lane this probe did not touch. `gwz-dev-docs-p4` was disposed by its
  own owner during §8.
- `.claude/settings.local.json`: block removed, byte-identical to its pre-probe
  content, md5 `bb5fdf6869587a943be4f7c65f3e291e` (§10).
- `.claude/settings.json`, `~/.claude/settings.json`, `gwz.conf/`,
  `AGENTS_GWZ.md`: untouched.
- `git status --porcelain` at the gwz-dev root and in all eight members: empty
  throughout the probe. The one exception is this note, which is an untracked
  file in `gwz-cli` and is the lane owner's to commit.
- `<root>/.gwz/claude-hooks.log`: five lines, 5,660 bytes, kept as evidence and
  ignored by `/.gwz/` in the managed exclude block. `~/.claude/gwz-lane-hooks.log`
  gained six lines from the fallback probe and was likewise kept.
- `<scratch>/plain-repo`: throwaway, no worktrees and no `worktree-*` branches
  left; it lives in the session scratchpad and disappears with it.
- Nothing was committed. The scratch edit of §4 was reverted inside the lane
  before disposal.
