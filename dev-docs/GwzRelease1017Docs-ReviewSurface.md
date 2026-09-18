# GwzRelease1017Docs Surface review

Tuple verified at start: root `4ff2834d05378eb9580373812eabdbf4c2e2d4d8`, gwz-cli `ad095e961d635d21c1533242cec20943e31aa0ae`, gwz-core `95138c7d9f48e1772720ce8f148d4c1a1445dfb1`, gwz-py `68de94cc367f063e028438e03776ac72d89dcc59`, binary `/Users/owebeeone/limbo/gwz-dev-docs-release/target/release/gwz` reporting `gwz 0.2.0-dev`, baseline `/Users/owebeeone/.cargo/bin/gwz` reporting `gwz 1.0.16`, free space on `/Users/owebeeone/limbo` 8.3 GB ; at end: identical (all four SHAs unchanged, `git -C gwz-cli status --porcelain` empty, binary still `gwz 0.2.0-dev`). Nothing in the lane was modified; every command ran read-only or inside `/private/tmp/claude-501/-Users-owebeeone-limbo-gwz-dev/f7a99a51-fc36-4c36-8f6f-db9ac84441d7/scratchpad/surface-review/`.

## Verdict: NO-GO

Three P2 findings. Every one has a bounded text-fixable remedy except P2-1, which has a text remedy the author must choose between two ways of writing. I pre-commit to GO on a revision that resolves P2-1, P2-2 and P2-3 as specified.

## Findings

### P2-1 `gwz --dry-run fetch` prints `no change` / `"result": "Unchanged"` when the upstream has in fact moved, contradicting the help paragraph this commit added

**Location.** `gwz-cli/src/fetch_long.rs` L24-30 (as added by `ad095e9`), reproduced verbatim in `gwz-cli/docs/CLI.md` (fetch block) and in `gwz-cli/docs/Releases.md:46-51`. The row vocabulary it collides with is defined in `gwz-cli/docs/commands/fetch.md:43-51`.

**Violated invariant.** A status token must mean one thing. The help added by this commit asserts of `--dry-run`: "the rows carry no result from any remote" and that it "answers `which repositories would be contacted` and never `what moved`". `docs/commands/fetch.md:45-46` defines `no change` as "the repository was contacted and its tracking ref did not move." The binary prints `no change` for a dry run, so the same token simultaneously means "contacted, nothing moved" and "not contacted, no result".

**Reproduction.** In the throwaway workspace, with a local bare `origin` and one commit pushed to it from a second clone so the tracking ref was genuinely one commit behind:

```
$ gwz --dry-run fetch
status: Noop
@root  .  no change
exit=0
$ gwz --dry-run --json fetch     # fetch_repos[0]
{"after":null,"ahead":null,"before":null,"behind":null,"branch":"main",
 "member_id":"@root","member_path":".","remote":"origin",
 "result":"Unchanged","source_kind":"Git","upstream":null}
$ gwz fetch
status: Ok
@root  .  5e5700a..2bde72b  (origin/main, +0 -1)
exit=0
```

The dry run reported `no change` / `Unchanged` at a moment when the real fetch immediately afterwards reported a one-commit move.

**Impact.** A first-day user who reaches for `--dry-run` because the docs present it as the safe preview (`docs/commands/fetch.md:85-89`, "Resolve the selection and report what would be contacted, contacting nothing") is told, in the documented vocabulary, that nothing moved upstream. The only cue that this is a plan and not an answer is the absence of the `(origin/main, +A -B)` suffix — which is also absent from the genuine `no upstream` row, and which `fetch.md`'s Output section never says is dropped. `--dry-run` is a global flag, so a wrapper or agent that passes it uniformly reads `"result":"Unchanged"` out of `fetch_repos` as an affirmative verdict. This ships forever in the machine schema.

**Required correction.** The dry-run row must carry a token distinct from every live-result token — e.g. `would contact` / `"result":"Planned"` — and `fetch_long.rs`, `docs/commands/fetch.md` and `docs/Releases.md` must show that row. If the token is not changed before release, the help must be rewritten to state plainly that a dry run prints `no change` for every repository regardless of upstream state and that the row is not an answer; that is the weaker fix, and after release the token can no longer be changed without a machine-output compatibility break.

