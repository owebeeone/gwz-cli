# gwz 1.0.17 release documentation — Consistency review, round 4

Tuple verified at start: root `301a140187101f1a07a0c071b4a2386d38f2e7ac`, gwz-cli `8702b6863166619c7e996d180363677545fd720a`, gwz-core `8a457d1f2bf1423400b428f9ef9d5e4a0f1c7b66`, gwz-py `fc42c8ab2dff5cde376de28f79b189690bbbf497` (unchanged) ; at end: identical, all three worktrees clean. Lane `/Users/owebeeone/limbo/gwz-dev-docs-release`, 8.5 GiB free, binary rebuilt at 20:46. Read-only: no cargo, no builds, no git mutations; throwaway workspaces only under `…/scratchpad/consistency-review/`.

Prior rounds: round 1 (P2-1..P2-3, P3-1..P3-5, all closed at round 2), round 2 (C2-P2-1, closed at round 3), round 3 (GO).

## Verdict: GO

My round-3 GO holds. The round-4 changes are consistent with the plan, the handler and the binary. One new P3 — a rustdoc comment that silently reassociated during the edit — which does not block.

## What I checked

**The rewritten exit-code passages against the binary.** Every claim reproduced in a throwaway workspace with a root and one member:

| Claim | Where | Observed |
| --- | --- | --- |
| Exit 2 is every selected repository refused before the network, e.g. `--remote <name>` none has | `docs/commands/fetch.md:136-145`, `docs/Releases.md:77-87`, `src/fetch_long.rs:37-42`, `docs/MachineOutput.md:705-712` | `gwz --remote nope fetch` → `status: Rejected`, both rows `failed  MissingRemote: … missing remote 'nope'`, exit 2 |
| A dry run shares the live codes and refuses the same name | same | `gwz --remote nope --dry-run fetch` → byte-identical output and exit 2 |
| A refusal before there is a selection is a typed error exiting 1, as on every verb | same | Outside a workspace: `gwz: WorkspaceNotFound: gwz.conf/gwz.yml missing`, exit 1. Unknown member id: `gwz: MemberNotFound: member path not found`, exit 1 — identical with and without `--dry-run` |
| `--json` carries the refusal on the members entry | `docs/MachineOutput.md` | `aggregate_status: Rejected`; rows `result: "Failed"`; `members[].status: "Rejected"`, `error.code: "MissingRemote"` |
| Round-3 closure (C2-P2-1) intact | — | Good member + unmaterialized member: dry run `Partial`/1, live `Partial`/1, still identical |
| Baseline unaffected | — | Plain `gwz fetch` → `status: Ok`, `no change` + `new e087889`, exit 0 |

**`GwzFetchPlan.md` §3.5.** The rewritten tables now match the plan exactly where they previously did not. §3.5's row "`Rejected` | 2 | nothing was contacted; every row was refused before the network" was the stricter statement I declined to file in round 3; `docs/Releases.md:81` and `docs/commands/fetch.md:139` have moved to it ("every selected repository was refused before the network, so nothing was contacted"), and `named_remote_refusal` gives it a reachable whole-batch case. The documents and the plan now say one thing.

**The `handle_fetch.rs` doc comment.** `fetch_aggregate_status`'s comment at `:456-461` is unchanged from round 3 and still correct: `Rejected` reserved for the wholly-refused batch, `Planned` counting as contacted. Re-confirmed against the binary in all four boundary cases (healthy, mixed, all-refused, `--remote` unknown).

**`GwzFetchPlan.md` §2.1 and §3, and the `--remote` help.** No contradiction. §2.1 describes `pull_fetch_remote_name` / `pull_root_remote_name` as "the policy `--remote` override, else …"; `named_remote_refusal` does not touch resolution — both helpers still return `Some(name)` for a named remote — it adds a local precondition beside them. §3.2 step 1's "No remote ⇒ `no upstream` row" is about a repository with no remote at all and is unaffected. The global `--remote` long help ("On `fetch` it selects the remote each selected repository contacts") still holds: the name is selected, and a repository that lacks it is refused rather than contacting something else; nothing in the docs promises a fallback, and the code has none.

