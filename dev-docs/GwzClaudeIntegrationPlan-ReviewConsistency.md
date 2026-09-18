# GwzClaudeIntegrationPlan.md — CONSISTENCY-AXIS REVIEW

**Review object:** `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `e5325c03144af35186cb32859d5e44ee3ab154765bb0dc06493e5dd2ee6306b6`, 594 lines, uncommitted working-tree modification of gwz-cli (`git status --porcelain` → ` M dev-docs/GwzClaudeIntegrationPlan.md`). Draft-stage review of an adopted plan before commit. 2026-09-17.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7`; gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8`. The tuple was verified with `shasum -a 256` / `git rev-parse` at the start and again at the end of the review; it did not move. Sources read with Read/Grep only; `gwz --version` (1.0.13), `gwz --help` and `gwz hook --help` (unrecognized subcommand, as expected) were the only binary invocations. No file, index or ref was modified.
**Date:** 2026-09-17
**Axis:** Consistency: the document against its controlling graph. Independent, adversarial, read-only. The other axis runs in parallel; nothing here relies on it. Filed verbatim by the lane owner.

**Verdict: NO-GO** — 0 P0, 0 P1, 2 P2, 7 P3. Every blocking finding is a bounded, text-only edit to the object. I pre-commit to GO on a revision that resolves {P2-1, P2-2} as specified.

---

## 0. Evidence base

Documents read in full or in the cited ranges:

- The object: `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` (all 594 lines).
- `gwz-core/dev-docs/GwzLaneCleanFixes.md` (all 144 lines, R0/R0.1 and R1–R19 verified verbatim at lines 29–105, the cause table at 111–122, the open questions at 137–144).
- `dev-docs/GwzLaneIssues.md` (register table at 13–16; L1 in full at 18–100).
- `gwz-cli/dev-docs/GwzClaudeIntegration-S0.1-Review.md` (findings F1–F18 at 53–281, the summary table at 300–319, and the round-1 fold record at 330–354).
- `gwz-cli/docs/LocalClones.md` (grepped for the refusal/flag facts the object quotes: lines 39–44, 88, 148–162, 310–345, 480–523).

Code read to test the object's unstated impacts:

- `gwz-cli/src/help.rs:11-17` — the hand-curated root-help command grouping (`Inspect: / Change: / Workspace: / Members: / Lanes: / Other:`), reproduced verbatim at `gwz-cli/docs/CLI.md:32`.
- `gwz-cli/src/cli_reference.rs:11-72` — `cli_reference_markdown()` renders the root help plus a help block for **every** clap subcommand path, recursively (`collect_command_paths`).
- `gwz-cli/src/tests/g00.rs:36-43` — `cli_reference_doc_matches_generated_clap_help` asserts `include_str!("../../docs/CLI.md")` equals the generated text exactly.
- `gwz-cli/scripts/release.py:407-416` — the release script runs `generate_cli_reference.py --check` and fails the release on drift.
- `gwz-cli/mkdocs.yml:70-95` — the nav lists `CLI.md` and a `commands/<name>.md` page per command family.

Citation audit result (all verified against the cited lines): R0, R0.1, R1–R8, R9, R10, R11, R12, R14–R16 as cited in the object's section 1, D3, D7, S3.1, S3.3, S3.4, O5 all exist in `GwzLaneCleanFixes.md` and say what the object says they say; O5's appeal to "its section 6" matches the first open question at line 139. L1 exists in `GwzLaneIssues.md` and supports the "1.0.13 changes nothing for L1" claim (line 8). F1–F18 all map to real findings, and every fold the 2026-09-12 trail entry claims is present in the object except F15 (see P3-4's neighbourhood; recorded in section 2, not raised as a separate finding). `gwz ls` as used in S1.3 is a real command (`gwz --help`), so F14 is genuinely cured. The 2026-09-17 script-to-subcommand fold is clean: no surviving sentence assumes shell scripts, `jq`, `python3`, a copy step, `integrations/claude-code/`, a stub `gwz`, or a "gwz missing" branch. The only residue is the dead fact in P3-4.

## 1. Findings

### [P2-1] D5's one-placement rule rests on a rationale that D1 abolished, and it forbids the placement the Goal is designed around

**Location:** object lines 25–28 (Goal), 190–198 (D1), 236–242 (D5), 413–420 (S1.4), 493–498 (S4.1).

**Root cause:** the one-placement rule and its enforcement were written for the pre-fold D1, in which each placement carried a *different* handler string (`$CLAUDE_PROJECT_DIR/gwz-cli/integrations/claude-code/...` versus a user-local copy). The 2026-09-17 fold replaced the handler with a single fixed string and kept the rule and its now-false justification.

**Violated invariant:** a decision's stated rationale must still hold after the decision it depends on is replaced; and no decision may forbid a configuration another section of the same document requires.

**Reproduction / state sequence.**
1. Section 1 line 73–75 (quoted from the Claude Code docs, given): "If you define the same handler in more than one settings file, it runs once. The dedupe is by identical handler; two different commands that both print a path both run."
2. D1 line 195–197: "the handler text is fixed so Claude's same-handler dedupe holds". So under D1 the two placements carry *identical* handlers and Claude runs the hook once. U4's undefined-winner hazard cannot arise.
3. D5 line 238–240 nevertheless states the rule with the pre-fold reason attached: "The hooks live in exactly one settings file per machine, project or user, never both: **two different commands that both print a path both run, and the winner is undefined (U4)**." The premise is false for the configuration D1 produces.
4. S1.4 line 413–418 commits the project-level block into the gwz-dev root's `.claude/settings.json`, "so every clone of the workspace has it".
5. The Goal line 25–28 states the reason the fallback exists at all: the hook "falls back to a plain git worktree ... **so it can live in user-level settings without breaking other projects**." User-level placement is the designed deployment for anyone who uses Claude Code outside gwz-dev.
6. After S1.4 lands, any developer who clones gwz-dev and also wants the integration in their other projects holds both placements. D1 line 193–195 says `gwz claude-code setup` "refuses when the other placement already carries the block (D5, enforced rather than documented)" — so the tool refuses, by enforcement, the exact configuration that step 2 made safe and step 5 designed for. There is no branch anywhere in the plan for "same handler in both files".

**Impact:** an implementer of S1.2 will build a refusal that blocks the plan's own primary deployment path, and S4.1 will document a "choose one" rule whose only stated reason is untrue. The gwz-dev clone user's choice is not free: the project block arrives by `git`, so to get user-level hooks they must delete a committed file's block. This is a self-contradictory contract between D1, D5, S1.4 and the Goal, not a style preference.

**Required correction:** rewrite D5's rule and D1's enforcement clause to key on the handler string, not the file: identical handler text in more than one settings file is explicitly permitted (Claude dedupes it); `gwz claude-code setup` refuses only when another placement carries a *different* handler for the same event — the `--command PATH` pinned form is the one case that can produce that. State that S1.4's committed project block and a user-level block coexist by design. Reassign U4's second half (which path wins) from "prevented" to "prevented for identical handlers; unresolved for differing handlers, which setup refuses".

**Closure test:** grep the revised D5 for the phrase "two different commands"; it must no longer appear as the justification for a rule that D1's fixed handler text disproves. Then trace a reader from the Goal line 27 ("user-level settings") through D5 and S1.4 to S4.1 and confirm no step forbids the resulting configuration.

### [P2-2] Two new top-level command families are added with no step owning the generated CLI reference, which a checked-in test and the release gate both enforce — and S1.4 waits on that very release

**Location:** object lines 336–366 (S1.1), 367–383 (S1.2), 413–420 (S1.4), 512–515 (S4.3); `gwz-cli/src/tests/g00.rs:36-43`; `gwz-cli/src/help.rs:11-17`; `gwz-cli/docs/CLI.md:32`; `gwz-cli/scripts/release.py:407-416`; `gwz-cli/mkdocs.yml:70-95`.

**Root cause:** the plan budgets S1.1 and S1.2 as code plus unit tests only, and does not account for the mechanically generated artefacts that adding a clap subcommand to gwz-cli obliges.

**Violated invariant:** a step's stated output and budget must cover every artefact the step's change makes mandatory in the repository it edits.

**Reproduction / state sequence.**
1. S1.1 (line 336–337) declares a "new `hook` command family; ~350 lines plus ~250 lines of test". S1.2 (line 367–369) declares "`claude-code setup`; ~150 lines plus ~120 lines of test". Together they add `gwz hook claude-code worktree-create`, `gwz hook claude-code worktree-remove` and `gwz claude-code setup`.
2. `src/cli_reference.rs:55-72` walks every clap subcommand path recursively, so all three new paths enter the generated reference; `src/help.rs:11-17` is the hand-curated root-help grouping (`Lanes: local clone|list|dispose|disband`, `Other: auth forall`) which is embedded verbatim in the generated root-help block.
3. `src/tests/g00.rs:36-43` asserts `docs/CLI.md` equals the generated text byte-for-byte, with the message "gwz-cli/docs/CLI.md is stale". Adding the subcommands makes this checked-in test fail until `python scripts/generate_cli_reference.py --write` is run and `docs/CLI.md` is committed.
4. `scripts/release.py:407-416` runs the same generator with `--check` as a release gate.
5. S1.4 (line 419–420) states "S1.4 waits for a gwz release that contains S1.1". That release cannot be cut while step 3/4 are unsatisfied, so the omission sits directly on S1.4's critical path — and on S4.3's, which must name "the minimum gwz version that carries the `hook` family".
6. Neither S1.1, S1.2, S1.4 nor S4.3 mentions `docs/CLI.md`, `src/help.rs`, `docs/commands/`, or `mkdocs.yml`. The plan's only documentation step, S4.1 (line 493), is a *new* page (`docs/ClaudeCode.md`) in Phase 4 — after the release S1.4 waits on.

**Impact:** an agent picking up S1.1 to its stated budget lands a change that fails gwz-cli's own test suite and, if merged, blocks the release. It also leaves an undecided question no step owns: whether `hook` and `claude-code` appear in the curated `gwz --help` grouping at all (they are agent/tooling surfaces, not user verbs), and whether each needs a `docs/commands/<name>.md` page and an `mkdocs.yml` nav entry.

**Required correction:** add to S1.1 (and S1.2, for `claude-code setup`) an explicit output line: "regenerate `docs/CLI.md` via `python scripts/generate_cli_reference.py --write`; decide and record whether the family appears in the curated grouping in `src/help.rs`; add a `docs/commands/` page and `mkdocs.yml` nav entry, or record why not." Raise the budgets to match, or split the reference/doc regeneration into its own step if the <500-line goal is to hold for S1.1 (which already stands at ~600 lines combined).

**Closure test:** the revised S1.1 text names `docs/CLI.md` and `src/help.rs`, and an implementer following S1.1 alone can run gwz-cli's test suite green. Confirm by re-reading `src/tests/g00.rs:36-43` against the revised step's output list.

### [P3-1] S1.2 is three goals in one step, and the third mints a lane the plan never retires

**Location:** object lines 367–383 (S1.2), 315–317 (the budget rule), 327–334 (S1.0), 384–412 (S1.3), 423–431 (S2.1), 470–481 (S3.4), 532–535 (the sketch).

**Root cause:** S1.2's title is literally conjunctive — "the setup command, **and** local adoption in gwz-dev" — and its body adds a third, evidentiary goal.

**Violated invariant:** the repo's plan rule (recorded in the operator's standing orders and echoed at object line 316) that each step is one goal; and the phase boundary the plan itself draws, in which Phase 1 is "the CLI path" and Phase 2 is "desktop chips and background sessions".

**Reproduction:** read S1.2 line 367–383. Goal one: implement `gwz claude-code setup` with its merge, placement and `--command` behaviour plus five test cases (~270 lines). Goal two: run `gwz claude-code setup --project --local --write` at the gwz-dev root. Goal three: "confirm the desktop app's environment supplies `PATH` with `gwz` ... **by triggering one creation from the app**" — a live desktop-app lane creation of the 65 GB gwz-dev workspace (section 1 line 134). D8 line 279 confirms this is a load-bearing probe: "S1.2 confirms `PATH` once."

**Impact.** (a) The step cannot be picked up by one agent as one goal, defeating the parallel-friendly requirement. (b) The third goal inverts the sketch: it performs a desktop-app creation inside Phase 1, before S1.3, whereas the sketch (line 533) orders every desktop probe (`S2.1`, `S2.2`) after S1.3, and D5 line 237–239 gates committed adoption on S1.3's evidence. (c) S1.0 says of its own probe lane "then retire it", and S1.3 says to retire its lanes through S3.4; S1.2 says nothing about the lane its desktop trigger creates. By section 1 line 120–126 and D3, dispose will refuse that lane, so it persists — and S3.4, the step that owns retirement, does not exist yet at that point in the order. (The same cross-surface leak appears once more in S1.3 line 388–389, which asks a "CLI probe" to record "what the desktop's diff and PR views show".)

**Required correction:** split S1.2 into "S1.2: the setup command" (code plus tests) and a separate adoption/probe step that owns the `--write` run, the desktop `PATH` confirmation and the retirement of the lane it creates; or move the desktop `PATH` confirmation into S2.1 and have D8 line 279 point there instead. Whichever is chosen, the step that triggers a creation must state how its lane is retired.

**Closure test:** no Phase 1 step title contains "and" joining two deliverables; every step that creates a lane names its retirement; the sketch's Phase 1 chain matches the revised steps.

### [P3-2] The dependency sketch omits S1.4's release precondition and the S1.4 → S4.1/S4.3 dependency

**Location:** object lines 529–541 (section 5), 419–420 (S1.4), 496–498 (S4.1), 512–515 (S4.3).

**Root cause:** S1.4's minimum-version condition was folded in on 2026-09-17 (trail line 591–592) without updating section 5.

**Violated invariant:** section 5's own contract — every stated dependency appears in the sketch. The sketch already demonstrates the notation for a non-step precondition at line 534 (`{ S2.3, S3.3, GwzLaneCleanFixes R0 in an installed gwz } -> S3.5`), so the omission is not a limitation of the form.

**Reproduction:** S1.4 line 419–420: "so S1.4 waits for a gwz release that contains S1.1, and S4.3 states the minimum version." The sketch line 532 shows only `... -> S1.3 -> S1.4`, and S1.4 is a sink — nothing is shown depending on it. Yet S4.1 line 497 requires "the minimum gwz version" and S4.3 line 514–515 requires "naming the minimum gwz version that carries the `hook` family (S1.4)". The prose at 537–541 discusses independence and S3.5 but never mentions a release.

**Impact:** an agent scheduling from the sketch alone (which is what the sketch is for) treats S1.4 as unblocked once S1.3 lands, commits a settings block naming a `hook` command no released `gwz` carries, and — by S1.4's own reasoning — every clone of gwz-dev aborts worktree creation. Two documentation steps are also shown as schedulable before the fact they must state exists.

**Required correction:** amend the sketch to `S1.3 -> { S1.4 requires a gwz release containing S1.1 }` in the same notation as line 534, add `S1.4 -> { S4.1, S4.3 }`, and add one sentence to the prose at 537–541 naming the release as the second external dependency alongside S3.5's.

**Closure test:** for each of S1.0 … S5.1, every dependency asserted in the step's prose has a corresponding arrow or brace entry in section 5, and vice versa.

### [P3-3] U3, U5 and O2 are assigned to no step, contradicting section 2's own preamble

**Location:** object lines 144–147 (the preamble), 156 (U3), 160–161 (U5), 547–548 (O2).

**Root cause:** the U and O lists were extended and renumbered across two folds without a completeness pass against the steps.

**Violated invariant:** section 2 line 145–147 asserts "These are the questions the probes in Phases 1 and 2 must answer before the integration is documented for other users." Every U must therefore be assigned to a step, or marked resolved with a reason (as U11 correctly is at line 179–181).

**Reproduction:** `grep -n "U3\|U5\|EnterWorktree\|Worktree location\|branch-prefix\|url-scheme"` over the object returns exactly three hits — lines 156, 160 and 548 — each being the item's own definition. No step text references U3, U5, `EnterWorktree`, "Worktree location", or `.gwz/url-scheme.yml`. By contrast U1→S2.1, U2→S1.3, U4→S1.1/D5, U6→S1.0/S1.3/S3.1, U7→S1.3, U8→S3.1, U9→S2.3, U10→S2.2 are all explicitly assigned. O2 line 548 claims "S1.3 decides from what the lane looks like", but S1.3's twelve-item record list (lines 384–412) never mentions lane-local state or `.gwz/url-scheme.yml`, so the assignment is asserted from the open item's side only.

**Impact:** Phase 4 documentation (S4.1) is gated on Phases 1 and 2 answering these questions, and two of them will still be open when S4.1 runs. U5 is not idle: if the desktop app's "Worktree location" setting still applies once a hook owns creation, it bears directly on D8's printed path and on the fallback's `.claude/worktrees/<name>` layout.

**Required correction:** add U3 and U5 to S2.1's record list (both are desktop/session-surface observations), and add `.gwz/url-scheme.yml` and lane-local state to S1.3's record list so O2's claim is true from both sides. If U3 or U5 is judged not worth a probe, mark it resolved with a reason in section 2 the way U11 is.

**Closure test:** every U1..U11 and O1..O5 string appears in at least one step's text, or carries an in-place "Resolved" note.

### [P3-4] Section 1's not-a-workspace error-code fact is dead after U11/D8 and invites an implementer to rebuild the branch the fold abolished

**Location:** object lines 138–142 (section 1), 179–181 (U11), 255–283 (D8), 281–283 (the historical note).

**Root cause:** the 2026-09-17 D8 simplification removed the external-script status probe but left the fact it existed to support.

**Violated invariant:** section 1 is the plan's "facts this plan rests on"; a fact no decision or step rests on, and whose subject matter a decision explicitly abolished, is misdirection.

**Reproduction:** lines 138–142 record "`gwz --root DIR status` in a directory without a manifest exits 1 with `ManifestNotFound`; the agent skill also names `WorkspaceNotFound`. Status fails for other reasons too ... which are not 'this is not a workspace'." This is F1's evidence verbatim (`GwzClaudeIntegration-S0.1-Review.md:55-65`). U11 line 179–181 now states the hook "never interprets its own error codes", D8 line 255–258 resolves the root in process, and D8 line 281–283 records the old external-script design as replaced. No remaining decision, step or test cites `ManifestNotFound` or `WorkspaceNotFound`.

**Impact:** S1.1's implementer, reading section 1 as binding facts, may implement code-set branching in the hook — reintroducing exactly the F1 failure mode D8 was simplified to eliminate.

**Required correction:** either delete lines 138–142, or retain them with an explicit closing clause such as "kept as background only; D8 supersedes it — the hook resolves in process and does not branch on error codes." While editing, correct the 2026-09-12 trail entry at line 575 ("F15 into D11"): F15's `jq` remedy now lives in D1's "no `jq` or `python3`" clause (line 198–199), and D11 contains no trace of it, so an auditor checking F15 against D11 finds nothing.

**Closure test:** grep the object for `ManifestNotFound`; every hit is either absent or inside an explicit supersession note. Grep for `F15`; the trail names D1.

### [P3-5] Section 1's lane count does not match the register it cites

**Location:** object lines 119–126; `dev-docs/GwzLaneIssues.md:20-36`.

**Root cause:** the sentence was written against `GwzLaneCleanFixes.md`'s 2026-09-16 snapshot (18 lanes) and was not re-checked against L1, which the same sentence cites and which has since grown.

**Violated invariant:** a claim attributed to a cited document must match that document at the reviewed baseline.

**Reproduction:** the object line 119–121 states "three on 2026-09-10, and **all 15 lanes** merged on 2026-09-15 and 2026-09-16 ... (gwz-dev `dev-docs/GwzLaneIssues.md`, L1)". L1's occurrence list at `GwzLaneIssues.md:24-36` enumerates rounds 1–9: 4 + 3 + 3 + 1 + 1 + 3 + 2 + 5 + 3 = **25** lanes on those two dates, not 15. The 15 figure reconciles with `GwzLaneCleanFixes.md:122` ("3, in 2 of 18 lanes", i.e. 3 + 15), which is a 2026-09-16 snapshot taken before rounds 8 and 9 were recorded.

**Impact:** the number is the plan's headline evidence that dispose refuses universally, and it is attributed to a document that now says something else. An auditor checking the citation finds a mismatch and must re-derive the claim; the understated count also weakens the D3/S3.4 case it exists to support.

**Required correction:** replace "15" with the count L1 actually carries at the cited baseline (25 on 2026-09-15/16, 28 including the three on 2026-09-10), or attribute the 18-lane figure to `GwzLaneCleanFixes.md:122` and its 2026-09-16 recount explicitly. Note while editing that L1's rounds 6–9 fall on 2026-09-16, the day `GwzLaneIssues.md:8` records gwz 1.0.13 as installed, so "with gwz 1.0.12" does not cover them all — the direction of the claim is still correct, since the register itself states 1.0.13 "changes nothing for L1 to L3".

**Closure test:** re-count L1's occurrence list and compare against the figure in section 1.

### [P3-6] The create timeout is assigned to S1.1 by S1.0 but is produced by S1.2

**Location:** object lines 332–334 (S1.0), 336–366 (S1.1), 369–371 (S1.2).

**Root cause:** a fold remnant — under the pre-fold D1, S1.1 owned both the scripts and the settings block, so "the creation timeout in S1.1" was true. The settings block moved to S1.2 at adoption; S1.0's pointer did not.

**Violated invariant:** a cross-reference must name the step that actually produces the artefact.

**Reproduction:** S1.0 line 332–334: "The numbers set D6's default and **the creation timeout in S1.1** (sized for two copies back to back)." S1.1's full text (336–366) never mentions a timeout — it cannot: the timeout is Claude Code's per-hook `timeout` field, a settings value. S1.2 line 369–371 correctly claims it: "the hooks block (`WorktreeCreate` and `WorktreeRemove`, each one command handler, **the create timeout from S1.0**)."

**Impact:** an implementer of S1.1 is handed a number with nothing to set it on; the S1.0 operator hands the measurement to the wrong step. Minor but concrete, and it is the residue the fold was expected to leave.

**Required correction:** in S1.0 line 333, change "the creation timeout in S1.1" to "the creation timeout emitted by S1.2's settings block".

**Closure test:** the only step naming the create timeout as its own output is S1.2.

### [P3-7] D6's and O3's environment-variable knobs are inoperative on the desktop app, by section 1's own fact

**Location:** object lines 62–65 (section 1), 244–248 (D6), 298–303 (D10), 551–553 (O3).

**Root cause:** F10's cure (a fixed log file) was applied to D10's logging only; the same constraint applies to every other environment-variable knob the plan introduces, and the plan does not say so.

**Violated invariant:** a configuration mechanism must work on the surfaces the plan targets, or the limitation must be stated where the mechanism is offered.

**Reproduction:** section 1 line 62–65: "On macOS the desktop app reads `PATH` from the shell profile but not other exported variables, so a hook launched by the desktop app **cannot rely on any environment variable of ours**." D10 line 301–303 acknowledges this for logging and fixes it with a fixed file path. But D6 line 245–246 makes the free-space threshold `GWZ_LANE_MIN_FREE_GB`, and O3 line 552–553 offers `GWZ_LANE_BASE_REF` as "the cheap override if anyone needs it" — both environment variables only, with no fixed-file or flag alternative, and neither carries the caveat.

**Impact:** the compiled-in defaults still apply, so nothing breaks silently; but a desktop-app user who trips D6's guard has no documented way to raise it, and the fallback's `baseRef` override that O3 proposes cannot be used on the surface (other, non-GWZ projects opened in the desktop app) it is proposed for. S4.1, which will document both, would publish an override that does not work there. S1.1's test "the guard with a forced threshold" is unaffected — an in-process Rust test can set the variable freely.

**Required correction:** add one clause to D6 and one to O3 recording that the variable is a terminal-session and test-time override only, and name the fallback for the desktop surface (a setting read from the workspace, a `gwz claude-code setup` flag baked into the handler string, or an explicit "the compiled default is the only value the desktop app sees").

**Closure test:** every environment variable the plan introduces (`GWZ_LANE_MIN_FREE_GB`, `GWZ_LANE_HOOK_LOG`, `GWZ_LANE_BASE_REF`) carries, at the point it is introduced, a statement of whether it reaches the desktop app.

## 2. Invariant analysis

Invariants tested, and what held:

**The D1/D8 fold left no script-era text.** Held. A targeted grep for `script`, `jq`, `python3`, `stub`, `.ps1`, `integrations/`, `copy step`, `1800` and `--json status` returns hits only inside the two explicit historical notes (lines 202–205, 281–283), D1's consequence list (198–200), U11's closing clause (181), and the trail entries. Phase 5 correctly reads as verification of one binary through `powershell.exe`, not a PowerShell twin (517–527), matching the trail's claim at line 592. S1.1's tests are in-process Rust against a fixture workspace with no stub `gwz` (354–366), consistent with D1. This was the primary predicted failure mode and the fold survived it; the only residue is P3-4's dead fact.

**Section 1's Claude Code facts versus D1/D8/S1.1.** Held, with one gap already noted. The stdout contract is honoured: S1.1 line 352–353 makes stdout discipline "the contract ... nothing but the path is ever written there", and progress goes to stderr (346–347). Canonicalisation is present — S1.1 prints "the canonical lane path" and tests "lane path printed and canonical" (347, 357) — which answers the symlink-refusal rule at line 47–48; the lane is a sibling of the workspace root (`../<root-dirname>-NAME`, D2 line 207–208), so it is not "below the repository root" and the symlink rule is not engaged for it. The outside-any-repository rule at 49–52 is honoured twice over: D2's walk-up parent check (208–214) for the lane, and D8's reliance on the "a directory that is its own repository is accepted" clause (line 51–52, restated at 91–94) for both the fallback's `git worktree add` output and the member-path-inside-a-lane case (261–262). The 600-second default timeout (64–65) is superseded by S1.0's measured value, correctly. The desktop-`PATH` fact (60–62) drives D1's `--command PATH` escape and S1.2's confirmation. The only unhonoured fact is the dedupe rule, which D1 satisfies and D5 then contradicts — P2-1.

**Every R-number, L-number and F-number cited.** Held. See section 0. The R citations are accurate down to the sub-clause (R11's "no broader than its category", R12's check-only mode, R9/R10's category reporting and exact waiver command, R8 as the basis for O5's `.claude/` question, and R14–R16 as the clean-lane set); R0/R0.1's split is used correctly by D3 and S3.5. L1 supports the "1.0.13 changes nothing" claim. All eighteen F-numbers map to real findings and all eighteen folds are locatable in the object — except the F15 pointer, which now points at the wrong decision (folded into P3-4's correction). The one factual divergence is the lane count, P3-5.

**The plan rule: milestones, one goal per step, <500 LOC aspirational, foundational first, parallel-friendly.** Partially held. Phase milestones are stated and are genuine shippable increments. Foundational ordering is correct — S1.0 and S1.1 precede everything, and the plan says so at 315–317. Parallel-friendliness is asserted and mostly earned (537–541). S1.1 is a single coherent goal (the hook family, both directions, including D8's fallback), but its stated budget of ~350 + ~250 = ~600 lines already exceeds the aspirational 500 before P2-2's mandatory reference regeneration is counted, and the enumerated surface (stdin parsing, name validation, in-process root resolution, in-process clone, two `cfg_if!`-bounded platform helpers, dual-location logging with `HOME` resolution, `git worktree add` with `origin/<default>` resolution, `git worktree remove`, and four-way `worktree_path` classification) against fourteen enumerated test cases makes ~600 optimistic rather than aspirational. Since line 316 explicitly declares budgets "aspirational targets, not limits", I do not raise this as a finding on its own; it is the reason P2-2's correction should consider a split rather than a budget bump. S1.2 does violate the one-goal rule outright — P3-1.

**The step dependency sketch against the step texts.** Failed in one direction — P3-2. In the other direction the sketch is loose but not wrong: line 533 shows `{ S2.1, S2.2, S2.3, S3.1, S3.2, S3.4 } -> S3.3`, while S3.3's own text (463) justifies only `S3.2` and `S2.3`; the brace notation reads as "after the phase's evidence", which the prose at 537–540 supports. Line 534's S3.5 entry, including the external `GwzLaneCleanFixes R0 in an installed gwz` precondition, is exactly consistent with S3.5's text (483–489) and with D3's staging. Line 540's "nothing else waits on S3.5" is consistent with S4.2's forward reference (511), since that is a documented rule awaiting revision, not a blocked step.

**Test and evidence sections satisfiable as written.** Held. S1.1's fourteen cases are all constructible in a temporary fixture workspace: the forced free-space threshold via `GWZ_LANE_MIN_FREE_GB` (an in-process test can set it, unlike the desktop app — P3-7), the open-coordinated-merge fixture via a workspace left mid-merge (`LocalClones.md:148-154` documents the refusal and its typed error), the `origin/<default>` and `HEAD` branch cases via a local bare remote, the member-rooted `CLAUDE_PROJECT_DIR` case via a fixture with one member, and the clean-versus-dirty dispose exit codes via a minimal fixture that carries none of the inherited hazards L1 describes for gwz-dev. Nothing in S1.3 is unobservable: the `gwz`-hidden-from-`PATH` probe (407–410) is a terminal-session probe and a modified `PATH` is available there; the exit-zero-with-directory case is S3.1's, not S1.3's. The one mis-scoped observation is S1.3's request for the desktop's diff and PR views (388–389), noted under P3-1.

**Unstated impacts on uncited documents.** Failed — P2-2, verified against gwz-cli source rather than inferred. The remaining uncited surfaces are benign: the gwz skill is owned by S4.2 (509–511) and `AGENTS_GWZ.md` by S4.3 (512–515); `docs/AgentBootstrap.md` and the mkdocs nav are named by S4.1 (493–494). `docs/LocalClones.md` needs no edit from this plan.

**O1..O5 against the steps.** Held except O2 (P3-3). O1→S4.1, O3→S4.1's "`baseRef` parity for the fallback" (503–504), O4→S1.3's explicit check (405–406) and D2's walk-up (208–214), O5→S3.5's explicit record (487–488) and R8.

## 3. Risks and next action

The 2026-09-17 fold is, on the consistency axis, substantially cleaner than its scale would predict: the script-to-subcommand rewrite left no orphaned mechanism, no stub-`gwz` test remnant, and no "gwz missing" branch. Both blocking findings are of one kind — a clause whose supporting premise moved out from under it (P2-1) and an obligation that only exists because the deliverable changed shape from shell scripts to clap subcommands (P2-2). Neither requires reopening a deferred decision.

The residual risk after those two are fixed is concentrated in Phase 1's shape rather than its content: S1.2 currently bundles a code deliverable, a configuration write and a live desktop-app lane creation (P3-1), and the sketch that agents will schedule from does not carry S1.4's release gate (P3-2). Both are cheap to fix now and expensive to discover when an agent has already committed a settings block naming a command no release carries.

Recommended next action: the drafter revises for {P2-1, P2-2} and, since each is a few lines, folds {P3-1 … P3-7} in the same pass. P2-2 should be resolved by reading `gwz-cli/src/tests/g00.rs:36-43`, `src/help.rs:11-17` and `scripts/release.py:407-416` directly, so the revised S1.1 names the artefacts rather than gesturing at them. On a revision that resolves P2-1 and P2-2 as specified, I pre-commit to GO.
