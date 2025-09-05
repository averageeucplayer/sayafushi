
Tauri updater setup

```
cargo tauri signer generate -w ~/.tauri/app.key
```

To generate icons

```
cargo tauri icon
```

Trigger release

```
$version = "0.0.1";
git tag "v$version";
git push origin "v$version";
```

Prune tags

```
git fetch --prune --prune-tags
```

Verbose build

```
$env:RUST_LOG="tauri=debug"
$env:TAURI_LOG_LEVEL="debug"
cargo tauri build --verbose
```