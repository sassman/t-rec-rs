# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 🎯 [Unreleased]

## [0.3.0] - XXXX
### Added
- command line parameter `-d` or `--decor` that allows to turn on and off effects [feat(decor:shadow)]
- command line parameter `-v` or `--verbose` that shows insights on the window name and window id for the curious
- turn on the new shadow decor effect by default 
![demo](./docs/demo-shadow.gif)

[feat(decor:shadow)]: https://github.com/sassman/t-rec-rs/commit/65ceeb2bb23da9243b57152755c865d8950af3c7

### Fixed
- white corners are now fixed and aligned with the radius of macos big sur [feat(decor:big-sur)]
- sometimes there were unexpected image dimensions, with a small stripe on the right of black pixel [feat(decor:big-sur)]

[feat(decor:big-sur)]: https://github.com/sassman/t-rec-rs/commit/ab64d3dd8e9b264b617acca9a0db9375525911d0

## [0.2.2] - 2020-11-26
### Fixed
- improve error handling for invalid captured image data [pull/15]
  
[pull/15]: https://github.com/sassman/t-rec-rs/pull/15

## [0.2.1] - 2020-11-17
### Fixed
- improve error handling for invalid window id [issue/13] / [pull/14]
  
[issue/13]: https://github.com/sassman/t-rec-rs/issues/13
[pull/14]: https://github.com/sassman/t-rec-rs/pull/14

## [0.2.0] - 2020-10-12
### Added
- command line parameter for natural recording `-n` or `--natural`
- feature to avoid identical frames, where nobody sees some progress happening [issue/10] / [pull/11]
  
[issue/10]: https://github.com/sassman/t-rec-rs/issues/10
[pull/11]: https://github.com/sassman/t-rec-rs/pull/11

## [0.1.2] - 2020-10-12
### Added
- CHANGELOG.md follows now a [new format](https://keepachangelog.com/en/1.0.0/)
- feature to check for `convert` on launch [issue/6] / [pull/7]
- feature to avoid overwriting existing final gif [issue/8] / [pull/9]

[issue/6]: https://github.com/sassman/t-rec-rs/issues/6
[issue/8]: https://github.com/sassman/t-rec-rs/issues/8
[pull/7]: https://github.com/sassman/t-rec-rs/pull/7
[pull/9]: https://github.com/sassman/t-rec-rs/pull/9

## [0.1.1] - 2020-10-11
### Fixed
- Segmentation fault on listing the windows `t-rec -l` [issue/4]

## [0.1.0] - 2020-10-10
### Added
- Basic recoding functionality with 4 FPS
- Generating a gif out of n frames of a recording
- CI pipeline as GitHub Actions workflow

[issue/4]: https://github.com/sassman/t-rec-rs/issues/4