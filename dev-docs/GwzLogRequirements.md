# GWZ Log

Status: **requirements RESOLVED 2026-08-29, rows reconciled inline after the
S0.1 round-1 review** — all thirteen questions in
[GwzLogAmbiguityRezo.md](GwzLogAmbiguityRezo.md) are answered; every row
below states its resolved value with the `Q-n` kept as provenance. Q-1's
answer **sharpens the product**: entries from the same coordinated workspace
commit COALESCE into one attributed entry — the Coalescing section is
normative. Two narrow re-confirmations are pending with the operator (the
Rezo's dated notes under Q-1 and Q-11: heuristic breadth; pager); their
standing resolutions govern until changed. The Rezo stays authoritative
where the two documents disagree. Ready for implementation planning.

Scope: a unified commit log across the workspace — `@root` plus member
repositories — as one command. Semantics in `gwz-core`; client surface in
`gwz-cli` **AND mirrored in `gwz-py`** (operator directive 2026-08-29,
verbatim: "all gwz-cli work needs to be mirrored as gwz-py work too") —
both clients lower to the same protocol, exactly as `gwz diff` and
`gwz status` do. The protocol therefore grows the log request/response
messages, additively (the Protocol and Python surface section).

## Problem

A workspace is the unit of work: one feature lands as commits spread across
several member repositories, often within minutes of each other — and one
coordinated `gwz commit` deliberately creates sibling commits (same message,
one operation) across several members and the root. Today, answering "what
happened in this workspace lately?" means running `git log` once per member
and mentally interleaving the results, and a single workspace-level change
shows up as N unrelated-looking rows. There is no view that presents
workspace history as a single stream with each entry attributed to the
repository — or set of repositories — that carries it.

Member repositories have fully independent histories ("individual log
records"): unrelated roots, different branches checked out, detached HEADs,
unborn branches (no commits yet), refs that exist in some members and not
others. A unified log must tolerate all of that heterogeneity rather than
failing because one member is odd (Q-1).

## Goals

- `gwz log` renders the commit history of the **selection** (default: `@root`
  plus all members) as **one interleaved, attributed stream**.
- **Sibling commits from the same workspace-level commit COALESCE into one
  entry**, attributed to the set of member repos (and/or `@root`) that carry
  it (Q-1's resolution; the Coalescing section). A plain single-repo commit
  is the singleton case of the same rendering.
- Operand grammar is **consistent with `gwz diff`**: revisions, ranges
  (`A..B`, `A...B`), `+snapshot` ids, and pathspecs after `--`, classified by
  core.
- **Read-only and tolerant.** The command never mutates anything, never
  contacts the network, never takes the workspace mutation lock, and never
  gates on `gwz.conf` integrity. A member that cannot contribute (missing
  ref, unborn branch, unreadable repo) degrades that member, not the command.
- Machine output (`--json` / `--jsonl`) suitable for tooling.
- **`gwz-py` parity**: the Python distribution ships the same command with
  the same semantics in the same release (the three-channel rule: the
  feature is not done until PyPI moves too).

## Non-Goals

- A cross-repository commit **graph**. Ancestry is only meaningful within one
  repository; the unified stream is an interleave, not a DAG merge (Q-8:
  no `--graph` in v0 at all).
- Replacing `git log` inside a single member.
- Any implicit `fetch`; the log reads only what is local.
- Rename-following across member boundaries.
- Reflog. (`gwz log` reads commit history, not reflogs.)

## Requirement Conventions

Follows `gwz-core/dev-docs/GWZRequirements.md`: `MUST` / `SHOULD` / `MAY`;
`v0` is the first implementation target; every v0 requirement the plan
implements — MUST or SHOULD — must be traceable to a named test before
implementation is accepted.

## Concepts

### Entry

One **workspace-level change**. An entry carries a **member set**: one or
more (member identity, commit hash, parents) tuples — member identity is the
id and workspace-relative path; `@root` uses the id `@root` and path `.` —
plus the shared author and committer (name, email, timestamp with offset),
subject, and body. A commit that belongs to no coalesced group is a
singleton entry; a coalesced entry (below) carries every sibling.

### Workspace commit (coalesced entry)

One coordinated `gwz commit` operation creates sibling git commits — same
message, one operation, different SHAs — across several members and/or the
root. Those siblings are ONE workspace-level change and render as ONE entry
attributed to the set of repos carrying it (Q-1). Merge identity, in
priority order: (1) the **`GWZ-Commit-ID` trailer** — one UUIDv7 per commit
operation. This is **landed, shipped machinery in production use** (minted
and written by `gwz-core/src/workspace_ops/handle_commit.rs`; wire flag
`commit_marker`; CLI `--commit-marker`/`--no-commit-marker`; verified
2026-08-29 against live history — `gwz-core/dev-docs/GwzCommitMarker.md`'s
"proposed" status line is stale and is corrected by plan step S1.2).
(2) For history predating the marker (and trailer-stripped messages), a
**conservative heuristic** — see L-COA-2. Known coverage limit, stated
honestly: a `gwz commit` that commits ONLY `@root` while member commits are
made with plain `git` (a common working pattern) yields trailerless member
commits that neither key can associate; the marker ARTIFACT
(`gwz.conf/markers/<uuid>.yaml`, whose `members:` map records member SHAs)
could associate them — recorded as the v2 candidate "artifact-assisted
association", not v0. (The marker doc's own "`gwz log --merged` finds the
root commit by locating the commit that contains the marker file" sentence
is superseded for this design: the root's own trailer already identifies
it.)

### Unified stream

The k-way merge of the selected repositories' commit sequences, coalesced
within a bounded reorder window (L-COA-7). Within a single repository the
order MUST be that repository's own `git log` default order (L-ORD-1).
Across repositories, entries are interleaved newest-first by committer date
(Q-4). Cross-repo ordering is therefore **timestamp-approximate**: clocks
skew, rebases re-stamp — the tool presents what the records say and does
not attempt to correct skew.

### Degraded member

A selected repository that cannot contribute entries for the given operands:
an operand ref that does not resolve there, an unborn branch, a repository
that fails to open. Degraded members are reported (Q-6: skip with a
per-member note; `--strict` promotes to error) and the command proceeds
with the rest.

## Requirements

### Command surface and selection

- **L-SEL-1 (v0).** `gwz log [OPTIONS] [operand]... [-- <pathspec>...]` MUST
  exist as a `gwz-cli` subcommand and MUST honor the standard global
  selectors (`--root`, `--target`, `--no-target`, `--member` aliases,
  `--all`) with the same semantics as other read commands.
- **L-SEL-2 (v0).** Default selection MUST be `@root` plus all members
  (Q-3: resolved as proposed).
- **L-SEL-3 (v0).** `--tagged` MUST narrow the selection to repositories
  containing every supplied local tag, exactly as `gwz diff --tagged` does
  (Q-7). Note the existing diff-side exclusion carried over: `--tagged`
  does not combine with `+`-prefixed operands (`+snapshot`, `+lock`) — the
  same refusal, same wording.

### Operands and ranges

- **L-RNG-1 (v0).** Operands MUST be classified by core using the same
  grammar as `gwz diff`: zero or more revisions, ranges (`A..B`, `A...B`),
  or `+snapshot` ids; pathspecs come after `--` and a leading `+` after `--`
  is a path.
- **L-RNG-2 (v0).** With no operands, each repository contributes its
  current `HEAD` history (detached HEAD included, from the detached commit).
- **L-RNG-3 (v0).** A `+snapshot` operand resolves per-member to the ref or
  commit that snapshot recorded for that member; `+snapA..+snapB` and
  `+snap..HEAD` MUST work wherever both endpoints resolve for a member.
  Members without an entry in the snapshot DEGRADE per L-TOL-2. `@root`
  under a `+snapshot` operand DEGRADES with a record (snapshots do not
  record the root) — a deliberate divergence from `gwz diff`'s silent
  root-omission, per L-SEL-2's spirit: the root's absence is visible, not
  silent.
- **L-RNG-4 (v0).** The lock-relative range — each member logging
  `<lock-recorded pin>..HEAD`, "what has moved since the recorded workspace
  state" — MUST exist, spelled **`+lock`** as a pseudo-snapshot operand
  (Q-10). Resolution is per-member from `gwz.lock.yml`'s `members:` map.
  **`@root` under `+lock` DEGRADES with a degradation record** — the lock
  artifact has no root entry (`LockArtifact` carries `schema`,
  `workspace_id`, `manifest_schema`, `members` only; decided at the S0.1
  round-1 fold, review F8) — it never errors and never silently vanishes.
  Classification note: core's operand grammar already routes any
  `+`-prefixed token away from path interpretation, so `+lock` is a
  resolution feature, not a classifier change.
- **L-RNG-5 (v0).** Ref resolution is strictly local. The command MUST NOT
  perform network operations.

### Coalescing (Q-1's resolution — normative)

- **L-COA-1 (v0).** Sibling commits carrying the same `GWZ-Commit-ID`
  trailer value MUST coalesce into one entry whose member set is every
  selected repo carrying that value. The trailer is landed production
  machinery (see the Workspace commit concept); this row is exercised
  against real history from day one.
- **L-COA-2 (v0).** Unmarked commits (history predating the marker, and
  trailer-stripped messages) MUST coalesce under a conservative heuristic
  ONLY: byte-identical full message AND identical author name+email AND
  committer timestamps within a fixed small window (default 10 seconds)
  AND author timestamps within the same window, across DIFFERENT repos.
  Anything less than all four stays separate. The author-timestamp
  conjunct exists for the rebase hazard (S0.1 review F6): a cross-member
  rebase re-stamps committer dates into the same few seconds, destroying
  exactly the timestamp diversity the committer-window test keys on, while
  author dates survive rebase and genuinely-unrelated commits do not share
  them. Declared consequence, intended: same-message `gwz forall -- git
  commit`/`cherry-pick` fan-outs inside the window DO coalesce, labeled
  `heuristic` (pending the operator's breadth re-confirmation, Rezo Q-1
  note). Same-repo commits NEVER merge (this also makes `--amend` twins
  safe by construction). The heuristic MUST never merge across different
  `GWZ-Commit-ID` values, and a commit that CARRIES a marker MUST never
  join a heuristic group at all (round-2 F24): marked commits coalesce by
  marker or not at all — the heuristic exists only for the unmarked.
- **L-COA-3 (v0).** `--no-coalesce` renders raw per-repo entries
  (singleton sets throughout), for debugging and for exact-git-parity
  consumers.
- **L-COA-4 (v0).** A coalesced entry's ordering timestamp is the LATEST
  committer timestamp among its siblings **found within the L-COA-7
  window**; ties across entries break by member id then hash for
  deterministic output.
- **L-COA-5 (v0).** Filters (`--author`, `--grep`, `--since`, `--until`)
  evaluate on the entry's shared message/author/date surface; selection
  narrowing (`--target`, `--tagged`) narrows the member set an entry can
  draw from — an entry whose every carrying repo is deselected disappears,
  and one partially deselected renders with the narrowed set (the machine
  record says so, → L-JSN-1).
- **L-COA-6 (v0).** Machine output records merge provenance per entry:
  `marker:<uuid>` or `heuristic` (or `none` for singletons), so consumers
  can distinguish proven identity from inference.
- **L-COA-7 (v0) — the bounded coalescing window (the L-COA-4 ↔ L-PRF-1
  contract; S0.1 review F3, ownership fixed per round-2 F21).** The
  streaming merge MUST hold emission only within a bounded reorder window
  **W = 60 seconds** of stream time: a group MUST be closed when every
  live cursor has advanced past (group's newest committer timestamp − W).
  Siblings falling OUTSIDE the window MUST emit as separate entries
  carrying the SAME provenance key (`marker:<uuid>` repeats), so
  consumers can re-join what the stream could not — memory MUST stay
  O(selected repos × entries within W), never whole-history. Ownership:
  **the window buffer and the emission hold live in the MERGE (plan step
  S2.5, which owns this row)**; the grouping logic (plan S2.4) assembles
  groups against this contract and exposes a group-assembly API the merge
  drives — S2.4 does not itself hold cross-cursor state. This is the
  explicit, tested relaxation of L-COA-4; `--no-coalesce` is unaffected.
  (Round-2 validation on this workspace's real history: 153 multi-repo
  marker groups, maximum committer-date spread 1 s — W has ~60×
  headroom.)
- **L-COA-8 (v0) — identity stability.** One workspace-level change MUST
  carry ONE `GWZ-Commit-ID` across partial-commit retries. The landed
  implementation is NOT retry-idempotent (a fresh UUIDv7 is minted per
  invocation; members commit before root, so a member-succeeded /
  root-failed retry permanently splits one change across two ids — S0.1
  review F5), and L-COA-2 forbids the heuristic from healing the split.
  **The retry-sameness predicate (round-2 F22):** whether an invocation
  is "a retry of the same operation" MUST be determined from **durable
  per-operation state recorded before the operation's first commit** —
  never inferred after the fact from message similarity or timing. When
  sameness cannot be PROVEN from that state, a NEW id MUST be minted:
  a false fuse (two different changes under one id) is strictly worse
  than a false split, so the predicate fails toward splitting. Plan step
  S1.1 designs the durable state (the marker artifact's own
  pending-then-finalized lifecycle is the natural candidate), implements
  the predicate, and lands the regression test driving the
  member-succeeded/root-failed retry to ONE id; `gwz log` itself carries
  no code for this row.

### Tolerance

- **L-TOL-1 (v0).** The command MUST NOT use whole-request plan rejection.
  Per-member failures degrade that member only. (This is the read-side
  complement of the mutation-side `DirtyMember`/`MissingRemote` wholesale
  refusals: correct for writes, wrong for reads.)
- **L-TOL-2 (v0).** An operand that resolves in some selected repositories
  and not others MUST NOT be a hard error by default: members where it does
  not resolve are degraded and reported (Q-6: stderr summary in human mode,
  explicit records in machine mode, exit stays 0); `--strict` turns any
  degradation into a hard error for scripting.
- **L-TOL-3 (v0).** Unborn/empty repositories (e.g. a workspace root with no
  commits yet) MUST contribute zero entries and MUST NOT fail the command.
- **L-TOL-4 (v0).** Detached members MUST log normally from their detached
  commit.
- **L-TOL-5 (v0).** A shallow member MUST contribute what it has; hitting a
  shallow boundary is not an error.
- **L-TOL-6 (v0).** `gwz log` MUST NOT gate on `gwz.conf` integrity (house
  rule: read-only and list commands never gate, so a damaged workspace stays
  inspectable).

### Ordering and depth

- **L-ORD-1 (v0).** Within one repository, entry order MUST equal that
  repository's own **`git log` default order** for the same operands (the
  revwalk's default sorting; this row deliberately does not say
  "topological" — `--topo-order` is a different, unclaimed mode).
- **L-ORD-2 (v0).** Across repositories, entries MUST be merged newest-first
  by **committer date** (Q-4; author date carried in machine output; no
  `--date-order=author` in v0 unless it falls out free). Ties break by
  member id then hash.
- **L-DEP-1 (v0).** Bare `gwz log` MUST NOT dump unbounded history: the
  default depth is a **workspace-global cap of the newest 50 entries**
  (post-coalescing), overridden by `-n <N>`, removed by `-n 0`/
  `--no-limit`, and LIFTED automatically when any explicit range or
  `--since`/`--until` filter is given (Q-2).
- **L-FIL-1 (v0).** The v0 filter passthrough set is `--since`, `--until`,
  `--author`, `--grep`, `--no-merges`, `--first-parent` (Q-5; everything
  else deferred). Filters apply per-repository before interleaving, with
  entry-level semantics per L-COA-5.

### Output — human

- **L-OUT-1 (v0).** Every rendered entry MUST be attributed to its member
  SET, clearly (Q-1: "clear on which log entries applies to which set of
  member repos"): the compact rendering shows the workspace-relative paths
  when the set is small (≤3) and a count form (e.g. `[root+5]`) beyond
  that; `--full` and machine output always carry the complete set. A
  singleton renders as the plain single path.
- **L-OUT-2 (v0).** The DEFAULT rendering is the compact one-line-per-commit
  form `<date> <member-set> <short-hash> <subject>`; `--full` switches to
  git-style block format with a member table (Q-9).
- **L-OUT-3 (v2 — DEFERRED).** The grouped rendering (per-repository
  sections, forall-banner style) is deferred to v2 by Q-9's resolution
  (`gwz forall -- git log` covers the gap meanwhile). No v0 step owns it;
  it is listed under Deferred below.
- **L-OUT-4 (v0).** Degraded members are reported out-of-band of the entry
  stream: a stderr summary in human mode (Q-6), so machine parsing of
  stdout stays clean.
- **L-OUT-5 (v0).** ANSI color on tty only, `--color=always|never|auto`.
  **No pager** per Q-11's standing resolution — with the dated caveat that
  the resolution's premise was wrong (house behavior is NOT uniformly
  pager-less: `gwz diff` pages by default with `--no-pager` offered; S0.1
  review F12) and a one-line operator re-confirmation is pending in the
  Rezo. Until it lands, no-pager stands and the divergence from `gwz diff`
  is deliberate and recorded here.

### Output — machine

- **L-JSN-1 (v0).** `--json` (one document) and `--jsonl` (one record per
  entry) MUST be supported. Each entry record carries at minimum: a
  `members` array (one element per sibling: member id, member path, hash,
  parent hashes — singleton entries carry a one-element array, the same
  shape), merge provenance (→ L-COA-6), the shared author
  name/email/timestamp, committer name/email/timestamp, and subject. Body
  is subject-only by default; `--body` includes the full message (Q-12).
- **L-JSN-2 (v0).** Machine output MUST also represent degraded members
  (member id + reason) so consumers can distinguish "no commits" from
  "could not read".

### Protocol and Python surface (operator directive 2026-08-29)

- **L-PRO-1 (v0).** The protocol (`gwz-core/protocol/gwz.taut.py`) gains
  the log surface — request, response/entry stream, and degradation
  records — **additively only**: existing messages and slots stay
  byte-untouched, the regenerated artifacts land in BOTH repos
  (`gwz-core` generated.rs; `gwz-py` generated IR), and the protocol
  drift/regen check is green in both. Precedent: `StatusRequest`/
  `DiffRequest` message shapes and the `commit_marker` slot's two-client
  consumption.
- **L-PY-1 (v0).** `gwz-py`'s CLI MUST expose `gwz log` with the same
  operands, flags, defaults, degradation reporting, and exit semantics as
  `gwz-cli`'s, lowered through the same protocol request (per-family
  mirror precedent: `cli_diff.py`, `cli_read.py`).
- **L-PY-2 (v0).** The Python API (`client.py`) MUST expose the log
  programmatically — entries and degradations as structured records
  carrying the L-JSN-1/L-JSN-2 shapes (precedent: `async def diff` /
  `diff_output`).
- **L-PY-3 (v0).** Human rendering parity: `gwz-py` renders via its
  `cli_render` pattern; semantic parity with `gwz-cli`'s compact and
  `--full` forms is MUST, byte parity SHOULD where the existing render
  layers make it cheap. Machine output (`--json`/`--jsonl`) MUST be
  byte-compatible between the two clients.

### Exit codes

- **L-EXIT-1 (v0).** Exit codes follow the house convention: `0` success
  (including benign degradations, Q-13), `1` partial/failed (a selected
  member could not be read at all), `2` rejected (invalid invocation,
  unreadable workspace). `--strict` promotes any degradation to `1`. Core
  reports per-member outcomes in its aggregate status; the process exit
  mapping is `gwz-cli`'s (the existing `exit_code_for_response` seam).

### Performance

- **L-PRF-1 (v0).** The interleave MUST stream (k-way merge over per-repo
  cursors) with memory bounded by the L-COA-7 window — O(selected repos ×
  entries within W), never whole histories — so that bare `gwz log` on a
  20-member workspace with deep histories stays fast and bounded.
- **L-PRF-2 (v0).** Per-repository reads MAY run concurrently under the
  standard `--jobs` ceiling; output ordering must be unaffected by
  concurrency.

## Deferred (v2 candidates)

- The grouped (per-repository sections) rendering — Q-9's deferral
  (L-OUT-3) — and per-repository `--graph` under it (Q-8).
- **Artifact-assisted association**: using the marker artifact's
  `members: {…, commit: <sha>}` map to associate gwz-native-but-trailerless
  member commits (the root-only-`gwz commit` + plain-`git`-member-commit
  working pattern) with their workspace change — the coverage limit named
  in the Workspace commit concept.
- Lock-relative conveniences beyond `+lock..HEAD` (e.g. `+lock` on the
  root once the lock records a root state).
- Full `git log` format passthrough (`--format=...`);
  `--date-order=author`.
- `--follow`, extended diff-filter options, `-S`/`-G` pickaxe.
- Reading histories of unmaterialized members.

## Orientation for the implementing agent

You are implementing a new read-only verb in an existing, conventions-heavy
codebase. Before writing anything:

- Read `gwz-cli/dev-docs/AgentQuickStart.md` (repo working rules), then this
  document, the resolved `GwzLogAmbiguityRezo.md`, and the plan
  (`GwzLogPlan.md`). If something material is unresolved and blocks you,
  stop and ask rather than guessing.
- Precedents to study, in order: `gwz diff` (operand classification,
  `+snapshot` handling, pathspec routing after `--`, `--tagged`, and the
  root-omission behavior L-RNG-3 deliberately diverges from) — `gwz log`
  MUST reuse that classification machinery, not reimplement it; `gwz
  status` (multi-repo read orchestration and attributed rendering);
  `gwz snapshot` (snapshot identity); the landed commit-marker machinery
  (`gwz-core/src/workspace_ops/handle_commit.rs`, `artifact::MarkerArtifact`
  and friends, tests in `workspace_ops/tests/g13.rs`); `gwz stash`
  spec/plan in this directory (house style for a coordinated verb's spec,
  and the machine-parseable-marker pattern).
- Module home (operator amendment 2026-08-29): the new engine lives at
  **`gwz-core/src/operation/commit_log/`**, re-exported with minimum visibility
  through the existing `operation` seam. The originally adopted top-level
  home was refused by the checked-artifact boundary as "compiler root manifest
  changed" (exit 1) for a `lib.rs` export and "Rust source-loading edge
  inventory changed" (exit 1) for a path mount. Those inventories, pins and
  `lib.rs` remain untouched. This preserves F15's substance — a distinct
  commit-history home with no extension of or collision with
  `gwz-core/src/diff/log_service.rs` (`DiffLog`, the diff OUTPUT log) — and
  does not change the core-owns-semantics split. S2.0 dispatches through this
  existing seam; S2.1 implements here and its single-axis review checks both
  placement and minimal visibility.
- Split: `gwz-core` owns semantics (selection, operand resolution, per-repo
  cursors, coalescing, merge, tolerance, structured events, aggregate
  status); `gwz-cli` owns the clap surface — ALL log-specific flags — and
  rendering, including the exit-code mapping; `gwz-py` mirrors the whole
  client surface (L-PY-1..3) over the same protocol. Core's
  message-oriented API is the boundary — rendering consumes
  entry/degradation events. gwz-py precedents: `cli_diff.py` /
  `cli_read.py` (command mirrors), `cli_render.py` + `cli_render_parts/`
  (rendering), `client.py`'s `diff`/`diff_output` (API), and the
  protocol drift/regen tooling its release script runs.
- House rules that bind you: read-only commands never gate on conf
  integrity; no network; `gwz.conf/` is machine-managed (never hand-edit —
  structural changes only via the gwz CLI; READING artifacts through
  existing core APIs is of course in charter); every implemented v0 row —
  MUST or SHOULD — lands with tests (the multi-repo fixtures used by
  status/diff tests are the pattern); the repo gate (fmt, clippy
  `-D warnings`, full test partitions) must be green before any landing.
