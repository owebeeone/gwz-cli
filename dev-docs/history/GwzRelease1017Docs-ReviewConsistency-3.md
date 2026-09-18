# gwz 1.0.17 release documentation — Consistency review, round 3

Tuple verified at start: root `ad8a21a6a1bd1c95d53186b690502bbd31770aee`, gwz-cli `5270fe4c359dda7bb54d9ae69b0b361770ebec66`, gwz-core `475947db5c8238304c53e5bd86e248dd56b54ae1`, gwz-py `fc42c8ab2dff5cde376de28f79b189690bbbf497` (unchanged from round 2), lane `/Users/owebeeone/limbo/gwz-dev-docs-release`, `df -h /Users/owebeeone/limbo` = 9.0 GiB available, binary `target/release/gwz` rebuilt at 20:40 (`gwz 0.2.0-dev`) ; at end: identical, all three member worktrees clean.

Prior rounds: `GwzRelease1017Docs-ReviewConsistency.md` (round 1, NO-GO, P2-1..P2-3, P3-1..P3-5 — all closed at round 2), `GwzRelease1017Docs-ReviewConsistency-2.md` (round 2, NO-GO, one open finding C2-P2-1). Remediation commits this round: gwz-core `475947db`, gwz-cli `5270fe4` (report filing only, two dev-docs files, no docs or source). Read-only throughout; no cargo, no builds, no git mutations; throwaway workspaces only under `…/scratchpad/consistency-review/`.

## Verdict: GO

## Closure table

| ID | Status | Evidence at the round-3 tuple |
| --- | --- | --- |
| **C2-P2-1** — a `--dry-run` fetch with any refused repository reported `Rejected` and exited 2 although rows were planned, contradicting `GwzFetchPlan.md` §3.5, `handle_fetch.rs`'s own doc comment, and the shipped exit tables | **Closed** | Fixed by option 1, as specified. `gwz-core/src/workspace_ops/handle_fetch.rs:461-470` now matches `Updated \| Unchanged \| Planned` in `contacted`, and the doc comment at `:456-461` states the reason ("A `Planned` row counts as contacted here: under `--dry-run` it stands for the repository the live run would have read, so a dry run and the live run of the same selection aggregate alike"). Original reproduction re-run below: the mixed dry run is now `Partial`, exit 1, byte-identical in aggregate to the live run of the same selection. |

### Reproduction, re-run verbatim

Throwaway workspace, root on `origin`, one member, the member's repository then removed (`member is not materialized` — the case a quiet-cloned private member also reaches):

```
### B. one member unmaterialized (the C2-P2-1 repro)
--- dry-run:
status: Partial
@root   .   would contact origin
mem_r3  r3  failed                MemberNotFound: member 'mem_r3' at 'r3': member is not materialized
exit=1
--- live:
status: Partial
@root   .   no change  (origin/main, +0 -0)
mem_r3  r3  failed     MemberNotFound: member 'mem_r3' at 'r3': member is not materialized
exit=1
```

At the round-2 tuple this dry run printed `status: Rejected` and exited 2. The dry run no longer fails harder than the live run it rehearses.

### Boundary cases checked, so the fix is not a swap of one inconsistency for another

| Case | Dry run | Live run | Against |
| --- | --- | --- | --- |
| Every selected repository healthy | `Noop`, exit 0 | `Ok`, exit 0 | Exit codes identical; the aggregate differs only because a live run can observe movement and a plan cannot, which is the intended distinction and shares exit 0 |
| One planned row, one refused row | `Partial`, exit 1 | `Partial`, exit 1 | `GwzFetchPlan.md:174-179`, "a failure alongside any contacted repository is `Partial`" |
| Every selected row refused (`--no-target @root`) | `Rejected`, exit 2 | `Rejected`, exit 2 | `GwzFetchPlan.md:166`, `handle_fetch.rs:456-458`; `Rejected` is still reserved for the wholly-refused batch and is still reachable |
| `--json` aggregate parity, mixed selection | `Partial`, rows `[('@root','Planned'), ('mem_r3','Failed')]` | `Partial`, rows `[('@root','Unchanged'), ('mem_r3','Failed')]` | `docs/MachineOutput.md:705-709`; the aggregate now agrees and `Planned`/`Unchanged` still distinguish plan from report |

