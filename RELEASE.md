# Release Process

This document describes how to publish a new release of `imgui-wgpu`.

## Prerequisites

- Push access to the `master` branch
- A crates.io API token with publish rights for `imgui-wgpu`
- `gh` CLI installed and authenticated (for creating the GitHub release)

## Steps

### 1. Determine the new version

Pick the new version number following cargo semver conventions. For this document,
we'll use `X.Y.Z` as a placeholder (e.g. `0.26.0`).

### 2. Update CHANGELOG.md

Make the following edits to `CHANGELOG.md`:

**a) Add the new version to the Table of Contents:**

Find the line:
```
- [Unreleased](#unreleased)
```
Add a new entry directly below it:
```
- [vX.Y.Z](#vXYZ)
```
(The anchor is the version with dots removed, e.g. `v0.26.0` -> `#v0260`)

**b) Add a version heading under Unreleased:**

Find:
```
## Unreleased
```
Add a blank line and a new version section below it, moving all existing unreleased
items under the new heading:
```
## Unreleased

## vX.Y.Z

Released YYYY-MM-DD

- (move all previously unreleased items here)
```

**c) Update the Diffs section at the bottom:**

Find the existing unreleased diff link:
```
- [Unreleased](https://github.com/Yatekii/imgui-wgpu-rs/compare/vPREVIOUS...HEAD)
```
Update it and add a new entry:
```
- [Unreleased](https://github.com/Yatekii/imgui-wgpu-rs/compare/vX.Y.Z...HEAD)
- [vX.Y.Z](https://github.com/Yatekii/imgui-wgpu-rs/compare/vPREVIOUS...vX.Y.Z)
```

### 3. Update Cargo.toml

Set the `version` field to the new version:
```toml
version = "X.Y.Z"
```

### 4. Update README.md

**a) Update the dependency snippet** in the "Getting Started" section to match the
new versions:

```toml
[dependencies]
imgui-wgpu = "X.Y"
imgui = "I"
wgpu = "W"
```

**b) Update the version compatibility table.** Change the `master` row to the new
version with the current `wgpu` and `imgui` dependency versions from `Cargo.toml`:

```
| imgui-wgpu | wgpu   | imgui |
|------------|--------|-------|
| X.Y.Z      | W      | I     |
```

If there are already unreleased changes on the branch, add a new `master` row above it.

### 5. Commit and tag

```bash
git add Cargo.toml CHANGELOG.md README.md
git commit -m "Release vX.Y.Z"
git tag vX.Y.Z
git push origin master --tags
```

### 6. Publish to crates.io

```bash
cargo publish
```

### 7. Create the GitHub release

Extract the release notes (the bullet points under the new version heading in
`CHANGELOG.md`) and create a release:

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "$(NOTES)"
```

Or interactively:

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --generate-notes
```

Then edit the release body to match the changelog entry.

### 8. Post-release

Verify:
- [ ] The crate is visible at https://crates.io/crates/imgui-wgpu/X.Y.Z
- [ ] Docs are building at https://docs.rs/imgui-wgpu/X.Y.Z
- [ ] The GitHub release exists at https://github.com/Yatekii/imgui-wgpu-rs/releases/tag/vX.Y.Z
