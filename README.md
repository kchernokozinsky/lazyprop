# lazyprop

[![CI](https://github.com/kchernokozinsky/lazyprop/workflows/CI/badge.svg)](https://github.com/kchernokozinsky/lazyprop/actions)

`lazyprop` is a lazygit-style terminal UI for working with **MuleSoft / Anypoint
secure properties**. You pick an environment (its algorithm, cipher mode and
key are read from a YAML file), type or paste a value, and encrypt or decrypt it
on the spot — a friendly front end over MuleSoft's `secure-properties-tool.jar`
so you never have to remember the Java command line.

Runs on **macOS, Linux and Windows**.

## Requirements

- A **Java runtime** (`java` on your `PATH`) — the encryption itself is done by
  MuleSoft's Secure Properties Tool. The jar is **embedded in the binary** and
  extracted to `~/.lazyprop` on first run, so you don't have to manage it.

## Installation

### macOS / Linux — Homebrew

```bash
brew install kchernokozinsky/tap/lazyprop
```

The formula pulls in a Java runtime automatically, so nothing else is needed.

### Windows — Scoop

```powershell
scoop bucket add lazyprop https://github.com/kchernokozinsky/scoop-bucket
scoop install lazyprop
```

If you don't have Java yet:

```powershell
scoop bucket add java
scoop install temurin-jre
```

### Windows — manual

1. Download `lazyprop-x86_64-pc-windows-msvc.zip` from the
   [Releases](https://github.com/kchernokozinsky/lazyprop/releases) page and
   extract `lazyprop.exe`.
2. Move it to a folder on your `PATH` (e.g. `%USERPROFILE%\bin`), or add its
   folder to `PATH` via *Settings → Environment Variables*.
3. Install a Java runtime if you don't have one — e.g.
   [Adoptium Temurin](https://adoptium.net/temurin/releases/) — and make sure
   `java` is on your `PATH`.
4. Open a new terminal and run `lazyprop`.

### Any platform — prebuilt binary

Download the archive for your platform from the
[Releases](https://github.com/kchernokozinsky/lazyprop/releases) page, extract
the `lazyprop` binary, and put it on your `PATH`. You still need `java`
available (see [Requirements](#requirements)).

### From source

```bash
cargo install --git https://github.com/kchernokozinsky/lazyprop
# or, from a checkout:
cargo build --release   # binary at ./target/release/lazyprop
```

## Usage

Just run it — on first launch it creates `~/.lazyprop/` with a sample
environments file and the extracted jar:

```bash
lazyprop
```

Point it at a specific environments file or jar if you like:

```bash
lazyprop --envs ./config/envs.yaml --jar /opt/secure-properties-tool.jar
```

There are four screens, shown as tabs in the header: **Main**, **Playground**,
**YAML** and **About**. Switch with `1` / `2` / `3` / `4` (or `?` to jump to
About).

Typical flow on the **Main** screen:

1. Select an environment in the **Environments** list with `w` / `s` (or the
   arrow keys).
2. Press `Tab` to focus the **Value** field and type/paste the text.
3. Press `Esc` (or `Enter`) to return to normal mode.
4. Press `e` to encrypt or `d` to decrypt — the result appears in the **Result**
   pane.

The **Playground** screen (`2`) is a one-off encrypt/decrypt form with no saved
environment — pick the Operation, Algorithm, State and Random-IV, type a Key and
Value, and press `Enter` to generate. Move between fields with `Tab` / `↑` `↓`,
change a choice with `←` `→`, and press `Esc` to return to Main.

The **About** screen (`4` or `?`) shows a brief description, the full list of
keybindings, and where lazyprop keeps its files; scroll it with `w` / `s`.

### The YAML editor (`3`)

Encrypt/decrypt individual values inside a `.yaml`/`.yml` file **in place**,
leaving comments, ordering and formatting untouched.

1. Press `3` to open the YAML screen, then `Ctrl-o` to open a file — either
   **browse** the filesystem or `Tab` to **type a path** (`~` is expanded).
   You can also start on it directly: `lazyprop --file ./config.yaml`.
2. The file is shown as a collapsible tree. Move with `w`/`s` (or arrows),
   fold/unfold with `←`/`→`, and `Tab` toggles focus between the environments
   list and the tree.
3. Select an environment (same environments as the Main screen).
4. On a scalar value, press `e` to encrypt or `d` to decrypt — only that value
   changes. Encrypted values are stored as `password: "![ciphertext]"` and are
   masked in the tree (`r` reveals them).
5. `Enter` on a scalar edits it manually; `Ctrl-s` saves (atomically);
   `Ctrl-r` restores the document to exactly how it was opened (with a
   confirmation if there are unsaved changes). The header shows `Modified` when
   there are unsaved edits.

Example — navigating to `database.password` and pressing `e` turns:

```yaml
database:
  username: admin
  password: secret   # unchanged comment
```

into:

```yaml
database:
  username: admin
  password: "![encrypted…]"   # unchanged comment
```

**Limitations:** values written with flow style (`{}`/`[]`), block/multiline
scalars (`|`/`>`), or anchors/aliases/tags are shown but not editable in place —
lazyprop refuses to edit them rather than reformat the file. Encrypting a
non-string scalar (e.g. a number) necessarily makes it a quoted string.

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

### Algorithm / mode compatibility

The **mode** (`state`) list is filtered by the selected **algorithm**, so you
can only pick combinations the tool actually accepts. Verified against the
bundled jar:

| Algorithm | Modes                |
| --------- | -------------------- |
| AES       | CBC, CFB, ECB, OFB   |
| Blowfish  | CBC, CFB, ECB, OFB   |
| DES       | CBC, CFB, ECB, OFB   |
| DESede    | CBC, CFB, ECB, OFB   |
| RC2       | CBC, CFB, ECB, OFB   |
| RCA (RC4) | *(none — stream cipher, unsupported by the tool)* |

When you change the algorithm, an incompatible mode is reset to the default
(CBC where supported). Selecting **RCA** shows the mode as `n/a`, and trying to
encrypt/decrypt with it reports a clear "not supported" error rather than
failing deep inside the jar.

## Keybindings

| Key         | Action                                   |
| ----------- | ---------------------------------------- |
| `s` / `Down`| Select next environment / scroll down    |
| `w` / `Up`  | Select previous environment / scroll up  |
| `1`/`2`/`3`/`4` | Jump to Main / Playground / YAML / About |
| `h` / `l`   | Previous / next screen                   |
| `Tab`       | Cycle focus (Environments ↔ Value)       |
| `/`         | Filter the environments list by name     |
| `e`         | Encrypt the current value                |
| `d`         | Decrypt the current value                |
| `Ctrl-y`    | Copy the result to the clipboard         |
| `r`         | Reveal / hide the selected key           |
| `p`         | Send selected environment → Playground   |
| `a`         | Add a new environment                    |
| `Enter`     | Edit the selected environment            |
| `x`         | Delete the selected environment          |
| `Esc`       | Leave input / cancel / back to Main       |
| `?`         | Open the About / help screen             |
| `q`         | Quit                                     |
| `Ctrl-c`    | Quit                                     |
| `Ctrl-z`    | Suspend                                  |

**YAML screen** (`3`): `Ctrl-o` open file · `w`/`s` navigate tree · `←`/`→`
fold/unfold · `Enter` edit scalar (or expand/collapse) · `e`/`d`
encrypt/decrypt the selected value · `Ctrl-s` save · `Ctrl-r` restore · `r`
reveal · `Tab` switch focus (environments ↔ tree) · `Esc` cancel/close.

In text fields (Value, and the form/playground Key/Value), `←` `→` `Home` `End`
move the cursor, `Backspace`/`Delete` edit at it, and long values scroll
horizontally to keep the cursor in view. Encrypt/decrypt runs in the background
(the pane shows `Working…`) so the UI never freezes during the JVM start-up.

Keybindings are configurable — see below. The interface uses ANSI-named colours
and dimmed secondary text, so it stays legible on both light and dark terminal
themes, and the footer hints shrink to fit narrow terminals.

## Configuration

### The `~/.lazyprop` home

Like Maven's `~/.m2`, lazyprop keeps its files in a home directory. On first run
it creates **`~/.lazyprop/`** containing a sample `envs.yaml` and the extracted
`secure-properties-tool.jar` (same location on macOS, Linux and Windows).

The **environments file** is resolved in this order (first match wins):

1. `--envs <path>` command-line flag
2. `LAZYPROP_ENVS` environment variable
3. `./envs.yaml` in the current directory (project-local, like a project's own
   `settings.xml`)
4. `~/.lazyprop/envs.yaml` (created from a sample on first run)

The **jar** is resolved the same way (`--jar`, `LAZYPROP_JAR`,
`./secure-properties-tool.jar`, then the copy in `~/.lazyprop`). Set
`LAZYPROP_HOME` to relocate the home directory.

### Keybindings, logs

Keybindings and styles are read from a `config.{json5,json,yaml,toml,ini}` in
the config directory (`$LAZYPROP_CONFIG`, otherwise the platform config dir),
falling back to the bundled [`.config/config.json`](.config/config.json). Logs
go to `lazyprop.log` in the data dir (`$LAZYPROP_DATA`); set `$LAZYPROP_LOG_LEVEL`
(e.g. `debug`) to change verbosity.

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
