# claude-code

Set this machine up to run GWZ's Claude Code hooks.

## `gwz claude-code setup`

Prints the hooks block, and with `--write` merges it into a settings file:

```sh
gwz claude-code setup --project                 # print the block
gwz claude-code setup --project --local --write # merge into settings.local.json
gwz claude-code setup --user --write            # merge into ~/.claude/settings.json
```

The block carries one `WorktreeCreate` handler and one `WorktreeRemove`
handler, each with its own timeout. Inside a workspace the timeouts and the
handlers' `--wait-secs` are computed from the same run-time estimate the create
hook uses — one wait plus one estimated copy plus 60 s for creation, one wait
plus 60 s for removal. Outside a workspace they are the compiled-in defaults.

The default handler is the bare command `gwz hook ...`, resolved through
`PATH`, so a committed project block is machine-independent and identical
everywhere, which is what Claude Code's same-handler dedupe keys on.
`--command PATH` pins an absolute binary instead, for a machine whose desktop
app cannot see `gwz` on its `PATH`. Any hook option passed to `setup` is baked
into the handler text.

## Writing

`--write` edits another program's configuration, so it:

- parses the existing file first and refuses one that does not parse, is a
  symbolic link, or is not a regular file;
- writes a temporary file beside the target, fsyncs it, re-parses it and
  renames it over the original;
- changes no byte outside the inserted block;
- creates the file when it is absent;
- does nothing when the block is already there.

## Placements

One placement is the recommendation. A block may sit in both a project's
settings and a user's: Claude Code runs an identical handler once, so identical
text is harmless. Two *differing* handlers both run, in parallel, for the same
name and session; `setup` warns, naming both files, when it finds one. They
stay harmless for creation — the second handler reuses the first's lane and
prints the same path — but a handler that refuses for a reason of its own makes
Claude abort the creation the other completed, and that lane is an orphan whose
inventory is `gwz local list`.

See [`gwz hook`](hook.md) for what the hooks themselves do.
