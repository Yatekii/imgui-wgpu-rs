# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to cargo's version of [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per Keep a Changelog there are 6 main categories of changes:
- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security

#### Table of Contents

- [Unreleased](#unreleased)
- [v0.16.0](#v0160)
- [v0.15.1](#v0151)
- [v0.15.0](#v0150)
- [v0.14.0](#v0140)
- [v0.13.1](#v0131)
- [v0.13.0](#v0130)
- [v0.12.0](#v0120)
- [Diffs](#diffs)

## Unreleased

#### Updated
- updated `wgpu` to 0.10

#### Fixed
- Internal: fix all warnings from static analysis (clippy).
- Internal: Do not render draw commands that fall outside the framebuffer
- Internal: Avoid wgpu logic error by not rendering empty clip rects

## v0.16.0

Released 2021-07-14

#### Added
- Internal: Vastly improved CI and release process.
- Internal: PR and Issue Templates

#### Changed
- Examples: Use `env_logger` instead of `wgpu-subscriber`
- Examples: Use `pollster` as block_on provider instead of `futures`

#### Fixed
- Rendering to multi-sampled images no longer errors.
- Examples: Simple API examples now properly depend on that feature existing.

#### Updated
- updated `wgpu` to 0.9

## v0.15.1

Released 2021-05-08

#### Fixed
- removed hack due to wgpu bug

#### Updated
- updated `wgpu` to 0.8.1

## v0.15.0

Released 2021-05-08

#### Updated
- updated `wgpu` to 0.8

## v0.14.0

Released 2021-02-12

#### Updated
- updated `imgui` to 0.7

## v0.13.1

Released 2021-02-01

#### Fixed
- Readme

## v0.13.0

Released 2021-02-01

#### Added
- Add experimental simple api behind feature `simple_api_unstable`
- Implemented `std::error::Error` for `RendererError`

#### Updated
- updated to `wgpu` 0.7
- support `winit` 0.24 as well as 0.23

## v0.12.0

Released 2020-11-21

#### Added
- A changelog!
- Shaders are now SRGB aware. Choose `RendererConfig::new()` to get shaders outputting in linear color
  and `RendererConfig::new_srgb()` for shaders outputting SRGB.

#### Updated
- `imgui` to `0.6`.
- `winit` to `0.23`

#### Removed
- GLSL shaders and `glsl-to-spirv`. If you want a custom shader, provide custom spirv to `RendererConfig::with_shaders()`, however you must generate it.

## Diffs

- [Unreleased](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.16.0...HEAD)
- [v0.16.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.15.1...v0.16.0)
- [v0.15.1](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.15.0...v0.15.1)
- [v0.15.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.14.0...v0.15.0)
- [v0.14.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.13.1...v0.14.0)
- [v0.13.1](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.13.0...v0.13.1)
- [v0.13.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.12.0...v0.13.0)
- [v0.12.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.11.0...v0.12.0)
