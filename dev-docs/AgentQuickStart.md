# Agent Quick Start — constructing and operating GWZ workspaces

Audience: an AI agent driving `gwz` non-interactively. This is the delta over
[docs/QuickStart.md](../docs/QuickStart.md) (human-paced tour) and
`AGENTS_GWZ.md` (entry-only bootstrap: install / materialize / status). Read
those for concepts; read this for verified semantics, machine-mode operation,
and verification-at-each-step recipes.

Verified against `gwz 0.9.0` (2026-07-11) by constructing a real workspace.
Re-verify the "Verified semantics" table if the version has moved.

## Operating rules

1. **Use machine mode.** Pass `--json` (single response object) or `--jsonl`
   (streamed records) and check `meta.aggregate_status == "Ok"` plus the exit
   code. Do not parse human-mode output; it is for terminals.
2. **Verify state after every mutating command** — `gwz status --json` and,
   where relevant, `git status --short` in root/member. The recipes below name
   the expected state; if observed state differs, stop and diagnose, don't
   continue the recipe.
3. **`--dry-run` before broad mutations** (`pull`, `push`, multi-member ops).
   Not supported for `gwz clone`.
4. **`gwz snapshot <name>` before any risky multi-repo change.** Restore with
   `gwz materialize --snapshot <name>`.
5. **Don't confuse the pairs:** `gwz clone` (whole workspace) vs
   `gwz repo clone` (one member into current workspace); `gwz add` (stage FILE
   content) vs `gwz repo add` (register an existing REPO as member).
6. Commands discover the workspace from CWD, including inside members. Use
   `--root <path>` only to override.

## Verified semantics (the things QuickStart leaves ambiguous)

| Question | Verified answer (gwz 0.9.0) |
|---|---|
| Does `gwz init` need a pre-existing git repo? | **No.** In an empty dir it creates `.git` (branch `main`, no commits), `.gwz/`, `gwz.conf/{gwz.yml,gwz.lock.yml}`, `AGENTS_GWZ.md` — and **stages** all of it. Nothing is committed. |
| What does `gwz repo create <path>` produce? | An **empty member git repo: zero commits, zero files** (unborn `main`). You add the first files and commit. |
| What does `repo create` stage? | It rewrites **and stages** `gwz.conf/gwz.yml` + `gwz.conf/gwz.lock.yml` in the root (member row: `active: true`, `desired.local_only: true`; lock row: `branch: main, materialized: true`). No explicit `gwz add gwz.conf` needed — unlike `repo sync`, which leaves its manifest rewrite **unstaged**, and `repo add`, which also requires explicit staging. |
| Does the lock update automatically? | For `repo create`, yes (row written at create time). `gwz commit` also records the committed member revisions into the lock in the same operation (verified: after a workspace-wide `gwz commit`, `gwz capture` was a no-op and every member reported `lock_match: Matches`). `gwz capture` is for recording live worktree state *without* committing. |
| Member-id conventions? | Default id is `mem_<basename>` and source id `src_<basename>`, derived from the member path's last segment (verified: `services/api` → `mem_api`; dashes sanitize to underscores, `wyred-contract` → `mem_wyred_contract`). The default is deterministic — omit `--member-id` on the fast path. A basename collision fails cleanly (`InvalidRequest: member id is already registered`, exit 1, no partial manifest mutation); only then pass `--member-id`. |
| Is `gwz commit` root-only? | `gwz commit` commits staged content across root and members in one operation (per-member commits with the same message where staged). |

## Recipe: construct a new multi-member workspace

```sh
mkdir -p <ws-path> && cd <ws-path>
gwz init --json
# EXPECT: exit 0; .git + gwz.conf/ + AGENTS_GWZ.md created and staged;
#         root branch main unborn (no commits yet); members: []

gwz repo create <member-path> --json   # repeat per member; id defaults to mem_<basename>
# EXPECT: exit 0; member dir contains only .git (zero commits);
#         gwz.conf/gwz.yml gains an active member row; both manifest+lock staged

gwz status --json
# EXPECT: aggregate_status Ok; each member lock_match "Matches",
#         branches[].unborn true (members have no commits yet)

# Optional root-level content (docs, plans) — root is a normal git repo:
mkdir dev-docs && <write files>
gwz add dev-docs/<file>

gwz commit -m "Define the workspace"
# EXPECT: exit 0; root has its first commit; `git -C <ws-path> log --oneline`
#         shows it; gwz status --json workspace_git_status.root_status.dirty false
```

## Recipe: publish a locally-created member (later)

Create the empty hosted repo first (e.g. `gh repo create`), then:

```sh
git -C <member-path> remote add origin git@github.com:<org>/<name>.git
gwz repo sync <member-path>
# CAUTION: repo sync leaves its gwz.conf rewrite UNSTAGED (verified in docs;
#          asymmetric with repo create) — stage it explicitly:
gwz add gwz.conf
gwz commit -m "Record member origin"
# Member needs >=1 commit before its first push:
gwz --member mem_<name> push
```

## Recipe: entering an existing workspace (summary)

`gwz clone <root-url> [dir]` materializes everything; if the root arrived via
plain `git clone`, run `gwz materialize --lock` then `gwz status --json`.
This path is what `AGENTS_GWZ.md` at any workspace root covers.

## Failure handling

- Non-`Ok` status or nonzero exit: read `errors[]` in the JSON response; see
  [docs/Troubleshooting.md](../docs/Troubleshooting.md).
- Half-completed multi-member operation: `--partial` governs whether that can
  happen; default policy rejects partial mutation when planning detects a
  member cannot proceed. Recover to a named state with
  `gwz materialize --snapshot <name>`.
- Identity errors on attach/add (`SourceIdentityMismatch`): stop and read
  [docs/RepoLifecycle.md](../docs/RepoLifecycle.md) — do not `--force` through
  identity checks; that's how designations get corrupted.

## Doc map

- [docs/Concepts.md](../docs/Concepts.md) — manifest/lock/snapshot/selection model
- [docs/MachineOutput.md](../docs/MachineOutput.md) — JSON/JSONL schema, exit codes
- [docs/commands/](../docs/commands/) — per-command reference
- [docs/RepoLifecycle.md](../docs/RepoLifecycle.md) — identity & recovery rules (read before replace/reattach)
