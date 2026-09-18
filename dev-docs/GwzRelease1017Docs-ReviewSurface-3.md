# GwzRelease1017Docs Surface review — round 3

Tuple verified at start: root `ad8a21a6a1bd1c95d53186b690502bbd31770aee`, gwz-cli `5270fe4c359dda7bb54d9ae69b0b361770ebec66`, gwz-core `475947db5c8238304c53e5bd86e248dd56b54ae1`, gwz-py `fc42c8ab2dff5cde376de28f79b189690bbbf497`, working tree clean, binary mtime 2026-09-18 20:40 (newer than the round-2 binary), free space 8.5 GB ; at end: all four SHAs unchanged, binary unchanged. One untracked file appeared during my run — `dev-docs/GwzRelease1017Docs-ReviewConsistency-3.md`, the other axis filing its own round-3 report. It is untracked, outside `docs/` and `src/`, and I did not open it. Peer-blind maintained.

Scope confirmed independently: `git -C gwz-cli diff --stat fe0f1d2 5270fe4` touches only two `dev-docs/` files; `git -C gwz-core log --oneline 8f798b5..475947d` is the single commit "Aggregate a dry-run fetch like the live run it rehearses". No `docs/` page, no `src/` help string, no `docs/CLI.md` line changed. My round-2 closures for P2-1 through P3-6 therefore stand on text I already verified, and I did not re-litigate them.

## Verdict: NO-GO

One new P2 and one new P3, both on the exit-code and dry-run/live parity surface that round 3 was convened to check. Both have bounded remedies. **I pre-commit to GO on a revision that resolves P2-4 and P3-7 as specified.**

The case I was handed passes. The failure is on an input one flag away from it.

## What I re-ran

Built a two-member workspace (`@root` + `mem_good` + `mem_gone`, each with its own local bare origin) under the scratchpad, then deleted `mem_gone`'s directory.

| Case | live | dry-run | agree? |
| --- | --- | --- | --- |
| all healthy, up to date | `status: Noop`, exit 0 | `status: Noop`, exit 0 | yes |
| **mixed: `mem_gone` directory deleted** | **`status: Partial`, exit 1** | **`status: Partial`, exit 1** | **yes** |
| healthy subset (`--target @root`) | `status: Noop`, exit 0 | `status: Noop`, exit 0 | yes |
| outside a workspace | `WorkspaceNotFound`, exit 1 | `WorkspaceNotFound`, exit 1 | yes |
| unknown member id (`--target mem_nope`) | `MemberNotFound`, exit 1 | `MemberNotFound`, exit 1 | yes |
| **unknown `--remote nope`, healthy selection** | **`status: Failed`, exit 1** | **`status: Noop`, exit 0** | **no** |

The specified mixed case is correct in every respect. Live and dry printed the same `status: Partial`, the same exit 1, and the same failure row for the broken member:

```
mem_gone  gone  failed  MemberNotFound: member 'mem_gone' at 'gone': member is not materialized
```

`--json` agreed too: `meta.aggregate_status = "Partial"` for both, with `fetch_repos` results `[Unchanged, Unchanged, Failed]` live and `[Planned, Planned, Failed]` dry — the only difference being the `Planned` token, which is the P2-1 distinction working as designed. The round-3 core change does what it says.

My round-2 walkthrough-1 checks all still hold on the rebuilt binary: `would contact origin` / `"result": "Planned"` for a dry run, `Unchanged` only for a genuinely contacted no-op, `Updated` for a real move, `NoUpstream` under `--dry-run` for a repository with no fetch remote.

## New findings

### P2-4 `gwz --dry-run fetch --remote <name>` reports `would contact <name>` and exits 0 for a remote no repository has, while the live run fails every row

**Location.** Behavioural, at gwz-core `475947db`, surfaced through `docs/commands/fetch.md:57-58` and `:62-81` and the exit-code tables at `docs/commands/fetch.md:124-135` and `docs/Releases.md:72-81`.

**Violated invariant.** A dry run must rehearse the live run it previews. Round 3's own acceptance criterion is that the two print the same `status:` line and exit code; `docs/commands/fetch.md:64-65` states the dry run "resolves the selection and reports the repositories it would contact". A remote that is not configured is resolved locally, with no network — the live run proves it, reporting `MissingRemote` per repository without connecting.

**Reproduction.** Minimal: one workspace, one repository, no git remote configured at all.

```
$ gwz --dry-run fetch                    # default remote
@root  .  no upstream                                exit 0
$ gwz fetch                              # default remote
@root  .  no upstream                                exit 0
$ gwz --dry-run --remote nope fetch
@root  .  would contact nope                         exit 0
$ gwz --remote nope fetch
@root  .  failed  MissingRemote: workspace root '@root' at '.': missing remote 'nope'
                                                     exit 1
```

