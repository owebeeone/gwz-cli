# gws-cli

`gws-cli` provides the `gws` command-line driver for `gws-core`.

The CLI is intentionally thin: it parses argv, builds GWS requests, calls
`gws-core`, and renders responses/events.

## Development

```text
cargo test
cargo run -- --version
```
