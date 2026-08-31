# `gwz log` S3.4 independent review

- Date: 2026-08-31
- Review round: 1 of 2
- Exact `gwz-py` base / sole parent:
  `0bc58a887a9548dbe8b2d2608c8d0ba15915b0ee`
- Exact `gwz-py` candidate:
  `8d717e329ae007358c71e0545ba97fb6d4ca7134`
- Candidate tree: `73d7ee676157874b6bbbd41594431e92efbc9e1d`
- Exact Rust CLI authority:
  `6266ba2bff28bf631d9eb1efa69dcf2c220f8944`
- Exact sibling `gwz-core`:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Scope: S3.4's two-client real-workspace battery, parity and determinism,
  mutation tightness, the narrow Python stdout-EPIPE cure, and package
  integrity only

## Verdict: NO-GO

The production correction is contract-correct and tightly scoped. The
existing stdout silencer was moved byte-for-byte to the shared CLI module and
is now called at both successful log-output BrokenPipe boundaries. Prefix
failure releases directly; later failure closes the output generator; every
successful stdout EPIPE returns 0 without diagnostic spray in deterministic
reviewer probes. Stderr and ordinary write failures still return 1 in the
reviewed implementation. No core, Rust CLI, protocol, renderer, native,
dependency, or unrelated-command behavior moved.

The tests-only acceptance package is not yet mutation-tight. Its central
parity oracle can silently invoke Python twice, three required depth-lift
forms are absent, several exact S3.4 semantic assertions permit wrong output,
the heuristic matrix omits four conjunct boundaries, and the human body-C0
arm is accidentally parsed as pathspecs. The new real EPIPE test is also
race-dependent and does not distinguish every new silencing call; adjacent
tests allow an ordinary streaming `OSError` to be reclassified as clean
termination.

Finding count: **0 P0 / 0 P1 / 7 P2 / 2 P3**. No current production defect
was found; every P2 is an acceptance blocker under the S3.4 charter.

## Findings

### S3.4-F1 — P2 — The two-client parity oracle can collapse to one client

[`_parity`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.4/gwz-py/src/tests/test_log_real_workspace.py:199) calls the two intended
commands at lines 205–206 and compares exit code, stdout, and stderr. Nothing
proves that the two returned processes came from distinct commands.

The exact reviewer mutation changed only the first call from
`workspace.run_rust(...)` to `workspace.run_python(...)`. The complete
26-case real-workspace module stayed green. Fixture setup still used the Rust
CLI, and a few bespoke tests still invoked it directly, but every semantic
case routed through `_parity` compared Python with itself. That defeats the
step's primary claim while retaining the appearance of byte equality.

Required remediation: make the oracle self-checking. Assert the exact command
identity in both `CompletedProcess.args` values, or keep an invocation trace
that proves one call to the configured Rust executable and one call to
`[sys.executable, "-m", "gwz.cli"]`. Add a compact false-oracle regression
that kills either-side collapse and loss of the stdout/stderr comparison.

### S3.4-F2 — P2 — Required depth aliases and automatic lifts are masked

The cap test at
[`test_log_real_workspace.py:844`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.4/gwz-py/src/tests/test_log_real_workspace.py:844)
checks default 50, `-n 2`, `--no-limit`, and a `--since` lift. It does not
drive `-n 0`, an `--until`-only lift, or an unmasked explicit range. Every
range invocation at lines 657–705 supplies `--no-limit`, so the override
hides the automatic range policy it purports to accept.

The snapshot assertion is also non-discriminating: changing
`+baseline..HEAD` to standalone `+baseline` leaves its root-reason plus
nonempty-entry assertion green.

An exact wrong Python lowering was applied in a disposable candidate. It:

- mapped explicit `-n 0` back to omitted/default depth; and
- forced 50 when depth was omitted with an explicit `..` operand or
  `--until`.

All **227/227** focused S3.4 and adjacent tests remained green. The unmodified
candidate passes reviewer-added real probes for all three missing forms, and
each arm becomes red when its corresponding wrong lowering is active.

Required remediation: extend one real >50-entry table across default, `-n 2`,
`-n 0`, `--no-limit`, `--since`, `--until`, ordinary explicit range,
`+snapshot..HEAD`, and `+lock..HEAD`, without a masking override on the lift
cases. Snapshot/lock cases must assert exact included post-pin hashes and
excluded pin/pre-pin hashes as well as the required root degradation.

### S3.4-F3 — P2 — Exact-one and absolute-order acceptance assertions are weak

The actual `gwz commit` arm at lines 483–501 uses `next(...)`; it proves that
one matching coalesced record exists but not that the coordinated change
renders as exactly one entry. Appending a duplicate decoded record leaves the
test green. It also asserts ids but not the three exact commit hashes already
recorded in the fixture.

