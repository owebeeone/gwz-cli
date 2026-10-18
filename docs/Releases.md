# Releases

GWZ releases are distributed through GitHub Releases for the `gwz-cli`
repository:

https://github.com/owebeeone/gwz-cli/releases

The [hosted documentation](https://owebeeone.github.io/gwz-cli/) is built from
the tag of the most recently published release, so its command model matches
the released CLI rather than unreleased work on `main`.

## Unreleased: 1.1.0, the transport release (DRAFT)

**This entry is a DRAFT** (2026-10-17, plan step C5 / S7.2). It is finalized after the switch removal (C4) and the acceptance matrix (S5.6). Statements marked *pending* wait on a number or a decision that is not in yet; nothing here is a release claim until the marker is gone.

1.1.0 makes gwz's own SSH and HTTPS transport the default for every network operation, on macOS (ARM64), Linux (x86-64) and Windows (x86-64). Connections are pooled within one command, a setup that fails is retried, and a stalled setup or body read ends with a message that names the limit. Windows has parity with gwz 1.0.17 (and a list of deliberate differences, below). libgit2's own transport, which 1.0.17 used, stays available as a choice you make before a command runs.

### What is new

- **`--transport gwz|native` chooses the transport for network operations.**
  `gwz` is the default. `native` is libgit2's own transport, as gwz 1.0 used.
  It is an escape hatch you choose before a command runs, not a fallback: gwz
  never moves an operation from one transport to the other, and a failed
  operation is reported as failed. The order, highest first: `--transport`,
  the `GWZ_TRANSPORT` environment variable (`gwz` or `native`), your global git
  configuration (`gwz.transport`), then the default. An invalid value is
  refused, not ignored. `gwz.transport` in a workspace's or a member's own git
  configuration does not select the transport, and gwz says so in a `note:`
  line. To try native once: `gwz --transport native fetch`. To keep it:
  `git config --file "$HOME/.gitconfig" gwz.transport native`; to remove it:
  `git config --file "$HOME/.gitconfig" --unset-all gwz.transport`. While
  native is selected, each command that uses the network says so once. See
  [`gwz fetch`](commands/fetch.md) and [Troubleshooting](Troubleshooting.md).
- **`--max-retries <n>`** (default 3) is the number of extra attempts after a
  failed setup. Setup is before the session is reusable: for SSH, before it is
  authenticated; for HTTPS, before the first request byte. A stall or a reset
  during a fetch, push or pull body is not retried. The wait after a failed
  attempt is 1 second, then 2, then 4, doubling, never more than 30 seconds,
  plus up to 0.25 second. `0` retries nothing and also turns off the adaptive
  per-host limit below. `--ssh-timeout 0` does not turn retries off.
- **`--ssh-timeout` means more and defaults higher.** It is the stall limit of
  one SSH setup attempt and of a stalled read during a fetch, push or pull body,
  on SSH and on HTTPS. The default is 9 seconds (it was 3). A setup also has a
  30 second budget, which HTTPS setup has alone; `--ssh-timeout 0` removes both
  clocks. With `--transport native` it is libgit2's connect and read timeout and
  the default is 3, as in gwz 1.0.
- **`--jobs` and `--max-per-host` default higher.** 100 jobs and 32 connections
  per host (they were 50 and 8). `--transport native` keeps 50 and 8 unless you
  give a value. Both are ceilings; see the next entry. *Pending S5.4: the
  defaults are confirmed or changed by the measure-then-select note.*
- **`--max-per-host` is a ceiling, and gwz may use fewer connections** (on gwz's
  transport). When a host refuses extra connections, gwz uses fewer at a time
  for the rest of the command, per scheme and port, tests that limit again as
  the command goes, and raises the number when the host recovers. A refused
  connection is retried inside `--max-retries` (default 3 extra attempts)
  instead of failing its repository at once, and a `Retry-After` from the host
  is waited for, at most 30 seconds at a time. Human output says what happened
  in `note:` lines on stderr; `--json`, `--jsonl` and `--porcelain` print none.
  `--max-retries 0` turns the lowering off. See
  [Troubleshooting](Troubleshooting.md) ("Fewer connections than --max-per-host, or a Capacity error").
- **A member that a host throttled, or that waited for gwz's own setup slots,
  fails with the new `Capacity` error code.** Scripts can tell "slow down, try
  later" from a Git failure by reading `error.code` in `--json` or `--jsonl`
  output. The code is a new value, 76, of the protocol's error codes
  (`capacity`); a reader that does not know it sees an unknown code, and
  `--json` writes it as the string `Capacity`. A final failure that was a 429, a
  503 with `Retry-After`, a `Retry-After` of more than 30 seconds, or a wait for
  one of gwz's own connection limits that expired (its 64 connection setups, its
  8 slots for resolving a host and setting up TLS, or the connections
  `--max-per-host` and the total allow) reports `Capacity`; every other failure
  keeps its code. The decision is made on the
  member's last attempt only: its last attempt was refused. What `error.code`
  was and is:

  | The member's own final failure | `error.code` before | `error.code` now |
  |---|---|---|
  | A 429 on its last attempt | `RemoteRejected` on a `fetch` or `push` row, `GitCommandFailed` elsewhere (`pull`, `clone`, `materialize`) | `Capacity` |
  | A 503 with `Retry-After` on its last attempt | the same | `Capacity` |
  | A `Retry-After` of more than 30 seconds | the same | `Capacity` |
  | Waiting for one of gwz's own connection limits expired | the same | `Capacity` |
  | A bare 503, a reset, a refused connect, a stalled setup | unchanged | unchanged |
  | A host that stopped answering (`not attempted: ...`) | unchanged | unchanged |

  See [Machine Output](MachineOutput.md) ("The `Capacity` error code").
