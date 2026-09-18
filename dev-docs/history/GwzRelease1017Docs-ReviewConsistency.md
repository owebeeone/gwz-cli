# gwz 1.0.17 release documentation (gwz-cli ad095e9) Consistency review

Tuple verified at start: root `4ff2834d05378eb9580373812eabdbf4c2e2d4d8`, gwz-cli `ad095e961d635d21c1533242cec20943e31aa0ae`, gwz-core `95138c7d9f48e1772720ce8f148d4c1a1445dfb1`, gwz-py `68de94cc367f063e028438e03776ac72d89dcc59`, lane `/Users/owebeeone/limbo/gwz-dev-docs-release`, `df -h /Users/owebeeone/limbo` = 8.3 GiB available (≥ 3 GB, no clone made) ; at end: identical (all four SHAs re-read, gwz-cli working tree clean). Binary exercised: `/Users/owebeeone/limbo/gwz-dev-docs-release/target/release/gwz` (`gwz 0.2.0-dev`, the unreleased 1.0.17 candidate; installed `gwz` on PATH is 1.0.16). Throwaway workspaces created only under `…/scratchpad/consistency-review/`. No file in any repository was modified.

## Verdict: NO-GO

Three P2 findings are open. Every one has a bounded text-only remedy.

I pre-commit to GO on a revision that resolves P2-1, P2-2 and P2-3 as specified.

## Findings

### P2-1 — the `MissingRemote` → `GitCommandFailed` correction was applied at two sites and left stale at three, one of them in the object's own file

- **Root cause.** The object corrected the error text a family name on `pull`/`push` produces, but only on the two pages it was editing for other reasons. The three other pages that state the same error, including a second statement inside `docs/LocalClones.md` itself, were not swept.
- **Location at the tuple.** Corrected: `gwz-cli/docs/LocalClones.md:618-624`, `gwz-cli/docs/commands/local.md:403-406`. Stale: `gwz-cli/docs/LocalClones.md:529-531` ("`MissingRemote` unless a Git remote of that name exists"), `gwz-cli/docs/commands/pull.md:68-69` ("answers `MissingRemote: missing remote 'C'` (`missing_remote` in this build's machine output)"), `gwz-cli/docs/commands/push.md:43-44` ("fails every selected target with `MissingRemote: missing remote 'hub'` (`missing_remote` in this build's machine output)").
- **Controlling source.** The binary at the tuple. `gwz-core/src/workspace_ops/handle_fetch.rs` is not involved; the resolution happens on the ordinary pull/push path, and the observed answer is `GitCommandFailed`.
- **Reproduction.**
  ```
  $ cd <throwaway gwz workspace>
  $ gwz --remote NOPE pull
  gwz: GitCommandFailed: remote 'NOPE' does not exist
  $ gwz --remote NOPE push
  status: Rejected
  @root . Rejected GitCommandFailed: remote 'NOPE' does not exist
  $ gwz --json --remote NOPE pull    # errors[0]
  {'code': 'GitCommandFailed', 'message': "remote 'NOPE' does not exist", ...}
  ```
  No `MissingRemote` and no `missing_remote` appears in either the human or the machine output.
- **Impact.** `docs/LocalClones.md` now tells a reader two different answers for the same command, seven pages apart. A script written against `docs/commands/push.md`'s documented `missing_remote` machine code never matches, and the operator who greps for `MissingRemote` after a refusal finds nothing. This is the first release in which the family-name fall-through is documented as corrected, so the stale text is shipped alongside the correction.
- **Required correction.** Replace the error text at `docs/LocalClones.md:531`, `docs/commands/pull.md:68-69` and `docs/commands/push.md:43-44` with `GitCommandFailed: remote '<name>' does not exist` and the machine code `GitCommandFailed`, matching `docs/commands/local.md:405-406`.
- **Closure test.** `grep -rn "MissingRemote\|missing_remote" gwz-cli/docs/` returns nothing, and `gwz --remote NOPE pull` / `gwz --json --remote NOPE pull` in a throwaway workspace print exactly the documented code and message.

### P2-2 — `ClaudeCode.md` tells the operator `--target @all` is required for a lane merge, contradicting `merge`'s documented default, `LocalClones.md`, and the hook's own printed remedy

