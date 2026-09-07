# SSH identity configuration

Use an identity for one invocation:

```sh
gwz push --identity ~/.ssh/work_key
gwz pull --remote-identity origin=~/.ssh/work_key
```

`--remote-identity NAME=PATH` can be repeated for different remote names. The
first `=` separates the name from the path, so paths may contain `=`. An override
for `origin` applies to every selected repository using that name. Unknown or
duplicate names refuse the operation. Root selection follows the command's
normal rules. Relative paths resolve against the invocation directory.

Save a setting in the selected repositories' local Git configuration:

```sh
gwz auth identity origin --set ~/.ssh/work_key
gwz auth identity origin --member @root --set ~/.ssh/workspace_key
gwz auth identity origin --all
gwz auth identity origin --unset
```

The configuration command defaults to active members; `--member @root` selects
only the root and `--all` selects root plus members. With neither `--set` nor
`--unset`, it reports the current local setting. `--dry-run` validates and shows
the proposed setting without writing. `--json` includes per-repository results.
Get and unset work even when the previously configured file is missing.

Precedence is per-remote invocation override, invocation default, local remote
setting, then the existing agent/helper behavior. Paths are stored as absolute
paths in repository-local Git configuration, never in `gwz.conf`. An SSH default
does not change HTTPS authentication. A named SSH override for a non-SSH
endpoint is rejected. Local-only commands refuse invocation identity options.

Explicit file selection offers only the selected key. If it is unavailable,
encrypted or rejected, GWZ cannot fall back to another agent key. Exact selection
of an encrypted key unlocked in an SSH agent remains unsupported by the pinned
native libraries. Configuring a path is not proof that it authenticates, and
successful authentication says nothing about Git author/committer attribution.

Transport results report the credential method and selection source, whether the
credential was offered, and whether authentication was proved. A failed request
may show `authenticated=unknown`: offering a key does not prove server acceptance.
These observations also appear in JSON and failed-operation output. Private-key
bytes and passphrases are never included; public fingerprints appear only when
known from the actual credential.

`--ssh-timeout SECONDS` sets the native connection/read timeout at process startup
(default: 3 seconds; 0 disables it). Python applications must configure it before
creating a native backend. Once fixed, later calls may repeat the same value but
cannot change the process setting while other requests could be running.
