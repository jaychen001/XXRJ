# Release Validation

## v0.1.0 - 2026-07-08

**Git commit**: `4e93953`

**Release scope**:
- Windows desktop offline application.
- Phase 9 delivery: report export, QA coverage audit, regression runner, Windows package artifacts.

## Build Artifacts

| Artifact | Size | SHA256 |
| --- | ---: | --- |
| `selector-desktop/src-tauri/target/release/bundle/msi/Selector Desktop_0.1.0_x64_en-US.msi` | 6,606,848 bytes | `73DD856DF6C16C6ED403FB697B54672E58BB3AE6A28A61433A4E410A3CB3DD7D` |
| `selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe` | 4,598,562 bytes | `A043CB79CF2B36ACE45F91287EA76520A168F35036EE3DED0310248F05322BC0` |

## Validation Commands

All commands below passed on 2026-07-08:

```powershell
cmd /c pnpm.cmd test
```

Result: 7 test files passed, 19 tests passed.

```powershell
cargo test
```

Result: 15 tests passed, including `qa::regression_runner::tests::regression_runner_passes_all_fixture_groups`.

```powershell
cmd /c pnpm.cmd package:windows
```

Result: MSI and NSIS package artifacts were generated.

## Privacy Audit

Bundle directory checked:

```text
selector-desktop/src-tauri/target/release/bundle
```

Checks performed:
- No `.env`, `.db`, `.sqlite`, credential, key, token, session, or `selector.db` files found in bundle artifacts.
- No matches for developer paths, user paths, or common API key markers:
  - `D:\codex`
  - `C:\Users`
  - `Users/`
  - `SCKJ`
  - `sk-ant-`
  - `sk-proj-`
  - `ANTHROPIC_API_KEY`
  - `OPENAI_API_KEY`

## Installed Package Smoke Test

Installer tested:

```text
selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe
```

Smoke steps:
- Silent install into `.codex/release-smoke/install`.
- Launch installed `selector-desktop.exe`.
- Confirm process starts and remains running.
- Confirm local SQLite database initialization.
- Silent uninstall after smoke test.

Observed result:
- Installer exit code: `0`.
- Installed app process started and stayed running for the smoke window.
- Uninstaller exit code: `0`.
- Database initialized at `%APPDATA%\com.sckj.selector\selector.db`.
- Database health: 2 applied migrations, 18 application tables.

Note: Tauri resolved the app data directory to the normal Windows roaming app data path even when `APPDATA` and `LOCALAPPDATA` were set for the launched process. Future automated smoke tests should add an explicit test-mode data directory hook if strict filesystem isolation is required.

## Status

Release validation status: **PASS with note**.

The installable Windows artifacts are usable for internal trial, with the caveat that unsigned installer warnings may appear because code signing has not been configured.
