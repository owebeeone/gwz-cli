# GWZ fetch: learn what moved upstream across the whole workspace without pulling

Status: plan. Phase 1 released in gwz 1.0.17 (2026-09-18), with two
refinements the release review added: a `--dry-run` row is `Planned` /
`would contact <remote>`, aggregates like the live run, and a `--remote <name>`
no repository has is refused before the network on both paths (see
`history/GwzRelease1017Docs-RemPlan.md`). Phases 2 and 3 are not started.

## 1. Goal and non-goals

### The gap

`gwz pull` integrates. `gwz push --check-remotes` reads every selected remote
and every root dependency, concurrently, but it reads them in order to decide
what to publish and it reports publication, not drift. Between the two there is
nothing that answers the question a multi-repo workspace asks constantly:

> What moved upstream, in the root and in each member, since I last looked?

Today the answer is `gwz forall -- git fetch` followed by
`gwz forall -- git status -sb`, which is serial, skips the root unless the
operator remembers `-A`, does not use GWZ's credential or URL-scheme machinery,
and reports in N separate voices.

### Goal

`gwz fetch` fetches the configured remote of every selected repository — the
root and its members, with `--target` semantics identical to `push` — reads them
concurrently the way `push --check-remotes` already does, updates
remote-tracking refs, and reports, one line per repository:

- the remote-tracking ref before and after the fetch;
- ahead/behind of that repository's current branch against its upstream;
- `no change` when the tracking ref did not move.

### Non-goals (permanently, not just for Phase 1)

- **Never integrates.** No merge, no rebase, no fast-forward, no reset. Nothing
  in `gwz fetch` writes `refs/heads/*`, `HEAD`, the index or the worktree.
- **Never writes workspace artifacts.** No lock write, no manifest write, no
  boundary sync. `gwz fetch` is a read of the world that leaves a side effect
  only inside each repository's `refs/remotes/*`.
- **Never prunes** unless asked (`--prune`, Phase 2).
- **Never skips the network.** There is no "unchanged since the last fetch"
  short-circuit. Contacting the remotes IS the operation; `push`'s
  `RemoteCheck::changed` default has no counterpart here and no
  `--check-remotes` flag is offered, because `always` is the only meaning.

## 2. Facts this plan rests on

Read on the lane tree at gwz-core `d1007a96` / gwz-cli / gwz-py.

### 2.1 The fetch half of `pull` is already factored

`gwz-core/src/workspace_ops/pull_head_member_preflight/`:

- `member_preflight.rs::pull_fetch_remote_name(member, policy)` — the policy
  `--remote` override, else the member's first manifest remote with
  `fetch: true`. `None` means the member has no fetch remote.
- `member_preflight.rs::pull_remote_host(member, policy)` — the host key for
  per-host concurrency bounding.
- `member_validation.rs::pull_branch(member, state)` — the locked branch, else
  the desired branch, else `main`.
- `root_lock.rs::pull_root_remote_name(backend, root, policy)` — the policy
  override, else `origin`, else the first remote.
- The fetch itself: `backend.fetch(path, &remote)`, then
  `backend.read_ref(path, "refs/remotes/<remote>/<branch>")`.

All of it is per-repository, side-effect-free outside the repository, and
already used under the same credential and URL-scheme machinery (SSH agent, the
`gh` credential helper, `pushurl`-style selection) because the backend is
scoped by `with_transport` before any of it runs.

### 2.2 The concurrent read path is already generic

`gwz-core/src/operation/par_map_per_host.rs::par_map_per_host` bounds work
globally (`--jobs`, `resolve_jobs`) and per host (`--max-per-host`,
`resolve_per_host`), preserves input order, and runs `f` on scoped threads.
`push`'s `ReadPreflight::read_before_transfers` and `pull`'s two-pass preflight
are both thin callers of it. `gwz fetch` is a third caller with the same shape:
N members + the root = N+1 items, one host key each.

### 2.3 What the response envelope already carries

