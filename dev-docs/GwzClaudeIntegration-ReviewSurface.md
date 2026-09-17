# gwz hook / claude-code surface — SURFACE-AXIS REVIEW

**Review object:** `gwz-alpha` 0.2.0-dev, sha256 `9f8352985967157c7dbc6dadf6865b77f2923bb0c0f4162660fb4e7d579daa78` (cli rev `6c352fe`, core 1.0.13) — the `hook`, `hook claude-code`, `hook claude-code worktree-create|worktree-remove`, `claude-code`, `claude-code setup` help, and the `--owner` / `--wait` surface on `local clone|dispose|list|disband`; plus `gwz-cli/docs/commands/hook.md`, `docs/commands/claude-code.md`, `docs/commands/local.md`, `docs/ClaudeCode.md`, `docs/LocalClones.md`.
**Date:** 2026-09-18 **Axis:** Surface: the interface as the person using it meets it. Read-only; help and docs only.

**Verdict: NO-GO** — 2 × P2, 10 × P3. (NO-GO while any P2 is open.)

---

## 0. What I read and ran

Ran: `gwz-alpha --version`, `--build-info`, `--help`; `hook --help`, `hook claude-code --help`, `hook claude-code worktree-create --help`, `hook claude-code worktree-remove --help`; `claude-code --help`, `claude-code setup --help` and `-h`; `local --help` and `clone|list|dispose|disband --help`; `auth --help`, `forall --help` for tone; `local list`. Three no-write probes of `claude-code setup` argument validation (`setup` bare, `setup --local`, `setup --project --user`) — all print-or-refuse paths, nothing written.

Read: the five docs pages named above, plus `docs/README.md`, `mkdocs.yml` nav, and greps of `QuickStart.md`, `Workflows.md`, `Troubleshooting.md`, `Install.md`, `CLI.md` for reachability.

Did not read: any source, plan or design document.

---

## 1. Findings

### F1 — P2 — One feature, two top-level families, and the split is baked into another program's committed config

**Root cause.** Installation lives at `gwz claude-code setup`; the runtime lives at `gwz hook claude-code worktree-create|worktree-remove`. Nothing about the feature is under one noun.

**Location.** `gwz-alpha --help` line `Other:      auth  forall  hook  claude-code`; `hook claude-code --help` ("Write the settings block with `gwz claude-code setup`."); `claude-code --help` ("The default handler is the bare command `gwz hook ...`"). Each family's help has to point at the other because neither is complete.

**What a first-time user experiences.** Cold, `hook` and `claude-code` read as two unrelated features sitting in a bucket labelled "Other" with `auth` and `forall`, and with no one-line summary next to either. Someone who knows only "I want Claude Code to use gwz lanes" has a 50/50 guess at the entry point, and once inside `gwz hook --help` gets a page about serving payloads on stdin — the opposite of what they wanted, which was to install something.

**Why it is P2 and not P3.** `claude-code setup` writes the literal string `gwz hook claude-code worktree-create` into `.claude/settings.json`, and `claude-code setup --help` states that Claude Code's same-handler dedupe keys on that exact text. So the split is not merely a help-page arrangement: it is a string committed into project settings files across machines. Unifying it later (`gwz claude-code install` + `gwz claude-code hook worktree-create`, say) changes the handler text, breaks the dedupe on every machine mid-migration, and produces exactly the two-differing-handlers orphan-lane failure that `claude-code.md` lines 44–49 spends a paragraph warning about. This is the shape that ships forever.

**Required correction.** Decide before release whether `hook` is a genuine multi-tool family (the help asserts it is — "Today one family is served" — but nothing else is served, and no second tool is named as a candidate). If it is not, collapse to one family and one handler string now. If it is, then `claude-code setup` belongs under `hook` as `gwz hook claude-code setup`, so the noun a user searches for owns both halves, and `hook` keeps its multi-tool shape for the second tool.

**Check.** From `gwz --help` alone, with no prior knowledge, a reader can name the single command that installs the integration. Today they cannot.

---

### F2 — P2 — `worktree-remove` accepts and documents four creation-only options, one of whose help text is literally about creation

**Root cause.** Both hook leaves share one option set instead of each carrying its own.

**Location.** `gwz-alpha hook claude-code worktree-remove --help`, Options block:

```
      --min-free-gb <gb>
          Refuse a creation when free space on the filesystem holding the destination's parent is
          below this many gigabytes. ...
      --max-lanes <n>
          Refuse a creation once the family holds this many ready lanes (default 8)
      --base-ref <ref>
          Base for the fallback git worktree (default origin/<default-branch>, else HEAD)
```