The extreme-time arm at lines 927–935 converts timestamps to a set. Reversing
the decoded records leaves it green, so the required far-future-before-
pre-epoch absolute order is not pinned.

Required remediation: collect all coordinated matches and assert exactly one,
with exact fixture member ids and hashes. Assert the ordered extreme subjects,
signed seconds, and recorded offsets rather than a set. The duplicate and
reverse-order mutations must be red.

### S3.4-F4 — P2 — The end-to-end heuristic matrix does not cover every arm

The fixture/table at lines 353–394 and 530–569 proves:

- a positive four-conjunct heuristic merge;
- the author-time/rebase MUST-NOT case; and
- the marked-versus-unmarked MUST-NOT case.

It does not independently vary the other required boundaries: different full
message, different author name/email, committer delta greater than 10 seconds,
or same-repository twins. A defect in any one of those branches can escape
S3.4 even though the earlier core suites remain useful upstream evidence.

Required remediation: add a compact one-axis real-history table for those
four MUST-NOT cases. Assert exact entry count, singleton member sets, hashes,
and `none` provenance through both real clients. Retain the positive, rebase,
and marked-versus-heuristic cases.

### S3.4-F5 — P2 — The human C0 body test passes rendering flags as pathspecs

`lossy_args` ends with `--`, `.` at lines 937–946. The human call then appends
`--full --color never` at lines 956–962. Both clients correctly parse those
three tokens as additional pathspecs:

```text
full=False
color=auto
pathspecs=[".", "--full", "--color", "never"]
```

The subject's invalid byte and ESC are still observed, but the body containing
U+0001 is never rendered. An exact Python sanitizer mutant that passes only
U+0001 through left the full 26-case battery plus 199 adjacent tests green.
The unmodified candidate passes the corrected invocation; moving the flags
before `--` makes that mutant red.

Required remediation: put `--full --body --color never` before the operand/
pathspec separator and assert the exact sanitized body fragment, including
U+FFFD in place of U+0001 and no raw SOH. Keep the machine assertions for
JSON-escaped `\u001b` and `\u0001`.

### S3.4-F6 — P2 — Real EPIPE acceptance is race-dependent and misses late machine positions

The new subprocess test at lines 967–997 starts the child with
`stdout=PIPE`, then closes the parent reader. The child may write before that
close. Removing only prefix silencing at
[`cli_log.py:245`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.4/gwz-py/src/gwz/cli_log.py:245) left the checked machine
node green in the reviewer run, although the builder had observed it red on a
different schedule. A deterministic pipe whose read descriptor was closed
before spawn made the same mutant exit 120 with interpreter-shutdown
BrokenPipe spray; production returned 0 with empty stderr.

A second mutation silenced only human output at the outer handler. All 12
focused EPIPE tests and the checked early-close machine node stayed green.
Synchronized post-prefix JSONL-record and JSON-suffix probes exposed exit 120
and spray; the production candidate returned 0 and released once.

Required remediation: preclose the read descriptor before spawning for the
prefix case. Add deterministic synchronized post-prefix JSONL-record and JSON
suffix cases, plus a later human-record case, asserting exit 0, empty stderr,
immediate stop, and one release. Removing either production silencing call or
making late silencing human-only must be red.

### S3.4-F7 — P2 — Streaming ordinary write failures can be mutated into success

Changing the outer handler at
[`cli_log.py:301`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.4/gwz-py/src/gwz/cli_log.py:301) from
`except BrokenPipeError` to `except OSError` makes ordinary failures during a
human record, JSONL record, or JSON suffix return 0. All **184/184** adjacent
CLI/render tests remained green. Reviewer position-specific probes passed
3/3 on production and 0/3 on the mutant, observing the wrong exit 0.

Required remediation: add ordinary-`OSError` writer cases at the first human
record, post-prefix JSONL record, and JSON suffix. Each must return 1 and
release exactly once. Add a fail-once stderr `OSError` case asserting exit 1,
the stderr-specific typed channel, and release; the existing stderr case
covers only BrokenPipe.

### S3.4-F8 — P3 — Two positive filter rows do not prove their flags are active

The `--grep` and nominal `--author` rows at lines 762–768 require only that
`filter subject` be present. That entry is present in the unfiltered history,
so deleting either flag leaves its row green. The separate marker-survivor
case does independently prove author filtering, but not the nominal row.
`--no-merges` likewise has no required survivor.

Tighten these rows to exact subject/hash sets or add known included and
excluded sentinels. Dropping each of the six filters must be red.

