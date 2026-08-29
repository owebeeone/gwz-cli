# GWZ Log S2.4 blind review

**Overall: NO-GO.** Round 1; no CURED findings.

## Findings

1. **[P1] Malformed marker values are accepted as authoritative marker identities.**
   [coalesce.rs:193](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce.rs:193), especially lines 204–209, accepts every nonempty UTF-8 trailer value as `Marker(String)`. It never applies the repository’s canonical UUIDv7 contract from [artifact/mod.rs:605](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/artifact/mod.rs:605).

   Exact-source probes demonstrated:

   ```text
   GWZ-Commit-ID: not-a-uuid
   => one cross-repo group, Marker("not-a-uuid")
   ```

   and:

   ```text
   GWZ-Commit-ID:
   GWZ-Workspace-ID: ws_test
   => Marker("\nGWZ-Workspace-ID: ws_test")
   ```

   These pairs had different messages, authors, and timestamps, proving the invalid value bypasses every heuristic safeguard and falsely asserts proven identity.

   **Remedy:** accept `Marker` only for a canonical UUIDv7 value, preferably through a private validated marker-ID type. Empty, continued, non-UTF-8, noncanonical, duplicate-ambiguous, or otherwise malformed values must become `Unusable`, never `Marker` or `Unmarked`.

2. **[P1] The trailer parser misses valid markers emitted by the shipped producer after a `---` line.**
   Production unconditionally appends marker lines after arbitrary user text at [handle_commit.rs:246](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/workspace_ops/handle_commit.rs:246). However, `git2::message_trailers_bytes` at [coalesce.rs:193](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce.rs:193) treats `---` as a patch boundary and ignores everything after it.

   A production-shaped message therefore produced:

   ```text
   subject

   ---
   notes

   GWZ-Commit-ID: 01987b0c-2f75-7c4a-9a32-8fd22f7d7c91
   GWZ-Workspace-ID: ws_test
   => Heuristic
   ```

   A slow coordinated operation could instead split completely. This violates L-COA-1 and L-COA-6 for commits created by existing shipped machinery.

   **Remedy:** within the private S2.4 seam, extract and validate the terminal GWZ trailer block without inheriting libgit2’s patch cutoff. Add a production-message fixture containing a `---` line.

3. **[P2] Rejected marker-shaped blocks are downgraded to genuinely unmarked input.**
   When libgit2 returns no parsed values, lines 201–202 return `Unmarked`. For example:

   ```text
   GWZ-Commit-ID: 01987b0c-2f75-7c4a-9a32-8fd22f7d7c91
   GWZ-Commit-ID nope
   ```

   yields no libgit2 trailers; identical cross-repo copies then merge as `Heuristic`. A missing-colon `GWZ-Commit-ID <uuid>` behaves likewise. Thus malformed marker-bearing commits enter the heuristic, contrary to L-COA-2’s fail-closed rule. Properly parsed conflicting duplicate values are correctly made opaque.

   **Remedy:** distinguish raw absence from a malformed/ambiguous GWZ marker claim. Only true absence may return `Unmarked`; any detected malformed claim must return `Unusable`.

4. **[P2] L-COA-1 is not exercised through real Git history.**
   The test named `l_coa_1_real_trailer_siblings_group_across_repositories` at [coalesce_tests.rs:12](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce_tests.rs:12) constructs direct `CommitLogEntry` values using the byte helper at line 249. It never opens repositories, consumes S2.1 cursors, or uses the shipped marker producer.

   I inspected a genuine shared-history pair—core `7d1a3e6a…` and CLI `3cca145c…`, marker `019ff193-31e7-742f-9ec9-f3ec2b7e0654`—but no automated candidate fixture exercises that seam.

   **Remedy:** create actual sibling commits through the production marker path, read them through S2.1, then assemble and assert one marker-provenance group. Include different bodies/authors/timestamps so the test cannot pass through heuristic equivalence.

## Canonical-row matrix

| Row | Verdict | Evidence |
|---|---|---|
| L-COA-1 | **FAIL** | Ordinary terminal UUID trailers group correctly, but production `---` messages lose marker identity; real-history coverage is absent. |
| L-COA-2 | **FAIL** | The four-conjunct algorithm itself is correct, including group-wide max/min checks and rebase re-stamp protection. Malformed marker claims can nevertheless enter it as `Unmarked`. |
| L-COA-3 | **PASS** | `coalesce=false` returns unchanged singleton entries with `None` provenance. |
| L-COA-4 | **PASS** | Lines 173–183 use the maximum sibling committer timestamp. Final tie-breaking remains available to S2.5 through member IDs and hashes. |
| L-COA-6 | **FAIL** | Valid groups produce `None`/`Heuristic`/`Marker`, but invalid strings can become marker provenance and valid hidden markers can be mislabeled heuristic. |
| L-COA-7 | **PASS** | Stateless assembly exposes `W=60`; marker fragments retain their marker key. Buffering, cursor advancement, closure, ordering, and emission remain outside this module. |

