# GWZ Log S3.5-B independent review

- Date: 2026-08-31
- Review round: 1 of 2
- Re-plan authority:
  `ba3d1fc23383ca51afb1a35611e58603c5d4a6d9`
- Exact `gwz-py` base / sole parent:
  `f827e30fd28c7322c9d70883b8a6d9a873bbfd0a`
- Exact `gwz-py` candidate:
  `12e3f5f1c928461e65c9bb6329c5b4e80c5269bf`
- Exact sibling `gwz-core`:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Scope: the S3.5-B residue only — all 13 named global singletons,
  before/after/split placement, truly repeatable selector aliases,
  no-abbreviation preservation, 200-LOC cap, and exact S3.5 base integrity

## Verdict: NO-GO

The candidate's production behavior is correct on the amended surface. Every
named global singleton is rejected when repeated before `log`, after `log`, or
split around it; inline `--option=value` spellings are also handled. Repeated
target/member/path selector aliases remain repeatable, singleton-looking
pathspecs after `--` remain pathspecs, long abbreviations remain disabled, and
the scanner is confined to `log`.

The checked-in acceptance matrix is not mutation-tight at three boundaries of
that raw-argv scanner. It uses only separated `[option, value]` spellings,
only invokes `log`, and never places singleton-looking literals after `--`.
Consequently three wrong implementations leave all 116 focused log/shared
parser tests green: ignoring inline `=`, scanning through the pathspec
separator, and applying the new rejection to every command. Each violates the
explicit S3.5-B charter even though the reviewed production lines are right.

Finding count: **0 P0 / 0 P1 / 1 P2 / 0 P3**.

## Finding

### S3.5-B-F1 — P2 — Raw-argv boundary mutants survive the acceptance matrix

`reject_repeated_log_global_singletons` correctly identifies the top-level
`log` command, stops before the post-command `--`, normalizes an inline token
with `token.partition("=")[0]`, and rejects a second occurrence from the
13-element singleton set. The table in `src/tests/test_cli_log.py`, however,
constructs every value option only as two argv elements, and its positive
cases remain inside the `log` command's option region.

Three exact mutants independently survive both `test_cli_log.py` and the
shared `test_cli_parser.py`: **116/116 passed** under each mutant.

| Exact mutant | Wrong behavior admitted by the mutant |
|---|---|
| Replace `option = token.partition("=")[0]` with `option = token` | All 21 inline duplicate placements for the seven value-taking singletons are accepted with last-value behavior. |
| Replace separator-bounded `end` with `len(raw_args)` | `log -- --root --root`, `log -- --json --json`, and `log HEAD -- --jobs --jobs` are falsely rejected as options instead of preserved as pathspecs. |
| Remove the `raw_args[command_index] != "log"` guard | Repeated globals on `status`, `diff`, and `ls` are newly rejected, contrary to the re-plan's explicit no-other-command-behavior boundary. |

Direct probes against the unmodified candidate prove these are acceptance
gaps, not current production failures. Inline and mixed `=` repetitions are
rejected with 2 in every placement; repeated singleton-looking values after
`--` remain exact pathspecs; and the existing non-`log` parser behavior is
unchanged.

Required remediation is tests-only unless a new regression disproves the
current code:

1. cover all seven value-taking singleton options with inline/mixed `=`
   duplicates before, after, and split around `log`;
2. preserve representative repeated singleton-looking pathspecs after `--`,
   including a case with a real singleton occurrence before the separator;
3. preserve representative non-`log` duplicate behavior before, after, and
   split around that command; and
4. prove the three exact mutants above are red while keeping the cumulative
   S3.5-B delta at or below 200 changed handwritten LOC.

## Charter matrix

| Surface | Result | Evidence |
|---|---|---|
| 13 global nonrepeatables | **PASS production / FAIL acceptance** | The exact 13-item set is complete. The checked-in 13 x 3 matrix rejects 39/39 separated duplicates with exit 2, and removing any one set member fails its exact three placements. F1 records the surviving inline-token mutant. |
| Before / after / split placement | **PASS** | Losing the pre-command side fails 26/39 cases; losing the post-command side fails 26/39; clearing state at `log` fails 13/39. |
| Repeatable selector aliases | **PASS** | `--target`, `--member`, `--no-target`, `--no-member`, `--member-path`, and `--no-member-path` preserve both values in all three placements. Adding `--target` or `--member` to the singleton set makes its three cases red. |
| No long abbreviations | **PASS / base-preserved** | `allow_abbrev=False` and its five focused regressions are byte-unchanged from `f827e30`; restoring abbreviation makes all five red. |
| `--` pathspec boundary | **PASS production / FAIL acceptance** | Production stops scanning at `--`; the boundary-blind mutant remains green per F1. |
| `log`-only scope | **PASS production / FAIL acceptance** | Production guards the exact command; removing the guard remains green per F1. |
| S3.5 F1/F2/F4/F5 cures | **PASS integrity** | `cli.py`, `cli_log.py`, `client.py`, `bridge.py`, the native source tree, and the focused protected tests are exact base blobs/trees. They are not regraded. |
| Hard cap | **PASS** | Delta is +106/-0 handwritten LOC: +36 production and +70 tests, 94 lines below the cap. |
| Protocol / renderer / lifecycle scope | **PASS** | No generated protocol, schema, native, bridge, client, renderer, manifest, lock, output lifecycle, request lowering, or documentation byte changes. |

