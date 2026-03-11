# Release Process

This document describes how to publish a new release of `imgui-wgpu`.

## Prerequisites

- Push access to the `master` branch
- A crates.io API token with publish rights for `imgui-wgpu`
- `gh` CLI installed and authenticated (for creating the GitHub release)

## Steps

### 1. Determine the new version

Pick the new version number following cargo semver conventions. For this document,
we'll use `X.Y.Z` as a placeholder (e.g. `0.28.0`).

### 2. Update CHANGELOG.md

**a) Add the new version to the Table of Contents:**

Find the line:
```
- [Unreleased](#unreleased)
```
Add a new entry directly below it:
```
- [vX.Y.Z](#vXYZ)
```
(The anchor is the version with dots removed, e.g. `v0.28.0` -> `#v0280`)

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

**b) Update the version compatibility table.** Add a new row for the release version
directly below the existing `master` row, with the current `wgpu` and `imgui` dependency
versions from `Cargo.toml`. Keep the `master` row as-is at the top:

```
| imgui-wgpu | wgpu   | imgui |
|------------|--------|-------|
| master     | W      | I     |
| X.Y.Z      | W      | I     |
```

### 5. Commit and tag

```bash
jj commit -m "Release vX.Y.Z"
jj tag create vX.Y.Z
jj git push
```

### 6. Publish to crates.io

```bash
cargo publish
```

### 7. Create the GitHub release

Extract the release notes from `CHANGELOG.md` and create a release:

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "<paste release notes here>"
```

### 8. Post-release

Verify:
- [ ] The crate is visible at https://crates.io/crates/imgui-wgpu/X.Y.Z
- [ ] Docs are building at https://docs.rs/imgui-wgpu/X.Y.Z
- [ ] The GitHub release exists at https://github.com/Yatekii/imgui-wgpu-rs/releases/tag/vX.Y.Z
