# Releasing WorldHunt

WorldHunt publishes one Rust crate, `worldhunt`. GitHub Releases contain notes
only: there are no prebuilt binaries, archives, checksums, or installers.

## Prerequisites

- The release candidate has been reviewed and merged into `develop`.
- `make check` and `make release-check-clean` pass from a clean checkout.
- Repository Actions may create pull requests, and the `crates-publish` GitHub
  Environment exists with the required reviewer protection.
- For releases after `0.1.0`, crates.io Trusted Publishing is configured for
  this repository's `publish.yml` workflow and the `crates-publish` Environment.

## Release Flow

1. Run **Bump Version** from the Actions tab and choose `patch`, `minor`, or
   `major`. It always creates a draft version-bump pull request against
   `develop` and regenerates `Cargo.lock`.
2. Review, merge, and validate the bump pull request on `develop`.
3. Promote the approved commit from `develop` to `main`.
4. The **Release** workflow validates the clean package. When a push to `main`
   changes the root package version, it creates the immutable `v<version>` tag
   and a GitHub Release with generated notes. A manual **Release** dispatch is
   validation only; it never creates a tag or release.
5. For versions after `0.1.0`, run **Publish to crates.io** manually from
   `main`. It selects the latest stable GitHub Release, verifies its tag is an
   ancestor of `main`, validates the tag's package, and publishes through the
   protected `crates-publish` Environment.

Never move, delete, or reuse a release tag. If the release workflow is retried,
it accepts an existing tag only when that tag resolves to the current `main`
commit.

## Initial 0.1.0 Publication

The first publication precedes Trusted Publishing because crates.io must first
have a crate to associate with the repository. After `v0.1.0` exists and points
to the promoted `main` commit, use a temporary publish-only crates.io token on
the local machine:

```powershell
git fetch --tags origin
git switch --detach v0.1.0
make release-check-clean
cargo publish --locked -p worldhunt --token $env:CARGO_REGISTRY_TOKEN
```

Create `CARGO_REGISTRY_TOKEN` only for this command, restrict it to publishing
this crate, and revoke it immediately after the publish succeeds. Do not store
the token in GitHub, a local repository file, or shell history.

Then configure crates.io **Trusted Publishing** for GitHub repository
`jfrz38/worldhunt`, workflow `.github/workflows/publish.yml`, and environment
`crates-publish`. The workflow requests an OIDC token only in its protected
publishing job.

## Recovery

Crates.io versions are immutable. If a published version must be withdrawn,
yank it rather than changing its tag:

```powershell
cargo yank --version 0.1.0 --token $env:CARGO_REGISTRY_TOKEN
```

Publish a corrected, new version through the normal bump and release flow.