- **HTTPS is authenticated through your configured git credential helpers.**
  `gh` is among them, and on Windows Git Credential Manager (`manager`) is the
  usual one. gwz reads your system, global and XDG git configuration (with
  unconditional includes, not `includeIf`) to find the helpers, asks them in
  their configured order, and sends what they return only as a Basic
  credential. It runs no `store` or `erase`. An `https://` URL that carries a
  password, a query or a fragment is refused before anything is sent, so a
  password or signed token in the URL is not a way to authenticate (a user name
  alone is allowed); use a credential helper. Windows adds one thing: a
  server that offers Negotiate or NTLM is answered with your Windows logon
  session, and no helper is asked for that. Credentials never cross to another
  origin on a redirect. See [Troubleshooting](Troubleshooting.md) ("HTTPS
  Credential Failure", "Windows Sign-In").
- **`gwz merge --remote <name>` no longer refuses a member only this workspace
  has.** A member registered here after the lane was cloned made every family
  merge from that lane refuse with `PairingMismatch`, even when `--target` left
  the member out. The merge now leaves such a member out and unchanged, and its
  summary message (`meta.message` in `--json` output) reports it as
  `not in source lane; unchanged`. Naming it with `--target` is refused before
  any transfer, and a member only the lane has is still refused. See
  [Local Clones](LocalClones.md).

### Migration notes for gwz-cli users

What to check when you move from 1.0.17. The first table is what changes for every user; the Windows list follows it.

| | 1.0.17 | 1.1.0 |
|---|---|---|
| Transport | libgit2's | gwz's own, by default; `--transport native` returns to libgit2's |
| `--jobs` / `--max-per-host` defaults | 50 / 8 | 100 / 32 (native: 50 / 8) |
| `--ssh-timeout` default | 3 | 9 (native: 3) |
| What `--ssh-timeout` covers | libgit2's connect and read timeout | a stalled SSH setup attempt and a stalled body read on SSH and HTTPS; native: libgit2's |
| A failed setup | one attempt | retried, 3 extra attempts by default (`--max-retries`) |
| A host that throttles | the repository fails | fewer connections, a wait for `Retry-After`, then `Capacity` (code 76) |
| A stalled body read | libgit2's timeout | ends that repository; not retried |
| Connections | one per member operation | pooled within one command |
| An `https://` URL with an embedded password, query or fragment | left to libgit2 | refused before anything is sent; use a credential helper |
| `--verbose` | authentication rows | also names the transport selected and what selected it |

Things that still use libgit2's own transport under the default setting. These are listed on purpose; none is a fallback:

- `http://` remotes. On Windows gwz prints a warning before the first request, on every operation, because a Basic password is sent unencrypted and a Windows sign-in has no channel binding. Use `https://` where the server offers it.
- `git://` remotes.
- `file://` remotes and local paths, including local clone families.
- Anything, when you choose `--transport native`.

On those remotes `--ssh-timeout` is libgit2's connect and read timeout, so under the default setting it is 9 seconds (it was 3), and `--max-retries` has no effect.

Scripts: `error.code` can now be `Capacity`. `--json` and `--jsonl` carry `meta.transport_setting` when the setting is not the plain default (native selected, a value ignored, an unreadable file skipped). gwz-cli's `meta.transport` rows are unchanged (see [Machine Output](MachineOutput.md)); gwz-py's gain four keys (see its migration notes). A reader of `meta.transport` rows from a Python or other client must treat the new keys as optional, and must not read an absent key as "the native route": the four receipts are absent unless the transport reported an open for that row (absent on the native route, on a transport row whose setup failed or was refused before the open, for remotes the transport does not carry, and from an older writer).

If something goes wrong after the upgrade, run the same command with `--transport native`, which is 1.0.17's behaviour for that command, and tell us what differed.

#### Windows

A Windows build of 1.1.0 uses Pageant, the OpenSSH agent, the machine's WinHTTP proxy and Windows authentication. The deliberate differences from 1.0.17 (the operator's list of 2026-10-10 with the decisions since; the draft Windows guide, `gwz-core/dev-docs/GwzTransportWindowsUserGuide-DRAFT.md`, has the messages and the commands, and moves into this documentation set at the release):

