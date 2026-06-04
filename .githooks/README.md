Enable local pre-commit checks with:

```bash
git config core.hooksPath .githooks
```

Current hook:

- `pre-commit`: runs the required fast gates before commit
  - prefers `just pre-commit` when `just` is installed
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