**Closure test.** With a tracking ref one commit behind, `gwz --dry-run fetch` and `gwz --dry-run --json fetch` must produce a human token and a `result` value that appear in no live fetch, and the same strings must appear in `fetch_long.rs`, `docs/commands/fetch.md` and `docs/Releases.md`.

---

### P2-2 `docs/ClaudeCode.md` promises that from 1.0.17 a lane session ends with no action; the binary refuses disposal of a lane whose session wrote the two files the same page names

**Location.** `gwz-cli/docs/ClaudeCode.md:188-204` (added by `ad095e9`), reinforced at `gwz-cli/docs/ClaudeCode.md:162-167` ("On gwz 1.0.17 the remove hook retires an integrated lane on its own, so the count only climbs where a removal was refused or never ran").

**Violated invariant.** A release note may not attribute a refusal to superseded releases when the release it describes still produces it. L197-204 reads "**On gwz 1.0.14 and 1.0.16 it does not.** There `dispose` refuses ... and under local adoption `.claude/settings.local.json` and `.claude/.cc-writes/` are among them" — the construction tells the reader those two names are the old releases' problem. L193-195 concludes "the ordinary end of a lane session is that the lane goes away and you do nothing."

**Reproduction.** Two runs against the built binary, both on lanes with nothing unmerged in git.

Case A — the files exist in the source, the lane's session modifies them (the doc's exact "carried into the lane by the copy" case):

```
$ gwz local dispose E --wait 30
gwz: UnwaivedHazard: local dispose `E` ... unwaived hazard(s) by category:
 regenerable 3: ... ; unchanged copy 1: ... (notes.txt);
 changed copy 2: `@root` ignored user data (...) (.claude/.cc-writes/),
 `@root` ignored user data (...), text content (.claude/settings.local.json);
 unique to the lane 0; to delete anyway ...: `gwz local dispose E --force dirty` ...
exit=1
```

Case B — the session creates them in the lane:

```
$ gwz local dispose D --wait 30
gwz: UnwaivedHazard: ... unique to the lane 2:
 `@root` ... (.claude/.cc-writes/), `@root` ..., text content (.claude/settings.local.json);
 ... `gwz local dispose D --force dirty` ... exit=1
```

The headline mechanism itself is sound and I confirmed it: a lane that only built (`target/.rustc_info.json`, `target/debug/{deps,.fingerprint}`, a `.dylib`) and touched nothing else disposed clean — `deleted local clone \`F\`: ... removed, its row removed`, exit 0. The defect is confined to the `.claude` claim.

**Impact.** This is the central promise of the 1.0.17 section for the page's only audience. The remove hook passes neither `--force` nor `--keep` (`docs/ClaudeCode.md:116-119`), so the refusal keeps the lane, and the `--max-lanes` default 8 ceiling climbs exactly as on 1.0.16 — which L162-167 tells the user it will not. The user upgrades to 1.0.17 expecting hands-off retirement and gets the 1.0.16 experience for every session that writes Claude Code's own local state.

**Required correction.** L197-204 must stop attributing the `.claude/settings.local.json` / `.claude/.cc-writes/` refusal to 1.0.14 and 1.0.16 alone, and state that a session which writes those files still produces a `changed copy` (or `unique to the lane`) refusal on 1.0.17, with the retirement branch at L206-239 named as the remedy. L193-195 and L162-167 must be qualified to "a lane the session did not write outside git" rather than the unconditional "you do nothing".

**Closure test.** A lane created from a source carrying `.claude/settings.local.json`, modified in the lane, then merged, must dispose or refuse exactly as `docs/ClaudeCode.md` says it will; run both Case A and Case B above and diff the outcome against the page.

---

### P2-3 `docs/Releases.md` states that an older gwz's index refusal "names the minimum version that reads it"; the observable refusal names no gwz version at all

**Location.** `gwz-cli/docs/Releases.md:148-151`; same claim in weaker form at `gwz-cli/docs/ClaudeCode.md:31-32` ("an older one refuses the whole index and says so").