1. Folder and key names with non-ASCII characters work.
2. A key path that starts with `~/` uses the same home as `known_hosts` (`HOME`, then `HOMEDRIVE` plus `HOMEPATH`, then `USERPROFILE`); 1.0.17 used `USERPROFILE`. A missing or unusable home says which source needs correcting.
3. A server that offers Digest beside a supported scheme is answered with the supported scheme; a server that offers only Digest fails at once, not after 15 replays. Neither transport answers Digest.
4. A server that rejects your Windows account ends the operation with one error naming the scheme. No replay loop, no helper fallback, no setting changes this.
5. Discovery redirects are followed on Windows (at most five; never from `https://` to `http://`; none after a Windows sign-in has started). Credentials never cross to another origin.
6. A server that challenges the upload step of a fetch is refused with a plain message, and the data is never sent again.
7. An SSH connection that drops before login is retried. The key exchange that Windows' crypto library (WinCNG) ends by itself about once in a hundred connections is retried up to twice on a new connection inside the same attempt; it does not count toward `--max-retries`. *Pending D32 landing.*
8. Messages that were opaque now name the cause and the fix (a host that is not in `known_hosts` and a key that changed are worded apart; a passphrase-protected key file is refused on every platform with its own message).
9. HTTPS uses the machine proxy setting (`netsh winhttp show proxy`) and no other. `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` and git's `http.proxy` are ignored. A setting with no entry for `https`, a PAC script, or a form gwz cannot read refuses the https remote with a message naming the setting, instead of going direct as 1.0.17 did; SSH and local operations are not affected. A proxy that answers 407 is refused: proxy sign-in is not supported.
10. gwz trusts the Windows store and also the CA file `GIT_SSL_CAINFO` names (else `SSL_CERT_FILE`); 1.0.17 ignored both. A stale or malformed file named there can now stop gwz before it connects.
11. A Pageant that is set to confirm every use, whose prompt stays open longer than `--ssh-timeout`, fails that attempt with a message that names the option (1.0.17 waited). Raise `--ssh-timeout` (up to 30 seconds; `0` removes the limit) or use `--transport native`. A passphrase entered on first use succeeds on the retry.
12. A plain `http://` remote prints the warning above (it goes native, as the previous list says).

Unchanged on purpose, so they are not differences: your Windows account can be offered to any host that challenges with Negotiate or NTLM (review the remote and any redirect; no setting turns it off in 1.1.0); only RSA host keys and unencrypted RSA PEM key files are used on Windows; agent RSA keys sign at `rsa-sha2-512` only; the agent pipe's creator is not checked and a Pageant owned by another user is not refused. Not promised: PAC scripts. Not supported in 1.1.0: hardware security keys, Kerberos, and certificates held by Pageant or by the Windows OpenSSH agent service (see *Known limits of this release*).

*Pending S7.2 finalization:* the capability answers for Windows selected-identity SSH (`file_identity`) and exact agent selection (`exact_agent_identity`) follow decision D30 (GO for both, the second after one exact-selection row passes); until the final answer is in, treat exact agent selection as not offered on Windows.

### Known limits of this release

Stated here because they are measured or decided, not because they are solved.

