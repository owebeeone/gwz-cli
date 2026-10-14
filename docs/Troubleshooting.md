# Troubleshooting

## Not A GWZ Workspace

Symptoms:

- A command cannot find workspace metadata.
- `gwz status` reports a workspace/root error.

Checks:

```sh
test -d gwz.conf && ls gwz.conf
gwz --help
```

Recovery:

- If this should be a new workspace, run `gwz init`.
- If this is a cloned root repository, run `gwz materialize --lock`.
- If you are in the wrong directory, move into the workspace or pass
  `--root <path>`.

## Members Not Materialized

Symptoms:

- Status reports members not materialized.
- A member path exists in metadata but not on disk.

Recovery:

```sh
gwz materialize --lock
gwz status
```

Use `gwz ls --unmaterialized` to include configured members in the listing.

## Dirty Member Blocks An Operation

Symptoms:

- Materialize or pull refuses to proceed.
- Status shows staged, unstaged, or untracked changes.

Recovery:

```sh
gwz status
gwz snapshot before-cleanup
```

Then commit, discard, or move the local changes using normal Git tools or GWZ
stage/commit commands. Use `--force` only when you understand the destructive
effect.

## Operation Would Partially Mutate Members

Symptoms:

- One selected member cannot proceed and GWZ rejects the wider operation.

Recovery:

- Fix the failing member and rerun the operation.
- Narrow the selection with `--member` or `--member-path`.
- Use `--partial` only when it is acceptable for some selected members to move
  while others do not.

## Remote Missing Or Wrong

Symptoms:

- Pull, push, fetch, or tag remote operations fail for missing remote names.

Checks:

```sh
gwz forall -- git remote -v
```

Recovery:

- Add or fix the remote in the affected member repository.
- Run `gwz repo sync <member-path>` after adding a remote outside GWZ so the
  workspace manifest records it.
- Use `--remote <name>` when the operation should use a non-default remote.

## Repository Source Identity Mismatch

Symptoms:

- `gwz repo attach`, bare `gwz repo add`, or explicit source-id reuse fails
  with `SourceIdentityMismatch`.
- The error lists a historical commit and the snapshot or marker that recorded
  it.

Meaning:

- The candidate repository does not currently contain every commit needed to
  support the historical identity claim. A matching remote URL is not proof.
- This is common with a shallow or otherwise incomplete checkout, but can also
  indicate that the checkout is a different repository.

Recovery:

1. Verify that the checkout is the repository you intended to attach.
2. Fetch sufficient history yourself; GWZ never fetches during identity
   verification.
3. Retry with the explicit historical member id.

```sh
git -C libs/shared fetch --all --tags
git -C libs/shared fetch --unshallow --all --tags  # for a shallow checkout
gwz repo attach mem_shared
```

If no snapshot or marker commit evidence exists, bare add cannot infer the old
identity. Explicit `gwz repo attach mem_shared` proceeds and warns:

```text
attached mem_shared; no snapshot or marker commit evidence was available to verify repository identity
```

An unreadable snapshot or marker is also rejected. Repair it rather than
bypassing the identity check. See
[Repository Member Lifecycle](RepoLifecycle.md) for the full contract.

## HTTPS Credential Failure

In the planned 1.1.0 transport candidate, GWZ runs the user's configured Git
credential helpers, including `gh`. Its helper timeout messages link here; the
same guidance applies to SSH's ambient password-only helper route. Ordinary
SSH agent/key selection is described below. The same helper messages apply on
Windows, where Git Credential Manager (`manager`) is the usual helper.

GWZ reads captured system/global/XDG configuration and session `GIT_CONFIG_*`
overrides with unconditional includes. It excludes repository-local files and
all `includeIf` entries. It preserves matching helper order and empty resets.
Native Git can therefore succeed through a different helper. To repair GWZ,
sign in to or repair the helper in its actual unconditional chain; keep legitimate
unconditional includes. Privately record original entries, their source and
order before changing them; remove only newly introduced entries and restore
the original chain to undo. Never print a helper's credential answer.

For an existing member, find its URL with
`git -C <member-path> remote get-url origin`. Before cloning use its manifest
URL, or the effective URL after `->` in `gwz --verbose materialize --lock` when
converting to HTTPS. `git ls-remote <member-url>` can unlock a store, but a
success through a local or conditional helper does not prove GWZ's helper was
repaired. `git config --get-urlmatch credential.helper <member-url>` inside a
repository has the same scope caveat.