**Violated invariant.** A compatibility note must describe the failure the user will actually see, because that note is what a user on an older release uses to judge upgrade risk. This is walkthrough 4's decisive fact for a 1.0.13 user.

**Reproduction.** I have no pre-1.0.14 binary on this machine, so I tested the refusal's *shape* in both directions available to me, by putting a schema neither build reads into a throwaway index:

```
$ sed 's|/v2|/v3|' ... > .gwz/local-family.yml
$ /Users/owebeeone/limbo/gwz-dev-docs-release/target/release/gwz local list
gwz: ManifestInvalid: local family list: local family: .../.gwz/local-family.yml is
 malformed: `schema: gwz.local-family/v3` is not `gwz.local-family/v1` or
 `gwz.local-family/v2`; this file is not in a format this store reads
exit=1
$ gwz local list            # installed 1.0.16 baseline
  ... byte-identical message ...
exit=1
```

The refusal calls the file **malformed**, names the schemas *it* reads, names no gwz version and offers no remedy. Both halves of the surrounding claim did check out: the index the new binary writes declares `schema: gwz.local-family/v2`, the 1.0.16 baseline reads it (`gwz local list` → the root row, exit 0), and a hand-downgraded `v1` index was read unchanged by the new binary and rewritten to `v2` on its first write (`local clone V`) — so only the older-gwz refusal wording is at issue.

**Impact.** A 1.0.13 user reading Releases.md is told the downgrade failure is self-describing. What they will meet is a `ManifestInvalid ... is malformed` line that reads like a corrupt or hand-edited file, sending them to Troubleshooting or to deleting the index rather than to upgrading. The wording lives in already-published binaries and can never be corrected; only the note can.

**Required correction.** Either verify against a real 1.0.13 binary and, if it names no version, rewrite L148-151 to quote the refusal the user will see and say plainly "upgrade every gwz on that workspace to 1.0.14 or later"; or, if it does name a version, leave it and record the evidence. `docs/ClaudeCode.md:31-32` needs the same treatment.

**Closure test.** Run a genuine pre-1.0.14 gwz (`v1.0.13`) `local list` against a v2 index and paste its exact line into Releases.md.

---

### P3-1 The docs name the waiver argument `<categories>`, but `--force` takes hazard names, not the four category names

**Location.** `gwz-cli/docs/Releases.md:92`, `gwz-cli/docs/LocalClones.md:380`, `gwz-cli/docs/commands/local.md:305` — all three added by `ad095e9`, all reading "The refusal then prints the exact `--force <categories>` command".

**Violated invariant.** A placeholder in a command template must name the vocabulary the flag accepts. The four categories this commit introduces are `regenerable`, `unchanged copy`, `changed copy`, `unique to the lane`. `--force` accepts only `open-merge`, `dirty`, `unpreserved-history` (`gwz local dispose --help`: "Known names: open-merge, dirty, unpreserved-history"; `gwz local dispose -h` usage line: `--force <hazard,...>`). `docs/commands/local.md:308-311` even calls the same strings "the waiver vocabulary's own order (`open-merge`, `dirty`, `unpreserved-history`)" three lines after calling them categories.

**Reproduction.** Both refusals I produced printed the hazard vocabulary, never a category:

```
... `gwz local dispose B --force unpreserved-history` ...
... `gwz local dispose C --force dirty` ...
```

**Impact.** A user reading the sentence before reaching a sample will type `--force "changed copy"` or `--force unique-to-the-lane` and get a parse error. In `Releases.md:92` there is no sample beneath it to correct the impression.

**Required correction.** Replace `<categories>` with `<hazard,...>` (or `<waivers>`) at all three sites, matching the binary's own usage line.

**Closure test.** `grep -rn 'force <categories>' gwz-cli/docs` returns nothing, and every `--force` placeholder in the docs matches the usage string in `gwz local dispose -h`.

---

### P3-2 The "Subagents" prohibition in `docs/ClaudeCode.md` is keyed to a condition this release satisfies, and the commit updated every other paragraph that shared it