Only `--wait-secs` and `--log` are meaningful on a removal, and `--wait-secs`'s own text ("Deadline for the attempt loop and for a removal's family lock") is written from the create side too.

**What a first-time user experiences.** Someone probing the remove hook by hand — the exact thing `hook --help` invites ("running one by hand is still useful for a probe") — reads a removal command that offers to refuse *a creation* for lack of disk space. They cannot tell which of the five options do anything here, and reasonably conclude the remove hook also creates something. The same option set is also baked verbatim into the `WorktreeRemove` handler by `setup`, so the noise ends up in the user's `settings.json` where they will see it again.

**Why P2.** Once released, those options are accepted arguments of a published command; removing them is a compatibility break, and worse, they are already written into users' settings files by `setup`, so a later removal turns a previously-working committed handler into a hard parse failure at hook time — a refused `WorktreeRemove`, which is the orphan-lane path.

**Required correction.** Give each leaf only its own options. `worktree-remove` keeps `--wait-secs` (reworded for removal: "Deadline for the family lock") and `--log`. `setup` emits only those in the `WorktreeRemove` handler.

**Check.** `gwz hook claude-code worktree-remove --help` contains the word "creation" zero times, and every option listed changes the removal's behaviour.

---

### F3 — P3 — `setup --write` has no paired undo; the only documented uninstall is "hand-edit the JSON"

**Root cause.** A write verb shipped without its inverse.

**Location.** `claude-code --help` names one subcommand, `setup`. `docs/ClaudeCode.md` lines 209–213, "Switching it off": *"Delete the block from the settings file it lives in."*

**What a first-time user experiences.** `setup --write`'s own help spends a full paragraph establishing that editing another program's configuration is delicate enough to warrant parse-first, temp-file, fsync, re-parse, atomic rename, and byte-exactness outside the block. Then the removal of that same block is left to the user with a text editor and no tooling — the asymmetry is conspicuous, and the careful half advertises exactly why the careless half is risky. A user who ran `--write` against all three placements while experimenting now has three files to find and hand-edit, and no command tells them which three.

**Required correction.** `gwz claude-code setup --remove` (or a sibling `gwz claude-code remove`), using the same parse/temp/fsync/rename discipline, removing only the inserted block, and a no-op when it is absent. Document it beside `--write` on the same page, not in a "Switching it off" section at the far end of a guide.

**Check.** `claude-code setup --help` shows install and uninstall adjacent, and `ClaudeCode.md` §"Switching it off" names a command rather than an editor. Additive, so P3 — but the docs' present answer is not a command surface at all.

---

### F4 — P3 — `--min-free-gb` is the one option in the family with no stated default

**Root cause.** Default omitted from the option text.

**Location.** `hook claude-code worktree-create --help` and `claude-code setup --help`: `--max-lanes` says "(default 8)", `--wait-secs` says "(default 300)", `--base-ref` says "(default origin/<default-branch>, else HEAD)". `--min-free-gb` says only that it "is a floor applied on top of the copy-cost estimate, never a replacement for it". `docs/commands/hook.md` line 65 repeats the omission in the options table. `ClaudeCode.md` line 137 also omits it.

**What a first-time user experiences.** Against three neighbours that all state a default, the silence reads as "there is a default and we forgot to tell you", not "there is none". The user cannot tell whether omitting the flag means no floor, or some compiled-in floor they are about to trip over on a small disk — and the failure it governs (a refused `WorktreeCreate`) surfaces as an aborted session, not as a message they went looking for.

**Required correction.** State it: `(default: no floor; only the estimate applies)` if that is the behaviour, or the number if it is not. Mirror into `hook.md`'s table.

**Check.** Every option on both hook leaves and on `setup` states a default or explicitly states that it has none.

---

### F5 — P3 — `--project` / `--user` are required and mutually exclusive, and `--help` shows them as two ordinary optional flags

**Root cause.** A required exclusive group rendered as plain options.

**Location.** `claude-code setup --help`:

```
Usage: gwz-alpha claude-code setup [OPTIONS]

Options:
      --project    Use the workspace root's .claude/settings.json
      --user       Use ~/.claude/settings.json
      --local      With --project, use settings.local.json instead
```