The blank Q-1 resolution leaves the standing rule active; the qualifying three-repository fan-out correctly coalesces as `Heuristic`.

## Algorithm and raw-byte audit

- Byte-identical message, author name/email, both ≤10-second spans, and distinct repository IDs are all enforced.
- The max/min calculation prevents transitive `0/10/20` collapse.
- Same-repository entries never merge, including marker groups.
- Distinct parsed marker values never merge.
- Parsed conflicting trailers fail closed.
- The rebase re-stamp fixture correctly rejects close committer dates with distant author dates.
- Actual raw Git `0xff` message bytes survive S2.1 unchanged, and a valid terminal marker remains extractable.
- First-fit choice is input-order dependent for overlapping valid cliques, but the reviewed rows provide no alternate partition rule; no finding filed.

## API/state ownership

**PASS.** `assemble_commit_log_groups` is a finite, per-call transformation with no retained state. It owns neither cursors nor the W=60 buffer, closure decisions, output ordering, or emission. Its memory is proportional to the candidate set S2.5 admits. The visibility remains private to `operation::commit_log`.

## Scope and gates

- Candidate `f535d93c…` directly parents baseline `14bd5acf…`.
- Diff: 505 insertions across only `coalesce.rs`, `coalesce_tests.rs`, and the private module declaration.
- No library export, handler behavior, protocol/generated surface, CLI, inventory, pin, lockfile, dependency, or `gwz.conf` changes.
- Worktree remained clean.

Gates:

- `cargo fmt --check` — pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `cargo test --lib operation::commit_log::` — pass, 25/25.
- Focused S2.4 tests — pass, 13/13.
- `git diff --check baseline..candidate` — pass.
- An exploratory full `cargo test --all-features` was stopped after roughly 15 minutes in unrelated long-running matrices; no failure had appeared, but no full-suite pass is claimed.
- Exact-source negative probes reproduced Findings 1–3.

**Final verdict: NO-GO due to P1/P2 findings.**

## Round 2 final peer-blind re-review

**Baseline:** `14bd5acf01485a6a72922ff7527d9275f0877869`
**Candidate:** `a2ab729f95f334c13dcd2bc1c0809152976bef48`
**Verdict:** **NO-GO — terminal under the two-round cap.**

The current Q-1 `Resolution (Gianni):` remains blank, so qualifying same-message fan-outs continue to coalesce as `heuristic`.

### Prior finding disposition

1. **[P1] UNCURED — malformed marker values can still establish authoritative identity.**

   Most filed examples are cured: arbitrary strings, uppercase, v4, empty, continued, non-UTF-8, and duplicate values now become opaque. However, [canonical_uuid_v7](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce.rs:251) validates the version nibble but not the RFC UUID variant nibble at byte 19.

   Exact-candidate probe:

   ```text
   GWZ-Commit-ID: 01987b0c-2f75-7c4a-1a32-8fd22f7d7c91
   different bodies/authors/timestamps across two repos
   => groups=1, provenance=Marker("01987b0c-2f75-7c4a-1a32-8fd22f7d7c91")
   ```

   The fourth UUID group begins with `1`, not the required RFC variant `[89ab]`; this is not a UUIDv7 authority. It therefore retains the original false-fusion defect.

   **Required remedy:** require `matches!(value[19], b'8' | b'9' | b'a' | b'b')` and add wrong-variant fixtures.

2. **[P1] CURED — shipped markers after `---` are recognized.**

   The generic Git trailer parser is gone. The terminal-block extraction at [coalesce.rs:206](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce.rs:206) correctly recognizes the shipped marker after arbitrary user text containing `---`, both with and without the optional origin hash.

   Exact-source probe now returns:

   ```text
   production marker after patch divider:
   groups=1, provenance=Marker("01987b0c-2f75-7c4a-9a32-8fd22f7d7c91")
   ```