**Location.** `gwz-cli/docs/ClaudeCode.md:134-139`.

**Violated invariant.** One page must not hold two answers to the same question. The paragraph reads "Do not use subagent worktree isolation in a GWZ workspace **until an integrated lane disposes in one command**". `ad095e9` deleted the sibling marker that carried that same condition ("**Unmeasured:** the release in which an integrated lane disposes in one command (S3.5)") and replaced L188-195 with "**From gwz 1.0.17, an integrated lane needs no waiver**", and rewrote the `--max-lanes` bullet at L162-167 for the same reason — but left this bullet on the old condition.

**Reproduction.** `git show ad095e9 -- docs/ClaudeCode.md` touches L159-167 and L185-239 and never L134-139; `grep -n 'until an integrated lane disposes in one command' docs/ClaudeCode.md` → line 134.

**Impact.** The reader cannot tell whether the subagent prohibition is lifted by this release. Given P2-2 the answer is genuinely "not reliably", which makes the paragraph's *reason* right and its *trigger* wrong — the worst combination to leave unstated.

**Required correction.** Restate the bullet against what actually holds after 1.0.17 (a subagent lane still refuses when the subagent wrote outside git), or say the prohibition stands and why.

**Closure test.** No sentence in `docs/ClaudeCode.md` gates behaviour on "until an integrated lane disposes in one command"; the subagent guidance names a release or a condition the reader can evaluate.

---

### P3-3 `setup` is documented as computing the handlers' `--wait-secs`, and emits no `--wait-secs`

**Location.** `gwz-cli/docs/ClaudeCode.md:81-87`; the same claim in `gwz hook claude-code setup --help` ("Inside a workspace the timeouts and the handlers' `--wait-secs` are computed from the same run-time estimate the create hook uses").

**Reproduction.** In the throwaway workspace:

```
$ gwz hook claude-code setup --project
  "command": "gwz hook claude-code worktree-create", "timeout": 361,
  "command": "gwz hook claude-code worktree-remove", "timeout": 360,
```

The timeouts confirm the documented arithmetic (300 wait + ~1 s estimated copy + 60 = 361; 300 + 60 = 360), but neither handler carries `--wait-secs`. The doc never says the option is omitted when the computed wait equals the compiled-in 300 s default.

**Impact.** This is a walkthrough-3 guess point: a user told the block is computed for their workspace inspects it, finds one of the two computed values absent, and cannot tell whether the estimate ran, whether their workspace is too small for it, or whether `setup` is broken. It also hides the one case that matters — a workspace whose estimate raises the wait above 300 s, where presumably the flag does appear.

**Required correction.** State that `--wait-secs` is emitted only when the estimate raises the wait above the compiled-in default, and that its absence means the default stands.

**Closure test.** `gwz hook claude-code setup --project` in a small workspace produces a block whose contents `docs/ClaudeCode.md` accounts for exactly, with no reader inference required.

---

### P3-4 The blessed refusal sample contains Rust `{:?}` struct syntax