The usage line says `[OPTIONS]` and nothing more. Running `gwz-alpha claude-code setup` bare exits 2 with `gwz: InvalidRequest: claude-code setup needs --project or --user`; `--project --user` exits 2 with a clap conflict error; `--local` alone exits 2 demanding `--project`. All three refusals are correct and clear — but none of that constraint is visible before you trip it.

**What a first-time user experiences.** They read the help, see three independent-looking flags, guess (reasonably) that omitting them prints "the default placement", and get an error. The behaviour is right; the help is the part that left them guessing. Note also `--local`'s help depends on `--project` in prose only, while the usage line shows no grouping.

**Required correction.** Render the constraint in the usage line — `Usage: gwz claude-code setup <--project [--local] | --user> [OPTIONS]` — so the required choice is visible cold. Mirror it in `commands/claude-code.md`, which currently only shows three worked examples and never states that a placement flag is mandatory.

**Check.** A reader of `setup --help` alone can say, before running anything, that exactly one of `--project` / `--user` must be given.

---

### F6 — P3 — `local list --wait` is accepted and does nothing, permanently

**Root cause.** Uniformity for a wrapper was bought with a dead option on a user-facing command.

**Location.** `gwz-alpha local list --help`: *"Accepted and ignored. `gwz local list` is observation-only and takes no family lock, so there is nothing to wait for; the option exists here so a wrapper may pass --wait to every family verb uniformly."* Repeated in `commands/local.md` line 111 and in `local clone --help`'s `--wait` text.

**What a first-time user experiences.** A person reading `local --help`'s four subcommands sees `--wait` on all four and assumes it means the same thing on all four. On three it changes behaviour; on one it is decoration. The honesty of the help text is good, but the surface now carries an option that can never be given meaning — if `list` ever takes a lock, `--wait` cannot change from "ignored" to "waits" without silently altering the behaviour of existing scripts.

**Required correction.** Either drop it (a wrapper that wants uniformity can hold a per-verb table; this is one line of wrapper code against a permanent public option), or keep it and say in the same sentence that it is retained for forward compatibility and may acquire meaning — which is a different promise from "there is nothing to wait for".

**Check.** No command in the family advertises an option that does nothing.

---

### F7 — P3 — `local clone`'s usage line advertises four options this build refuses

**Root cause.** Reserved-but-refused flags left in the usage synopsis.

**Location.** `gwz-alpha local clone --help`:

```
Usage: gwz local clone <name> [dest] [--clean | --bare] [-b <branch>] [--from <name|path>] [--owner <token>] [--wait <secs>]
```

Four of the six bracketed options are, per their own help text, "Unsupported in this build; parsed but refused before copying."

**What a first-time user experiences.** The usage line is the first thing read and the last thing remembered. A user setting up lanes for agents reads `[--clean | --bare]`, decides a clean lane is what an agent wants, types it, and is refused after the fact. The long-form option descriptions do say so, but they say so *after* three sentences describing what the flag would do, so the reader has already formed the intent.

This matters more for this review than for `local` generally, because `ClaudeCode.md` line 101 (*"A lane is a snapshot of a live source"*) and the rebuild caveat make `--clean` the first thing a Claude Code user will reach for.

**Required correction.** Drop the unsupported four from the usage synopsis and keep them only as a short "reserved, refused in this build" note; or prefix each description with the refusal instead of ending with it.

**Check.** Every option in a usage synopsis does something in the shipped build.

---

### F8 — P3 — Top-level help gives the feature no one-liner and files it away from Lanes

**Root cause.** The group line is bare words, and the grouping is by implementation family rather than by what the user wants.

**Location.** `gwz-alpha --help`:

```
Lanes:      local clone|list|dispose|disband
Other:      auth  forall  hook  claude-code
```

Every other group either names its verbs concretely or is self-describing. `Other` gets four bare nouns, two of which are this feature.

**What a first-time user experiences.** The user's goal — "have Claude Code work in a lane" — is a *lane* goal. They scan `Lanes:` and find only the four manual verbs. They never scan `Other:`, and if they do, `hook` and `claude-code` carry no text to tell them which one they want. `gwz help COMMAND` is offered as the way to get more, but you have to already suspect the command.

**Required correction.** Either add the pair to the Lanes line (`Lanes: local clone|list|dispose|disband; claude-code setup`) or give `Other:` one-liners the way the Selection block gets them. This is presentation only, hence P3, but it is the first screen every user meets.

**Check.** A reader of `gwz --help` who wants agent-driven lanes reaches the right command without a second guess.

---

### F9 — P3 — The lane guide shows a Claude session token with no path to the page that explains it

