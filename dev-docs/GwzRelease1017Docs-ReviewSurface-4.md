# GwzRelease1017Docs Surface review — round 4

Tuple verified at start: root `301a140187101f1a07a0c071b4a2386d38f2e7ac`, gwz-cli `8702b6863166619c7e996d180363677545fd720a`, gwz-core `8a457d1f2bf1423400b428f9ef9d5e4a0f1c7b66`, gwz-py `fc42c8ab2dff5cde376de28f79b189690bbbf497`, working tree clean ; at end: identical, clean.

**Binary provenance — one discrepancy, resolved behaviourally.** The coordinator stated the binary's mtime would be "after 20:55". The actual mtime is **20:46**, and the wall clock during this review was 20:49, so 20:55 had not yet occurred. The binary is nevertheless not stale and is not the round-3 one: it differs in size from round 3's (14,596,144 vs 14,619,392 bytes), and it demonstrably contains gwz-core `8a457d1f`'s change — `gwz --dry-run --remote nope fetch` now refuses instead of printing `would contact nope`, which is behaviour that did not exist at the round-3 tuple. The source commits are timestamped 20:48:54, three minutes after the build, which is the ordinary build-then-commit order. I record the stated-mtime mismatch for the audit trail; it does not affect the verdict, because every finding below rests on observed behaviour rather than on build metadata.

Change scope confirmed independently: `git -C gwz-cli diff --stat 5270fe4 8702b68` touches `docs/CLI.md`, `docs/MachineOutput.md`, `docs/Releases.md`, `docs/commands/fetch.md`, `src/fetch_long.rs` and two `dev-docs/` report files; `git -C gwz-core log --oneline 475947d..8a457d1` is the single commit "Refuse a `--remote` name a repository lacks before the network, on both paths". `docs/ClaudeCode.md`, `docs/LocalClones.md`, `docs/commands/local.md` and `skills/gwz/SKILL.md` are untouched since the round-2 tuple.

Peer-blind maintained: `dev-docs/GwzRelease1017Docs-ReviewConsistency-3.md` is now tracked in this tuple and I did not open it.

## Verdict: GO

Both open findings are closed. No new findings at P0–P3. No regression in any previously closed finding.

## Closure table

| ID | Status | Evidence |
| --- | --- | --- |
| **P2-4** — `--dry-run --remote <name>` promised `would contact <name>` and exited 0 for a remote no repository has, while the live run failed every row | **Closed** | Fixed in gwz-core, which is the remedy I recommended over the documentation-only alternative. Minimal reproduction, one repository with no git remote configured at all: `gwz --dry-run --remote nope fetch` and `gwz --remote nope fetch` now print **byte-identical** output — `status: Rejected`, then `@root  .  failed  MissingRemote: workspace root '@root' at '.': missing remote 'nope'` — and both exit **2**. The string `would contact nope` no longer appears on any path. Parity also holds in machine output: on the two-member workspace, both paths return `meta.aggregate_status = "Rejected"`, `fetch_repos` results `[Failed, Failed]` and `members` statuses `[Rejected, Rejected]`, identical field for field. Documented on all four surfaces: `docs/commands/fetch.md`'s "A dry run contacts nothing" section ("A `--remote <name>` that a repository does not have is refused here too, as `failed` with `MissingRemote`, because that answer is also local: a dry run never promises to contact a remote the live run cannot"), `docs/Releases.md`, `docs/MachineOutput.md`, and `src/fetch_long.rs` with `docs/CLI.md` regenerated. |
| **P3-7** — exit code `2` was documented but unreachable; three different refusals all exited `1` | **Closed** | The table now reads `2` = "every selected repository was refused before the network, so nothing was contacted", and both `docs/commands/fetch.md:124-140` and `docs/Releases.md` add the boundary paragraph distinguishing a whole-batch refusal from a pre-selection typed error, and state that a dry run shares the live codes. Every published row now has a reproducible command, live and dry identical in all eight pairs (see matrix below). The two stated exceptions reproduce exactly as written: no workspace → `gwz: WorkspaceNotFound: gwz.conf/gwz.yml missing`, exit 1; unknown member id → `gwz: MemberNotFound: member id not found`, exit 1. |