`credential_helper_timeout` (code 75) reports the exact available allowance:
either a helper could not start while waiting for resources, or it did not
answer after admission. The maximum interaction allowance is 120 seconds;
caller/configured limits can shorten it. Slot waiting has a separate allocation
allowance, so increasing the network timeout does not repair a waiting helper.
Finish a pending sign-in/unlock, or repair the selected helper, then retry the
operation. There are eight helper slots per endpoint and eight shared host
slots, including cleanup; member jobs and connection limits are separate.

Three messages say why no credential came from the helpers; the first two end with
a pointer to this section:

- **`The credential helper in your git configuration gave no password for <host>`**
  (the helper declined). The helper may have been cancelled (for example a Git
  Credential Manager window that was closed), may hold no sign-in for this server,
  or may have failed; GWZ cannot tell which, because git's prompts are off and its
  diagnostics are discarded, and it does not show the helper's name. No credential
  was sent. Run `git ls-remote <member-url>` with the member's URL: it runs the same
  helper where it can ask you or show its own error. Sign in, then retry.
- **`gwz did not use your git configuration to find a credential helper, because it
  refused to read it`** (configuration refused). GWZ refuses a configuration that is
  too large, `include.path` chains that are too deep or too many, an include it
  cannot follow (unreadable, not a regular file, or written as `~user/` or
  `%(prefix)/`), a value too large for the environment of the `git` it starts, and a
  `git config` that itself fails. It asked no helper and sent no credential.
  `git config --list --show-origin` shows what git reads: reduce it or fix the
  include it names, then retry.
- **`The server asks for a user name and password (Basic) and no git credential
  helper is configured`** (Windows). Configure one, for example
  `git config --global credential.helper manager` (`manager` is Git Credential
  Manager, which comes with Git for Windows), and retry.

