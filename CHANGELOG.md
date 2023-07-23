# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- New configuration options for modes. These options apply to all keybindings in a mode.
- `swallow` mode option: all keybindings associated with this mode do not emit events
- `oneoff` mode option: automatically exits a mode after using a keybind
- `DESTDIR` variable for the `install` target in the `Makefile` to help
  packaging and installation. To install in a subdirectory, just call `make
  DESTDIR=subdir install`.
- Detection of added/removed devices (e.g., when plugging or unplugging a
  keyboard). The devices are grabbed by `swhkd` if they match the `--device`
  parameters if present or if they are recognized as keyboard devices otherwise.
- `Altgr` modifier added (https://github.com/waycrate/swhkd/pull/213).

### Changed

- The project `Makefile` now builds the polkit policy file dynamically depending
  on the target installation directories.
- Alt modifier no longer maps to the right aly key. It only maps to the left alt key. Right alt is referred to as Altgr (alt graph).
- Tokio version bumped from 1.23.0 to 1.24.2 (https://github.com/waycrate/swhkd/pull/198).

### Fixed

- Mouse cursors and other devices are no longer blocked when running `swhkd`.
- Option prefixes on modifiers are now properly parsed. e.g., `~control` is now
  understood by `swhkd` as the `control` modifier with an option
- Install mandocs in the correct locations.