3. **[P2] UNCURED — some malformed marker-shaped claims still enter the heuristic.**

   The filed missing-colon and malformed-adjacent examples are cured. However, [marker_shaped_claim](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce.rs:243) recognizes only end-of-line, colon, space, or tab after the key. A conventional malformed assignment using `=` is treated as having no marker claim and reaches `Unmarked`.

   Exact-candidate probe:

   ```text
   GWZ-Commit-ID=01987b0c-2f75-7c4a-9a32-8fd22f7d7c91
   => groups=1, provenance=Heuristic
   ```

   This violates the rule that malformed marker claims fail closed and never enter heuristic grouping.

   **Required remedy:** recognize `=` and other unambiguous key/value delimiters as marker-shaped malformed claims, then return `Unusable`. Add an explicit `GWZ-Commit-ID=<uuid>` fixture.

4. **[P2] CURED — the real Git history/S2.1 integration fixture now exists.**

   [coalesce_tests.rs:41](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce_tests.rs:41) now exercises:

   `workspace_ops::handle_commit` → stored root/member Git objects → rewritten heuristic-ineligible sibling → S2.1 cursors → one marker-provenance group.

   It includes `---`, the optional origin hash path, different message and author bytes, and an exact 60-second committer spread.

### Additional requested gap checks

- **CURED:** same author name with different email is isolated at [coalesce_tests.rs:252](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce_tests.rs:252).
- **CURED:** marker authority over heuristic-ineligible siblings at exactly W=60 is covered at [coalesce_tests.rs:397](/Users/owebeeone/limbo/gwz-log-worktrees/s2.4/gwz-core/src/operation/commit_log/coalesce_tests.rs:397).
- Non-UTF-8 body/message comparisons remain byte-exact.
- Parsed duplicate, continued, nonterminal, and blank-separated claims fail closed.
- A boundary probe placing an otherwise valid marker block before another blank paragraph produced two `None` singletons, confirming the parser does not cross the last blank boundary.
- Ordinary prose containing the token away from a marker-shaped line cannot acquire marker authority.
- Same-repository exclusion, distinct-marker separation, pairwise 10-second spans, rebase re-stamp protection, no-coalesce singletons, maximum committer timestamp, and valid provenance remain correct.
- No new findings beyond the residual instances of Findings 1 and 3.

### Canonical-row matrix

| Row | Round-two result | Evidence |
|---|---|---|
| L-COA-1 | **FAIL** | Shipped terminal markers and real-history grouping are cured, but a non-RFC-variant value can still establish marker authority. |
| L-COA-2 | **FAIL** | The four-conjunct heuristic remains correct, but an equals-delimited malformed marker claim enters it. |
| L-COA-3 | **PASS** | Raw singleton groups with `none` provenance remain intact. |
| L-COA-4 | **PASS** | Ordering timestamp remains the maximum sibling committer timestamp. |
| L-COA-6 | **FAIL** | Invalid UUID authority can emit marker provenance; malformed `=` claims can emit heuristic provenance. |
| L-COA-7 | **PASS** | W=60 is exposed without acquiring cursor, buffer, closure, ordering, or emission state. |

### API and scope

The assembly API remains private and stateless. S2.5 still owns the W=60 buffer, live cursors, group closure, ordering, and emission.

The candidate is one squashed commit directly over the baseline. The complete diff changes only:

- `src/operation/commit_log/coalesce.rs`
- `src/operation/commit_log/coalesce_tests.rs`
- the private module/test declarations in `src/operation/commit_log/mod.rs`

No `lib.rs`, handler, protocol/generated surface, CLI, inventory, pin, dependency lockfile, or `gwz.conf` change occurred. The worktree is clean.

### Command and direct-exit evidence

- `cargo test --lib operation::commit_log::coalesce_tests -- --nocapture` — exit `0`, 19/19 passed.
- `cargo test --lib operation::commit_log::` — exit `0`, 31/31 passed.
- `cargo fmt --check` — exit `0`.
- `cargo clippy --all-targets --all-features -- -D warnings` — exit `0`.
- `git diff --check 14bd5acf…a2ab729f` — exit `0`.
- Forbidden-surface `git diff --quiet` checks — exit `0`.
- Candidate/worktree equality and porcelain status — exit `0`, empty.
- Exact-source wrong-variant probe — exit `0`, reproduced invalid `Marker(...)` fusion.
- Exact-source equals-delimiter probe — exit `0`, reproduced `Heuristic` fusion.
- Exact-source blank/nonterminal boundary probe — exit `0`, correctly produced `None` singletons.

**Final: NO-GO. Findings 1 and 3 remain UNCURED at P1/P2, making the result terminal under the two-round cap.**