The new regression test `gwz-core/src/workspace_ops/tests/g27.rs:415-462`, `a_dry_run_with_a_refused_member_aggregates_like_the_live_run`, pins exactly the case I asked for: it builds a good member and an unmaterialized one, asserts `FetchResult::Planned` and `FetchResult::Failed` on the two dry-run rows, asserts the live aggregate is `Partial`, and then asserts dry and live aggregates are equal with the message "a dry run must not fail harder than the live run it rehearses". That is the closure test from my round-2 report, in the file I named.

## New findings

None.

## Re-checks carried forward

| Check | Result |
| --- | --- |
| Round-1 findings P2-1, P3-1 and the P2-2 claim not reintroduced | **Pass** — `grep -rn "MissingRemote\|missing_remote" docs/`, `grep -rn -- "force <categories>" docs/` and `grep -rn "@all so the lane" docs/ skills/` all return nothing |
| `docs/CLI.md`'s `gwz fetch` section equals what the rebuilt binary prints | **Pass** — `COLUMNS=100 gwz fetch --help` diffed against the block: exact match |
| Round-2 C-P2-3 surfaces still hold against the rebuilt binary | **Pass** — dry-run rows still render `would contact origin` / `"result": "Planned"`, live rows `no change` / `"Unchanged"`; a repository with no fetch remote is still `no upstream` / `"NoUpstream"` under `--dry-run` |
| gwz-cli `5270fe4` scope | **Pass** — two dev-docs review reports added, 126 insertions, no `docs/`, `src/` or `skills/` file touched, so nothing closed in round 2 could regress |
| gwz-py unchanged | **Pass** — `fc42c8ab`, as at round 2; the fix is entirely in the aggregate, which gwz-py reads from the envelope rather than recomputing |

## Observed and deliberately not filed

A selection whose only non-refused row is `NoUpstream` — for example a root with no remote plus one unmaterialized member — aggregates `Rejected`, exit 2, on both the dry and the live path:

```
### D. root has no upstream, member refused
status: Rejected
@root   .   no upstream
mem_r3  r3  failed       MemberNotFound: member 'mem_r3' at 'r3': member is not materialized
exit=2
```

This is the same shape as C2-P2-1 with `NoUpstream` in place of `Planned`, and it is **not** a finding here, for three reasons an auditor should be able to check. It is pre-existing, not a regression: `contacted` has never matched `NoUpstream`, in any of the three tuples, and `475947db` did not touch that. It falsifies no shipped document: `docs/Releases.md:81` and `docs/commands/fetch.md:135` say exit 2 means "the request was refused before any remote was contacted", and in this case no remote was contacted and the one unrefused row needed no network to answer. Only the dev-doc plan's stricter phrasing at `GwzFetchPlan.md:166,176-178` ("every row was refused before the network") and the matching sentence at `handle_fetch.rs:456-458` are literally wider than the code, and a plan wording that is stricter than the behaviour it governs is not a defect in the object under review. Recording it so a later reader of §3.5 knows the gap was seen and weighed, not missed.

## What I could not check and why

- **The test suites.** `cargo test`, `g27`, gwz-cli `g15` and the protocol corpus tests were not executed — read-only, no cargo. I read `a_dry_run_with_a_refused_member_aggregates_like_the_live_run` line by line and reproduced its scenario against the rebuilt binary instead; the binary's behaviour satisfies every assertion in it. The drift pins moved in round 2 remain unverified by execution, as recorded in round 2; `475947db` touches no protocol file, so nothing changed there this round.
- **R0 end to end** and **the two dispose refusal samples** in `docs/LocalClones.md` — unchanged from rounds 1 and 2, still accepted on traced tests read but not run.
- **The Surface axis.** Not seen and not requested in any round. This verdict rests only on my own evidence.