- **Root cause.** The new "the work is wanted" recipe asserts that without `--target @all` a lane merge is member-only. `merge`'s default selection already includes the root, so the justification is false and the flag is redundant rather than necessary.
- **Location at the tuple.** `gwz-cli/docs/ClaudeCode.md:224-226`: "Use `@all` so the lane's root commits travel with its member commits; a member-only merge leaves the lane's root history unpreserved and the dispose refuses again."
- **Controlling sources it contradicts.**
  - `gwz-cli/docs/commands/merge.md:15-17` (unchanged): "With no selection, the workspace root and all active members participate. … For the previous member-only default, use `--target @all --no-target @root`."
  - `gwz-cli/docs/LocalClones.md:279-281` (unchanged, in the file the object edited): "**Root and members participate by default.** `gwz merge --remote A` carries the lane's root commits as well as its member commits."
  - `gwz-cli/docs/LocalClones.md:326-328` (unchanged, immediately above the object's own insertion): "After a completed default merge, ordinary disposal needs no separate root merge."
  - `gwz-cli/src/hook/remove.rs:293-297` — the remove hook's own hazard remedy string prints the bare form: `integrate it first: \`gwz merge --remote {name}\` from the main workspace, then \`gwz local dispose {name}\``.
- **Reproduction.** Read `ClaudeCode.md:224-226`; then `sed -n '15,17p' docs/commands/merge.md` and `sed -n '279,281p' docs/LocalClones.md`; then `grep -n "integrate it first" gwz-cli/src/hook/remove.rs`. The doc and the binary's own message prescribe different commands for the same refusal, and the doc says the binary's command will be refused again.
- **Impact.** The operator meets this text exactly when a lane removal has just been refused. It teaches that GWZ's default merge selection is member-only — which is false and load-bearing elsewhere — and it tells them the remedy the binary just printed is wrong. A subsequent dispose refusal (for any other reason) will be misdiagnosed as a missing `@all`. The command itself is harmless, so this is a diagnosability defect, not a destructive one.
- **Required correction.** Rewrite `ClaudeCode.md:224-226` to state that the default selection already carries the lane's root commits, and that `--target @all` is only needed if a narrower selection was used. Either drop `--target @all` from the example at `:216-218` to match the hook's own message, or keep it and say explicitly that it is the default spelled out.
- **Closure test.** The sentence at `ClaudeCode.md:224-226` agrees with `docs/commands/merge.md:15` and `docs/LocalClones.md:279-281`, and the command it shows is the one `src/hook/remove.rs`'s hazard remedy prints.

### P2-3 — the `--dry-run` exception is documented on three new surfaces and left uncorrected on the two pages that define `fetch`'s contract, one of which the binary now contradicts

- **Root cause.** The object added "`--dry-run` is the one exception … `gwz --dry-run fetch` contacts no remote at all" to `src/fetch_long.rs` (hence `docs/CLI.md`), `docs/Releases.md` and `skills/gwz/SKILL.md`, but did not propagate it to `docs/commands/fetch.md` or `docs/MachineOutput.md`, which still state the unqualified absolute — and `MachineOutput.md` states it as a machine-consumer guarantee that the binary breaks under `--dry-run`.
- **Location at the tuple.**
  - Added: `gwz-cli/src/fetch_long.rs:24-29`; `gwz-cli/docs/CLI.md:1162-1167`; `gwz-cli/docs/Releases.md:44-50`; `gwz-cli/skills/gwz/SKILL.md:39-41`.
  - Not corrected: `gwz-cli/docs/commands/fetch.md:27-29` ("**It never skips the network.** … a fetch that does not connect has answered nothing."); `gwz-cli/docs/MachineOutput.md:662-663` ("`Unchanged`: the repository was contacted and its tracking ref did not move. This is a successful read, not a skip — `gwz fetch` never skips the network.").
- **Controlling source.** `gwz-core/src/workspace_ops/handle_fetch.rs` — the `--dry-run` branch returns before `par_map_per_host`, and `FetchTarget::planned_row` maps a repository with a remote and a branch to `crate::FetchResult::Unchanged`. Nothing before that branch touches the network: `backend.validate_transport_remotes` resolves to `identities.validate_remote_names` (`gwz-core/src/git/gitbackend.rs:210-212`), a local check of `--remote-identity` names.
- **Reproduction.**
  ```
  $ gwz --json --dry-run fetch
  … "fetch_repos":[{"…","member_id":"@root","remote":"origin","result":"Unchanged","upstream":null},
                   {"…","member_id":"mem_ws","remote":"origin","result":"Unchanged","upstream":null}] …
    "aggregate_status":"Noop"
  $ gwz --dry-run fetch
  status: Noop
  @root   .   no change
  mem_ws  ws  no change
  ```
  Exit 0, no remote contacted. `result: "Unchanged"` on every row, which `MachineOutput.md:662` defines as "the repository was contacted".
- **Impact.** A script that follows `MachineOutput.md` and treats `Unchanged` as contacted-and-answered will report a dry run as a completed fetch of the whole workspace. The human `--dry-run` rows read `no change`, identical to a real contacted no-op, with nothing in the output to distinguish them. The object asserts the exception matters enough to state three times; leaving the contract pages stating the opposite is the defect.
- **Required correction.** Add the `--dry-run` exception to `docs/commands/fetch.md:27-29`, and qualify `docs/MachineOutput.md:662-663` so `Unchanged` means contacted only outside `--dry-run` — saying what a dry-run row carries (`result: "Unchanged"`, `members[].status: "Planned"`, `upstream`/`before`/`after` null) and how a consumer tells the two apart.
- **Closure test.** `gwz --json --dry-run fetch` and `gwz --json fetch` are both run against the same workspace; every statement in `docs/MachineOutput.md`'s Fetch JSON section and `docs/commands/fetch.md`'s "What it does not do" holds for both outputs.

## P3 findings

### P3-1 — the refusal's waiver command is described as taking categories, which the hazard vocabulary rejects

- **Root cause.** The four report categories and the three `--force` waiver names are deliberately different vocabularies in this release; three new passages name the wrong one in the `--force` placeholder.
- **Location.** `gwz-cli/docs/Releases.md:92`, `gwz-cli/docs/commands/local.md:305`, `gwz-cli/docs/LocalClones.md:380` — all "prints the exact `--force <categories>` command".
- **Controlling source.** `gwz-core/crates/local-disposal/src/hazard.rs:100-108`: "They are the **report's** vocabulary, not the wire's … In this release several categories still share one waiver name — narrowing the waiver vocabulary so each category has its own is R11". `HazardWaiver::ALL` is `open-merge`, `dirty`, `unpreserved-history`; `HazardWaiver::parse` rejects anything else with `unknown hazard \`{name}\`; known hazards: …`. `GwzLaneCleanFixes.md` R11 is explicitly not yet implemented.
- **Contradicted in the same files.** `docs/commands/local.md:8` and `docs/LocalClones.md:466,552` write `--force <hazard,...>`.
- **Impact.** A reader who takes the placeholder literally types `gwz local dispose C --force regenerable` and is refused. Recovery is immediate — the refusal prints the correct command — so the cost is one failed attempt and a wrong mental model of R11's status.
- **Required correction.** Change the three placeholders to `--force <hazard,...>`, keeping the surrounding sentence (which is otherwise accurate: the printed command does waive exactly the refusing entries, per `required_waivers` in `hazard.rs:213-224`).
- **Closure test.** `grep -rn -- "--force <categories>" gwz-cli/docs/` returns nothing.

### P3-2 — `Releases.md`'s enumeration of `gwz fetch`'s row kinds omits the row a first fetch prints

- **Root cause.** The row-kind list was copied from `docs/commands/fetch.md`'s Output section, which itself omits two of the five shapes `movement()` produces.
- **Location.** `gwz-cli/docs/Releases.md:19-22`: "prints one row each: the tracking ref before and after, how far the current branch is ahead of and behind it, or `no change`, `no upstream`, `failed`."
- **Controlling source.** `gwz-cli/src/append_branch_summary/fetch.rs:15-27` — `Updated` renders `{before}..{after}`, **or `new {after}` when the tracking ref did not exist before**, or `updated` when neither id is known; then `no change`, `no upstream`, `failed`.
- **Reproduction.** In a throwaway workspace whose member has never been fetched:
  ```
  $ gwz fetch
  status: Ok
  @root   .   no change    (origin/main, +0 -0)
  mem_ws  ws  new 6a44c31
  ```
- **Impact.** `new <oid>` is the row a reader sees the first time they run the new verb against a fresh clone, and it is in no user-facing document. `docs/MachineOutput.md:659-661` covers the JSON side ("`before` is absent when the ref did not exist yet") but names no human row.
- **Required correction.** Add `new <after>` (and, if it is reachable, `updated`) to `Releases.md:19-22` and to `docs/commands/fetch.md`'s Output list.
- **Closure test.** Every arm of `movement()` in `src/append_branch_summary/fetch.rs` appears in `docs/commands/fetch.md`'s Output list.

### P3-3 — the `gwz fetch` sample block is not what the binary prints

- **Root cause.** The sample was carried over from `GwzFetchPlan.md` §3.3's hand-drawn mock-up rather than captured from a run; its first and third columns are one character narrower than the renderer's.
- **Location.** `gwz-cli/docs/Releases.md:27-35` (identical block at `gwz-cli/docs/commands/fetch.md:33-41`).
- **Controlling source.** `gwz-cli/src/append_branch_summary/fetch.rs:60-63`: `format!("{:<id_width$}  {:<path_width$}  {movement:<movement_width$}", …)` — two spaces between columns, each width the max of its column.
- **Reproduction.** Applying that format to the sample's own five rows (`id_width` 9, `path_width` 8, `movement_width` 16) yields `'mem_local  local     no upstream'` where the doc has `'mem_local local     no upstream'`, and `'mem_core   gwz-code…'` alignment throughout; the doc's movement column is likewise 19 wide against the renderer's 18.
- **Impact.** Cosmetic only — row order, content, `status:` line, `(origin/main, +A -B)` suffix and the `RemoteRejected: connection refused` tail all match the renderer exactly. The severity contract lists "a sample the binary cannot print" at P2; I grade it P3 because the divergence is column padding and nothing a reader would act on differs.
- **Required correction.** Regenerate the block from a real run, or re-pad it to the renderer's widths, in both files.
- **Closure test.** The block, fed through the `format!` in `fetch.rs:60-63` with its own widths, reproduces itself byte for byte.

### P3-4 — the retitled "Compatibility Notes" lead cites a note the same commit deleted

- **Root cause.** The local-clone bullet was removed from the section and the replacement lead paragraph still introduces it.
- **Location.** `gwz-cli/docs/Releases.md:257-260`: "Every note here describes **released** behaviour: the local clone family shipped in the 1.0 line (see [1.0.4](#104-the-10-line-ships)), and the merge, log and `gwz.conf/` notes below shipped in the 0.10 to 0.12 releases that preceded it." The section's bullets now begin at `:263` with `gwz log`; no local-clone note remains.
- **Controlling source.** `git show ad095e9 -- docs/Releases.md` removes the twelve-line "A local clone family." bullet in the same hunk that adds this lead.
- **Impact.** A reader looks for the local-clone note the lead promises and finds none. The substance is not lost — `docs/Concepts.md:213-225`, `docs/LocalClones.md:84-85`, `docs/commands/local.md:19` and `docs/commands/merge.md:461-462` all still carry "never written into `gwz.conf/` … never become Git remotes", and `docs/commands/local.md:397-406` carries the `--clean`/`--bare`/`--from` refusals.
- **Note on the retitling itself, which is sound.** Every surviving bullet's text originates in a released tag: `git blame -L 263,311` + `git describe --contains` gives `e2a70742 → v0.12.0`, `177f25d6 → v0.11.1`, `0cd9742c/b5866630/c36bf1bf → v0.10.0`, `9b73bff0/9e23d0a6 → v0.10.5`, `373c9815 → v1.0.0-rc.1`. Dropping "Unreleased" is justified. The lead's range "0.10 to 0.12" is one release short — line 287's sentence entered at `v1.0.0-rc.1` — and the anchor-directory bullet at `:277-281` is neither a merge, a log nor a `gwz.conf/` note, though it shares a commit with the `gwz.conf/` one.
- **Required correction.** Drop the local-clone clause from the lead, or restore a one-line local-clone note; widen "0.10 to 0.12" to "the 0.10 to 1.0 releases" or drop the range.
- **Closure test.** Every noun phrase in `Releases.md:257-261` names a bullet that is present in the section below it.

### P3-5 — `ClaudeCode.md` states a measured lane-creation range that no record in the lane supports

- **Root cause.** A measurement was written into user-facing documentation without an evidence record; the two observations that do exist in the lane give different numbers.
- **Location.** `gwz-cli/docs/ClaudeCode.md:99-101`: "On this workspace a lane takes about two minutes to appear (observed 97 to 146 s for a 380k-file, 135 GB logical workspace on APFS)".
- **Controlling sources.** `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md:190-198` (U1, the amendment this commit's parent `5112780` corrected): "the session looked stalled for the ~2 min the hook took (lane directory born 7 s after the click, tree copy 106 s, copy record and index write 10 s; no other lane or build was running)" — 123 s, one observation, not a range. `gwz-cli/dev-docs/GwzClaudeIntegration-Probe-20260918.md:131` records the only other timed create: 207.42 s, under three-way contention. `grep -rn "97\|146\|380k\|135 GB" gwz-cli/docs gwz-cli/dev-docs` returns only this one line in `ClaudeCode.md`; the same search over the lane root's `dev-docs/` returns nothing, and `docs/LocalClones.md`'s "Disk Space and Copy Speed" section (`:46-70`) records neither a file count nor a byte total for this workspace.
- **Impact.** The one claim on this page that an operator can hold GWZ to is unfalsifiable from the repository, in a project whose own process files probes as dated evidence documents (`EVIDENCE.md`, `GwzClaudeIntegration-Probe-20260918.md`). The adjacent paragraph at `:159-162` is careful to label its figures "conservative estimates, not measurements"; this one is not.
- **Required correction.** Cite the record the range comes from, or restate it as U1's single observation ("~2 min: 7 s to the lane directory, 106 s of tree copy, 10 s of index write, on a workspace of this size, with nothing else running"), or file the missing measurements as a dated probe record.
- **Closure test.** Each number in `ClaudeCode.md:99-101` is traceable to a line in a committed evidence or probe document.
- **Scope note.** This is not the D8 deferral. D8 postpones the `--min-free-gb` share and per-file cost measurements, which `:159-162` correctly labels as postponed. This finding is about a measurement the page asserts as taken.

## Claims checked

| Claim | Page:line at the tuple | Controlling source | Result |
| --- | --- | --- | --- |
| Regenerable recogniser list is exactly what this build implements (6 rules) | LocalClones.md:406-425; commands/local.md:333-346; Releases.md:96-106 | `gwz-core/crates/repo-inspect/src/regenerable.rs:39-95,103-118,142-166` | **Pass** — all six present, none extra; `CACHEDIR.TAG` prefix test, `__pycache__` `.pyc`/`.pyo`-only test, `*.egg-info` + `PKG-INFO`, `bazel-*`/`razel-*` link outside the workspace, `.so`/`.pyd`/`.dylib`, and the `UNTAGGED_BUILD_DIRECTORIES` marker table all match |
| "never by a directory's name"; copy record not consulted; unreadable is not regenerable; ignored tree once, untracked file by file, recognised through ancestors to the repository boundary | LocalClones.md:408-410,427-437; commands/local.md:331-332,348-353 | `regenerable.rs:16-31` (the three rules), `recognise_under:173-189`, `children_all:299-311` | **Pass** |
| Four report categories, their meanings, which refuse, empty ones printed | LocalClones.md:365-378; commands/local.md:292-302; Releases.md:80-89 | `gwz-core/crates/local-disposal/src/hazard.rs:110-159,182-212`; `gwz-core/src/local_clone/dispose.rs:495-530` | **Pass** — names, order, refusal semantics and the empty-category rule all match |
| The printed waiver command names only the refusing categories | LocalClones.md:380-384; commands/local.md:305-308 | `hazard.rs:213-224` (`required_waivers` filters on `HazardFinding::refuses`); `dispose.rs:470-486` | **Pass** (placeholder wording is P3-1) |
| D9: `forced past:` in waiver-vocabulary order, `unused waiver:` in operator order | LocalClones.md:393-401; commands/local.md:310-318; Releases.md:109-112 | `gwz-core` commit `95138c7d`; `dispose.rs:113-131,594-624`; `crates/local-disposal/src/tests/hazards.rs` (`a_forced_deletion_carries_the_hazards_it_actually_waived`) | **Pass** — both sample lines are byte-identical to the unit test's expectation |
| `dirty` no longer raised by regenerable data or an unchanged copy | commands/local.md:282 | `gwz-core/crates/work-detector/src/builder.rs:203-212`; `crates/work-detector/src/provenance.rs:59-69` | **Pass** |
| An integrated lane disposes with no waiver (R0) | LocalClones.md:328-334; Releases.md:69-74; ClaudeCode.md:188-196 | `GwzLaneCleanFixes.md` status line and R0/R0.1; `gwz-core` `96338a63` (S2.4 trace, R17-R19) | **Pass** — asserted by a traced test; not independently reproduced here (see below) |
| `--dry-run` refused for every local family verb, with the two distinct messages | LocalClones.md:625-629 | binary | **Pass** — `local create with dry_run …` for `local clone`; `local family operations with dry_run …` for `list`, `dispose`, `disband`, all `UnsupportedOperation` |
| Family name on `pull`/`push` answers `GitCommandFailed: remote '<name>' does not exist` | LocalClones.md:618-624; commands/local.md:403-406 | binary | **Pass** here; **P2-1** for the three stale sites |
| `--dry-run fetch` contacts no remote at all | fetch_long.rs:24-29; CLI.md:1162-1167; Releases.md:44-50; SKILL.md:39-41 | `handle_fetch.rs:107-110`; `gitbackend.rs:210-212` | **Pass** on the code; **P2-3** for the uncorrected contract pages |
| Global `--remote` selects the remote each selected repository contacts | Releases.md:65-66; parser.rs:169; CLI.md (×20) | `handle_fetch.rs` `root_fetch_remote_name`; `pull_head_member_preflight/member_preflight.rs:302-316` | **Pass** |
| `docs/CLI.md` equals what the binary prints (`fetch`, global `--remote`) | CLI.md:1139-1272 | `COLUMNS=100 gwz fetch --help` | **Pass** — diff is empty apart from the generator's trailing-whitespace strip |
| `--json` carries the rows under `fetch_repos`; exit codes 0/1/2; `no change` means contacted | Releases.md:52-66 | `handle_fetch.rs` `fetch_aggregate_status`; `src/tests/g15.rs:217-273`; `GwzFetchPlan.md` §3.5 | **Pass** |
| `--prune`, `--tags`, per-remote and multi-remote not implemented | Releases.md:68-72 | `GwzFetchPlan.md` §5 Phase 2 steps 2.1-2.3 | **Pass** (Phase 2/3 absence is a declared deferral, not reported) |
| Family index schema v2; 1.0.14 floor; "a create, a dispose, a `--keep` or a family merge" is a write; older gwz refuses whole and names the minimum | Releases.md:145-155 | `gwz-core/crates/family-model/src/lib.rs:50-88` (`INDEX_SCHEMA`, `INDEX_SCHEMAS_READ`, `INDEX_MIN_GWZ_VERSION = "1.0.14"`, `index_schema_min_gwz_version`) | **Pass** — the rule is stated exactly as the code enforces it |
| 1.0.14 carried the Claude Code hooks, `--owner`/`--wait`, index v2, disposal Phase 1 S1.1-S1.4, `gwz ls` materialized note | Releases.md:118-168 | `git log v1.0.13..v1.0.14` (gwz-cli `51ee0b7`, `ab433dd`, `ef865de`, `23828e1`, `8c8b66a`, `1c050fe`); gwz-core `a921ed69`, `05921a35`, `76dee09e` before `1264845a` | **Pass** |
| 1.0.16 completed Phase 1 with the four-category refusal and the exact waiver command | Releases.md:114-116,161-168 | gwz-core `aae0d1ea`, `af994944`, `e53813b6`, `1f5062b8` in `v1.0.14..v1.0.16`; `GwzLaneCleanFixes.md` status line | **Pass** |
| Release dates 1.0.14 / 1.0.16 = 2026-09-18 | Releases.md:118-119 | `git log -1 --date=short v1.0.14 / v1.0.16` | **Pass** |
| "Compatibility Notes" retitling justified — every bullet is released | Releases.md:255-310 | `git blame -L 263,311` + `git describe --contains`: v0.10.0, v0.10.5, v0.11.1, v0.12.0, v1.0.0-rc.1 | **Pass**; lead's scope wording is P3-4 |
| Remove hook runs the bare dispose, never `--force`, never `--keep`, one summary line | ClaudeCode.md:199-210,215; Releases.md:124-127 | `gwz-cli/src/hook/family.rs:179-198` (`keep: None, force_hazards: Vec::new()`); `src/hook/remove.rs:258-301`; gwz-cli `ad94fad` (F3) | **Pass** |
| `--max-lanes` counts every ready lane | ClaudeCode.md:162-167 | `gwz-cli/src/hook/create.rs:139` (`check_lane_ceiling(family.ready_rows(), …)`) | **Pass** |
| A hook killed mid-copy leaves a `creating` row and its files | ClaudeCode.md:102-103 | `docs/LocalClones.md:165` | **Pass** |
| `--target @all` needed so root commits travel | ClaudeCode.md:224-226 | `docs/commands/merge.md:15-17`; `docs/LocalClones.md:279-281,327-328`; `src/hook/remove.rs:293-297` | **Fail — P2-2** |
| Human row kinds enumerated completely | Releases.md:19-22; commands/fetch.md:43-54 | `src/append_branch_summary/fetch.rs:15-27` | **Fail — P3-2** (`new <oid>`, `updated` missing) |
| `gwz fetch` sample is producible | Releases.md:27-35 | `src/append_branch_summary/fetch.rs:60-63` | **Fail — P3-3** (column widths) |
| AGENTS_GWZ template needs no change | — | `gwz-core/src/workspace_ops/agents_gwz_template.md` (last line: "Claude Code worktree hooks … gwz 1.0.14 or later") | **Pass** — the hooks still require 1.0.14; the 1.0.17 no-waiver disposal is an improvement, not a new floor |
| `docs/Concepts.md`, `docs/QuickStart.md`, `docs/README.md` need no change | — | Concepts.md:213-225 states the pull/push fall-through without naming an error code; README.md indexes guides, not commands; `mkdocs.yml:93` already carries `commands/fetch.md` | **Pass** |
| `docs/CLI.md` has no "Command page:" link for `fetch` although `docs/commands/fetch.md` exists | CLI.md:1137 | `src/cli_reference.rs:116-141` (`command_page` has no `fetch` arm; `auth` likewise) | **Not a finding** — pre-existing gap from gwz-cli `7a6814f`, untouched by the object, and `docs/CLI.md` at the tuple equals what the binary prints |

## What I could not check and why

- **The R0 claim end to end.** I did not build a workspace with stashes, reflog-only commits, tagged and untagged caches, `__pycache__`, ignored user data and a merged lane, and dispose it. That is `GwzLaneCleanFixes.md` R18's test and gwz-core `96338a63` runs it; reproducing it needs a multi-repository workspace, a lane, a build in the lane and a family merge, which exceeds what a read-only reviewer can safely stand up under the scratchpad. I verified the claim's mechanism instead — the recognisers, `Provenance::refuses`, `WorkVerdict` derivation, and `inspect`'s partition — all of which support it. **Consequence:** the headline behavioural claim of the release is accepted on the strength of a traced test I read but did not run.
- **The two dispose refusal samples at `LocalClones.md:388` and `:476`.** I verified their grammar against `render_hazards`/`render_category` in `gwz-core/src/local_clone/dispose.rs:470-530`, the detail strings against `gwz-core/crates/work-detector/src/builder.rs:226-242`, the `gwz: <Code>: ` envelope against observed output, and the truncation threshold (`MAX_LISTED = 16`, so neither is truncated). Each reads as producible. I did not produce either, for the reason above. The `:476` sample is unchanged by the object.
- **Test suites.** `cargo test` and `python scripts/generate_cli_reference.py --check` were not run (read-only, no builds). `docs/CLI.md` was checked instead by diffing the binary's own `--help` output, which is what the generator renders.
- **The workspace-size figures in P3-5.** I did not count files or bytes in `/Users/owebeeone/limbo/gwz-dev` — the brief forbids touching it, and a `du` over 135 GB is not an inspection command. P3-5 rests on the absence of any record, not on a contradicting measurement.
- **The other reviewer's axis.** Not seen, not requested, not reasoned about. This verdict rests only on the evidence above.