### S3.4-F9 — P3 — The extra pathspec test is not a native-Git oracle

The pathspec arm at lines 716–755 proves cross-client workspace/member-CWD
equality and one long-form exclusion. It does not compare complete hash order
against native `git rev-list`, and it omits short exclusion aliases and top
magic. S2.2-B's accepted native-parity suite remains the primary evidence, so
this is not a separate S3.4 blocker; the new test's “match” claim should still
be made exact or narrowed.

## Battery and requirement audit

| S3.4 surface | Result | Evidence |
|---|---|---|
| Actual Rust and Python subprocesses | PASS implementation / FAIL oracle | The configured exact Rust executable and `python -m gwz.cli` run over one real workspace. F1 records the self-collapse mutation. |
| Real `gwz init` + `gwz commit` marker group | PASS implementation / FAIL exact-one assertion | Exact clients coalesce root/api/web with marker provenance; F3 permits a duplicate. |
| Heuristic and marker boundaries | PARTIAL | Positive, rebase author-time, marked/unmarked, invalid-marker, inclusive W=60, and non-monotone fragmentation pass. F4 lists missing one-axis arms. |
| `--no-coalesce` and selection narrowing | PASS | Three raw singleton entries and a one-member selected marker group are exact. |
| Unborn/detached/unreadable/strict | PASS | Unborn root/member, detached HEAD, unreadable member, benign degradation, Partial/Failed exits, channels, and records are real and cross-client exact. |
| Snapshot/lock/tagged/ranges | PARTIAL | Root degradation and tagged selection pass; F2 records masking and endpoint weakness. |
| Six filters and typed filter refusals | PASS in production / PARTIAL acceptance | Since/until, no-merges, first-parent, survivor narrowing, invalid regex, and approxidate refusal are substantive. F8 records two weak positive rows. |
| Global depth and lifts | FAIL acceptance | Default/explicit/no-limit/since pass; F2 records three missing forms and a 227/227 green wrong lowering. |
| Jobs/ties/deterministic bytes | PASS | Jobs 1/2/4 are byte-identical and exact member-id tie order is pinned. |
| Extreme time, non-UTF-8, and C0 | PARTIAL | Exact signed values, offsets, machine lossy/escaping, and human subject sanitization pass. F3 and F5 record order/body gaps. |
| EPIPE and non-EPIPE | PASS production / FAIL acceptance | Deterministic reviewer probes pass every position and release. F6/F7 record surviving mutants. |
| Network/workspace mutation | PASS | Histories are local; fixtures live under pytest temp roots; no production write or network path was added. |

## Production EPIPE audit

The 18-line `_silence_broken_stdout` implementation moved byte-for-byte from
`cli.py` to `cli_shared.py` (extracted text SHA-256
`95cf9256897bb18a9ee3cb85eb32fe34fb53a9fe0970eae9bb6b68ede72faaff`).
Its consumers are now:

- machine-error stdout in `cli.py`;
- machine prefix output in `cli_log.py`; and
- streamed human/JSON/JSONL records plus the JSON suffix through the outer
  `cli_log.py` handler.

The prefix branch releases directly before returning. The streamed branch
returns inside a `try/finally`, whose `aclose()` releases the generator. Human
degradation stderr catches `OSError` inside the outer stdout-BrokenPipe
handler and remains a typed execution failure. Reviewer deterministic probes
at preclosed prefix, delayed JSONL record, delayed JSON suffix, and second
human record all returned 0 with empty stderr and one release. Ordinary
failures at the same positions returned 1 with one release. The findings are
acceptance defects, not production exceptions.

## Scope, size, and integrity

The candidate is one clean commit on the required sole parent, is
non-shallow, has no replacement refs or trailers, and passes `git diff
--check`. A temp-index replay of the exact binary patch reproduces the final
tree.

The five-file delta is `+1026/-21`:

```text
production:  src/gwz/cli.py, cli_log.py, cli_shared.py       +23/-19
tests:       test_log_real_workspace.py, test_native_log.py +1003/-2
```

Production is a net four-line correction: the byte-identical helper move,
two imports, and two calls. The 997-line harness makes the package roughly
3.4 times the plan's aspirational ~300-line target including tests. That is
not a standalone blocker; every production seam is necessary and private.
The surviving mutations show why raw test volume cannot substitute for the
missing distinctions.

The remaining `test_native_log.py` edit replaces two stale empty-output
expectations with the already-reviewed real behavior: a contributing root
entry on stdout and the unreadable-member degradation on stderr for both
Partial and strict Failed. It changes no runtime seam.

