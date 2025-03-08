# Release Process

1. After a `feat`, `feat!`, `fix`, or `fix!`, merge the [release-please] PR
   after commiting a dummy commit to trigger CI. **Please remember to update
   `Cargo.lock` too!** (Remove this extra notice once you figure out how to have
   this be done automatically.)
2. After any further fixes are required and commited, run `cargo release
   --workspace --execute` from the project root.

## Multiple semantic changes in one release

After a `feat`, `feat!`, `fix`, or `fix!`, [release-please] will create a PR for
a new version following [semver]. A subsequent semantic change can explicit set
the version release field in `release-please-config.json` and remove it on the
CI-trigger commit to the [release-please] PR.

## Excluding crates

Any crate to be excluded from publishing should contain the following section in
its `Cargo.toml`:

```toml
[package.metadata.release]
release = false
publish = false
```

[release-please]: https://github.com/googleapis/release-please
[semver]: https://semver.org
