# AGENTS

Follow the root `AGENTS.md` rules.

- Work TDD-first: failing test, implementation, green tests, then refactor.
- Keep CLI behavior thin; workspace semantics belong in `gws-core`.
- Do not call Git directly from the CLI.
- Do not read or write GWS artifacts directly from the CLI.