**Cross-document sweep.** `MissingRemote` now appears in exactly one place in `docs/` — `docs/commands/fetch.md:81`, describing this refusal — so it does not collide with the round-1 sweep that removed the stale `MissingRemote` claims from `pull.md`/`push.md`/`LocalClones.md`, which describe a different path (`GitCommandFailed`, still what `gwz --remote NOPE pull` prints). `grep -rn "missing_remote" docs/` and `grep -rn -- "force <categories>" docs/` still return nothing, so rounds 1 and 2 have not regressed.

**`docs/CLI.md`.** Regenerated correctly: the `gwz fetch` block is byte-identical to `COLUMNS=100 gwz fetch --help` from the rebuilt binary, including the new exit-code sentence from `src/fetch_long.rs`.

**The new test.** `g27::a_named_remote_no_repository_has_is_refused_before_the_network_on_both_paths` asserts, for the live and the dry response alike, `FetchResult::Failed`, `MemberStatus::Rejected`, `GwzErrorCode::MissingRemote` and `AggregateStatus::Rejected`. That is the behaviour I observed; not executed (no cargo), read and matched against the binary.

## New findings

### C4-P3-1 — `named_remote_refusal`'s rustdoc opens with the sentence that documented `root_fetch_remote_name`, which now has none

- **Root cause.** The new function was inserted between an existing doc comment and the function it documented, so the two-line comment silently reassociated.
- **Location.** `gwz-core/src/workspace_ops/handle_fetch.rs:501-506` — `named_remote_refusal`'s rustdoc reads "The root's fetch remote: the policy `--remote` token, else `origin`, else whatever remote the root has first. Identical to pull's rule." followed by the intended four lines; `root_fetch_remote_name` at `:531` now carries no doc comment.
- **Evidence.** `git show 8a457d1f -- src/workspace_ops/handle_fetch.rs` adds the new lines directly beneath the unchanged `/// The root's fetch remote: …` pair, with the blank line and `fn root_fetch_remote_name` following after.
- **Impact.** Bounded and documentation-only: no behaviour changes and nothing shipped is affected. The consequence is that the rustdoc for `named_remote_refusal` states something false about it — it is not the root's resolver, it applies to members too, and it does no defaulting — while the resolver a reader would look up is undocumented. This is the reassociation-during-edit class the workspace's own standing rule was written against, so it is worth naming rather than absorbing.
- **Required correction.** Move `:501-502` back below `named_remote_refusal`, immediately above `fn root_fetch_remote_name`, leaving `:503-506` as the new function's whole doc comment.
- **Closure test.** `named_remote_refusal`'s doc comment begins "A `--remote <name>` the repository does not have …" and `root_fetch_remote_name`'s begins "The root's fetch remote: …".

## Observed and deliberately not filed

`GwzFetchPlan.md` §3.2's numbered per-repository sequence (resolve remote → resolve branch → read ref → fetch → read ref → count) does not mention the new pre-network refusal, so the plan is now silent about a step the code takes. This is an omission in a dev-doc, not a contradiction: §3.2 step 1's "No remote ⇒ `no upstream`" covers a different case and stays true, and every user-facing surface — `fetch.md`, `Releases.md`, `MachineOutput.md`, `CLI.md` — does describe the refusal. The plan's precedent for a contract refinement is to amend it (gwz-cli `acf36ca`), so a one-line addition to §3.2 would be tidy, but nothing shipped is falsified and I am not filing it. Recorded so a later reader of §3.2 knows the gap was seen and weighed.

Separately, the three-row exit tables in `Releases.md` and `fetch.md` still have no row for the all-`Failed` batch (every repository reached the network and errored), which aggregates `Failed` and exits 1 while the table's row 1 reads "some answered and some failed". This is pre-existing across all four rounds, is not what this round touched, and the complete aggregate-to-exit mapping is in `docs/MachineOutput.md`'s own table. Not filed.

## What I could not check and why

- **The test suites.** `cargo test`, `g27`, gwz-cli `g15` and the protocol corpus tests were not executed — read-only, no cargo. Each new assertion was read and matched against observed binary behaviour instead.
- **The drift pins** moved in round 2 remain unverified by execution; neither `8a457d1f` nor `8702b68` touches a protocol file, and gwz-py is unchanged at `fc42c8ab`.
- **R0 end to end** and **the two dispose refusal samples** — unchanged from rounds 1–3, still accepted on traced tests read but not run.
- **The Surface axis.** Not seen in any round. The gwz-core and gwz-cli commit messages this round cite Surface P2-4 and P3-7; I formed no view on those findings and my checks above are my own reproductions against my own controlling documents.