`ResponseEnvelope.members: List(MemberResponse)` gives id, path, source kind,
status and error per repository, and the CLI's generic human renderer already
prints one line per row. `BranchRepoSummary` shows the precedent for a verb that
needs more than `MemberResponse` carries: a parallel `repos` list on the verb's
own response message, rendered by a dedicated renderer
(`render_branch_response`). `BranchRepoSummary` already declares
`upstream`/`ahead`/`behind` as optional ints — and nothing in the tree ever
populates them, because no backend method counts.

### 2.4 What is missing

- No `ActionKind` for fetch (next free tag: 30).
- No `FetchRequest`/`FetchResponse`, no `FetchRepoSummary`.
- No backend primitive that **counts** ahead/behind. `is_ancestor` answers the
  boolean; `git2::Repository::graph_ahead_behind` answers the counts and is
  already linked.
- No CLI verb, no gwz-py client method, no docs.

## 3. Behaviour contract (Phase 1)

### 3.1 Selection

`--target` semantics are exactly `push`'s: `action_policy` row
`(All, Allow, "fetch")`. Default selection is the root plus every active member.
`--all`, `--target`, `--member`, `--member-path`, `--no-target @root` and the
exclusion forms all behave as they do under `push`.

### 3.2 Per repository

For each selected repository, in parallel under the global and per-host
ceilings:

1. Resolve the fetch remote (§2.1). No remote ⇒ `no upstream` row, `Noop`.
2. Resolve the branch (§2.1 for a member; the attached HEAD branch for the
   root). Detached or unborn ⇒ `no upstream` row, `Noop` — a detached HEAD is
   a normal state to fetch from, not an error.
3. Read `refs/remotes/<remote>/<branch>` — **before**.
4. `backend.fetch(path, remote)`.
5. Read the same ref again — **after**.
6. If the repository has a local commit and an after-ref, count ahead/behind of
   the local commit against the after-ref.

Row status: `Ok` when the tracking ref moved, `Noop` when it did not or there
was no upstream, `Failed` when the remote refused or the fetch errored.

### 3.3 Human output

One line per repository, root first when selected, then members in manifest
order:

```
$ gwz fetch
status: Partial
@root        .         no change          (origin/main, +0 -0)
mem_core     gwz-core  a1b2c3d..9f8e7d6   (origin/main, +0 -3)
mem_cli      gwz-cli   no change          (origin/main, +2 -0)
mem_py       gwz-py    no upstream
mem_private  private   failed             RemoteRejected: ...
```

The `status:` line comes first, as it does under every other verb that renders
an envelope.

`+A -B` is ahead/behind of the current branch against the tracking ref after
the fetch. Abbreviated object ids are 7 hex characters, as `git` renders them.

### 3.4 `--json` / `--jsonl`

`--json` emits the ordinary response object plus a top-level `repos` array, one
object per repository carrying `member_id`, `member_path`, `source_kind`,
`result`, `remote`, `branch`, `before`, `after`, `upstream`, `ahead`, `behind`.
`--jsonl` streams the same rows through the existing event stream and closes
with the same final object. `docs/MachineOutput.md` gains the `fetch` section.

### 3.5 Exit codes

Follow `push`'s conventions exactly, through the unchanged
`exit_code_for_response`:

| Aggregate | Exit | When |
| --- | --- | --- |
| `Ok` / `Noop` | 0 | every selected repository fetched, or nothing moved |
| `Partial` | 1 | at least one repository contacted and at least one failure |
| `Failed` | 1 | every repository failed or was refused |
| `Rejected` | 2 | nothing was contacted; every row was refused before the network |

