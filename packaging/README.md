# Packaging

Distribution manifests for the install methods documented in the main README.
Both reference the archives produced by the release workflow
(`.github/workflows/cd.yml`), which also uploads a `sha256` checksum for each.

## Homebrew (`homebrew/lazyprop.rb`)

One-time setup:

1. Create a public repo named **`homebrew-tap`** under your account.
2. Add `homebrew/lazyprop.rb` to it as `Formula/lazyprop.rb`.

Per release: bump `version` and replace the three `sha256` placeholders with the
checksums from the release (each archive has a matching `.sha256` asset). Users
then install with:

```bash
brew install kchernokozinsky/tap/lazyprop
```

## Scoop (`scoop/lazyprop.json`)

One-time setup:

1. Create a public repo named **`scoop-bucket`** under your account.
2. Add `scoop/lazyprop.json` to it as `bucket/lazyprop.json`.

The manifest has `checkver`/`autoupdate`, so future versions can be bumped with
`scoop update` tooling; for the first version, replace the `hash` placeholder
with the checksum of the Windows `.zip`. Users then install with:

```powershell
scoop bucket add lazyprop https://github.com/kchernokozinsky/scoop-bucket
scoop install lazyprop
```