**Root cause.** One-way cross-linking: the Claude Code pages link to Local Clones, not the reverse.

**Location.** `docs/LocalClones.md` line 210 renders a worked `gwz local list` with the owner column `claude-code:session_7`, and the word "Claude" appears nowhere else in the file's 568 lines — no link to `ClaudeCode.md`, no sentence saying where such a row comes from. `docs/commands/local.md` uses the same token at lines 87, 99, 108, 221, 262 and likewise never links out. `docs/README.md`'s "Choose A Path" table and "Guides" list omit Claude Code entirely, though `mkdocs.yml` does carry it in the nav. `Workflows.md` §"Work In An Isolated Lane" — the recipe page for exactly this — does not point to it either.

**What a first-time user experiences.** A user who has lanes running, sees a row owned by `claude-code:session_7` in `gwz local list`, and wants to know what made it, has three of the four natural entry points (README, LocalClones, local.md) dead-end on them. They find it only by scrolling the site nav.

**Required correction.** One "Lanes made by Claude Code: see [Claude Code](ClaudeCode.md)" line in `LocalClones.md` beside that example and in `commands/local.md` §"Lanes made by a tool"; one row in README's "Choose A Path" table ("Let Claude Code sessions work in lanes"); one pointer from `Workflows.md` §"Work In An Isolated Lane".

**Check.** Every page that prints a `claude-code:` owner token links to the page that explains it.

---

### F10 — P3 — The user-facing guide ships marked as an unverified draft against a gwz that does not exist yet

**Root cause.** Plan scaffolding left in a published docs page.

**Location.** `docs/ClaudeCode.md` lines 3–7:

> Status: draft written from the accepted plan before its probes have run. Paragraphs marked **Unmeasured** describe intended behaviour that no probe has yet confirmed; the plan's steps S1.3, S2.1, S2.2 and S3.2 replace them with observed behaviour. Requires a gwz that carries `gwz hook` and `gwz claude-code` (the release after 1.0.13); the installed 1.0.13 does not.

Ten further **Unmeasured** markers follow (lines 88, 105, 110, 118, 137, 168, 189, 202, 205, 218), each citing an internal plan step id (S1.3, S2.1, S2.2, S2.3, S3.2, S3.5). Line 219 says of the refusal table: *"the table below is the design."* Line 161 documents the retirement procedure against gwz 1.0.13 behaviour and cites `GwzLaneCleanFixes`, R0–R19 — internal requirement ids.

**What a first-time user experiences.** This is the page linked from the site nav as "Claude Code" — the feature's front door. It opens by telling the reader that the author has not run it, cites six work-item ids that mean nothing outside the team, and warns that the release they have does not carry the commands. Ten times through the page it disclaims the paragraph they just read. The effect on a first-day reader is not caution, it is distrust of the whole page — including the parts that are solid, and much of it is solid.

**Required correction.** Before this page ships: strike the status block, replace each Unmeasured paragraph with what the probes observed (or cut the claim entirely — an absent paragraph is better than a disclaimed one), remove every plan-step and requirement id, and state the minimum gwz version as a plain prerequisite line rather than as an apology.

**Check.** `grep -ci 'unmeasured\|S1\.\|S2\.\|S3\.\|GwzLaneCleanFixes\|the plan' docs/ClaudeCode.md` returns 0.

---

### F11 — P3 — The two leaf hook commands carry no examples, on the one page where a probe example is needed

**Root cause.** Examples attached to the parent only.

**Location.** `hook claude-code --help` ends with two examples, one of which is the useful probe:

```
  echo '{"name":"fix-123","session_id":"abc"}' | gwz hook claude-code worktree-create
```

`hook claude-code worktree-create --help` and `worktree-remove --help` end at the Global Options block with no Examples section at all — confirmed on the raw tails. Every other command I checked (`local clone`, `local dispose`, `local list`, `local disband`, `claude-code setup`) ends with examples.

**What a first-time user experiences.** `hook --help` explicitly invites hand-probing. The user drills down to the leaf for detail — the normal direction of travel — and the payload shape they need disappears. `worktree-remove` is worse: nothing at any level shows a `WorktreeRemove` payload, so the one field the command is built around, `worktree_path`, is named in prose but never shown in a runnable line.

**Required correction.** Move the probe example down to `worktree-create`, and add the matching `worktree-remove` one: `echo '{"worktree_path":"/path/to/lane"}' | gwz hook claude-code worktree-remove`.

**Check.** Both leaves end with a runnable example showing their own payload.

