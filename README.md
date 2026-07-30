# lazyprop

[![CI](https://github.com/kchernokozinsky/lazyprop/workflows/CI/badge.svg)](https://github.com/kchernokozinsky/lazyprop/actions)

`lazyprop` is a lazygit-style terminal UI for working with **MuleSoft / Anypoint
secure properties**. You pick an environment (its algorithm, cipher mode and
key are read from a YAML file), type or paste a value, and encrypt or decrypt it
on the spot — a friendly front end over MuleSoft's `secure-properties-tool.jar`
so you never have to remember the Java command line.

## Requirements

- A Java runtime (`java` on your `PATH`) — the encryption itself is done by the
  MuleSoft Secure Properties Tool jar.
- The `secure-properties-tool.jar` (bundled in this repo).

## Installation

Build a single static binary with Cargo:

```bash
cargo build --release
# binary at ./target/release/lazyprop
```

Or install it into your Cargo bin directory:

```bash
cargo install --path .
```

## Usage

Run it from a directory that contains your `envs.yaml` and the jar (the defaults
resolve relative to the current directory):

```bash
lazyprop
```

Override the environment file or jar location explicitly:

```bash
lazyprop --envs ./config/envs.yaml --jar ./tools/secure-properties-tool.jar
```

There are two screens, shown as tabs in the header: **Main** and **About**.
Switch with `1` / `2` (or `?` to jump to About, `Esc` to come back).

Typical flow on the **Main** screen:

1. Select an environment in the **Environments** list with `w` / `s` (or the
   arrow keys).
2. Press `Tab` to focus the **Value** field and type/paste the text.
3. Press `Esc` (or `Enter`) to return to normal mode.
4. Press `e` to encrypt or `d` to decrypt — the result appears in the **Result**
   pane.

The **About** screen (`2` or `?`) shows a brief description, the full list of
keybindings, and where lazyprop keeps its files; scroll it with `w` / `s`.

### Searching environments

Press `/` to filter the environments list by name — type to narrow it down,
`Esc` (or `Enter`) to stop filtering while keeping the filter applied.

### Managing environments

- `a` opens a form to **add** a new environment.
- `Enter` (on the selected item) opens the same form to **edit** it.
- `x` **deletes** the selected environment after a `y`/`n` confirmation.

In the form, move between fields with `Tab` / `↑` `↓`, edit **Name** and **Key**
by typing, and cycle **Algorithm** / **Mode** / **Random IV** with `←` `→` (or
`Space`). `Enter` validates and **writes the change back to your environments
file**; `Esc` cancels.

### The environments file

Environments are described in a YAML file (default `envs.yaml`):

```yaml
environments:
  - name: DefaultEnv
    algorithm: AES        # AES | Blowfish | DES | DESede | RC2 | RCA
    state: CBC            # CBC | CFB | ECB | OFB
    use_random_ivs: false
    key: secret1234567890
```

`key` must be a valid length for the chosen algorithm (e.g. 16 characters for
AES). When `use_random_ivs` is `true`, the `--use-random-iv` flag is passed to
the tool.

## Keybindings

| Key         | Action                                   |
| ----------- | ---------------------------------------- |
| `s` / `Down`| Select next environment / scroll down    |
| `w` / `Up`  | Select previous environment / scroll up  |
| `1` / `2`   | Switch to the Main / About screen        |
| `Tab`       | Cycle focus (Environments ↔ Value)       |
| `/`         | Filter the environments list by name     |
| `e`         | Encrypt the current value                |
| `d`         | Decrypt the current value                |
| `a`         | Add a new environment                    |
| `Enter`     | Edit the selected environment            |
| `x`         | Delete the selected environment          |
| `Esc`       | Leave input / cancel / back to Main       |
| `?`         | Open the About / help screen             |
| `q`         | Quit                                     |
| `Ctrl-c`    | Quit                                     |
| `Ctrl-z`    | Suspend                                  |

Keybindings are configurable — see below. The interface uses ANSI-named colours
and dimmed secondary text, so it stays legible on both light and dark terminal
themes.

## Configuration

`lazyprop` reads its configuration from the first of these it finds in the
config directory (`config.json5`, `config.json`, `config.yaml`, `config.toml`,
`config.ini`), falling back to the bundled defaults.

- **Config directory**: `$LAZYPROP_CONFIG`, otherwise the platform config dir
  (e.g. `~/Library/Application Support/com.kchernokozinsky.lazyprop` on macOS,
  `~/.config/lazyprop` on Linux).
- **Data / log directory**: `$LAZYPROP_DATA`, otherwise the platform data dir.
  Logs are written to `lazyprop.log`; set `$LAZYPROP_LOG_LEVEL` (e.g. `debug`)
  to change verbosity.

The bundled config lives in [`.config/config.json`](.config/config.json) and
defines `jar_path`, `envs_path`, keybindings and styles.

## Development

```bash
cargo test                       # unit tests
cargo test -- --ignored          # also run the java round-trip test (needs a JRE)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run                        # run the TUI
```

## License

MIT — see [LICENSE](LICENSE).
