# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to cargo's version of [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

- [Unreleased](#unreleased)
- [v0.12.0](#v0120)
- [Diffs](#diffs)

## Unreleased
#### Updated
- `winit` to `0.24`

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
- GLSL shaders and `glsl-to-spirv`. If you want a custom shader, provide custom spirv to `RendererConfig::with_shaders()`, however you generate it.

## Diffs

- [Unreleased](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.12.0...HEAD)
- [v0.12.0](https://github.com/Yatekii/imgui-wgpu-rs/compare/v0.11.0...v0.12.0)
