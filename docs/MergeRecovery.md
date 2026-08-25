# Merge Recovery Runbook

This page is the operator procedure for a coordinated merge that will not
finish and will not close. It covers every refusal and stuck state GWZ 0.11.0
can leave you in, what each one means, what it did and did not change on disk,
and how to get out.

Read [`gwz merge`](commands/merge.md) first for the ordinary flow: resolve,
`gwz merge --continue`, or `gwz merge --abort`. Come here when one of those
refuses, or when a rollback reported success and the result still looks wrong.

Nothing on this page happens automatically. Every step is something you decide
to do, and several of them cannot be undone.

## Four Rules Before You Touch Anything

1. **Stop other activity in the workspace.** GWZ's mutation lock serializes
   cooperating GWZ commands. It cannot protect a hand operation, an open
   editor, or a raw `git` writer.
2. **Copy before you move.** Take the evidence snapshot in
   [step 2 of the wedged-merge procedure](#step-2-copy-everything-first)
   before any `mv`, `reset`, or `checkout` on this page.
3. **Park and re-checkout; never delete, never hand-edit.** No procedure here
   deletes a merge record, a `refs/gwz/` ref, a `gwz:`-prefixed stash, or a
   stash bundle. Those may be the only surviving copy of your work, and nothing
   in this runbook asks you to remove them.
4. **Never edit `.gitattributes` to get past a refusal.** It is the obvious
   workaround for cases A and C and it is the worst one: it silences the
   refusal without changing a single byte of what GWZ then writes. Each case
   below says what to do instead.

While a merge is open, `gwz forall` is blocked along with the other mutating
commands, so run per-repository commands directly with `git`.

**Shell convention.** Most blocks use `git -C <member> …` so you can stay at
the workspace root. Blocks that need a plain `rm` open with `cd <member>` and
then drop the `-C`. Destructive sequences are chained with `&&` so that a
refused command stops the ones after it — keep the `&&` when you paste.

## Which Case Am I In

| What you saw | Case |
| --- | --- |
| A refusal ending `refusing before any ref or worktree mutation` | [A](#a-refused-a-recovery-checkout-under-a-configured-filter) |
| Abort or rollback reported success, but filtered files hold the wrong content | [B](#b-a-rollback-succeeded-and-left-divergent-files) |
| `participant is neither at the exact rollback before nor after state` | [C](#c-rollback-is-unavailable-in-a-crlf-worktree) |
| `merge '<id>' is open; this command is blocked…` and no merge command can close it | [D](#d-an-open-merge-no-command-can-close) |
| Git LFS files hold pointer text; everything reports clean | [E](#e-lfs-paths-hold-pointer-text) |
| None of the above | [F](#f-stop-and-collect-evidence) |

GWZ prints human errors as `gwz: <Code>: <message>`. Some codes are shared by
several unrelated refusals, so match the cases below on the **message**, not on
the code.

## A. Refused: A Recovery Checkout Under A Configured Filter

### How it presents

```text
gwz: DirtyMember: recovery checkout would rewrite 'secrets/api.key' through configured foreign filter 'crypt' (filter.crypt.clean/process); refusing before any ref or worktree mutation
```

The quoted path is the first covered file the check reached, and the quoted
name is that path's `filter` attribute. There may be more; the check stops at
the first. A driver name that is not valid UTF-8 is reported as
`'<non-utf8 filter name>'`. `DirtyMember` is the general cleanliness-refusal
code and is not specific to this case — the trailing
`refusing before any ref or worktree mutation` is.

### What it means

`gwz merge --abort` and coordinated rollback restore a worktree **blob-exact**:
they write the bytes git has stored, with content filters switched off, because
recovery verification compares those exact bytes. That is safe for git's own
filters and for Git LFS pointers, which all round-trip unchanged.

It is not safe for a filter that rewrites content on the way in — the
encrypt-on-clean family, git-crypt being the common example. Written raw, such
a path is immediately divergent from what the filter would produce, and the
divergence appears only after the branch has already moved.

So GWZ checks first. For every path the checkout would write, it reads the
`filter` attribute and refuses if that driver has a `filter.<name>.clean` or
`filter.<name>.process` command in the effective configuration. `lfs` is
allowlisted by name.

### What was changed

**Nothing.** The check runs before the worktree rewrite and before the branch
ref moves. The merge stays open in exactly its previous state, and every
repository is untouched. Re-running the same command without changing anything
reproduces the same refusal.

### Recovery

Pick one:

- **Find and remove the driver configuration**, run the recovery command, then
  put it back and repair the worktree.

  Both keys matter and either one at any level triggers the refusal, so list
  every occurrence rather than the winning one:

  ```sh
  git -C <member> config --show-origin --get-all filter.crypt.clean
  git -C <member> config --show-origin --get-all filter.crypt.process
  ```

  Unset each occurrence at the level that reported it, then re-run the
  recovery command:

  ```sh
  git -C <member> config --local --unset-all filter.crypt.clean
  git -C <member> config --global --unset-all filter.crypt.clean
  gwz merge --abort
  ```

  Now put the keys back — and repair, because the covered paths hold the
  stored form. **This step is mandatory**, and the obvious command for it does
  not work: see below.

- **Run the operation from a checkout that does not configure the driver.**
  The attribute alone never triggers the refusal; the configured `clean` or
  `process` command does. The same repair applies afterwards.

Do not instead edit `.gitattributes` in the worktree to hide the coverage from
the check. That suppresses the refusal without changing what gets written, and
lands you in [case B](#b-a-rollback-succeeded-and-left-divergent-files) with no
record of why.

### The mandatory repair, and why `git checkout --` alone does nothing

After a filters-off recovery checkout, git's index holds the *raw* file's size
and timestamp. Git therefore believes the path is up to date and will not run
the filter again. Three things follow, and all three are traps:

- `git status` prints **nothing**, however wrong the content is;
- a bare `git checkout -- <paths>` is **skipped as up to date** and changes
  nothing;
- a `git status` check afterwards **reports success on the unrepaired
  worktree**.

Remove the files first. That is what forces git to re-materialize them through
the filter:

```sh
cd <member>
rm <paths> &&
git checkout -- <paths>
```

Verify by content, not by `git status`:

```sh
head -c 120 <path>
```

To sweep a whole member, defeat the stat cache first — touching the files
makes git re-check them:

```sh
cd <member>
git ls-files -z | xargs -0 touch
git status --short          # anything listed still holds the stored form
```

This is the same repair as
[case B](#b-a-rollback-succeeded-and-left-divergent-files), performed
deliberately instead of discovered later. Do not skip it.

### The deliberate false positives

GWZ refuses on the *presence* of a configured `clean`/`process` command, not on
proof that the driver changes content. It cannot run the driver to find out. A
custom filter that is genuinely idempotent — a formatter that is stable on
already-formatted bytes, a pass-through wrapper — is refused too.

That is intended. Failing closed costs you one manual step; guessing wrong
costs a silently divergent worktree under a moved branch.

## B. A Rollback Succeeded And Left Divergent Files

### How it presents

`gwz merge --abort` (or a coordinated rollback) reports **success**, and every
status command you can think of agrees: `gwz status`, `gwz merge --status`,
and plain `git status` are all clean. What is wrong is the *content* of the
covered files — they hold the stored form rather than the form the filter
should produce.

**This case has no ordinary symptom.** A plain `git status` cannot see it: the
index holds the raw file's size and timestamp from the filters-off checkout, so
git believes the path is up to date and never re-runs the filter. To make git
look, touch the files first:

```sh
cd <member>
git ls-files -z | xargs -0 touch
git status --short
# M  config/secrets.env
```

Anything listed there holds the stored form. Looking at the file works too:

```sh
head -c 120 <path>
```

Suspect this case when all of the following were true of the rollback:

- `.gitattributes` was itself among the files the rollback restored;
- the `filter=` coverage for the affected paths exists **only in the restored
  state** — the checkout brought the coverage back together with the bytes;
- the driver is configured (`filter.<name>.clean` or `.process`).

The usual direction — rolling back a change that *added* coverage — is caught
by [case A](#a-refused-a-recovery-checkout-under-a-configured-filter) instead
and refuses safely. Reaching this case takes the inverse: rolling back across a
merge that had **deleted** the covering `.gitattributes`.

### What was changed

The rollback completed. The branch ref moved and the worktree was rewritten.
The affected paths hold git's stored bytes, while the freshly restored
`.gitattributes` and the configured driver mean git now expects the filtered
form on disk.

Coordinated merge state is consistent; the divergence is confined to those
paths in that worktree.

### Why everything reports clean

GWZ's own status does not execute configured filter commands, so those paths
look unchanged to it. In this release GWZ cannot detect this shape in advance
and cannot see it afterwards.

Real `git` is blind to it too, for a different reason: the filters-off checkout
left the raw file's size and timestamp in the index, so git treats the path as
up to date and never re-runs the filter. That is why the detection above has to
`touch` the files first, and why the repair below has to `rm` them. Neither
step is optional.

### Recovery

Remove the affected paths, then check them out again. The removal is what
forces git to re-materialize them through the filter — **a bare
`git checkout -- <paths>` silently does nothing here**, and a `git status`
afterwards will report success whether or not the repair happened.

```sh
cd <member>
rm <paths> &&
git checkout -- <paths>
```

Verify by content:

```sh
head -c 120 <path>
```

Or re-run the whole-member sweep from the detection step above and expect it to
list nothing.

This overwrites the working copy of those paths. If you have real edits there,
save them first.

## C. Rollback Is Unavailable In A CRLF Worktree

### How it presents

`gwz merge --abort` refuses with a message of the form:

```text
gwz: MergeRecoveryRequired: participant is neither at the exact rollback before nor after state
```

`--json` output carries `member_id` and `member_path` naming the repository.

### What it means

GWZ verifies this rollback by comparing worktree bytes against the recorded
commits exactly. In a worktree that was materialized through a line-ending
filter, the bytes on disk are the smudged form and match neither recorded
commit, so GWZ cannot tell which state it is looking at and stops.

GWZ is refusing because it cannot prove the state, not because something is
broken.

### Which repositories are affected

- **Adopted and legacy worktrees only** — repositories created before GWZ
  0.11.0, or cloned into the workspace by ordinary `git`. Repositories GWZ
  creates or clones from 0.11.0 onward get repo-local `core.autocrlf=false` and
  `core.eol=lf` pinned at birth, and a GWZ clone materializes its first
  worktree with filters off, so those repositories stay blob-exact. Check with
  `git -C <member> config --get core.autocrlf`: the pins are ordinary config
  and a later hand edit can undo them.
- **The trigger is the filter configuration, not the operating system.**
  `core.autocrlf=true` is the common Windows default, which is where this is
  usually met, but any host carrying that setting is affected, and the
  attribute-driven forms (`eol=crlf`, `ident`, a foreign `filter=`) apply
  everywhere.
- **This is the `--no-ff` recovery path.** `--no-ff` starts always use it; an
  ordinary start can also reach it once its record is migrated during abort.
  Match on the message, not on how you started the merge.

### What was changed

**Nothing.** The classification happens before any rollback mutation.

### First: is the smudge from config, or from an attribute?

The repair below resets `core.autocrlf` and `core.eol`. It cannot touch
attribute-driven smudge, and if you run it against an attribute-driven member
it will **report success and change nothing**. Check before you start:

```sh
git -C <member> check-attr -a -- <path>
```

If that reports `eol: crlf`, `ident: set`, or a `filter:` value, the repair
below does not apply to this member. Go to
[case F](#f-stop-and-collect-evidence). Do not "fix" it by editing
`.gitattributes` — rule 4.

### Recovery, for the config-driven case

> **`git reset --hard` destroys uncommitted work.** Staged changes and
> unstaged edits to tracked files are gone; untracked and ignored files
> survive. Commit anything you want to keep **before** you paste the block.
> The block refuses to start if it finds either, but read this first.

Apply this only to a member that is not mid-conflict. A conflicted member is a
*normal* way to reach this case, so the first two lines of the block exist to
catch exactly that.

```sh
cd <member>

# 1. Guard. STOP if either line prints anything:
#      the first means uncommitted work  - commit it, or go to case F
#      the second means a live conflict  - go to case F, do not continue
git status --porcelain -uno
git ls-files -u

# 2. Only when both printed nothing. Chained with && so that a refusal stops
#    everything after it. `git rm --cached -r .` is INDEX-ONLY: it empties the
#    index and does not delete a single file.
git config core.autocrlf false &&
git config core.eol lf &&
git rm --cached -r . &&
git reset --hard
```

**If you stop between the last two commands**, the index is empty and every
file reports as both staged-deleted and untracked. Nothing has been deleted —
the files are all still there. Run `git reset --hard` to put the index back. In
that state do not run `git clean`, `git commit -a`, or `git stash`: each of
them would turn an intact worktree into a real deletion.

Verify by bytes, because `git status` is green either way:

```sh
git -C <member> ls-files -z | xargs -0 file | grep CRLF   # expect no output
```

Then retry `gwz merge --abort`.

The alternative — re-cloning the member through GWZ, which materializes it
blob-exact — needs `gwz repo` or `gwz materialize`, and both are blocked while
a merge is open. It is a fix for the general condition, not for a merge you are
currently stuck in. Use it once the workspace is unblocked, so the next merge
is not exposed.

> A command that performs this normalization for you is planned. **It is not
> part of 0.11.0.** Use the manual steps above.

## D. An Open Merge No Command Can Close

### How it presents

Every mutating command refuses:

```text
gwz: OpenOperation: merge 'merge_20260725_1234' is open; this command is blocked until it is recovered; use merge status, merge continue, or merge abort
```

Blocked while a merge is open: `commit`, `capture`, `snapshot`, `pull`, `push`,
`materialize`, `forall`, `init --update`, branch/tag/stash/repo mutation, and
starting another merge. Among those still available are `status`, `ls`, `diff`,
the list commands, `gwz merge --status`, and `gwz add` for staging conflict
resolution in a repository the record already lists as conflicted.

`gwz merge --gc` also runs while a merge is open — but it **deletes archived
records**, including ones step 4 below depends on. Do not run it while you are
working through this page.

You are in this case when the suggested commands cannot move it either:

- `gwz merge --continue` and `gwz merge --abort` both refuse, because the live
  state can no longer be verified — a member checkout was deleted or moved,
  history was force-rewritten and the recorded commits pruned, a preservation
  stash was dropped, or publication files were hand-edited; or
- discovery itself fails, so even `gwz merge --status` dies:

  ```text
  gwz: MergeRecoveryRequired: multiple merge records exist under '<workspace>/.gwz/merge'
  ```

  or a `MergeRecordUnreadable` error naming a record GWZ cannot decode.

### What it means

A coordinated merge is durable. Its record lives at
`<workspace>/.gwz/merge/<merge-id>.yaml`, and while that file is there GWZ
protects the half-finished operation by blocking coordinated mutation. When the
world can no longer produce the exact state the record expects, the operation
cannot be finished or undone — and the block does not lift by itself.

The procedure below parks the record so the workspace becomes usable again,
preserving every byte of evidence. It does not finish the merge and it does not
undo it.

There is deliberately no command that force-abandons a merge record. A record
that cannot be completed or undone stays parked; parking it **is** the end
state, not a step toward one.

### Step 1: Stop everything

No GWZ command, no `git` command, no editor writing into the workspace, until
the procedure is done.

### Step 2: Copy everything first

Nothing further is permitted until this exists.

```sh
# Records: open, archived, and anything already parked.
cp -a <workspace>/.gwz/merge /somewhere/outside/merge-backup

# Preservation bundles, if the directory exists.
cp -a <workspace>/.gwz/stash/bundles /somewhere/outside/bundles-backup

# In the workspace root repository AND in every member — save the output.
git -C <path> for-each-ref refs/gwz/
git -C <path> stash list

# Confirm the copies actually exist before going any further.
diff -r <workspace>/.gwz/merge /somewhere/outside/merge-backup   # expect no output
```

If `gwz merge --status` still runs, save its output too, in both forms — it
prints the recorded pre-merge commits you will need in step 5:

```sh
gwz merge --status
gwz --json merge --status
```

If it does not run — which is the case when discovery itself failed — you are
not stuck. Those same commits are inside the record's own YAML, which you have
just copied and can read at any time, and each preserved branch tip is reachable
through its `refs/gwz/merge/…/head` backup ref. Step 5 says how to use both.

### Step 3: Park the record

Move it. Do not delete it, and do not edit it in place.

```sh
mkdir -p <workspace>/.gwz/merge/quarantine
mv <workspace>/.gwz/merge/<merge-id>.yaml \
   <workspace>/.gwz/merge/quarantine/<merge-id>.yaml
```

GWZ's open-record discovery reads only `*.yaml` files sitting **directly**
inside `.gwz/merge/`. A file under `quarantine/` is invisible to it, exactly as
archived records under `done/` are. The block lifts, ordinary workspace
commands work again, and the record bytes, backup refs, stashes, bundles, and
member worktrees are all untouched.

Two warnings:

- **The protection is gone, not the problem.** Member repositories stay in
  whatever half-merged state they were in: branches may sit at merged commits
  that were never published, and the workspace lock is still at its pre-merge
  baseline. Deal with the members deliberately in step 5.
- **Keep `.gwz/merge/` clean.** Anything named `*.yaml` directly in that
  directory is treated as an open merge record — a file or a directory alike.
  Put notes, copies, and exports somewhere else entirely.

### Step 4: More than one record

If discovery reported multiple records, move them one at a time and decide
about each one separately.

If the same id exists in both `.gwz/merge/<id>.yaml` and
`.gwz/merge/done/<id>.yaml` with different contents, an archive step was
interrupted. GWZ keeps both precisely so a human can choose. The open copy is
the authoritative one unless it is the damaged one. Move the copy you reject
into `quarantine/` under a distinct name — for example
`quarantine/<id>.rejected.yaml`. Never delete either.

### Step 5: Cleaning up inside a member

This is the complete list of what GWZ owns inside a member repository, and the
only things there that are GWZ's to remove at all:

- refs named `refs/gwz/merge/<merge-id>/<owner-key>/head`, where `<owner-key>`
  is the member id or `root`;
- one native git stash whose message is exactly
  `gwz:stash_<merge-id>: merge preservation`.

Nothing else in a member repository belongs to GWZ — and nothing in this
runbook asks you to remove even these two. They are the preserved copies of
your work. If you ever do delete them, do it only once that work is somewhere
else.

Half-merged integration branches are **your** branches, not GWZ's. To put one
back you need its recorded pre-merge commit. Three places have it: the
`gwz merge --status` output you saved in step 2, the parked record's own YAML
(copy it out and read the participant's `before_commit`), and — for work that
was preserved — the backup ref, which is a usable branch point on its own.

```sh
git -C <member> status
git -C <member> log --oneline -5
git -C <member> reset --hard <before-commit>
```

`git reset --hard` discards work. Check the log, `git stash list`, and any
`refs/gwz/merge/…/head` backup ref before you run it — a backup ref is a
branch point:

```sh
git -C <member> branch recovered-work refs/gwz/merge/<merge-id>/<owner-key>/head
```

### Step 6: Never touch

- anything under a member's `.git/` other than the two items in step 5;
- `.gwz/merge/done/`, the archived records;
- the bytes you parked under `quarantine/` — copy them out to read them;
- `.gwz/stash/bundles/`, which may be the only surviving record of preserved
  work.

### Step 7: Re-entry, if you repair the world

Parking is reversible. If you later restore what was missing — re-clone the
deleted member from its remote, recover the stash — you can move the parked
record back into the open slot.

Two checks first, both of which must pass:

```sh
# 1. The bytes must be the ones you parked. Compare against the step-2 copy.
diff <workspace>/.gwz/merge/quarantine/<merge-id>.yaml \
     /somewhere/outside/merge-backup/<merge-id>.yaml       # expect no output

# 2. The open slot must be empty. Restoring alongside another record turns a
#    working workspace into the multiple-records wedge.
ls <workspace>/.gwz/merge/*.yaml                           # expect no output
```

Then move it back:

```sh
mv <workspace>/.gwz/merge/quarantine/<merge-id>.yaml \
   <workspace>/.gwz/merge/<merge-id>.yaml
```

Resume with the ordinary `gwz merge --continue` or `gwz merge --abort`; whether
they can proceed still depends on the live repositories matching the record.

**Editing a record's YAML by hand and putting it back is not supported.** The
record's internal consistency is what makes recovery safe; a hand-edited record
has undefined behaviour. If you edited it, leave it parked — and remember that
parked is a legitimate final state, not a failure to finish.

## E. LFS Paths Hold Pointer Text

### How it presents

Files that Git LFS manages contain the short LFS pointer text instead of the
real content. Both `git status` and `gwz status` report clean. This happens in
two situations, and they need different commands:

- **after GWZ cloned or materialized the repository** — the LFS objects were
  never downloaded at all;
- **after a recovery checkout touched LFS-managed paths** — the objects are
  normally already in the repository, only the working copy is the pointer.

### What it means

GWZ materializes and restores blob-exact — the bytes git has stored. For an
LFS-managed path that is the pointer. Pointers round-trip through the LFS
filter unchanged, so nothing is dirty and nothing is inconsistent; only the
working copy's content is a surprise. This is why `lfs` is allowlisted by the
refusal in [case A](#a-refused-a-recovery-checkout-under-a-configured-filter):
it cannot wedge anything.

No merge state is involved and nothing needs recovering.

### Recovery

**After a GWZ clone or materialize**, the objects have to be fetched before
anything can be written. `git lfs checkout` only materializes objects that are
already local, so it is not the command here:

```sh
git -C <member> lfs pull
```

**After a recovery checkout**, the objects are already present and a checkout
is enough:

```sh
git -C <member> lfs checkout
```

If you are not sure which situation you are in, this form works for both,
because the smudge filter fetches what it needs:

```sh
cd <member>
rm <paths> &&
git checkout -- <paths>
```

## F. Stop And Collect Evidence

If what you are seeing does not match A-E, collect evidence **before** touching
anything. Save the exact error text, including the `gwz: <Code>:` prefix.

```sh
gwz merge --status
gwz --json merge --status
gwz status

# Per repository — the workspace root and every member.
git -C <path> status
git -C <path> log --oneline -5
git -C <path> for-each-ref refs/gwz/
git -C <path> stash list

# The record and artifact plane.
ls -la <workspace>/.gwz/merge
ls -la <workspace>/.gwz/merge/done
ls -la <workspace>/.gwz/stash/bundles
```

Then stop. In particular, do not:

- delete a merge record, or edit one in place;
- delete a ref under `refs/gwz/`, a `gwz:`-prefixed stash, or a stash bundle;
- run `git reset --hard`, `git rm --cached`, `git checkout --`, `git clean`, or
  `git stash drop` in a member before you know which case you are in;
- edit `.gitattributes` to make a refusal go away;
- move a record before the copy in
  [step 2](#step-2-copy-everything-first) exists.

With the evidence above in hand, the parking procedure in
[case D](#d-an-open-merge-no-command-can-close) is the general unblock: it is
reversible, it destroys nothing, and it leaves every artifact available for a
later decision.
