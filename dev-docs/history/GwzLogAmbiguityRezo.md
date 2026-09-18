# GWZ Log — Ambiguity Resolution

Status: **RESOLVED 2026-08-29** — all thirteen questions answered by the
operator; Q-2 through Q-13 endorse the proposed defaults as written; Q-1
confirms the heterogeneous-histories reading and **sharpens the product**:
same-workspace-commit entries coalesce (see the confirmed reading under
Q-1). `GwzLogRequirements.md` is reconciled to this file as of the same
date; this file remains authoritative where the two disagree.

Post-review addendum (2026-08-29, after the S0.1 round-1 review): two
dated lane-owner notes below carry corrections and each ends in a fresh
**Resolution (Gianni):** line — Q-1 (a factual correction to the lane
owner's annotation: the commit marker is LANDED, not proposed; plus a
one-line breadth re-confirmation on the heuristic) and Q-11 (the pager
premise was wrong; standing "no pager" governs until you flip it). Blank
still means the standing resolution stands.

Companion to [GwzLogRequirements.md](GwzLogRequirements.md); the `Q-n`
references there point here. Write your answer under each
**Resolution (Gianni):** line — a bare "yes" endorses the proposed default.
Contract: **a question left blank means the proposed default stands**, so you
only need to comment where you disagree or want to sharpen. Once commented,
the requirements doc gets reconciled to this file and implementation follows
this file where the two disagree.

---

## Q-1. What does "tolerate repos which have individual log records" mean?

I read it as: member repositories have fully independent histories —
unrelated roots, different branches, detached HEADs, unborn branches, refs
that exist in some members and not others — and the unified log must handle
all of that per-member rather than failing wholesale. The requirements doc is
written to that reading (the whole Tolerance section).

If you meant something more specific — e.g. members that keep their own
changelog/notes conventions, or members whose local branches have diverged
from their remotes — say so and I'll rework the scope.

**Proposed default:** the heterogeneous-histories reading above.

**Resolution (Gianni):** Correct on hetero-histories. In essence 'gwz log' should merge log entries where they came from the same commit. The output should be clear on which log entries applies to which set of member repos.