The asymmetry is internal to the dry run, not a general "it contacts nothing" limitation: with the *default* remote the dry run already performs local upstream resolution and correctly prints `no upstream`. It simply skips that same local check when `--remote` names a remote explicitly. Confirmed again on the two-member workspace with the broken member excluded: live `status: Failed` / exit 1 with `MissingRemote` on every row, dry `status: Noop` / exit 0 with `would contact nope` on every row.

**Impact.** `gwz --remote upstream fetch` is one of the six documented examples on both `gwz fetch --help` and `docs/commands/fetch.md`, and `--dry-run` is the documented way to check a request before committing to the network. A user or CI gate that previews `--remote upstream` gets a clean exit 0 and a row-by-row promise to contact it, then the real run fails on every repository — the exact failure a preview exists to prevent. The row asserts a plan the binary already knows, locally, it cannot carry out. This case also regressed in round 3: per the coordinator's description of the prior aggregation, it previously surfaced as `Rejected` / exit 2, so the change turned a loud wrong answer into a silent one. (I infer the prior behaviour from the change description; I could not rebuild the old binary to observe it. The finding stands at this tuple either way.)

**Required correction.** Apply the same local remote resolution to an explicit `--remote <name>` that the dry run already applies to the default: a repository that has no remote of that name must report the live run's answer (`failed` / `Failed`, or `no upstream`), not `would contact`. The aggregate and exit code then follow the live run, satisfying round 3's own criterion. If the behaviour is instead to be kept, `docs/commands/fetch.md:62-81` must say plainly that a dry run does not check whether `--remote`'s name exists and that its exit 0 is not a pre-flight for the named remote — but that leaves the preview unfit for its documented example, so I do not recommend it.

**Closure test.** In a workspace where no repository has a remote called `nope`: `gwz --dry-run --remote nope fetch` and `gwz --remote nope fetch` print the same `status:` line and the same exit code, and no row reads `would contact nope`. Re-run the mixed case above to confirm `Partial`/1 parity is unaffected.

---

### P3-7 Exit code `2` is documented on two pages but no `gwz fetch` refusal I can construct produces it; three different refusals all exit `1`

**Location.** `docs/commands/fetch.md:124-135`, `docs/Releases.md:72-81`, and the same sentence in `src/fetch_long.rs` (hence `docs/CLI.md`): "2 when the request was refused before any remote was contacted".

**Violated invariant.** A published exit-code table is a machine contract; every row must be reachable and distinguishable.

**Reproduction.** Three refusals that occur before any remote is contacted, all unpiped so `$?` is gwz's:

```
gwz fetch                      (outside a workspace)  → exit 1   gwz: WorkspaceNotFound: gwz.conf/gwz.yml missing
gwz --target mem_nope fetch    (unknown member id)    → exit 1   gwz: MemberNotFound: member id not found
gwz --remote nope fetch        (no such remote)       → exit 1   status: Failed
```

Exit 2 does exist in the binary, but for argument-parsing errors — `gwz hook claude-code setup --write --remove` returns 2, and the 1.0.16 baseline returns 2 for `error: unrecognized subcommand 'fetch'`. That is "usage error", not "the request was refused before any remote was contacted".

**Impact.** The table's row 1 ("some answered and some failed; the report is incomplete") and row 2 ("refused before any remote was contacted") are not distinguishable in practice: a script branching on exit 2 to detect a refusal never fires, and reads a workspace-not-found or bad-selector refusal as a partial success with an incomplete report. Compounding this, neither page says what a dry run's exit code means at all — every row in the table is phrased in terms of contacting or answering, which a dry run never does, so a reader cannot learn from the docs that `--dry-run fetch` shares the live codes, which is precisely what the round-3 change established.

**Caveat.** My axis forbids reading the implementation, so I cannot enumerate every refusal path; a refusal I could not cheaply construct (an open coordinated merge, a manifest newer than the binary) may well exit 2. The finding is that the three refusals a first-day user is most likely to meet all exit 1, and the table offers no way to tell them from a partial success.

**Required correction.** Either make the pre-contact refusals exit 2 as documented, or correct both tables to describe what the binary does and reserve the `2` row for usage errors. In the same edit, state that `--dry-run` uses the same exit codes as the live run it rehearses, with one sentence in `docs/commands/fetch.md`'s `### A dry run contacts nothing` section.

**Closure test.** Each row of the published table has a named, reproducible command that produces it, and `gwz --dry-run fetch`'s exit code is documented on the page.

## What I could not test

- **The pre-`475947db` aggregation.** I cannot rebuild, so the round-3 regression claim inside P2-4 rests on the coordinator's description of the prior behaviour, not on observation. P2-4 itself is established from the round-3 tuple alone.
- **An exhaustive refusal enumeration for P3-7**, for the axis reason stated above.
- Everything listed in my round-2 report's "could not test" section remains untested and unchanged: a real pre-1.0.14 binary, `gwz fetch` with a coordinated merge open, the hooks driven by a live Claude Code session, `--max-lanes` / `--min-free-gb` / the free-space estimate, and Windows.
