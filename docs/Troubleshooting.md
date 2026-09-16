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
- Use `--jobs` and `--max-per-host` to reduce concurrency against a host.
- To publish over HTTPS from a workspace cloned over SSH, switch its remotes as
  [Publication](Concepts.md#publication) describes.
- For public repositories on github.com, gitlab.com, or bitbucket.org, clone or
  materialize over HTTPS instead: `gwz clone --url-scheme https <url>`, or
  `GWZ_URL_SCHEME=https gwz materialize --lock`. This changes only the URLs used
  for repositories this run clones; members already checked out keep their
  remotes, and the manifest is not rewritten.

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
