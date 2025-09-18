# Releasing

1. Update the version in `Cargo.toml` (and update `Cargo.lock`, `cargo` will do
   this for you).
2. Tag and push the commit with the new version number. For example, `v0.3.4`
3. Wait for the CI to complete. This will build Docker images and publish
   them to Docker Hub with the correct tags.

This can be automated with this command:

```
cargo release <bump_level> --sign --no-publish --execute --verbose --push-remote upstream
```