If `git` is missing or cannot execute, repair the captured `PATH`/installation
and start a new operation. If the credential is rejected, renew it where it is
stored and retry. GWZ does not run credential `store` or `erase` and does not
automatically switch to the native transport. The candidate's
`gwz --transport native fetch` permits an explicit comparison; use
`--transport gwz` or remove your override to return. See
[authentication](commands/auth.md) for precedence and persistent setting removal,
and the [core helper guide](https://github.com/owebeeone/gwz-core/blob/main/docs/CredentialHelpers.md)
for timeout and configuration details.

## SSH Or Credential Failure

Symptoms:

- Network operations fail authentication.
- A host read stalls or times out.

Checks:

```sh
ssh -T git@github.com
gwz --ssh-timeout 10 pull --head
```

Recovery:

- Confirm SSH agent keys or HTTPS credentials.
- Increase `--ssh-timeout <secs>` for slow networks.
- Use `--jobs` and `--max-per-host` to reduce concurrency against a host. If
  gwz already printed `note:` lines about using fewer connections, or a
  repository failed after a host kept refusing connections, see
  "Fewer connections than --max-per-host, or a Capacity error" below.
- To publish over HTTPS from a workspace cloned over SSH, switch its remotes as
  [Publication](Concepts.md#publication) describes.
- For public repositories on github.com, gitlab.com, or bitbucket.org, clone or
  materialize over HTTPS instead: `gwz clone --url-scheme https <url>`, or
  `GWZ_URL_SCHEME=https gwz materialize --lock`. This changes only the URLs used
  for repositories this run clones; members already checked out keep their
  remotes, and the manifest is not rewritten.

- If the message says an SSH key file "is protected by a passphrase", see
  [An SSH Key File Is Protected By A Passphrase](#an-ssh-key-file-is-protected-by-a-passphrase).

Two messages point at that remedy:

- No usable SSH identity on a known host; the message ends with
  `; for public repositories, retry with --url-scheme https or set GWZ_URL_SCHEME=https`.
- `invalid or unknown remote ssh hostkey`; the message ends with
  `; run ssh -T git@<host> once to record the host key, or retry with --url-scheme https`.

A URL on a known host that cannot be converted, such as one with a nonstandard
port, an `http://` URL, or an empty path, is refused with
`UrlSchemeUnavailable` before anything is fetched. Use `--url-scheme manifest`
for that run, or record a remote in the form you want and run
`gwz repo sync <member-path>`. `--remote-identity NAME=PATH` names an SSH
identity, so a clone, fetch, push or tag that would reach that remote over
HTTPS refuses it before any network access, including a push whose root proof
reads a dependency through that remote. The message ends with
`; use --identity PATH, which non-SSH destinations ignore, or no override for that remote`.

## An SSH Key File Is Protected By A Passphrase

Symptoms:

- A clone, fetch, pull or push over SSH fails, on any platform, with an
  `UnsupportedOperation` error that starts:
  `The SSH key file <path> is protected by a passphrase, and gwz cannot ask for a passphrase, so it cannot use the file.`
- `<path>` is the key file you named with `--identity`, `--remote-identity` or a
  remote's `gwzSshIdentity` setting. The message is given before gwz connects, and
  nothing is retried.

Cause: gwz reads a key file you name without a passphrase and cannot ask you for
one, so it cannot use a key file that has one. gwz recognizes such a file by its
`Proc-Type: 4,ENCRYPTED` or `DEK-Info:` header, an `ENCRYPTED PRIVATE KEY` block,
or an OpenSSH-format key whose cipher is not `none`.

Recovery, either way:

- Load the key into an SSH agent, and name no key file. A named key file is the only
  key gwz offers, and it does not turn to the agent, so drop `--identity` and
  `--remote-identity`, and unset the remote's setting with
  `gwz auth identity <remote> --unset`. The agent asks for the passphrase when you
  add the key:

  ```sh
  ssh-add <path>
  ```

  On Windows the agent is the OpenSSH Authentication Agent service (`ssh-agent`),
  which is disabled on many machines and which an administrator starts, or Pageant,
  into which you load the key with "Add Key".
- Or remove the passphrase from a copy of the key, and name the copy:

  ```sh
  cp <path> <path>-nopass
  ssh-keygen -p -f <path>-nopass
  ```

  `ssh-keygen` asks for the old passphrase and then for a new one: press Enter for
  the new one. The copy is unprotected, so keep it private and delete it when you no
  longer need it. On Windows gwz reads a key file only if it is an RSA key in PEM
  form, so for an RSA key use `ssh-keygen -p -m PEM -f <path>-nopass`; an ECDSA or
  Ed25519 key can only come from an agent there.

## Fewer connections than --max-per-host, or a Capacity error

In the transport build (planned for 1.1.0), `--max-per-host` is a ceiling, not
a promise. Servers cap how many connections one account or address may hold,
and other programs on the same account or address count against that cap. When
a host refuses extra connections, gwz uses fewer at a time, keeps checking
whether it may use more, and raises the number again when the host allows it.
The number is kept for each scheme and port of a host, so SSH and HTTPS to one
host are separate, and it never goes above `--max-per-host` (default 32). It
lasts for one command only; the next command starts at the ceiling again.

### The notes

gwz says what it is doing in lines that start with `note: `, on stderr, one
line per event for the host, not one per repository:

```text
note: github.com:443 refused extra connections; using 8 at a time (ceiling 32 from --max-per-host); continuing, no action needed
note: github.com:443: now using 12 connections
note: github.com:443: back at the ceiling of 32 connections
note: github.com:443 asked gwz to wait 20 s; waiting
note: github.com:443 asked gwz to wait 120 s; one command waits at most 30 s
note: github.com:22 is not answering; waiting up to 30 s to retry
```

- The first line means the host refused extra connections, so gwz is using
  fewer, and the command continues. No action is needed. The next two lines
  follow when the host lets gwz use more, at most one change line per host
  every 2 seconds.
- The fourth line means the host sent a `Retry-After` of more than 1 second and
  gwz is waiting for it. gwz waits at most 30 seconds at a time for one host.
  The fifth line is a `Retry-After` of more than that: it says what the host
  asked and what gwz waits, and a repository that is still refused after the
  30 seconds fails with the "asked to wait" message below.
- The last line means the host did not answer gwz's connection attempts.
  Repositories on that host wait up to 30 seconds and then try once more. A
  repository that carried a retry and failed again reports the same failure
  with `; still failing after a retry about 30 s later` after it. If two retries
  in a row fail, the remaining repositories on that host fail at once, with the
  failure they share inside `not attempted: <host:port> failed earlier in this
  run (<the reason>)`. Check the host and your network, then run the command
  again. For an SSH host whose connections stalled:

  ```text
  not attempted: github.com:22 failed earlier in this run (ssh setup timeout: no progress for 9 seconds (--ssh-timeout 9))
  ssh setup timeout: no progress for 9 seconds (--ssh-timeout 9); still failing after a retry about 30 s later
  ```

  Neither line has an attempt count: the repository made no attempt of its own,
  or shares the retry's failure. They keep the error code of the failure they
  share, never `Capacity`.

A `note:` line is information, never a failure. A failure line starts with
`gwz: `. The notes are printed in human output, also when stderr is not a
terminal, and they are left out of `--json`, `--jsonl` and `--porcelain` output.
A single refused connection that a retry fixes prints nothing.

### When a repository fails

A repository gives up when it has used all of its attempts, which is
`--max-retries` plus one (4 by default), and a refusal by the host counts as an
attempt. Its error is the error of its last failure, and ends with
`(attempt N of M)`, where M is the number of attempts allowed. There are three
Capacity messages. Each starts with `member '<id>' at '<path>': `. The error
code is `Capacity` (in `--json`, `error.code`), so a script can tell "slow down,
try later" from a Git failure without reading the message; see
[Machine Output](MachineOutput.md) ("The `Capacity` error code").

The host refused connections:

```text
github.com:443 kept refusing connections (using 8 at a time, ceiling 32 from --max-per-host); gave up after 21 s; retry later, or lower --max-per-host (attempt 4 of 4)
github.com:443 refused the last attempt (using 8 at a time, ceiling 32 from --max-per-host); gave up after 21 s; retry later, or lower --max-per-host (attempt 4 of 4)
```

This is a 429, or a 503 with `Retry-After`, and it is decided by the
repository's last attempt: its last attempt was refused. The first line is said
when more than one of its attempts was refused. The second is said when the
last attempt was the only one refused, as when earlier attempts failed in other
ways (a reset, a stall), or at `--max-retries 0`, where the first refusal is
the only attempt. The part in brackets is left out when gwz never had to lower
its number. Run the command again later. If the account or address is shared
with other jobs, lower `--max-per-host` (for example `--max-per-host 8`) so
that this command leaves room for them. `--max-retries` gives repositories more
attempts.

The host asked for a long wait:

```text
github.com:443 asked to wait 120 s, longer than the 30 s one command holds a host; retry in 2 min (attempt 1 of 4)
```

One command waits at most 30 seconds for a host, so a repository whose host
asks for longer stops. Run the command again after the time it names.

gwz waited for one of its own limits until its time ran out. The message names
the limit that was full, and how long gwz waited:

```text
gwz's 64 connection-setup slots stayed full for 30 s (setups that stopped answering hold them until they time out); retry later
gwz's 8 slots for resolving a host and setting up TLS stayed full for 30 s; retry later
every connection gwz may open at once was in use for 30 s (--max-per-host limits them per host, and a total limit applies across hosts); retry later, or raise --max-per-host
```

The first line is the fixed limit of 64 connection setups at once (not
`--jobs`). It is full when many setups have stopped answering and hold their
slot until they time out, which points at a host or network that is not
answering. A lower `--ssh-timeout` makes stuck SSH setups give up sooner. The
second is the 8 slots that resolve a host name and set up TLS: a name lookup or
a TLS setup that stopped answering holds one. The third means other members
held every connection gwz may have open at once: they are healthy, and the
command only has to wait for them, or `--max-per-host` can be raised. These
messages have no attempt count. Retry later, and check the host.

Other failures keep the message they had: a bare 503 without `Retry-After`, a
reset, a refused connect or a stalled setup is reported as that failure, also
when the host's limit caused it. SSH servers that cap unauthenticated
connections (OpenSSH's `MaxStartups`) usually drop the connection, so a failure
there is reported as the drop. If gwz had lowered its number, a `note:` line
says so. An error about a host that stopped answering is reported with the
reason for that failure, never as a Capacity error.

### Turning the adjusting off

`--max-retries 0` turns it off. gwz then never lowers the number of
connections because of a refusal, a repository whose connection is refused
fails at once, and a `Retry-After` is still waited for. `--transport native`
has no adjusting either: it is the transport of gwz 1.0 with the fixed
`--max-per-host` (8 by default there).

## Push Refuses Root Publication

Symptoms:

- `gwz push` rejects the root with
  `root publication blocked: cannot prove member <id> commit <commit> is available at its committed fetch remote <remote>`,
  followed by `(read through <url>)` when the proof read another URL.
- A push that needs the URL scheme for a lock member that is not checked out
  refuses with `workspace URL-scheme preference <path> is unreadable: <detail>; delete or repair the file`.

Meaning:

- A push that contacts the root proves every member commit the committed lock
  names before the root transfer. Member pushes that already succeeded stay
  published, and the root remote is unchanged.

Recovery:

- Publish the member by pushing a branch that contains the commit.
- If someone else pushed to that member since your last fetch, fetch it (for
  example `git -C <member-path> fetch`) and retry, so GWZ can see that the
  remote's newer history contains the commit.
- Run `gwz push --check-remotes`, which re-checks the members and pushes those
  whose remote lacks their branch's commit.
- Repair `.gwz/url-scheme.yml` to keep the workspace's URL scheme, or delete it;
  members that are not checked out are then read, and later cloned, at their
  manifest URLs.

## Push Skips A Member Whose Remote Changed

Symptoms:

- `gwz push` reports a member `Noop` with
  `up to date with origin/<branch> as of the last fetch or push`, but its
  remote branch was rewound or deleted.

Meaning:

- By default a push decides from the member's remote-tracking ref, which
  records what the last fetch or push saw. A fetch repairs a rewound branch.
  GWZ does not prune, so unless `fetch.prune` is set, a branch deleted on the
  remote keeps its tracking ref, and its member stays up to date, through
  `gwz pull` and plain `git fetch`.
- A root that needs that member's commit is still refused when it is published,
  because its proof reads the remote.

Recovery:

- Run `gwz push --check-remotes`, which reads every remote and pushes what is
  missing.
- Drop the stale ref with `git -C <member-path> fetch --prune`, or set
  `fetch.prune`.
- Recreate the branch with a push outside the default check, for example
  `git -C <member-path> push origin <branch>`.

## Sync Rejected

Symptoms:

- Pull refuses because the default fast-forward policy cannot apply cleanly.

Recovery:

```sh
gwz --dry-run pull --head
gwz --sync fetch-only pull --head
```

Inspect the member state, then choose an explicit sync policy if needed:
`ff-only`, `merge`, `rebase`, `reset`, or `driver-selected`.

## Conflicts

Symptoms:

- An operation exits non-zero with conflicted or failed aggregate status.

Recovery:

1. Run `gwz status`.
2. Resolve conflicts in affected member repositories.
3. Stage and commit or otherwise finish the member-level Git operation.
4. Rerun the GWZ command or materialize the intended target.

## A Merge Will Not Finish Or Close

Symptoms:

- Mutating commands refuse with `merge '<merge-id>' is open`.
- `gwz merge --continue` or `gwz merge --abort` refuses, or `--abort` reports
  success and the affected files still look wrong to plain `git`.
- A recovery checkout is refused because a path is covered by a configured
  content filter.
- Every merge verb refuses with `this is a pre-0.14 merge`. Nothing is broken:
  the merge was started by GWZ 0.13 or earlier and 0.14 cannot act on it. Close
  it with a 0.13.x build, then return to 0.14.

Recovery:

- Follow the [Merge Recovery Runbook](MergeRecovery.md). It identifies each
  case by the message it prints, says what was and was not changed, and gives
  the manual procedure for a merge no command can close.
- Collect `gwz merge --status` output before changing anything, and do not
  delete merge records, `refs/gwz/` refs, `gwz:`-prefixed stashes, or stash
  bundles.

## A Local Clone Will Not Dispose

Symptoms:

- `gwz local dispose <name>` refuses with `UnwaivedHazard`, naming
  `open-merge`, `dirty` or `unpreserved-history` per repository.
- It refuses with `UnknownEvidence`, typically because the lane holds a GWZ
  stash record.

Meaning:

- Deletion is allowed only when the lane's history is verifiably preserved in
  another surviving family member. A clean working tree is not proof, and
  neither is a push to a network remote.
- After `gwz merge --remote <name>`, the lane's **root** commits are usually
  the unpreserved ones: the default merge selects members only, and every
  `gwz commit` in the lane produced a root commit.

Recovery:

- Preserve first, then dispose: `gwz --target @root merge --remote <name>`
  for the root, and a fetch into a surviving member for anything only a
  network remote holds.
- Pop or drop a GWZ stash in the lane; unknown evidence cannot be waived.
- Keep the tree and only forget the lane: `gwz local dispose <name> --keep`.
- Accept the loss explicitly: `gwz local dispose <name> --force <hazard,...>`,
  naming every hazard the refusal reported.

See [Local Clones](LocalClones.md#the-deletion-position-in-plain-terms).

## Machine Output Looks Unexpected

Checks:

```sh
gwz --json status
gwz --jsonl pull --head
gwz status --porcelain
```

Notes:

- `--json` and `--jsonl` cannot be combined.
- `status --porcelain` cannot be combined with `--json` or `--jsonl`.
- `forall` rejects `--json` and `--jsonl` because child process output streams
  directly.

### Filesystem capabilities and recovery

Windows recovery admission checks open-by-file-ID capability, a successful
nonzero 128-bit identity query and a local volume GUID, along with required
case-mode and handle probes. Filesystem names are diagnostic labels. An
unavailable name does not disable otherwise supported recovery. Missing
capabilities warn and allow ordinary `--no-ff` merges without crash recovery;
`--filesystem-strict` refuses. CLI and Python use the same core decision.
Successful block cloning proves a separate copy capability, not recovery
support.