---

### F12 — P3 — `Usage: gwz` at top level, `Usage: gwz-alpha` everywhere below

**Root cause.** A hardcoded program name in the root help against clap's derived name in subcommands.

**Location.** `gwz-alpha --help` → `Usage: gwz [OPTIONS] <COMMAND>`; `gwz-alpha hook --help` → `Usage: gwz-alpha hook [OPTIONS] <COMMAND>`; examples throughout say `gwz ...`; the suggestion on a typo says `gwz-alpha`.

**What a first-time user experiences.** Someone evaluating the pre-release copies `Usage: gwz ...` from the first screen and gets "command not found", then copies `gwz-alpha` from the second screen and it works. Minor, and plausibly an artefact of the alpha binary name rather than of the release build — but it lands on the very first screen, and I could not verify from the surface alone which name the release build prints.

**Required correction.** Confirm the release binary prints one name consistently at every level, including in examples. If the alpha is meant to be run as `gwz-alpha`, its examples should say so.

**Check.** `gwz-alpha --help | grep -c 'Usage: gwz '` and the subcommand form agree.

---

## 2. The first-day walkthrough

Done on paper from `--help` only, as a user who knows gwz and Claude Code but has not read the design. Numbered guesses are points where help alone was insufficient.

```sh
gwz --help
```
→ `Other:      auth  forall  hook  claude-code`. **Guess 1:** two candidate entry points, neither with a one-liner, and neither under `Lanes:` where I looked first. I pick `claude-code` because the noun matches my goal. (F8, F1)

```sh
gwz claude-code --help
```
→ Good. Tells me `gwz claude-code setup`, tells me `--write` and the three placements in the header paragraph. This is the strongest page in the set.

```sh
gwz claude-code setup --help
```
→ Options read as three independent optional flags. **Guess 2:** is there a default placement? I try the bare command; it exits 2 with a correct message. The help should have told me, not the error. (F5)

```sh
gwz claude-code setup --project
```
→ Prints the block. I read the handler text and see `gwz hook claude-code worktree-create --wait-secs ... --max-lanes ...`. I now understand the two families are one feature — but only because I read generated JSON. (F1)

```sh
gwz claude-code setup --project --local --write
```
→ (Not run.) **Guess 3:** how do I confirm it took? There is no `gwz claude-code status` / `check` / `--verify`. The help says `--write` "does nothing when the block is already there", which means re-running is my only confirmation, and a silent success is indistinguishable from a silent no-op. I would end up `cat`-ing `settings.local.json` by hand.

**Use it once.** **Guess 4 — the largest gap.** Nothing in any `--help` says what *I* type to get a session into a lane. `hook --help` says hooks "are meant to be run by the tool, not by hand". `claude-code setup --help` ends at the settings block. The answer (`claude --worktree <name>`, or a background session, or a subagent) exists only in `ClaudeCode.md` — once at line 10 in a sentence about when Claude makes a copy, and once at line 207 inside a Caveats bullet that is itself marked **Unmeasured**. From the help alone, I installed something and have no idea how to trigger it.

**Find what it made.** **Guess 5.** `claude-code setup --help` never names `gwz local list`. `hook claude-code --help` never names it either. `worktree-create --help` mentions "the family" and "the family index" without ever naming the command that shows them. I get there only because I already knew `gwz local list` from the Lanes group.

```sh
gwz local list
```
→ On a fresh workspace: `root  checkout  ready  /Users/owebeeone/limbo/gwz-dev`. No `owner` column, which `list --help` correctly explains appears only when some row carries a token. Good.

**Undo the one use.** **Guess 6.**
```sh
gwz local dispose <name>
```
`local dispose --help` is clear that this refuses on hazards and that I cannot dispose the member I am standing in — so I know to run it from the root. What it does *not* say is that a verbatim lane will almost always refuse until I have merged it first. `local clone --help` does say "Integrate work from the receiving workspace with `gwz merge --remote <name>`", so the information exists in the family — but it is on the *create* page, not the *undo* page, and the undo page is where I am standing when I need it. `ClaudeCode.md` line 161 confirms the refusal is the normal outcome. So from `dispose --help` alone I hit a hazard refusal and my next instinct is `--force dirty,unpreserved-history`, which is exactly the destructive move the design does not want me making before a merge.

```sh
gwz --target @all merge --remote <name>
gwz local dispose <name>
```
→ Correct order, learned from `local clone --help` and `ClaudeCode.md`, not from `dispose --help`.

