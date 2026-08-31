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
  is a path. Repository routing MUST be Git-pathspec-magic-aware: preserve
  the complete long-form `:(...)` envelope and the short-form `:!` / `:^`
  prefix byte-for-byte, and reroot only the pattern payload. Root and member
  subdirectory routing, top magic, and workspace-root fan-out MUST match
  native `git rev-list`'s complete commit sequence in fixtures that include `.`
  and companion exclusions.
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
- **L-RNG-6 (v0) — snapshot-ID/range compatibility.** Snapshot artifacts
  remain schema `gwz.snapshot/v0`; read-side compatibility is permanent.
  Listing, reading, and standalone exact `+<id>` access MUST continue to
  accept every previously valid v0 ID, including IDs with adjacent dots or
  a leading/trailing dot. Creation validation alone MUST reject NEW IDs
  containing adjacent dots or a leading/trailing dot; separated internal
  dots remain valid. An exact whole-token match to a stored legacy ID MUST be
  treated as standalone access before range interpretation. A legacy
  ambiguous `+` snapshot endpoint may therefore be used standalone, but when
  it participates in `..` or `...` the shared operand grammar MUST return a
  typed teaching refusal rather than silently choosing another snapshot or
  range meaning. Snapshot validation and operand parsing MUST be shared by
  diff and log, not forked. The artifact module MUST carry an explicit
  schema-v0 compatibility note at the creation/read validation split. Named
  creation/read/parser tests plus end-to-end diff AND log regressions MUST
  cover internal, adjacent, leading, and trailing dots on range boundaries.
  (Lane-owner-dictated amendment, `S2.2 terminal NO-GO, F5/F6`, 2026-08-29.)

### Coalescing (Q-1's resolution — normative)

- **L-COA-1 (v0).** Sibling commits carrying the same valid
  `GWZ-Commit-ID` trailer value MUST coalesce into one entry whose member
  set is every selected repo carrying that value. A value is valid iff it
  is canonical lowercase RFC-4122 text with version 7 and the RFC variant
  bits (`10xx` in octet 8); this is the strict reading of
  `GwzCommitMarker.md`'s "Lowercase canonical UUID text. Version 7 only."
  Only valid values key marker coalescing. The trailer is landed
  production machinery (see the Workspace commit concept); this row is
  exercised against real history from day one. (Lane-owner-dictated
  amendment, S2.4 terminal NO-GO, 2026-08-29.)
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
  `marker:<uuid>`, `marker-invalid`, or `heuristic` (or `none` for ordinary
  singletons), so consumers can distinguish proven identity, invalid
  marker claims, and inference. The `marker-invalid` token is additive.
  (Lane-owner-dictated amendment, S2.4 terminal NO-GO, 2026-08-29.)