> **Confirmed reading (lane owner, 2026-08-29), now normative in the
> requirements:** "the same commit" = the sibling git commits created across
> members and the root by **one coordinated `gwz commit` operation** — one
> logical workspace-level change with N per-repo SHAs. `gwz log` coalesces
> those siblings into ONE entry attributed to the **set** of repos that
> carry it (a plain single-repo commit is the singleton case, rendered the
> same way). Merge identity: the `GWZ-Commit-ID` trailer per
> `gwz-core/dev-docs/GwzCommitMarker.md` (UUIDv7 per commit operation —
> that doc's own motivation names "merged `gwz log`") when present; plus a
> conservative heuristic for history predating the marker (identical full
> message + author identity, committer AND author timestamps within a
> small window, different repos only), flagged as heuristic in machine
> output, with `--no-coalesce` to see raw per-repo entries. Requirements:
> the Coalescing section (L-COA-*).
>
> **Correction (lane owner, 2026-08-29, from the S0.1 round-1 review's
> P0):** this annotation originally claimed the marker design was "status
> *proposed*" with its trailer half "this project's dependency/companion".
> That was FALSE — the marker is landed, shipped machinery in daily
> production use, BOTH halves (trailers minted in
> `handle_commit.rs`; artifact written to `gwz.conf/markers/` — 316 on
> disk at review time; this workspace's own commits carry the trailers),
> verified independently by the reviewer and re-verified by the lane
> owner. The stale line is `GwzCommitMarker.md`'s own `Status: proposed`
> header, which plan step S1.2 corrects at source. Consequences folded:
> the plan's Phase 1 is now the marker AUDIT (the retry-identity defect,
> review F5 / requirement L-COA-8), not a build; L-COA-1 exercises
> against real history from day one. Gianni's resolution text above is
> untouched by this correction.
>
> **Breadth re-confirmation (one line, when convenient) — S0.1 review
> F18:** the heuristic as specified also coalesces same-message
> `gwz forall -- git commit`/`cherry-pick` fan-outs inside the window —
> commits that were NOT one `gwz commit` operation. Current requirements
> say YES, coalesce them, labeled `heuristic` (L-COA-2). If you meant
> strictly one-operation-only, say so and the heuristic narrows.
>
> **Resolution (Gianni):**
---

## Q-2. Default depth for bare `gwz log`

Unbounded interleaved history across ~20 members is unusable and slow to
terminate. Options:

- (a) **Workspace-global cap** — newest N entries across the whole stream
  (proposed N=50). Honest "what happened lately" view; quiet repos may show
  nothing.
- (b) **Per-member cap** — newest N per member (say 10). Every member
  guaranteed representation, but the tail is dominated by dormant repos'
  ancient commits, which makes the interleave misleading.
- (c) **Unbounded**, exact git parity.

`-n <N>` sets the cap explicitly; `-n 0` (or `--no-limit`) removes it.
Explicit ranges/`--since` also lift the default cap (you asked for a window,
you get the window).

**Proposed default:** (a) global cap, N=50, lifted when any range/filter is
given.

**Resolution (Gianni):** ok

---

## Q-3. Is `@root` in the default selection?

`gwz push` and `gwz status` treat the root repo as a first-class target, and
root commits (dev-docs, conf changes) are part of workspace history.

**Proposed default:** yes — `@root` plus all members, standard exclusions
(`--no-target @root`) available.

**Resolution (Gianni):** as proposed

---

## Q-4. Interleave timestamp: committer or author date?

Cross-repo ordering has to key on some timestamp. Committer date reflects
when the commit actually entered the repository (rebases/cherry-picks get
re-stamped); author date preserves original authorship time but interleaves
rebased work back into the past, which reads oddly in a "what happened"
stream.

**Proposed default:** committer date for ordering; both dates carried in
machine output; a `--date-order=author` escape hatch only if it falls out
cheaply (otherwise defer).

**Resolution (Gianni):** as proposed

---

## Q-5. v0 filter passthrough set

Proposed v0 set: `--since`, `--until`, `--author`, `--grep`, `--no-merges`,
`--first-parent`, `-n`. Everything else (`-S`/`-G` pickaxe, `--follow`,
diff-filters, full `--format`) deferred.

**Proposed default:** the set above.

**Resolution (Gianni):** as proposed

---

## Q-6. Missing-ref policy and degraded-member reporting

When an operand (`v0.11.0..`, a branch name, a `+snapshot` endpoint) resolves
in only some selected members:

- (a) **Degrade-and-note (proposed):** members where it doesn't resolve are
  skipped, with a per-member note (stderr summary in human mode; explicit
  degraded records in `--json`/`--jsonl`); exit stays 0. A `--strict` flag
  turns any degradation into a hard error for scripting.
- (b) **Hard error by default**, `--lenient` to opt into skipping.

**Proposed default:** (a), with `--strict` available.

**Resolution (Gianni):** as proposed

---

## Q-7. Does `--tagged` apply to `gwz log`?

`gwz diff --tagged` narrows the selection to repositories containing every
supplied local tag. The same narrowing is natural for e.g.
`gwz log --tagged v0.11.0..v0.11.1`.

**Proposed default:** yes, same semantics as diff.

**Resolution (Gianni):** as proposed

---

## Q-8. `--graph`

A cross-repo graph is meaningless (no shared ancestry). A per-repo graph is
only coherent inside a grouped (per-repo sections) rendering.

**Proposed default:** no `--graph` in v0 at all; revisit under the grouped
mode in v2.

**Resolution (Gianni):** sure - as proposed

---

## Q-9. Default human rendering

- (a) **Compact one-line-per-commit (proposed default):**
  `<date> <member-path> <short-hash> <subject>` — columnar, the natural shape
  for an interleaved stream. `--full` switches to git-style block format
  (with a `Member:` line).
- (b) **Git-style blocks by default** (muscle-memory parity), `--oneline`
  to compact.

And: does the **grouped** rendering (per-member sections, forall-banner
style) need to be in v0, or is interleave-only acceptable for v0?

**Proposed default:** (a) compact default + `--full`; grouped mode deferred
to v2 (`gwz forall -- git log` covers the gap meanwhile).

**Resolution (Gianni):** yes - as proposed

---

## Q-10. Lock-relative range ("what moved since the recorded state")

Each member logging `<lock-recorded pin>..HEAD` is the workspace-native view
plain git can't express, and it's cheap (pins are already in the lock). But
it's also new range surface: a spelling like `gwz log --since-lock` (or a
pseudo-operand like `+lock..HEAD`, reusing the `+` namespace snapshots use).

**Proposed default:** in v0, spelled `+lock` as a pseudo-snapshot operand
(so `gwz log +lock..` and `gwz diff +lock..` can eventually share the
mechanism); if core classification makes that awkward, fall back to
`--since-lock` and note the diff-parity gap.

**Resolution (Gianni):** sure

---

## Q-11. Pager and color

Other gwz commands write straight through (no pager) and this suits
composition; git users expect `git log` to page.

**Proposed default:** house behavior — no pager, ANSI color on tty only,
`--color=always|never|auto`. Users who want paging pipe to `less -R`.

**Resolution (Gianni):** yes - no pager

> **Premise correction + re-confirmation (lane owner, 2026-08-29 — S0.1
> review F12):** the question's premise ("other gwz commands write
> straight through — no pager") was WRONG for the nearest neighbour:
> **`gwz diff` pages by default** and offers `--no-pager`
> (`gwz-cli/src/pager.rs`, wired in `diff_exec.rs`). Your "no pager"
> answer therefore makes `gwz log` the one long-output read command that
> does not page, diverging from the command its grammar is modelled on.
> The standing resolution (no pager) GOVERNS until you say otherwise —
> one line here flips it to diff-parity (page by default, `--no-pager`
> offered) if that is what you actually want.
>
> **Resolution (Gianni):**

---

## Q-12. Commit body in machine output

Full bodies bloat `--jsonl` streams; subjects usually suffice for tooling.

**Proposed default:** subject only by default; `--body` includes the full
message in both human `--full` and machine records.

**Resolution (Gianni):** sure

---

## Q-13. Exit code for benign degradation

With Q-6(a): skipped members (missing ref, unborn) are benign. Should the
command still exit 0, reserving 1 for real read failures (repo unreadable),
2 for invalid invocation?

**Proposed default:** yes — 0 with benign degradations; 1 only when a
selected member could not be read at all; 2 rejected invocation. `--strict`
promotes any degradation to 1.

**Resolution (Gianni):** sure