**Uninstall.** **Guess 7 — no command at all.** `gwz claude-code --help` lists one subcommand. There is no removal verb anywhere. The only instruction is `ClaudeCode.md` line 211: delete the block by hand, from whichever of three files I put it in, in a JSON file that gwz itself was careful enough to rewrite atomically. If I had experimented with two placements I would now be hand-editing two files with no command to tell me which. (F3)

**Summary of the walkthrough:** install is discoverable and well-written once you guess `claude-code` over `hook`; *use it once* is invisible from help; *undo it once* is reachable but routed through the wrong page; *uninstall* does not exist as a command.

---

## 3. What is right and should not be changed

- **`worktree-create` / `worktree-remove` are a correctly named, correctly paired lifecycle.** Same noun, opposite verbs, documented on one page, with the create-side guards explicitly stated as not applying to a reuse. This is the part of the surface that reads cleanest cold.
- **The output contract stated in `hook claude-code --help`** — prints only a path it created or verified; deletes only what `worktree_path` canonically names; every failure one line `gwz: <cause>; <remedy>` on stderr with empty stdout — is a genuinely good piece of help writing. It tells the reader the invariant, not the implementation, and it is the right level of detail for someone deciding whether to trust the hook with their tree.
- **`local dispose --help`** is the model the rest should follow: it names the hazard vocabulary (`open-merge`, `dirty`, `unpreserved-history`), states the `--keep` non-destructive alternative in the same breath as the destructive path, calls `--force` "an operator loss waiver, not crash recovery", and ends with five examples covering the whole matrix. Do not touch it.
- **`--owner`'s honesty.** "GWZ stores it, reports it in `gwz local list`, and never interprets, matches or acts on it", plus the explicit index-format-2 consequence and which gwz versions read it. A user can predict exactly what this flag will and will not do. That is rare.
- **The `local list` state column design** — one word when recorded and observed agree, `recorded/observed` when they do not — makes an interrupted create visible without a second command, and the help says so in those words. Good surface thinking.
- **`setup --write`'s stated write discipline** (parse first, temp beside, fsync, re-parse, rename, no byte outside the block, no-op when present) is the right thing to put in `--help` rather than hide in a design doc: it is precisely what a user weighing "do I let this program edit my Claude config" needs.
- **The `--project` / `--user` / `--project --user` refusals** are all exit 2 with accurate, specific messages. The behaviour is right; only its advertisement (F5) is not.
- **`mkdocs.yml` nav placement** — `Claude Code` under Guides beside `Local Clones`, `hook` and `claude-code` under Reference → Commands in the right order. The site structure is correct even where the in-page cross-links (F9) are not.

---

## 4. Risks and next action

**The one risk that outlives the release** is F1. Every other finding here is a help string, a doc line, or an additive command — all fixable in a patch release without breaking anyone. F1 is different in kind: the family split is serialized into `.claude/settings.json` files on user machines and in committed project settings, and Claude Code's same-handler dedupe is keyed on that exact string. Ship it and the arrangement is permanent, because the migration cost is the orphan-lane failure mode the docs already warn about. This decision is due *before* the first release that carries `gwz claude-code setup --write`, not after.

**F2 compounds F1** for the same reason: the creation-only options are not merely printed in help, they are baked by `setup` into the `WorktreeRemove` handler text on disk. Dropping them later turns a working committed handler into a parse failure at removal time.

**The largest first-day gap is not a P2 at all** — it is the "use it once" hole in §2, Guess 4. The surface can install the integration and can (awkwardly) undo it, but no `--help` anywhere tells a user how to make a session actually take a lane. That is one sentence in `claude-code setup --help`'s trailing text and one in `commands/claude-code.md`, and it converts the walkthrough from "I installed something and nothing appears to happen" into a working first day.

**Next action, in order:**

1. Settle F1 — one family or two, decided on whether a second tool is genuinely coming. Blocking.
2. Split the hook leaves' option sets (F2). Blocking.
3. Add the uninstall verb (F3) and the "how to trigger a lane session" sentence (§2 Guess 4) — both cheap, both close the walkthrough.
4. Sweep F4–F8 and F11–F12 as one help-text pass; sweep F9–F10 as one docs pass. Note that F10 alone is a rewrite of `ClaudeCode.md`'s ten Unmeasured paragraphs, which means the probes it cites must actually run before the docs can ship — that is likely the long pole, not the code.

Re-review the surface after (1) and (2) land; the remainder can be verified against the checks stated per finding without a second full pass.