## Mutation audit

The primary table is strong on its literal axes:

- remove any one of the 13 singleton set entries — 3 failed / 36 passed;
- lose the pre-command scan — 26 failed / 13 passed;
- lose the post-command scan — 26 failed / 13 passed;
- clear seen state at the command boundary — 13 failed / 26 passed;
- classify `--target` or `--member` as a singleton — 3 failed / 15 passed;
- restore long abbreviation — 5 failed / 0 passed.

The three F1 mutants each remain 116/116 green. That distinction is why the
verdict is an acceptance NO-GO rather than a production-correct GO.

## Scope and integrity

The exact candidate is one clean commit whose sole parent is the chartered
`f827e30` base. It is non-shallow, has no replacement refs, carries no commit
trailers, and passes the diff check. The delta changes only two files, with no
rename or mode change:

```text
src/gwz/cli_shared.py     +36/-0
src/tests/test_cli_log.py +70/-0
total                    +106/-0
```

Both diffs are additions-only. Removing precisely those additions reproduces
the base files byte-for-byte. Protected base identities include:

```text
cli.py                 838be8340a4c578e340fec10ea25810cadba60fe
cli_log.py             776e13fa2da42da960c695f114af21cc569e01a7
client.py              407d906da6b0988044f6150148748723d8e4d080
bridge.py              0a68ec97caf2a969838df4a1bdd468af2bd0146a
native/src tree        2a94024ccd7a785aa5d1527312dab36d1341f6bc
generated protocol     251da68c56f8feb027ce51e67cde97a37d1d5f8a
renderer-parts tree    21d314c7da95b849be3ad78038a429e015f49ac4
```

A temp-index replay of the exact binary patch on the stated base reproduces
the candidate tree.

```text
base tree:            69d96c9ddf355246087de4aeba4e55ad08f7319e
candidate tree:       ece6d6e08cfdedc0848dd3b197611b0735775a4a
binary diff SHA-256:  fc1a0ada7f20ce6adf18e0fdb3d415a31f00f26937789ea6cdc3f541b7b9e44f
stable patch id:      6906407a2228121a2610e785f8f901352b610bac
format-patch SHA-256: f3c4908e8d3d03ca00192ac763b25746843234dbafd22a932f34fde4fc0bb408
```

The exact candidate and sibling core worktrees remained clean. The report
branch contains only this review file, and the Python candidate was read-only
throughout review.

## Proportional evidence

The fast-test instruction was followed. No broad Python suite, core suite, or
compiler matrix was run.

- Exact amended charter cases: 62/62 passed.
- Complete focused log parser file: 97/97 passed.
- Focused log/shared parser combination used for mutants: 116/116 baseline.
- Focused parser/log/protocol/codec/native-bridge set: 107 passed / 3 skipped.
- Direct extended matrices: 102/102 singleton space/inline/mixed cases
  rejected; 57/57 selector/alias cases accepted; 69/69 abbreviations rejected;
  5/5 separator cases accepted.
