# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **YAML editor screen** (`3`): open a `.yaml`/`.yml` file (browse or type a
  path, or `--file`), navigate it as a collapsible tree, select an environment,
  and encrypt/decrypt individual scalar values **in place** — comments, order
  and formatting are preserved (only the selected value token changes).
  Encrypted values use the Mule `![…]` wrapper and are masked (reveal with `r`).
  Manual scalar editing, atomic save (`Ctrl-s`), restore-to-opened (`Ctrl-r`)
  with confirmation, a dirty indicator, external-modification detection, and
  node-targeted background crypto that ignores stale results.

## [0.1.1] - 2026-08-04

### Fixed

- The header no longer clips the app name — it showed as `azyprop` instead of
  `lazyprop`.
- Replaced the `▶` selection marker (environments list and form fields) with an
  ASCII `>` so it renders correctly on Windows.

## [0.1.0] - 2026-07-30

First release.

### Added

- **Main screen** — pick an environment, type a value, and encrypt or decrypt
  it with MuleSoft's Secure Properties Tool; add, edit, delete and search
  environments (persisted to `envs.yaml`); reveal/hide keys; copy results to the
  clipboard.
- **Playground screen** — ad-hoc encrypt/decrypt with parameters entered
  directly, no saved environment.
- **About screen** — logo, keybindings with friendly descriptions, and app info.
- Algorithm-aware cipher-mode selection (only valid modes are offered).
- Encrypt/decrypt runs off the UI thread with a `Working…` indicator.
- Text inputs with a real cursor and horizontal scrolling.
- Self-contained binary: the jar is embedded and extracted to `~/.lazyprop` on
  first run; environments resolve via `--envs` / `LAZYPROP_ENVS` /
  project-local `./envs.yaml` / `~/.lazyprop/envs.yaml`.
- Cross-platform (macOS, Linux, Windows) with CI and release binaries.

[0.1.1]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.1.1
[0.1.0]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.1.0