**Location.** `gwz-cli/docs/LocalClones.md:346` (pre-existing sample, now surrounded by this commit's new four-category prose at L363-445, which describes the report as printing "the paths or object ids it holds").

**Reproduction.** My lane-with-a-unique-commit refusal reproduced the sample's shape exactly:

```
... unique to the lane 1: `@root` 2 protected root(s) of @root are preserved whole
in no surviving family repository: Head 2161f793b7c1b489bd2b9333763b3db143a6dea4,
Ref { name: "refs/heads/main" } 2161f793b7c1b489bd2b9333763b3db143a6dea4; ...
```

**Impact.** `Ref { name: "refs/heads/main" }` is neither a path nor an object id; it is an internal type's debug formatting, in the one line a user reads when gwz refuses to delete their work. Documenting it as the expected output freezes it. A user cannot copy `Ref { name: "refs/heads/main" }` into any gwz or git command.

**Required correction.** Print `refs/heads/main` (and `HEAD`) and update the sample. If the format cannot change for this release, the surrounding prose should not claim the report prints "the paths or object ids it holds".

**Closure test.** No sample in `gwz-cli/docs/` contains `{ name:`, and a refusal naming a protected ref prints a ref name a user can paste into `git`.

---

### P3-5 `docs/commands/fetch.md` was not brought forward with the `--dry-run` carve-out the commit added to the help

**Location.** `gwz-cli/docs/commands/fetch.md:27-29` versus `gwz-cli/src/fetch_long.rs:24-30`.

**Violated invariant.** The commit message says it "state[s] that fetch `--dry-run` contacts nothing". It added that paragraph to `fetch_long.rs` (hence to `docs/CLI.md`) and to `docs/Releases.md:46-51`, but `docs/commands/fetch.md` — the page `docs/Workflows.md:109` and `docs/Releases.md:70` both send the reader to, and the first page of walkthrough 1 — still carries the unqualified bullet "**It never skips the network.** ... a fetch that does not connect has answered nothing", with the `--dry-run` example 60 lines below under Examples and no cross-reference between them.

**Reproduction.** `git show ad095e9 --stat` lists nine files; `docs/commands/fetch.md` is not among them. `grep -n 'dry-run' docs/commands/fetch.md` → only the Examples block at L85-89.

**Impact.** The one page a new user is pointed at asserts a rule the help calls out by name as having exactly one exception, and never mentions the exception where the rule is stated. Compounded by P2-1, a reader of this page has no way to learn that a dry-run row is not an answer.

**Required correction.** Add the carve-out to the "What it does not do" bullet in `docs/commands/fetch.md`, and add `gwz --dry-run fetch` to the Examples in `fetch` clap help so the two surfaces list the same examples.

**Closure test.** The "never skips the network" bullet in `docs/commands/fetch.md` names `--dry-run`, and `gwz fetch --help`'s Examples and `docs/commands/fetch.md`'s Examples list the same invocations.

---

### P3-6 Pre-existing on `docs/ClaudeCode.md`, shipping in this release: draft banner, unresolved plan-step references, and an unfinished sentence

Not introduced by `ad095e9`, but this commit is the page's release-readiness pass and these ship with it.

- **L64.** "**Which placement.** One is the recommendation." The paragraph never says which one. The user's literal question is the heading; the answer is missing. A first-day reader must guess between `--project`, `--project --local` and `--user`.
- **L3-6.** The page opens "Status: draft written from the accepted plan before its probes have run" and refers the reader to "the plan's steps S1.3, S2.1, S2.2 and S3.2" — dev-docs identifiers no user of the published site can resolve. Eight **Unmeasured** markers remain (L108, 125, 130, 138, 264, 277, 280).
- **L5 vs L139.** The header's list of steps and the body's citations disagree: S3.2 is listed in the header and cited nowhere; S2.3 is cited at L139 and absent from the header. Verified with `grep -no 'S[0-9]\.[0-9]' docs/ClaudeCode.md`.

**Required correction.** Name the recommended placement at L64. Either finish the probes or replace the plan-step identifiers with plain statements of what is unverified; a published page should not cite an unpublished document's step numbers.

**Closure test.** L64 names a placement; `grep -n 'S[0-9]\.[0-9]' docs/ClaudeCode.md` returns nothing, or every identifier resolves to something the reader can open.

## Walkthrough log

### Walkthrough 1 — learning what moved upstream with `gwz fetch`

Sources used: `docs/Workflows.md:100-118`, `docs/commands/fetch.md`, `gwz fetch --help`, `gwz help fetch`.

| Step | Command | Printed | Guess point |
| --- | --- | --- | --- |
| Find the verb | `gwz --help` | `Inspect: status ls diff log fetch` | none — `fetch` sits with the read-only verbs, which is where a git user looks |
| Read the verb | `gwz fetch --help` | full long help, ends with 5 Examples | `--dry-run` is discussed at length in the prose but is absent from the Examples list (P3-5) |
| First run, no remote | `gwz fetch` | `status: Noop` / `@root  .  no upstream` / exit 0 | none — row kind matches `fetch.md:46-48` |
| With a local bare `origin`, up to date | `gwz fetch` | `status: Noop` / `@root  .  no change  (origin/main, +0 -0)` / exit 0 | doc sample shows `status: Partial`; `Noop` and `Ok` also occur and no doc lists the status words. Minor. |
| After the upstream really moved | `gwz fetch` | `status: Ok` / `@root  .  5e5700a..2bde72b  (origin/main, +0 -1)` / exit 0 | none — matches `fetch.md:43-51` exactly |
| The preview, with the upstream one commit ahead | `gwz --dry-run fetch` | `status: Noop` / `@root  .  no change` / exit 0 | **P2-1.** I had to run the live fetch immediately afterwards to discover the preview's `no change` was not an answer. |
| Machine form | `gwz --json fetch` | one `fetch_repos` row with `result`, `before`, `after`, `ahead`, `behind`, `remote`, `upstream` | none; `--dry-run --json` gave `"result":"Unchanged"` (P2-1) |
| Another remote | `gwz --remote upstream fetch` | `status: Failed` / `@root  .  failed  MissingRemote: workspace root '@root' at '.': missing remote 'upstream'` / exit 1 | none; exit 1 matches the documented table |

### Walkthrough 2 — create a lane, build in it, merge it, retire it, then hit a refusal

Sources used: `docs/LocalClones.md:323-445`, `docs/commands/local.md:279-350`, `gwz local --help`, `gwz local dispose --help`.

1. `gwz init`, commit `gwz.conf` — `status: Ok`.
2. `gwz local clone A --wait 30` → `created local clone \`A\` at .../demo-A (verbatim; recorded as ../demo-A; 47 files copied ...; family fam_..., founded)`. `gwz local list` shows `root` and `A`, both `checkout ready`. No owner column appears when no `--owner` was given; `docs/ClaudeCode.md:174` calls it "its owner column", which reads as always present. Minor guess point.
3. `gwz local dispose A --wait 30` (untouched lane, no waiver) → `status: Ok` / `deleted local clone \`A\`: .../demo-A removed, its row removed`. Matches `docs/LocalClones.md:357-359` form exactly. **The no-waiver claim holds for an untouched lane.**
4. Lane `B` with a commit made only in the lane → refusal, exit 1, the four categories in the documented order with the empty ones printed, and `to delete anyway ...: \`gwz local dispose B --force unpreserved-history\``. Matches the form of the `docs/LocalClones.md:346` sample line for line, including the `Ref { name: ... }` debug syntax (P3-4).
5. Followed the printed command, adding one waiver the report did not ask for: `gwz local dispose B --force unpreserved-history,dirty --wait 30` → `status: Ok` / `... removed, its row removed; forced past: unpreserved-history; unused waiver: dirty`. **The unused-waiver report works and matches `docs/commands/local.md:313` exactly.**
6. Lane `C`: gitignored `notes.txt`, `target/`, `cache/`, `__pycache__/` in the source; in the lane, a committed `feature.txt` plus an edit to `notes.txt`; then `gwz --target @all merge --remote C` → `status: Ok`, fast-forwarded. `gwz local dispose C --wait 30` → refusal with `regenerable 3` (`__pycache__/`, `cache/`, `target/`), `unchanged copy 0`, `changed copy 1` (`notes.txt`), `unique to the lane 0`, offering `--force dirty` and naming none of the regenerable entries. **This is the `docs/LocalClones.md:376-383` "offered `--force dirty` alone" claim, reproduced.**
7. `gwz local dispose C --force dirty --wait 30` → `status: Ok ... forced past: dirty` (no unused waiver). Correct.
8. Lane `F`: nothing but a freshly built `target/` (`.rustc_info.json`, `debug/deps/libfoo.dylib`, `debug/.fingerprint`) → `gwz local dispose F --wait 30` → `status: Ok`, deleted, **no waiver**. The regenerable recognisers work as `docs/commands/local.md:320-339` describes.
9. The three "Not Yet" `--dry-run` refusals rewritten by this commit are exact: `local create with dry_run is not supported by this gwz-core build` for `local clone`, and `local family operations with dry_run is not supported by this gwz-core build` for `list`, `dispose` and `disband`. The `pull`/`push` fall-through is also as rewritten: `gwz: GitCommandFailed: remote 'NOSUCH' does not exist`.

### Walkthrough 3 — a Claude Code user installs the hooks, gets a lane, retires it

Sources used: `docs/ClaudeCode.md`, `gwz hook --help`, `gwz hook claude-code --help`, `gwz hook claude-code setup --help`.

1. `gwz hook claude-code setup --project` → the block, then `gwz: not written; pass --write to merge it` and `gwz: install or refresh the agent skill too: copy \`skills/gwz/SKILL.md\` to \`~/.claude/skills/gwz/\``. Timeouts 361 and 360; no `--wait-secs` on either handler (P3-3).
2. `gwz hook claude-code setup --project --local --write` → `gwz: merged WorktreeCreate and WorktreeRemove into .../.claude/settings.local.json`, and the pre-existing `{"v":1}` key survived untouched, as `docs/ClaudeCode.md:57-62` promises.
3. Re-run → `gwz: ... already carries the block`, exit 0. Idempotent as documented.
4. `--project --local --remove` → `gwz: removed WorktreeCreate and WorktreeRemove from ...`; the file is back to `{"v":1}` — the `hooks` object was dropped, as `docs/ClaudeCode.md:292-294` says. Re-run → `gwz: ... does not carry the block; nothing was changed`, exit 0. `--write --remove` together → clap error, exit 2. **The lifecycle pair is complete, symmetrically named, and documented together.** `--remove` also reprints the whole block on stdout before the outcome line, which `docs/ClaudeCode.md:284-297` does not mention; a trivial surprise, not filed.
5. Retiring a lane with unmerged work — followed `docs/ClaudeCode.md:206-239`. The three branches (merge then dispose, `--force` exactly what was named, `--keep`) all map onto real commands and real output. The `@all` warning at L224-226 is the one thing I would not have worked out from `--help` alone, and it is present.
6. **Where it broke:** the "ordinary end" case. A lane whose session wrote `.claude/.cc-writes/` and `.claude/settings.local.json` — created in the lane, or inherited and modified — refuses disposal on this build, in both directions (P2-2). `docs/ClaudeCode.md:197-204` attributes that refusal to 1.0.14 and 1.0.16.
7. **Gap:** `docs/ClaudeCode.md` has no remove-hook outcomes table. The page's step 4 (L116-119) covers only "runs `gwz local dispose`" and "a refusal keeps the lane". `already-absent` appears only in the log-classification list at L332-335 and in prose in `docs/commands/hook.md:72`, which is not part of this commit. A user asking "what does the remove hook do when the lane is already gone, or when `worktree_path` is not a lane" must leave the page. Not filed separately; folded into the P3-2/P3-6 assessment of the page's readiness.

### Walkthrough 4 — reading `docs/Releases.md` to decide on upgrading from 1.0.13 and from 1.0.16

- **Structure.** Sections run 1.0.17 → 1.0.4 → 1.0.0 → 0.14.0. The 1.0.17 section carries a "Catching up: 1.0.14 and 1.0.16" subsection. `git tag -l 'v1.0.*'` confirms no 1.0.15 exists, so a 1.0.14→1.0.17 reader is fully served and a 1.0.13→1.0.17 reader gets 1.0.14 + 1.0.16 + 1.0.17. The sentence "No release notes were written for 1.0.14 or 1.0.16" implies the intervening 1.0.5, 1.0.8 and 1.0.10-1.0.13 do have notes; they do not, and there is no section for any of them. It does not obstruct either walkthrough, so it is not filed.
- **Every named command and flag exists.** Verified against the built binary: `gwz fetch`, global `--remote`/`--json`/`--dry-run`, `gwz local dispose --force <hazard,...>`/`--keep`/`--wait`, `gwz local clone --owner <token>`/`--wait`, `gwz local list`, `gwz local disband`, `gwz merge --remote <name>`, `gwz hook claude-code worktree-create|worktree-remove|setup` with `--project`/`--local`/`--user`/`--write`/`--remove`, `gwz ls --unmaterialized`. `--prune` and `--tags` are correctly declared not offered (deferred; not filed).
- **Index schema v2.** The index the new binary writes declares `schema: gwz.local-family/v2` — matches L144-147. The 1.0.16 baseline reads it (`gwz local list`, exit 0). A hand-downgraded v1 index was read unchanged and rewritten to v2 on the next write — matches "writes v2 on its first write of any kind". The one claim I could break is the older-gwz refusal wording (P2-3).
- **Compatibility consequence a 1.0.13 user needs and gets:** the bolded "Once a 1.0.14 or later gwz has written a workspace's family index, every gwz used on that workspace must be 1.0.14 or later" is present and prominent. Good.
- **Compatibility consequence a 1.0.16 user needs and does not get:** the 1.0.17 section's lane-disposal promise is stated without the `.claude` exception (P2-2), and `--force <categories>` misnames the waiver vocabulary in the release note itself (P3-1).

### Spot-check — `docs/CLI.md` against the binary

Extracted the `gwz fetch`, `gwz local dispose` and `gwz hook claude-code setup` blocks from `docs/CLI.md` and diffed each against the built binary's `--help`. All three differ only by one line of trailing whitespace inside the `--sync` possible-values block, stripped by the generator. The global `--remote` long text, including the new "On `fetch` it selects the remote each selected repository contacts" sentence, appears in all three blocks and matches `src/globalargs/parser.rs:169` character for character. **CLI.md is a faithful regeneration; no finding.**

### Spot-check — `skills/gwz/SKILL.md`

The three added sentences describe `gwz fetch` correctly against the binary, except that "`gwz --dry-run fetch` contacts no remote at all" carries P2-1's problem into the agent skill without saying what the dry-run rows look like — an agent reading a `no change` row after a dry run will report "nothing moved upstream". Covered by P2-1's correction; not filed separately.

## What I could not test and why

- **The pre-1.0.14 refusal of a v2 index (P2-3).** No gwz older than 1.0.16 exists on this machine (`~/.cargo/bin/gwz` is 1.0.16; the only other binaries are this lane's and `gwz-dev`'s, both 0.2.0-dev), and downloading a release asset is outside my read-only remit. I tested the closest available analogue — an unreadable schema against both the new binary and the 1.0.16 baseline — and both produced a `ManifestInvalid ... is malformed` line naming no gwz version. The forward direction (1.0.16 reading a v2 index the new binary wrote) I did test, and it works.
- **`gwz fetch` while a coordinated merge is open.** The help asserts fetch "is one of the few network verbs that still runs while a merge is open" and `docs/commands/fetch.md:23-25` and `docs/Releases.md:42-44` repeat it. I could not cheaply construct a stuck open merge in a single-repository throwaway; every `gwz merge --remote` I ran closed cleanly. Claim untested in both directions.
- **The hooks under Claude Code itself.** I ran `gwz hook claude-code setup` in all its forms, but not `worktree-create` / `worktree-remove` driven by a real Claude Code session, so the "Claude Code shows nothing while `WorktreeCreate` runs" paragraph and the two-minute figure, the `Error creating worktree: ...` wrapping at `docs/ClaudeCode.md:305-307`, the owner-token grammar refusal and the differing-handler warning at L72-73 are all unverified. P2-2 is nevertheless established: I reproduced the `gwz local dispose` refusal the remove hook invokes, with the files the page names, which is the step the hook's success depends on.
- **The multi-member case throughout.** My throwaway workspace had only `@root`; every `fetch`, `local` and `dispose` row I compared is a single-repository row. The `mem_*` rows in the doc samples, the `--target`/`--member` selectors, `--partial`, and the per-repository breakdown of a dispose refusal across several members are untested. Building a multi-member workspace would have meant several more clones against 8.3 GB of free space.
- **`--max-lanes`, `--min-free-gb`, the free-space estimate and the block-sharing probe.** Not exercised; the throwaway is a few dozen kilobytes and the disk figures in `docs/ClaudeCode.md:155-161` are themselves declared unmeasured by the page.
- **Windows.** `docs/ClaudeCode.md:36-38` defers it to the plan's Phase 5; this is a macOS host.
