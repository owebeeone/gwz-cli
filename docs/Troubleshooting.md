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