- **L-COA-7 (v0) — the bounded coalescing window (the L-COA-4 ↔ L-PRF-1
  contract; S0.1 review F3, ownership fixed per round-2 F21; CLOSURE
  RULE REPLACED 2026-08-30 by the S2.5 terminal review's own analysis —
  its F3 proved the original "every live cursor has advanced past"
  phrasing unimplementable with bounded memory under the accepted
  non-monotone envelope).** The streaming merge MUST hold emission only
  within a bounded reorder window **W = 60 seconds** of stream time,
  closed by the **group-eligible frontier rule**: a group MUST close
  once EVERY live cursor satisfies ANY of —
  (a) it has yielded, at any point, an entry with committer instant
  below (group newest − W) — seen-below-boundary, order-independent,
  so inversions cannot un-satisfy it;
  (b) it is exhausted;
  (c) it is already REPRESENTED in the group (same-repo siblings are
  forbidden by L-COA-2, so it can contribute nothing further);
  (d) **bounded patience**: it has yielded **K = 64** entries since the
  group became closure-pending without satisfying (a)-(c) — the group
  closes anyway.
  **A closed group is IMMUTABLE** — output-blocked or not, no later
  sibling is ever absorbed into it; any later compatible entry is a
  window FRAGMENT (a separate entry sharing the provenance key —
  L-ENV-2's mechanism, which rules (c) and (d) feed by construction).
  Buffering is therefore hard-bounded at O(selected repos × (entries
  within W + K)) for ANY input, monotone or not.
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
- **L-COA-8 (v2 — DEFERRED) — identity stability.** One workspace-level
  change SHOULD carry one `GWZ-Commit-ID` across partial-commit retries,
  and v2 will use artifact-assisted association to heal retry splits
  without false fusion. **There is no v0 retry-identity guarantee:** the
  shipped writer may mint a fresh UUIDv7 after a partial commit, and v0
  `gwz log` renders the resulting marker groups separately because
  L-COA-2 forbids heuristic healing across marked commits. This is the
  pre-authorized S1.1-B terminal fallback, executed 2026-08-30 after the
  final review returned NO-GO (`GwzLog-S1.1-B-Review.md`): the capped
  read-only proof could not cover ordinary Git cleanup configuration and
  crash-cut evidence safely without guessing. No v0 step owns this row.
- **L-COA-9 (v0) — invalid marker disposition.** A commit whose
  `GWZ-Commit-ID` trailer key is present in any recognizable form but whose
  value fails L-COA-1's validity rule MUST NOT join any heuristic group
  (it "carries a marker" per L-COA-2/F24) and MUST NOT marker-coalesce. It
  MUST render as a singleton with machine provenance `marker-invalid`;
  invalid identity always fails toward splitting. Detection is deliberately
  asymmetric: exclusion uses broad detection of any trailer-like line whose
  key is `GWZ-Commit-ID`, tolerating separator mangling such as `=` for `:`,
  while marker keying requires the strict canonical Git-trailer colon form
  and a valid value under L-COA-1. Broad exclusion, strict keying — fail
  closed in both directions. (Lane-owner-dictated amendment, S2.4 terminal
  NO-GO, 2026-08-29.)

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
  consumption. **Operator-approved S2.7 rider (2026-08-31):** every message
  and slot that predates the log surface remains byte-untouched. Within the
  still-unshipped `LogEntry` alone, slot 7 (`ordering_timestamp_ms`) becomes
  optional, and additive slots 8–11 carry exact author seconds, committer
  seconds, ordering seconds, and the source-byte `lossy` fact. This is the
  narrow exception forced by L-ENV-1/L-ENV-12; no other protocol shape moves.
- **L-INT-1 (v0) — public dispatch and output lifecycle.** The public core
  `operation::handle_log` seam MUST execute the completed commit-log engine
  using a caller-owned, operation-scoped commit-log output registry. A
  successful request MUST mint one non-empty opaque `log_id`, project every
  merged group and degradation exactly once and in engine emission order into
  valid `LogOutputRecord` messages, seal the output, and return a `LogResponse`
  whose final aggregate status is the S2.6 aggregate and whose output id
  resolves in that same registry. The public reader MUST provide an opaque
  cursor, bounded batch reads, explicit EOF, typed refusal for unknown,
  released, or invalid cursors, and idempotent release. Projection or spool
  failure MUST return a typed core error and leave no resolvable id.

  Retention MUST stay bounded in memory for explicit ranges and no-limit: the
  finite result is held in an anonymous/automatically removed process-temp
  spool, not a whole-history collection. The spool is operation state, never a
  workspace/repository write; the command remains read-only, lock-free,
  network-free, and independent of `gwz.conf` integrity. Coalesced members are
  projected in `(member_id, commit)` order; the least such sibling is the
  deterministic representative for the shared text and identities when hooks
  made siblings differ, while the group's latest admitted committer instant
  remains the ordering time. The service lives under
  `src/operation/commit_log/`, is re-exported through `operation` with only the
  visibility clients require, and MUST NOT use or modify the unrelated diff
  output-log service. (Operator-approved amendment, 2026-08-31.)
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

**L-RNG-6 addendum (2026-08-30, from the S2.2 terminal re-charter
review's exact-source probes):** the typed teaching refusal applies at
**open range boundaries too**. After exact whole-token stored-id matching
has been given its chance, a stored ambiguous legacy id participating at
an OPEN `..` or `...` boundary (`+legacy..`, `..+legacy`, and the
three-dot forms) is a participating endpoint and MUST refuse — never
silently resolve to a shorter stored id (`+trailing...` must not select
snapshot `trailing`; `+adjacent..dots..` must not become
`Snapshot("adjacent")..Revision("dots..")`). Standalone exact legacy
matching stays valid.

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

## Validity envelopes (the S0.2 sweep, EXECUTED 2026-08-30)

One pass minting the previously-undefined input/edge envelopes for every
remaining unbuilt step, after four envelope-class terminal findings
(marker validity; pathspec magic; dotted ids twice; the L-COA-8 trust
envelope). Each row names its owning step; step reviews key on these rows
exactly like L-* rows. House defaults honored: lossy conversion is the
standing non-UTF-8 idiom (`to_string_lossy` throughout gwz-cli); machine
outputs carry `schema` fields (the `git_status_json`/`gwz.protocol/v0`
precedent); refusals teach; read sides never break.

**Ordering, time, and the window — owner S2.5:**

- **L-ENV-1 (v0).** The ordering key is the committer time's ABSOLUTE
  instant (epoch seconds, i64); the recorded UTC offset affects display
  only, never order. Any timestamp git records is accepted — pre-epoch
  (negative) and far-future values included, no clamping, no warnings
  (present what the records say). Total order: epoch seconds, then the
  deterministic tiebreak — for coalesced entries the tiebreak key is the
  lexicographically-least sibling member id, then that sibling's hash.
  The public log record carries the exact author, committer, and ordering
  seconds. Its legacy millisecond convenience slot is optional and MUST be
  absent, never clamped or wrapped, when exact conversion would overflow.
- **L-ENV-2 (v0).** The L-COA-7 window boundary is INCLUSIVE: a sibling
  joins its group iff its committer instant ≥ (group's newest − W).
  Cursors are NOT assumed monotone (git's default order can invert
  around topology): an entry arriving BEHIND the emission frontier that
  would have joined an already-emitted group emits separately under the
  same provenance key — the L-COA-7 escape extended to frontier
  violations, tested with a deliberately non-monotone fixture.
- **L-ENV-3 (v0).** Reaching the depth cap terminates the walk; any
  group still open at termination closes as-is with the siblings seen so
  far (cap truncation is honest truncation — the machine record carries
  no false completeness claim; L-JSN-1's members[] is what was seen).
  `-n` takes an unsigned integer (clap rejects negatives); `--jobs`
  inherits the existing global's validation unchanged. **Sentinel
  addendum (2026-08-30, S2.5 terminal):** after cap termination the
  merge MUST NOT request further cursor yields, and the regression MUST
  prove it by killing a beyond-cap-read mutant — an instrumented
  yield-counting cursor asserting zero post-termination yields, not an
  output-only check.
- **L-ENV-4 (v0).** Determinism row: identical inputs produce
  byte-identical output for every `--jobs` value — L-PRF-2 made
  testable. **High-water addendum (2026-08-30, S2.5 terminal F3):** the
  L-PRF-1 memory-bound regression MUST use NON-MONOTONE cursors in the
  terminal review's own probe shape (an inverted frontier followed by a
  long W-sparse tail, two cursors): max buffered entries MUST stay flat
  as the post-inversion tail grows. A monotone-only high-water test is
  insufficient by demonstrated counterexample (23 → 103 on constant
  window density).

**Filters — owner S2.6:**

- **L-ENV-5 (v0).** `--grep` and `--author` take Rust `regex`-crate
  patterns (a NAMED divergence from git's POSIX flavors, stated in
  help); `--grep` matches the FULL message (subject + body) regardless
  of `--body`; `--author` matches the combined `Name <email>` string;
  matching is case-sensitive; an invalid pattern is a typed refusal at
  invocation (exit 2) naming the pattern error. No locale-dependent
  behavior anywhere: comparisons are over bytes and absolute instants.
- **L-ENV-6 (v0).** `--since`/`--until` accept RFC3339/ISO-8601
  timestamps (date-only forms mean local midnight; offset-less forms are
  LOCAL time) and `@<epoch-seconds>`; anything else — including git's
  approxidate ("yesterday") — is a typed teaching refusal naming the
  accepted forms (approxidate is a v2 candidate). Both bounds are
  INCLUSIVE and compare against the committer instant.
- **L-ENV-7 (v0).** Filters run per-repo pre-merge (L-COA-5): a
  coalesced group whose siblings are partially removed by a filter
  (e.g. `--no-merges` removing one repo's merge sibling) renders with
  the SURVIVORS — the same narrowing rule as selection, one rule, no
  special case. An empty result set is success: empty stdout, exit 0,
  no degradation.

**Flags and process behavior — owner S3.1:**

- **L-ENV-8 (v0).** `-n` conflicts with `--no-limit` (clap-refused);
  `--color` admits exactly always|never|auto; repeated flags follow
  clap's standard last-wins/deny semantics — no bespoke handling.
  Exit precedence: invalid invocation (2) beats everything; otherwise
  the worst observed class wins (read-failure 1 over degraded-ok 0;
  `--strict` promotes per L-EXIT-1).
- **L-ENV-9 (v0).** EPIPE on stdout (e.g. piped to `head`) is clean
  early termination: stop emitting, exit 0, no error spray — the
  composability twin of the no-pager decision.

**Human rendering — owner S3.2:**

- **L-ENV-10 (v0).** Non-UTF-8 in messages, names, or paths renders
  lossy (U+FFFD), never panics. Subjects are the first line; C0 control
  characters in rendered subjects/names are sanitized (tab becomes one
  space; other C0 bytes become U+FFFD) — terminal-escape injection via
  commit message must be impossible in human mode. Machine mode
  preserves content via JSON escaping instead (L-ENV-12).
- **L-ENV-11 (v0).** The compact line's date renders as
  `YYYY-MM-DD HH:MM:SS ±hhmm` in the COMMIT'S OWN recorded offset
  (present what the records say; no locale, no local-time conversion).
  No width-aware truncation anywhere in v0 (pipe-friendly); `--color=
  auto` keys on stdout tty-ness only. Zero entries: empty stdout,
  exit 0.

**Machine output — owner S3.3 (byte-parity binds S3.6):**

- **L-ENV-12 (v0).** JSON is UTF-8: any field whose source bytes were
  not valid UTF-8 is emitted lossy AND the entry carries `"lossy":
  true` (absent otherwise) — cheap honesty consumers can key on.
  Hashes are full 40-char lowercase hex; parents keep git's recorded
  order; times are `{"time": <epoch_seconds>, "offset_min": <n>}`;
  JSONL records are guaranteed single-line (JSON escaping suffices).
  Core protocol projection records this source-byte fact in the additive
  log-entry `lossy` field; clients MUST NOT infer it by searching for U+FFFD.
- **L-ENV-13 (v0).** Machine output is schema-tagged per house
  precedent: the `--json` envelope object carries
  `"schema": "gwz.log/v0"`; `--jsonl` begins with a header record
  `{"record": "header", "schema": "gwz.log/v0"}`; every subsequent
  record carries `"record": "entry"` or `"record": "degradation"`.
- **L-ENV-14 (v0).** gwz-py's machine output byte-matches gwz-cli's
  for identical inputs INCLUDING the lossy rule — the same U+FFFD
  substitution points (L-PY-3's byte-compatibility made testable at
  the envelope's hardest edge).

**Battery additions — owner S3.4:** fixtures for: a non-UTF-8 path and
a non-UTF-8 + C0-control commit message (human sanitization AND machine
lossy-flag asserted, both clients); pre-epoch and far-future timestamps;
equal-timestamp tiebreak determinism across `--jobs` values; the
inclusive W-boundary case; the non-monotone-cursor frontier case; an
invalid `--grep` pattern and an approxidate `--since` refusal.

**S4.1:** the traceability sweep covers L-ENV-1..14 like every other
implemented row.

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
- Public integration (operator amendment 2026-08-31): S2.7 replaces S2.0's
  refusal at that existing operation seam and adds the commit-history output
  registry under the same module home. The registry is a bounded-memory,
  automatically removed process-temp spool with cursor/EOF/release semantics;
  it is not `diff::log_service`, does not extend that subsystem, and never
  writes the workspace.
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
