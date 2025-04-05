# Release Process

1. After a `feat`, `feat!`, `fix`, or `fix!`, merge the [release-please] PR
   after running `/bin/sh prepare-release.sh` from a named branch at project
   root.
2. After any further fixes are required and commited, `git pull` the main
   branch, switch to it, and run `cargo release --workspace --execute` from the
   project root.

## Multiple semantic changes in one release

After a `feat`, `feat!`, `fix`, or `fix!`, [release-please] will create a PR for
a new version following [semver]. A subsequent semantic change can explicit set
the version release field in `release-please-config.json` and remove it on the
CI-trigger commit to the [release-please] PR. However, multiple `fix`es in a row
will be considered part of the same minor patch bump

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
