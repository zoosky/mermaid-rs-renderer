# Release and crates.io Publish

This repository publishes binaries and the Rust crate from a version tag.

## One-time setup

Add `CARGO_REGISTRY_TOKEN` to GitHub repository secrets:

1. Create a crates.io API token with publish permission.
2. In GitHub: `Settings -> Secrets and variables -> Actions -> New repository secret`.
3. Name: `CARGO_REGISTRY_TOKEN`.

## Release checklist

1. Update version in `Cargo.toml` (for example `0.2.2`).
2. Update `CHANGELOG.md`.
3. Run local checks:
```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --no-default-features --lib
cargo test --doc --all-features
cargo publish --dry-run --locked
```
4. Commit and push to `master`.
5. Create and push a version tag that matches `Cargo.toml`:
```bash
git tag v0.2.2
git push origin v0.2.2
```

## What CI does on tag push

Workflow: `.github/workflows/release.yml`

- Builds release binaries for Linux/macOS/Windows.
- Uploads assets to GitHub Release.
- Publishes `mermaid-rs-renderer` to crates.io.
- Verifies tag version matches `Cargo.toml`.
- Skips publish if that exact crate version already exists.

## Verify publish

```bash
cargo search mermaid-rs-renderer --limit 1
cargo info mermaid-rs-renderer
```

## Manual fallback (if needed)

If CI publish fails but release is ready:

```bash
cargo login <token>
cargo publish
```