`fetch_aggregate_status` follows push's exit codes but is computed from
fetch's own report rows, **not** delegated to `push_aggregate_status`
(amended at step 1.3, after the handler's tests): the two verbs mean different
things by a `Noop` row. Push's `Noop` is "nothing to publish"; fetch's
`unchanged` is a repository that WAS contacted and answered. Delegating would
report `Failed` for a batch where one remote refused and every other
repository read cleanly, which claims nothing was read. So the rule is:

- a failure alongside any contacted repository is `Partial`;
- a batch in which nothing was contacted and every row was refused before the
  network is `Rejected`;
- otherwise any `updated` row is `Ok`, and a batch that moved nothing is
  `Noop`.

### 3.6 What `gwz fetch` never does

It takes no workspace mutation guard and writes no workspace artifact, so it is
`OpenMergeGateDecision::Allow` — it runs with a merge open, like `status` and
`ls`. It is the one network verb that is safe to run at any time.

## 4. Decisions, with the alternatives considered

**D1. The root is fetched by default.** *Alternative: members only, root on
`--target @root`.* Rejected. The root repository holds the manifest and the
lock; "what moved upstream" without the root is the least interesting half of
the answer, and it is precisely the repository whose drift an operator forgets
to check. `push` includes the root by default and `fetch` is push's mirror, so
the selection policy row is literally the same `(All, Allow, …)`. An operator
who wants members only writes `--no-target @root`, exactly as under `push`.

**D2. A repository with no upstream is a `no upstream` row, not an error.**
*Alternatives: (a) fail that repository, (b) omit the row.* Both rejected.
(a) makes a single local-only member break the exit code of a whole-workspace
observation, which turns `gwz fetch` into something an operator stops running.
(b) hides the fact — and the whole point of the verb is that the report is
complete. `pull` already treats this as `SkipNoFetchRemote` rather than a
failure; this is the same ruling, spelled for a report. A detached or unborn
HEAD lands in the same row for the same reason.

**D3. Partial failure exits 1, a refused request exits 2.** *Alternative: exit 0
whenever any repository was read, on the grounds that fetch is an observation.*
Rejected: a remote that refused means the report is **incomplete**, and a script
that polls `gwz fetch` must be able to tell a complete answer from a partial
one. Following `push`'s table costs nothing and keeps one convention across the
two network verbs.

**D4. Always contact the remote; no `--check-remotes` and no `changed` mode.**
*Alternative: mirror `PushRequest.remote_check`.* Rejected. `push`'s default
exists because a push that publishes nothing should not pay for a connection;
a fetch that skips the connection has answered nothing. Offering the knob would
invite the one call that cannot mean anything. `FetchRequest` therefore carries
no `remote_check` field — not one pinned to `always`.

**D5. A dedicated `FetchRepoSummary` list, not `MemberResponse.planned.message`.**
*Alternative: reuse `PlannedChange.message` as free text, as `push`'s `Noop`
rows do.* Rejected: before/after object ids and ahead/behind counts are
structured facts a driver will want to switch on, and burying them in a human
string forces every consumer to parse prose. `BranchResponse.repos` is the
precedent; `FetchResponse.repos` follows it exactly.

**D6. Count ahead/behind with a new backend method, do not infer from
`is_ancestor`.** *Alternative: report the three booleans (up-to-date, strictly
behind, diverged) that `is_ancestor` can already answer.* Rejected: `+2 -0` is
the answer an operator acts on and `git status -sb` sets the expectation.
`GitBackend::ahead_behind` is one delegate over
`git2::Repository::graph_ahead_behind`, with an `unsupported_backend` default
so no existing backend implementation changes. It also fills in
`BranchRepoSummary.ahead`/`.behind`, which have been declared and unpopulated
since the branch verb landed — Phase 2 can do that for free.

**D7. Ahead/behind is measured after the fetch, against the tracking ref.**
*Alternative: measure against the configured Git upstream
(`branch.<name>.merge`).* Deferred, not rejected: the tracking ref this
operation just wrote is the fact this operation established, and a workspace
member's upstream is normally exactly that ref. Per-remote selection (Phase 2)
is where the distinction starts to matter.

**D8. `fetch` is allowed while a merge is open.** *Alternative: `Block`, as
`pull` and `push` are.* Chosen because the gate exists to stop workspace-state
mutation during an unrecovered merge, and `fetch` mutates no workspace state.
An operator recovering a merge is exactly the operator who needs to see what
moved upstream.

**D9. No `gwz fetch --dry-run` special case.** The global `--dry-run` resolves
selection and reports the rows it would contact with `Planned`, contacting
nothing — the same shape `push --dry-run` has.

## 5. Phases and steps

### Conventions

- Foundational steps first. Steps within a phase are independent unless a step
  says otherwise.
- Budgets are aspirational: under 500 LOC per step, tests included.
- One commit per member per step, through gwz:
  `gwz add <paths>` then `gwz --target <member> commit -m ...`.
- Gates before each commit: gwz-core `python3.13 scripts/run_tests.py` and
  `cargo clippy --all-targets -- -D warnings`; gwz-cli `cargo test`, clippy and
  `python3 scripts/generate_cli_reference.py --check`; gwz-py
  `.venv/bin/python run_tests.py`.
- Every control-flow body is braced; no `#[cfg]` on an unbraced declaration.
- No new crates. No AI attribution trailer.

### Phase 1: the verb, end to end (this lane)

**Step 1.1 — protocol surface (gwz-core).** Foundational; everything else
depends on it.

- `ActionKind.fetch = 30`.
- `FetchResult` enum: `updated`, `unchanged`, `no_upstream`, `failed`.
- `FetchRepoSummary` message: `member_id`, `member_path`, `source_kind`,
  `result`, `remote?`, `branch?`, `before?`, `after?`, `upstream?`, `ahead?`,
  `behind?`.
- `FetchRequest` (`meta` only — see D4) and `FetchResponse`
  (`response`, `repos`).
- `GwzCore.fetch` service method.
- `protocol/regen.py`; `check_log_additive.py` pin moved with a measured note;
  `docs/MessageCatalog.md` and `docs/Protocol.md` updated;
  `tests/protocol.rs` parity vectors refreshed.

**Step 1.2 — the ahead/behind primitive (gwz-core).** Independent of 1.1.

- `GitBackend::ahead_behind(path, local, upstream) -> (i64, i64)`, defaulting
  to `unsupported_backend`.
- `refs::ahead_behind` over `graph_ahead_behind`; one `delegate!` line; one
  `forward!` line in the test repository.
- Contract test against a real local repository.

**Step 1.3 — the handler (gwz-core).** Depends on 1.1 and 1.2.

- `workspace_ops/handle_fetch.rs`: selection via
  `resolve_action_targets(…, ActionKind::Fetch)`, the `(All, Allow, "fetch")`
  policy row, `OpenMergeCommand::Fetch => Allow`, transport scoping,
  `validate_transport_remotes`, `par_map_per_host` over the N+1 repositories,
  before/fetch/after per repository, `fetch_aggregate_status`, transport
  attachment on success and on error.
- Tests against local bare remotes: a repository ahead, a repository unchanged,
  a repository with no upstream, a remote that refuses, and N+1 concurrency.

**Step 1.4 — the CLI verb (gwz-cli).** Depends on 1.3.

- `CommandArgs::Fetch`, `FETCH_LONG`, `FETCH_AFTER`, `CliRequest::Fetch`,
  dispatch to `handle_fetch_with_events`, `CliResponse::fetch`.
- Human renderer (§3.3), `--json`/`--jsonl` rows (§3.4).
- CLI tests: the verb, `--target`, `--json`.

**Step 1.5 — CLI docs (gwz-cli).** Depends on 1.4.

- `docs/commands/fetch.md`, `mkdocs.yml` nav, `src/help.rs` grouping,
  `docs/MachineOutput.md`, and `docs/CLI.md` regenerated with
  `python3 scripts/generate_cli_reference.py --write` so the g00 byte compare
  passes.

**Step 1.6 — gwz-py (gwz-py).** Depends on 1.1 and 1.3; independent of 1.4.

- Lane hygiene first: repoint every `.venv` file naming the donor workspace,
  delete copied `__pycache__`/`.pytest_cache`, set `VIRTUAL_ENV`.
- `scripts/regen_protocol.py` regenerates `src/gwz/protocol/generated.py`
  (never hand-edited); the codec pins move with a measured note.
- Native bridge: `"fetch"` in `dispatch::call` and `dispatch::submit`.
- `client.fetch()` / `client.fetch_stream()`; the `fetch` CLI verb in
  `cli_mutation.py`; parity tests.

### Phase 2: the knobs (later)

Each step is independent of the others and depends only on Phase 1.

**Step 2.1 — `--prune`.** `FetchRequest.prune` (next free tag), passed to a new
`backend.fetch_with_options`. Pruned refs are reported as their own row kind so
the report says what disappeared, not just what moved.

**Step 2.2 — `--tags`.** `FetchRequest.tags`, the refspec decision, and the
interaction with `gwz tag --fetch`, which already fetches tags by another door.

**Step 2.3 — per-remote selection.** `FetchRequest.remote` overriding the
policy token the way `PushRequest.remote` does, plus multi-remote fetch for a
member that has more than one. This is where D7 has to be revisited: with an
explicit remote, ahead/behind against the configured upstream and against the
fetched ref stop being the same number.

**Step 2.4 — `gwz status --remote`.** Read what `fetch` wrote — the
remote-tracking refs — and fold ahead/behind into the status projection without
contacting anything. Populates `BranchRepoSummary.ahead`/`.behind` from Step
1.2's primitive at the same time.

### Phase 3: connection reuse (later, not designed here)

`gwz fetch` is N+1 remote reads, which is exactly the shape that pays for
connection reuse: the measured SSH cost is 2–3.5 s to connect versus ~0.5 s per
reused channel (`gwz-core/dev-docs/GwzRemoteTransportRequirements.md` §3.1,
§5.2). libgit2 cannot pool connections itself; a registered custom transport
can.

**This plan does not design that.** It names it as the dependency:
`gwz fetch`'s wall-clock time on a workspace of N members is governed by the
transport placement program in
`gwz-core/dev-docs/GwzRemoteTransportRequirements.md`, and `gwz fetch` should be
one of that program's measurement subjects — it is the cleanest N+1 read in the
product, with no publication semantics to confound the timing. Nothing in
Phases 1 and 2 may assume a pooled transport, and nothing in them blocks one:
the handler reaches the network only through `backend.fetch`.

## 6. Dependency sketch

```
  1.1 protocol ──┐
                 ├──> 1.3 handler ──┬──> 1.4 CLI verb ──> 1.5 CLI docs
  1.2 ahead/behind┘                 └──> 1.6 gwz-py
                                          (needs 1.1 for the schema,
                                           1.3 for the bridge call)

  Phase 1 ──> 2.1 --prune
          ──> 2.2 --tags
          ──> 2.3 per-remote selection   (revisits D7)
          ──> 2.4 status --remote        (reuses 1.2's primitive)

  GwzRemoteTransportRequirements.md ──> Phase 3 (connection reuse)
        (a dependency of fetch's SPEED, never of its correctness)
```

1.1 and 1.2 are independent and can run in parallel. 1.4/1.5 and 1.6 are
independent of each other once 1.3 lands: the CLI and the Python driver are
separate members and separate commits.

## 7. Test matrix (what "done" means for Phase 1)

| Case | Where |
| --- | --- |
| a repository ahead of its tracking ref | gwz-core handler tests, local bare remote |
| a repository unchanged (`no change`) | gwz-core handler tests |
| a member with no fetch remote (`no upstream`) | gwz-core handler tests |
| a remote that refuses | gwz-core handler tests |
| N+1 repositories read concurrently | gwz-core handler tests |
| the verb, `--target`, `--json` | gwz-cli tests |
| `docs/CLI.md` byte-compares | gwz-cli `src/tests/g00.rs` |
| client method and CLI parity | gwz-py tests |
| protocol additive, pins measured | `check_log_additive.py`, `tests/protocol.rs`, `test_codec.py` |