## Exit-code matrix (published table vs binary)

| Published row / stated exception | Command | live | dry-run |
| --- | --- | --- | --- |
| `0` every selected repository answered | `fetch --target @root` (healthy) | `Noop`, exit 0 | `Noop`, exit 0 |
| `1` some answered and some failed | `fetch` with `mem_gone`'s directory deleted | `Partial`, exit 1 | `Partial`, exit 1 |
| `2` every selected repository refused before the network | `--no-target mem_gone --remote nope fetch` | `Rejected`, exit 2 | `Rejected`, exit 2 |
| exception: no workspace → typed error, exit 1 | `fetch` outside a workspace | `WorkspaceNotFound`, exit 1 | `WorkspaceNotFound`, exit 1 |
| exception: unknown member id → typed error, exit 1 | `--target mem_nope fetch` | `MemberNotFound`, exit 1 | `MemberNotFound`, exit 1 |
| boundary I added: refusal plus an unmaterialized member | `--remote nope fetch` with `mem_gone` deleted | `Rejected`, exit 2 | `Rejected`, exit 2 |

The boundary row is consistent with the table's wording: `MemberNotFound: member is not materialized` is itself a refusal before the network, so every selected repository was refused and nothing was contacted. Live and dry produced identical rows here too.

## Re-verification of the specified cases

- **P2-4 closure test** — same `status:` line, same exit code, no `would contact nope`: **pass**, as detailed above.
- **Mixed `Partial`/1 case** — still correct and unaffected by the change. `gwz fetch` and `gwz --dry-run fetch` on a workspace with `@root`, a healthy `mem_good` and a deleted `mem_gone` both print `status: Partial`, both exit 1, and both carry the same failure row `mem_gone  gone  failed  MemberNotFound: member 'mem_gone' at 'gone': member is not materialized`. JSON aggregates agree (`Partial` / `Partial`).
- **P2-1's fix survives the round-4 change.** This was the risk worth checking: a change that makes the dry run refuse more could have collapsed the `Planned` token. It did not. A dry run over a repository that genuinely would be contacted still prints `@root  .  would contact origin` with `"result": "Planned"` and `members[].status = "Planned"`, and the mixed case still shows `[Planned, Planned, Failed]` against the live run's `[Unchanged, Unchanged, Failed]`. The one-meaning-per-token property that P2-1 was about is intact.

## Regression sweep on earlier closures

- `docs/ClaudeCode.md`, `docs/LocalClones.md`, `docs/commands/local.md` and `skills/gwz/SKILL.md` are unchanged since the round-2 tuple where I closed P2-2, P2-3 and P3-1 through P3-6, so those closures stand on text already verified. Spot greps re-confirm: no `force <categories>`, no "names the minimum", no `S<n>.<m>` identifiers, no "until an integrated lane disposes".
- Because gwz-core changed, I re-ran the dispose surface on the new binary rather than assuming it: a lane whose session created `.claude/.cc-writes/` still refuses as `unique to the lane 1`, offering `gwz local dispose Z --force dirty` — matching P2-2's corrected page — and `--force dirty,unpreserved-history` still reports `forced past: dirty; unused waiver: unpreserved-history`.
- `docs/CLI.md` re-diffed against the binary for `gwz fetch`, `gwz local dispose` and `gwz hook claude-code setup`: all three **MATCH** under trailing-whitespace normalisation, including the rewritten exit-code sentence in the fetch long help.

## New findings

**None at P0–P3.**

## What I could not test

Unchanged from rounds 2 and 3, and none of it bears on the two findings closed here: a real pre-1.0.14 binary for the index-schema refusal; `gwz fetch` with a coordinated merge open; the hooks driven by a live Claude Code session; `--max-lanes`, `--min-free-gb` and the free-space estimate; Windows. One note specific to this round: my axis forbids reading the implementation, so I cannot prove by inspection that no *other* refusal path disagrees between the live and dry paths — I can only report that every case I could construct now agrees, which is a broader set than the one I was asked to check.
