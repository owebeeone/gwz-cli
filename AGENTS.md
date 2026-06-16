# AGENTS

Follow the root `AGENTS.md` rules.

- Work TDD-first: failing test, implementation, green tests, then refactor.
- Keep CLI behavior thin; workspace semantics belong in `gwz-core`.
- Do not call Git directly from the CLI.
- Do not read or write GWZ artifacts directly from the CLI.