No native Rust, generated protocol, schema, renderer, client transport,
bridge, manifest, lockfile, dependency, core, Rust CLI, mode, or path moved.
Protected base identities include the native source tree, protocol tree,
renderer-parts tree, `cli_render.py`, `bridge.py`, `client.py`, Cargo files,
`pyproject.toml`, and `uv.lock`.

```text
base tree:                     3653d0f51a1f521f9c46070617515c1ecab40cbc
candidate tree:                73d7ee676157874b6bbbd41594431e92efbc9e1d
binary diff SHA-256:           7c02c3f771800fd34c8f56badcf0a84ea8050d84f16be0645c993483fe76b7a7
full-index diff SHA-256:       8f593b07a5557dadd269b0ed7af029e09db9547dde245e759a24d08259d176c4
stable patch id:               5fb4f500f6d4e8cea81031fa766b83fef997cf39
format-patch SHA-256:          081f490f8c7ab7f7e51ea8870e773fdb033bede3dd6d4a145635ba0b4ab82101
exact Rust authority tree:     317f3bb125ce91293fbb37d6a6b582f683f9addf
exact core authority tree:     20b52eb0b425e8482f4bd853fe4a6a580deb28e3
reviewed Rust binary SHA-256:   e6d986d4e3589cc7d558f091ede0cb658f40a75042c133cfdccadc1dcf35fac6
reviewed native module SHA-256: 7dd4642ae8d9554e686ebc95b094436e48fee5022975a3a08d3c978038535941
```

## Proportional evidence

The fast-test boundary was preserved. No broad Python/Rust/core suite and no
compiler mutation matrix was run.

Reviewer-run evidence:

- fresh `cargo build --locked` at exact Rust authority — exit 0;
- exact real-workspace battery — **26/26 passed**;
- S3.4 plus adjacent CLI/render/client/native/protocol gate —
  **227/227 passed**;
- parity-oracle one-side collapse — **26/26 stayed green**;
- combined `-n 0`/range/until depth mutant — **227/227 stayed green**;
- C0-body pass-through mutant — **225/225 stayed green**;
- prefix-silencer removal — checked machine node stayed green, deterministic
  preclosed probe red with exit 120/spray;
- late machine-silencing mutation — focused EPIPE set plus checked machine
  node stayed green; synchronized probes red;
- outer `OSError`-as-EPIPE mutation — **184/184 stayed green**;
- deterministic production EPIPE/non-EPIPE position probes — green;
- protocol drift check — exit 0, fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`;
- protocol regeneration check — exit 0; and
- topology, replay, protected blobs, diff check, and cleanliness — green.

Builder evidence was reviewed rather than needlessly repeated: Python
S3.4/adjacent 227, Rust g09/g10/g11 at 21/10/11, protocol drift/regeneration,
compile-all, native fmt/check/strict Clippy, CLI docs freshness, and the core
boundary all exited 0. The builder observed both silencer-removal mutants red;
the reviewer's repeat demonstrated that the prefix result depends on process
scheduling, so that single red run is not credited as a deterministic kill.

Cleanliness audit note: a delegated reviewer probe created untracked `.gwz/`
and `gwz.conf/` fixture directories at the candidate root at 15:14. They were
inspected, isolated, and moved recoverably to
`/Users/owebeeone/.Trash/gwz-s34-review-stray-dotgwz` and
`/Users/owebeeone/.Trash/gwz-s34-review-stray-gwz-conf`. No tracked candidate
byte or index entry changed; the candidate, authority, core, and report
worktrees are clean at handoff. The stray fixture is not product evidence.

## Final round-2 acceptance gate

Round 2 is final and should be tests-only unless the deterministic EPIPE
probes require a minimal private seam. Accept only when:

1. `_parity` proves one exact Rust and one exact Python process and both
   collapse mutants are red;
2. the full depth table covers `-n 0`, unmasked ordinary/snapshot/lock ranges,
   and until-only lift with >50 sentinels, killing every depth mutant;
3. actual `gwz commit` is exactly one group with exact hashes, extreme records
   are ordered, and the full heuristic one-axis MUST-NOT table is present;
4. the corrected human full/body invocation proves U+0001 sanitization;
5. deterministic prefix, late-record, and suffix EPIPE probes kill every
   silencer omission/conditional mutation while proving immediate 0, no
   spray, and release;
6. human/JSONL/JSON-suffix/stderr ordinary write failures remain 1 with one
   release and kill the broad-`OSError` mutant;
7. filter and pathspec assertions are tightened as described in F8/F9; and
8. production stays byte-identical unless a strictly private test seam is
   necessary; all authority, replay, protocol, and focused fast gates remain
   green.

No renderer redesign, protocol/core/Rust change, new semantics, broad suite,
or compiler matrix belongs in remediation.