- Protocol drift check: exit 0, fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`.
- Protocol regeneration check: exit 0.
- Candidate identity, cap, replay, protected-blob, and cleanliness checks:
  green.

## Round-2 acceptance gate

Round 2 is final under the fresh S3.5-B cap. Restrict it to S3.5-B-F1 and
integrity. Accept only if:

1. compact checked-in regressions cover inline/mixed `=` duplicates for all
   seven value-taking singleton options in all three placements;
2. singleton-looking repeated pathspecs remain untouched after `--`;
3. the scanner's `log`-only command boundary is pinned without changing
   other commands;
4. all three F1 mutants are red; and
5. cumulative changed LOC remains at or below 200, the protected S3.5 base
   bytes remain exact, and no new production or scope change rides the cure.

If this final round fails, S3.5-B freezes and returns to the operator with no
further remediation round.

## Terminal round-2 review

- Date: 2026-08-31
- Exact final `gwz-py` candidate:
  `dc6915545b8c65d01cebc02ba1c3c7f1df9a5f8b`
- Exact base / sole parent:
  `f827e30fd28c7322c9d70883b8a6d9a873bbfd0a`
- Round-1 comparison candidate:
  `12e3f5f1c928461e65c9bb6329c5b4e80c5269bf`
- Scope: S3.5-B-F1 and terminal integrity only

### Terminal verdict: GO

The round-1 P2 is fully cured. The amendment adds only the missing acceptance
tests; production is byte-identical to the already-reviewed round-1
implementation. Inline and mixed value spellings, the post-`--` pathspec
boundary, and the `log`-only command guard are now directly pinned. All three
exact round-1 mutants are red.

No P0, P1, P2, or P3 finding remains in the bounded terminal regrade. The
exact candidate above is approved for landing without conditions.

### Round-1 finding regrade

| Round-1 gate | Terminal result | Exact evidence |
|---|---|---|
| Inline/mixed `=` duplicates | **CURED** | Seven value-taking singletons x two spellings x three placements produce 42 checked-in cases. Replacing `token.partition("=")[0]` with `token` makes all 42 fail. |
| Post-`--` pathspec boundary | **CURED** | Three repeated singleton-looking pathspec cases plus one real-option-before-separator sentinel pass. Scanning to `len(raw_args)` makes all four fail. |
| `log`-only command scope | **CURED** | Status, diff, and ls preserve their existing duplicate behavior across the representative placements. Removing the command guard makes all three fail. |
| Cumulative hard cap | **PASS** | +176/-0 handwritten LOC, leaving 24 lines of headroom under 200. |
| Base F1/F2/F4/F5 integrity | **PASS** | Protected production, tests, native, protocol, and renderer blobs/trees are exact against `f827e30`; none is regraded. |

The exact four remediation tests contain 49 parameterized cases and pass
49/49. Running each mutant against that same gate produces:

- inline-token mutant — 42 failed / 7 passed;
- scan-through-`--` mutant — 4 failed / 45 passed;
- all-commands mutant — 3 failed / 46 passed.

### Final integrity and scope

The final candidate remains one clean commit with the required sole parent.
It is a sibling rewrite of the round-1 candidate: round 2 changes only
`src/tests/test_cli_log.py` by +70/-0. The reviewed production blob
`src/gwz/cli_shared.py` is identical at both candidates:
`1891260597af11330a22e3a2f9595b15f3b5a505`.

The cumulative base-to-final delta is additions-only:

```text
src/gwz/cli_shared.py     +36/-0
src/tests/test_cli_log.py +140/-0
total                    +176/-0
```

No other Python source, native code, generated protocol, renderer, manifest,
lock, documentation, mode, or path changes. The protected S3.5 blobs and
trees listed in round 1 remain exact, including `cli.py`, `cli_log.py`,
`client.py`, `bridge.py`, native source, generated protocol, and renderer
parts. F1/F2/F4/F5 remain base evidence, not a new review surface.

Temp-index binary replays from both the charter base and the round-1 candidate
reproduce the final tree exactly.

```text
final tree:                     33a0225fb91059a0fcb2b5cf2e810215122ffd13
cumulative binary diff SHA-256: d07bfab2abe4bf4739cea6f726d91622a51fe809a0987cb90fba156745f20b50
cumulative stable patch id:     04550e001f85a7441c36d90ea13a80287373df3d
cumulative format-patch SHA-256: f5254ef73879135aabca7d9df21144ca99bdc1a1ee2149f4448d962855414dd3
round-2 binary diff SHA-256:     1034d7ec38eef4ffd5c844e7cf581b79046b757b2d9cd59beede1d6b9b2e3aed
round-2 stable patch id:         9d89d4d3ebe2eca9e12f576da55d206d505287c3
```

The repository is non-shallow, has no replacement refs, and passes the diff
check. Candidate, sibling core, and report worktrees are clean after their
authorized operations. The Python candidate remained read-only.

### Proportional terminal evidence

The fast-test boundary was preserved. No broad Python suite, core suite, or
compiler matrix was run.

- Exact remediation cases: 49/49 passed.
- Focused parser regrade: 118 passed / 28 deselected.
- Focused parser/log/shared/protocol/codec/native-bridge set:
  175 passed / 3 skipped.
- All three exact round-1 mutants: red with the counts above.
- Protocol drift check: exit 0, fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`.
- Protocol regeneration check: exit 0.
- Cap, production-byte identity, protected-blob comparison, two-source replay,
  topology, and cleanliness: green.

### Landing identity

Land exactly `dc6915545b8c65d01cebc02ba1c3c7f1df9a5f8b` as the one-commit child
of `f827e30fd28c7322c9d70883b8a6d9a873bbfd0a`. No substitute tree or
additional amendment was reviewed.
