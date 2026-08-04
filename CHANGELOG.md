# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- YAML editor: a manually edited scalar value is now also written as a quoted
  string, consistent with encrypt/decrypt results.

## [0.2.1] - 2026-08-04

### Changed

- YAML editor: encrypt and decrypt results are now always written as a
  **quoted** string (e.g. `password: "secret"`), for consistency with the
  quoted cipher wrapper.

### Removed

- Mouse capture is disabled so the terminal's own text selection works
  everywhere. This removes the 0.2.0 scroll-wheel navigation and clickable
  header tabs; all navigation has keyboard equivalents (`w`/`s`, `h`/`l`,
  `1`–`4`).

## [0.2.0] - 2026-08-04

### Added

- **YAML editor screen** (`3`): open a `.yaml`/`.yml` file (browse or type a
  path, or `--file`), navigate it as a collapsible tree, select an environment,
  and encrypt/decrypt individual scalar values **in place** — comments, order
  and formatting are preserved (only the selected value token changes).
  Encrypted values use the Mule `![…]` wrapper and are masked (reveal with `r`).
  Manual scalar editing, atomic save (`Ctrl-s`), restore-to-opened (`Ctrl-r`)
  with confirmation, a dirty indicator, external-modification detection, and
  node-targeted background crypto that ignores stale results.
- **YAML: bulk encrypt/decrypt** (`E`/`D`) of every scalar under the selected
  subtree, **tree search** (`/`), **undo/redo** (`Ctrl-z`/`Ctrl-y`), and
  **add-environment** (`a`) without leaving the screen.
- **YAML: modified-property highlighting** — a subtle `●` marks each property
  whose value differs from the file as opened (containers flag a modified
  descendant); editing a value back to its original, saving, or restoring
  clears the mark.
- **Config-driven theme** — a `theme` section (`accent`/`success`/`error`)
  recolours the whole UI.
- **Mouse support** — scroll-wheel navigation and clickable header tabs.
- **Unsaved-changes guard** — leaving the YAML screen or opening another file
  with unsaved edits prompts to Save / Discard / Cancel.

### Changed

- **Contextual, consistent keyboard hints** — the footer now shows only the
  actions valid for the current screen, focus and mode from a single shared
  hint system, dropping the least important hints first as the terminal
  narrows. Confirmation and unsaved-changes popups always show their actions,
  even in tiny terminals, and never render off-screen.
- **About** is now a set of page-specific guides (Main, Playground, YAML,
  General) switched with `←`/`→`.
- Numeric screen shortcuts (`1`–`4`) still work but are no longer shown in the
  footer or the header tabs.

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

[0.2.1]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.2.1
[0.2.0]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.2.0
[0.1.1]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.1.1
[0.1.0]: https://github.com/kchernokozinsky/lazyprop/releases/tag/v0.1.0
