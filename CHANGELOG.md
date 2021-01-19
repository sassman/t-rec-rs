# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 🎯 [Unreleased]

## [0.4.3] - 2021-01-05
### Added
- Quiet (-q|--quiet) mode to suppress Ctrl+D banner: 'Press Ctrl+D to end recording' [pull/39](https://github.com/sassman/t-rec-rs/pull/39), thanks to [@Daviey](https://github.com/Daviey)
### Changed
- Changelog now contains the release links

## [0.4.2] - 2021-01-04
### Added
- ArcoLinux 5.4 on Xfwm4 to the list of tested distros

![demo-arco](./docs/demo-arco-xfwm4.gif)

### Fixed
- fixed issues on terminals with transparency (or where the compositor caused transparency) on Linux see [issue/26](https://github.com/sassman/t-rec-rs/issues/26) / [pull/38](https://github.com/sassman/t-rec-rs/pull/38)

## [0.4.1] - 2021-01-03
### Added
- Snap support on it's way to [snapcraft.io](https://snapcraft.io/t-rec) [pull/25], thanks to [@popey](https://github.com/popey)
### Fixed
- reduced crate size from 4.8MB to 34kB [pull/32], thanks to [@Byron](https://github.com/Byron)
- fixed a panic when the active window cannot be identified on Linux [pull/31] / [issue/30]
- fixed `t-rec -l` did not show any window names on Linux [pull/31]
- fixed system freeze on "Applying Effects" caused by too many threads [issue/29]

[pull/32]: https://github.com/sassman/t-rec-rs/pull/32
[pull/31]: https://github.com/sassman/t-rec-rs/pull/31
[pull/25]: https://github.com/sassman/t-rec-rs/pull/25
[issue/30]: https://github.com/sassman/t-rec-rs/issues/30
[issue/29]: https://github.com/sassman/t-rec-rs/issues/29

## [0.4.0] - 2020-12-27
### Added
- t-rec runs now on linux (X11 only) [issues/1] and has been tested on the following systems:
  - ubuntu 20.10 on GNOME ![demo-ubuntu](./docs/demo-ubuntu.gif)
  - ubuntu 20.10 on i3wm ![demo-ubuntu-i3wm](./docs/demo-ubuntu-i3wm.gif)
  - mint 20 on cinnamon ![demo-mint](./docs/demo-mint.gif)
  
[issues/1]: https://github.com/sassman/t-rec-rs/issues/1

### Fixed
- clear screen before starting the recording was somehow broken, it behaves now better

## [0.3.1] - 2020-12-18
### Added
- Readme badge for dependencies and latest version on crates.io
### Fixed
- updated dependencies

## [0.3.0] - 2020-12-07
### Added
- command line parameter `-d` or `--decor` that allows to turn on and off effects [issues/18] / [pull/19]
- command line parameter `-b` or `--bg` that allows to change the target background color to white, black or transparent [pull/19]
- command line parameter `-v` or `--verbose` that shows insights on the window name and window id for the curious [pull/19]
- turn on the new shadow decor effect by default [pull/19]
![demo](./docs/demo-shadow.gif)

[pull/19]: https://github.com/sassman/t-rec-rs/pull/19
[issues/18]: https://github.com/sassman/t-rec-rs/issues/18

### Fixed
- white corners are now fixed and aligned with the radius of macos big sur [issues/17] / [pull/19]
- sometimes there were unexpected image dimensions, with a small stripe on the right of black pixel [pull/19]

[pull/19]: https://github.com/sassman/t-rec-rs/pull/19
[issues/17]: https://github.com/sassman/t-rec-rs/issues/17

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

[Unreleased]: https://github.com/sassman/t-rec-rs/tree/v0.4.3...HEAD
[0.4.3]: https://github.com/sassman/t-rec-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/sassman/t-rec-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/sassman/t-rec-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/sassman/t-rec-rs/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/sassman/t-rec-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/sassman/t-rec-rs/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/sassman/t-rec-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/sassman/t-rec-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/sassman/t-rec-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/sassman/t-rec-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/sassman/t-rec-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sassman/t-rec-rs/compare/v0.1.0-beta.1...v0.1.0
[0.1.0-beta.1]: https://github.com/sassman/t-rec-rs/compare/v0.1.0-beta.1...v0.1.0