- **Platforms.** macOS ARM64, Linux x86-64 and Windows x86-64. Linux ARM64 and macOS x86-64 are not supported in 1.1.0. Building with Bazel's `@gwz_core_crates` is unsupported until the release's registry pins; it was already unbuildable before this release.
- **Speed on Windows (decision D29).** Over SSH, gwz 1.1.0 costs a fixed 20 to 30 ms more per command than 1.0.17 (8 ms more at 32 members in run `2026-10-16-tr84speed`). This is accepted as parity. Over HTTPS the first measurement was 4.7 ms more per member (0.165 s at 32 members); the cause was found (every member's certificate check queued on one thread of the HTTPS endpoint) and the fix, a multi-thread runtime on every platform, is decision D33. *Pending the re-time on macOS, Linux and Windows; the numbers go here.*
- **Speed on Linux.** *Pending the Linux x86-64 rows.* The Raspberry Pi 5 runs (aarch64, not a supported platform) put HTTPS to 32 members at +0.062 s against 1.0.17 after the fixed-cost fix, down from +0.071 s; at 1 member +0.005 s.
- **No partial result on the transport route (D22).** On the transport route a fetch of 16 or 32 members does not lose a member to a key exchange that Windows ends on its own (1.0.17 lost members in up to half of the SSH runs on the Windows speed rows). The native route keeps 1.0.17's behaviour here and is not retried; about 1 in 160 to 190 member fetches on the Windows test host (7 of 1,280 native, 8 of 1,280 on 1.0.17).
- **A very large blob pushed over SSH may be charged to the stall clock.** The pump cannot tell a push whose producer has paused (libgit2 compresses an object whole before writing it) from a finished pack waiting for the server's report, so a pause longer than `--ssh-timeout` can end the push with a stream stall. HTTPS does not have this. This is a hypothesis that has not been reproduced; a 1 GiB single-blob push row is the test. If it bites, raise `--ssh-timeout`. *Pending that row.*
- **A pooled SSH connection can be stale.** The pool's 60 second idle timer starts when a connection is released, not at its last network byte, so a connection a NAT dropped can be pooled and fails on reuse; that costs one stall and one retry.
- **Credential-helper launches on Windows are serialized, with limits.** gwz launches a helper in a window during which no other gwz child process may inherit its pipes. The serialization is per command (one host per `gwz` command): a program that embeds gwz and runs two operations at once in one process does not serialize them against each other, and `gwz log`'s `git rev-list` is not serialized. gwz's own sign-in worker launch is covered by a gwz-sspi launch guard (decision D31, *pending gwz-sspi 0.1.2*).
- **A Windows SSH server that refuses a signature it accepted the key for** (`PUBLICKEY_UNVERIFIED`, about 1 in 6,000 connections against Windows' `sshd.exe`) is not retried.
- **Hardware security keys are not supported.** gwz offers an SSH agent's `sk-ssh-ed25519@openssh.com` and `sk-ecdsa-sha2-nistp256@openssh.com` keys, and on macOS and Linux they authenticate in gwz's tests through a software authenticator that answers the agent's signature request. Signing with a physical key, including the wait for its touch, has not been tried. gwz waits for the agent's answer under `--ssh-timeout` (9 seconds by default), so a touch that takes longer ends the attempt with a message that names the option; raise `--ssh-timeout`. A key that the agent declines to sign, such as a security key that is not plugged in, is skipped and the next key in the agent is tried; to offer only one key, give its public-key file to `--identity` (exact agent-key selection, decisions D38 and D39, *pending its landing*). On Windows no security-key row has run yet (*pending the Windows security-key rows*).
- **Kerberos is not supported (Windows).** A server that offers `Negotiate` is answered with the logon session's default credentials, as in 1.0.17. On a domain-joined host Windows may choose Kerberos inside `Negotiate`; gwz does not prevent that, asks for neither delegation nor mutual authentication, and has tested the exchange only where NTLM was chosen, so no claim is made for Kerberos. If a `Negotiate` sign-in that worked with 1.0.17 fails, run the command with `--transport native`, which is 1.0.17's behaviour for that command. A sign-in that Windows refuses is reported with its own message naming Kerberos and NTLM policy and `--transport native` (decision D48, *pending its landing*). An exchange that ends without a verdict after a token was sent is reported as "HTTPS endpoint request failed: Protocol", which does not name Kerberos.
- **Certificates held by Pageant or by the Windows OpenSSH agent service are not supported.** On macOS and Linux, certificates of Ed25519, ECDSA (P-256, P-384, P-521), RSA and the two security-key types (through the software authenticator of item 1) authenticate from an SSH agent against a server that trusts the signing CA; a DSA certificate is skipped. On Windows, Ed25519 (as with 1.0.17) and ECDSA (P-256, P-384, P-521; no 1.0.17 baseline) certificates authenticate from an agent on a pipe other than the OpenSSH service. On Windows, an RSA certificate did not authenticate in the tests, as with 1.0.17: libssh2 on Windows offers it only under the algorithm `ssh-rsa-cert-v01@openssh.com`, which the test servers' default list refuses, and a server that accepts it was not tried. A certificate held in Pageant or in the OpenSSH Authentication Agent service has not been tried, and naming one with `--identity <certificate>.pub` is equally unsupported.

For Python, including the async `call()` exit limitation, see the gwz-py migration notes.

## 1.0.17: `gwz fetch`, and lanes that dispose clean

1.0.17 (2026-09-18) adds one read-only network verb and finishes the lane
disposal clean-up that 1.0.14 and 1.0.16 began.

### `gwz fetch`

`gwz fetch` contacts every selected repository's configured remote, updates
that repository's remote-tracking refs, and prints one row each: the tracking
ref before and after, how far the current branch is ahead of and behind it, or
`new <after>` for a tracking ref that did not exist before, or `updated`,
`no change`, `no upstream`, `failed`. Plain `gwz fetch` covers `@root` plus the
configured members, and the selectors are `gwz push`'s. It answers "what moved
upstream while I was working?" across the whole workspace in one pass, without
changing a file.

```text
$ gwz fetch
status: Partial
@root      .         no change         (origin/main, +2 -0)
mem_core   gwz-core  90ef552..330a174  (origin/main, +0 -3)
mem_cli    gwz-cli   no change         (origin/main, +0 -0)
mem_local  local     no upstream
mem_priv   private   failed            RemoteRejected: member 'mem_priv' at 'private': failed to connect to 127.0.0.1: Connection refused
```

What it never does:

- **It never integrates.** No merge, no rebase, no fast-forward, no reset: no
  branch, no `HEAD`, no index, no working-tree file. Use `gwz pull` to
  integrate what a fetch showed you.
- **It never writes workspace artifacts.** No lock, no manifest, no boundary
  sync. Because of that it still runs while a coordinated merge is open,
  unlike `pull` and `push`.
- **It never prunes.**
- **It never skips the network, with one exception: `--dry-run`.** There is no
  `--check-remotes` and no "unchanged since the last fetch" short-circuit as
  there is on `gwz push`: a fetch that does not connect has answered nothing.
  `--dry-run` is not git's. `git fetch --dry-run` contacts the remote and
  declines to write; `gwz --dry-run fetch` contacts no remote at all, resolving
  the selection and printing the planned rows.

A planned row carries a token of its own, so it can never be read as an answer:

```text
$ gwz --dry-run fetch
status: Noop
@root      .         would contact origin
mem_core   gwz-core  would contact origin
mem_cli    gwz-cli   would contact origin
mem_local  local     no upstream
mem_priv   private   would contact origin
```

`would contact <remote>` is `"result": "Planned"` in the machine output, and no
live fetch prints either. `no change` and `Unchanged` keep their one meaning:
the repository was contacted and its tracking ref did not move. A repository
with no fetch remote is still `no upstream` under `--dry-run`, because that
answer needs no network.

Exit codes follow `gwz push`, with one difference worth knowing: `no change`
means contacted-and-answered rather than skipped, so a run in which one remote
failed and every other repository read cleanly exits `1`, because the report is
incomplete, even though nothing moved.

| Exit | Meaning |
| --- | --- |
| `0` | every selected repository answered |
| `1` | some answered and some failed; the report is incomplete |
| `2` | every selected repository was refused before the network, so nothing was contacted |

Exit `2` is the whole-batch refusal, for example `--remote <name>` naming a
remote no selected repository has; a refusal before there is a selection (no
workspace, an unknown member id) is a typed error and exits `1` as on every
verb. A dry run shares these codes with the live run it rehearses, and refuses
a `--remote` name a repository lacks just as the live run does.

The global `--remote <name>` selects the remote each selected repository
contacts, and `--json` carries the rows under `fetch_repos`.

**Not offered yet.** This release is the verb and nothing more. `--prune`,
`--tags`, per-remote selection and multi-remote fetch for a member that has
more than one remote are all planned and none is implemented; this note does
not say when. See [`gwz fetch`](commands/fetch.md).

### Lane disposal: an integrated lane needs no waiver

`gwz local dispose <name>` now succeeds, with no `--force` and no operator
comparison, on a lane whose work the surviving family already holds, even one
that was built in. Disposal compares the lane against the family, and against
the record the clone wrote of what it copied, so the caches, ignored user data
and stash and reflog entries a verbatim copy inherits are reported without
refusing. What refuses is what only the lane holds.

A refusal sorts what it found into four categories, printing each with its
count and with a description of what it holds, the empty ones included. Most
entries are paths; a protected root is described by its object id and the head
or ref that reaches it:

- **regenerable**: a tool made it and the same tool remakes it. Never refuses.
- **unchanged copy**: the clone copied it, the lane has not touched it, and a
  surviving member still holds it. Never refuses.
- **changed copy**: the clone copied it and it is not the family's any more.
  Refuses.
- **unique to the lane**: the lane alone holds it, including any protected
  root no single surviving family repository preserves whole. Refuses.

The refusal then prints the exact `--force <hazard,...>` command that waives
exactly what it found, and names nothing more: a lane whose only refusing entry
is dirt is offered `--force dirty` even when the same report lists regenerable
entries and unchanged copies beside it.

**Regenerable is recognised by marker and by shape, never by a directory's
name.** A directory holding a `CACHEDIR.TAG` that begins with the Cache
Directory Tagging Specification's signature line; a `__pycache__/` holding
nothing but `.pyc` and `.pyo` files; an `*.egg-info/` holding a `PKG-INFO`; a
`bazel-*` or `razel-*` symlink whose target lies outside the workspace; a file
ending `.so`, `.pyd` or `.dylib` inside a worktree; and an untagged build
directory proved by its tool's own markers (cargo's `.rustc_info.json`, a
`debug/.fingerprint` beside a `debug/deps`, a `pyvenv.cfg`). A `target` with no
cargo marker in it is not regenerable, and neither is anything a probe cannot
read. Recognition does not consult the copy record, so a cache the lane rebuilt
or created from nothing is still a cache.

**A forced deletion now reports what it was actually forced past**, not the
names you typed. A waiver that covered a refusing entry is listed after
`forced past:`; a waiver you named that covered nothing is listed after
`unused waiver:`, so an over-broad `--force` says so in its own report.

The four categories and the exact waiver command shipped in 1.0.16 with
`regenerable` always empty. 1.0.17 fills it. See
[Local Clones](LocalClones.md) and [`gwz local`](commands/local.md).

### Catching up: 1.0.14 and 1.0.16

No release notes were written for 1.0.14 (2026-09-18) or 1.0.16
(2026-09-18). What they carried:

- **Claude Code worktree hooks.** `gwz hook claude-code worktree-create` gives
  a Claude Code session started with `--worktree` a lane, a local clone of the
  whole workspace, instead of the `git worktree` Claude Code would have made,
  which in a GWZ workspace has no members. `gwz hook claude-code
  worktree-remove` disposes that lane through `gwz local dispose`, with GWZ's
  own refusals intact and never `--force` or `--keep`. `gwz hook claude-code
  setup` prints or writes the settings block (`--project`, `--project --local`
  or `--user`, with `--write`) and `--remove` takes it back out, changing no
  byte outside the block. Outside a GWZ workspace the same hooks make and
  remove the plain worktree Claude Code would have made, so the block can live
  in user-level settings without changing other projects. See
  [Claude Code](ClaudeCode.md) and [`gwz hook`](commands/hook.md).
- **`--owner <token>` and `--wait <secs>` on the family verbs.** `gwz local
  clone --owner <token>` records an opaque caller token, up to 128 bytes of
  `[A-Za-z0-9._:-]`, on the new member row, in the same index write that
  reserves the row. It never changes afterwards, `gwz local list` reports it,
  and GWZ never interprets it: it is the caller's identity for the caller's own
  reuse decisions. `--wait <secs>` makes a busy family lock retry until the
  deadline instead of refusing `Busy` at once, on every family verb and on
  `gwz merge --remote <name>`; a wait that wins rereads the index before
  acting.
- **Family index schema v2.** Recording an owner needs a place to put it, so
  the index carries `gwz.local-family/v2`: v1 plus the optional per-row
  `owner`. A 1.0.14 or later gwz reads a v1 index unchanged and writes v2 on
  its first write of any kind, a create, a dispose, a `--keep` or a family
  merge. Going the other way is a refusal rather than a downgrade: an older gwz
  refuses a v2 index as a whole, and its refusal names no gwz version at all.
  On 1.0.13 it reads ``gwz: ManifestInvalid: ... .gwz/local-family.yml is
  malformed: `schema: gwz.local-family/v2` is not `gwz.local-family/v1` (format
  version 1); this file is not in a format this store reads``. That line is
  derived from the 1.0.13 source
  (`v1.0.13:crates/family-store/src/format.rs:37-42`), not from a run on this
  machine. It calls the file malformed, which it is not, and offers no remedy;
  the remedy is to upgrade. **Once a 1.0.14 or later gwz has written a
  workspace's family index, every gwz used on that workspace must be 1.0.14 or
  later.**
- **Lane disposal clean-up, Phase 1.** `gwz local clone` records what it
  copied per repository, and `gwz local dispose` uses that record so an
  unchanged copy the family still holds is reported instead of refusing. Where
  no record exists, because the lane came from an older gwz or was copied
  outside gwz, dispose makes the comparison against the family itself. The
  identical-copy witness settled `unpreserved-history` for a merged lane: a
  copy the surviving family holds object for object is not a loss. 1.0.16
  completed the phase with the four-category refusal and the exact waiver
  command described above.
- **`gwz ls` says why a listed member is not on disk.** A member the lock
  records but that is absent is still listed, rather than quietly hidden, with
  `materialized: false` and a human note: `(private, skipped)` for the
  quiet-clone case, `recorded in the lock but absent on disk` for any other.
  Members the lock never materialized are unchanged, omitted unless
  `--unmaterialized` and carrying no note. Read `materialized`, not the note,
  to decide anything. See [`gwz ls`](commands/ls.md).

## 1.0.4: the 1.0 line ships

1.0.4 (2026-09-08) is the first published release of the 1.0 series — the
1.0.0 release candidates and 1.0.2/1.0.3 were pre-publication and packaging
iterations from the same morning. If you are upgrading from 0.14 or earlier,
read this section together with the lane-recovery notes below.

The 1.0 series delivers the **local clone family** (`gwz local`): a verbatim,
isolated second copy of the whole workspace — its own repositories, its own
runtime state, its own merges — created with `gwz local clone NAME`, integrated
back by name with `gwz merge --remote NAME`, and disposable only when its
history provably survives elsewhere. See [Local Clones](LocalClones.md) for the
lane workflow and [measured disk/speed data](LocalClones.md#disk-space-and-copy-speed)
(on reflink filesystems, ten whole-workspace lanes cost ~20.5 MiB; on ext4 the
same ten cost ~537 MiB as ordinary copies).

Release verification for 1.0.4 (2026-09-08) covered install, workspace clone,
local clone, detach, and delete on Linux ARM64 and Windows x86_64, alongside
the release pipeline's own gates.

## 1.0.0: lane recovery

This release makes the ordinary lane cycle work with one default merge:

```sh
gwz local clone A
# Work and use gwz add / gwz commit in A.
gwz merge --remote A
gwz local dispose A
```

- **Merge selection changes:** ordinary and family merges include the workspace
  root and active members by default. To merge members only, use
  `gwz --target @all --no-target @root merge`. An explicit partial merge can
  leave root history unpreserved and prevent lane disposal.
- Untouched clones can be disposed despite a regenerated configuration marker.
  Only freshly verified generated bytes are discounted; actual edits and unique
  history remain protected. The marker can still appear modified in status.
- Network publication captures source refs before transfers and withholds the
  root if selected member publication fails or locked dependencies cannot be
  proved available. Earlier member pushes may already have succeeded; repair
  the reported failure and retry.
- SSH file selection is explicit through `--identity PATH` or
  `--remote-identity NAME=PATH`, without agent fallback for the selected file.
  Exact selection of an encrypted key inside an agent remains unsupported.
- Only verbatim local clones are supported. `--clean`, `--bare`, `--from`, and
  their branch option remain refused, as do family pull/push. Use the receiving
  workspace's `merge --remote NAME` to integrate a lane.

Windows publication and fresh-lock verification, Windows/Linux lane lifecycles,
and Linux ARM64 ext4 operation have focused acceptance evidence. XFS supports
space-efficient CoW copies; ext4 uses independent ordinary copies. See
[Local Clones](LocalClones.md#disk-space-and-copy-speed) for measured scope.

The older open-merge upgrade restriction below still applies.

## Upgrading To 0.14.0

**Close any open coordinated merge before you upgrade.** GWZ 0.14 has one merge
implementation, and it cannot act on a merge record written by 0.13.x or
earlier. On 0.13.x, run `gwz merge --status`, then finish the merge with
`gwz merge --continue` or undo it with `gwz merge --abort`. Only then install
0.14.

If a pre-0.14 merge is still open after the upgrade, GWZ says so and stops.
Every merge verb, and every command a merge blocks, refuses with one sentence:

```text
gwz: OpenOperation: this is a pre-0.14 merge; use gwz 0.13.0 (the last release before 0.14) to continue or abort
```

`--status`, `--continue` and `--abort` all refuse alike, so the usual
open-merge advice does not apply and is deliberately not printed. Nothing is
wrong with the record and nothing needs repairing: reinstall a 0.13.x build,
close the merge with it, and return to 0.14. There is no conversion step and no
in-place upgrade for an open record.

Merges that were already closed are unaffected. Archived records still project
read-only through `gwz merge --status <merge-id>`, and `gwz merge --gc` never
deletes an archive it cannot read.

Starting a merge is otherwise unchanged. Ordinary, `--ff-only`,
custom-message and `--no-ff` starts now all write the same coordinated merge
record, so status, continue, abort and recovery behave identically whichever
way a merge was started.

## Compatibility Notes

Behaviour a consumer of GWZ's output or metadata has to account for. Every
note here describes **released** behaviour: the log, merge, anchor-directory
and `gwz.conf/` notes below all shipped across the 0.10 to 1.0 releases.
Unreleased work is described in the version section it will ship in, not here.

- `gwz log` adds one newest-first history across the workspace root and selected
  member repositories, with coordinated-marker and conservative heuristic
  coalescing, revision/snapshot/lock ranges, six filters, compact and full human
  rendering, and the dedicated `gwz.log/v0` JSON/JSONL record schema. The
  Python API and `gwz-py log` expose the same records and rendering contract.
- Structural workspace commands refuse uncommitted hand edits to `gwz.conf/`
  (the machine-managed manifest and lock). The refusal names the sanctioned
  `gwz repo` verbs and the acceptance path (`gwz init --update --force`);
  states produced by git itself — clone, pull, branch switch — reconcile
  silently, and read-only/list commands never gate. Workspace bootstrap also
  emits a machine-managed banner on `gwz.yml`, records digests in
  `gwz.conf/markers/conf-integrity.yml`, and writes or merges a
  `.claude/settings.json` deny rule (`Edit(/gwz.conf/**)`) for agent sessions
  started at the workspace root.
- A checked-artifact private anchor directory holding a foreign,
  non-canonical retired-anchor rendering (for example
  `.ca1-anchor-retired-007`) now refuses operations on that family until the
  foreign file is removed; such names were previously adopted silently. The
  canonical rendering is unpadded (`.ca1-anchor-retired-7`).
- First-class merge JSON and JSONL include the complete current merge-response
  key set, including finalization progress. Structured errors include
  `target_kind` and retain member id/path context even for whole-operation
  preflight failures. Durable record compatibility errors also include typed
  `record_context` rather than encoding merge id, schema/version, required
  wave, or legacy mode only in prose. Consumers must tolerate additive keys within these versioned envelopes.
- Merge status rows expose durable pending-action reconciliation as
  `NotStarted`, `ExpectedConflict`, `CompletedExactly`, or `Ambiguous`.
  Ambiguity is also reported as dedicated structured drift and remains
  mutation-blocking.
- Merge and `pull --sync merge` reject source and target commits with unrelated
  histories, matching Git porcelain. GWZ does not implicitly allow unrelated
  histories.
- While a coordinated merge is open, the accepted workspace lock remains the
  exact pre-merge baseline. Clean, conflicted, failed, and unattempted outcomes
  are retained in the local durable operation record rather than published as
  a partial composition.
- Merge commits use the quoted default message
  `Merge '<source>' into '<target-branch>'`, or the body supplied with `-m`,
  with mandatory `GWZ-Merge-ID` and `GWZ-Operation-ID` identity lines. The
  exact final message is frozen before mutation and survives restart and
  conflict resolution. A request that creates a commit supplies its
  author/committer identity when present; otherwise the target repository
  identity is used.
- Coordinated merge start, dry-run, status, continue, and safe abort are
  available together. Successful changed merges publish a checked root
  composition commit; interrupted finalization resumes idempotently.
  Recovery must not substitute raw `git merge --abort` for the coordinated
  operation.

## Install Latest

macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Windows PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "iex (irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1)"
```

## Install A Pinned Version

Replace `latest` with a concrete tag:

```text
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.sh
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.ps1
```

## Verify Assets

Release assets are checksummed and have GitHub artifact attestations. For
stronger verification:

1. Download the release archive and checksum file.
2. Compare the archive SHA-256 with the checksum.
3. Verify the GitHub artifact attestation.
4. Run `gwz --version`.
5. Run `gwz --help`.

## Smoke Test A Unix Installer

```sh
tmp="$(mktemp -d)"

curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh \
  -o "${tmp}/gwz-installer.sh"

GWZ_UNMANAGED_INSTALL="${tmp}/bin" \
GWZ_NO_MODIFY_PATH=1 \
sh "${tmp}/gwz-installer.sh"

"${tmp}/bin/gwz" --version
"${tmp}/bin/gwz" --help
```

## Smoke Test A Windows Installer

```powershell
$ErrorActionPreference = "Stop"

$tmp = Join-Path $env:TEMP "gwz-test-$([guid]::NewGuid())"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

$installer = Join-Path $tmp "gwz-installer.ps1"
Invoke-WebRequest `
  "https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1" `
  -OutFile $installer

$env:GWZ_UNMANAGED_INSTALL = Join-Path $tmp "bin"
$env:GWZ_NO_MODIFY_PATH = "1"

Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& $installer

$exe = Join-Path $env:GWZ_UNMANAGED_INSTALL "gwz.exe"
& $exe --version
& $exe --help
```

## Local Release Checks

From the development workspace:

```sh
cargo fmt --check
cargo test
cargo run -q -p gwz -- --help
python scripts/generate_cli_reference.py --check
```

When changing command docs, inspect the generated reference and spot-check
command help for the affected commands:

```sh
cargo run -q -p gwz -- help status
cargo run -q -p gwz -- help tag
```

`scripts/release.py` runs the generated CLI reference check by default before it
commits the release worktree. If the release must proceed while docs are being
reconciled separately, pass `--no-doc-check`; otherwise update the reference
with:

```sh
python scripts/generate_cli_reference.py --write
```
