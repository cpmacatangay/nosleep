# Implementation Plan: Keep-Awake Tray Toggle

**Branch**: `001-keep-awake` | **Date**: 2026-11-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-keep-awake/spec.md`

## Summary

A single-file Windows tray utility that holds an operating-system wake lock while On, so the machine
neither sleeps nor turns off its display. Left-click toggles; the icon and tooltip report state;
right-click offers Quit. No window, no persistence, no network, no administrator rights, no installer.

Technical approach: a small native Rust program using the Windows API directly — one hidden top-level
window for the tray-icon callback message, one thread that owns the execution-state request, and no
GUI framework. The wake lock is an RAII guard so it cannot outlive the process.

## Technical Context

**Language/Version**: Rust 1.99.1 (stable, `x86_64-pc-windows-msvc`)

**Primary Dependencies**: `windows` crate (Win32 bindings) only. No async runtime, no GUI framework, no
logging crate, no serialization crate.

**Storage**: None. State is a single `bool` in memory. No configuration file, no registry.

**Testing**: `cargo test` for the state machine; a build-time integration test that inspects the release
executable's import table; documented manual protocol for the power-behaviour acceptance criteria.

**Target Platform**: Windows 10 1809+ / Windows 11, x86_64. Portable executable, no runtime prerequisite.

**Project Type**: Desktop application (single binary, tray-resident).

**Performance Goals**: cold start to interactive < 100 ms; idle CPU 0.0%; working set ≤ 10 MB.

**Constraints**: release binary ≤ 1 MB; zero writes; zero network imports; `asInvoker`; no console window;
per-monitor DPI aware.

**Scale/Scope**: one user, one machine, one toggle. Roughly 500 lines of source.

## Constitution Check

*GATE: Must pass before implementation.*

| Principle | Gate | Status |
|---|---|---|
| I. Physically Inert When Off | Wake request lives inside a guard whose `Drop` releases it; no persistence; no autostart | PASS — by construction |
| II. One Click, No Ceremony | Single tray icon, click-to-toggle, no window | PASS |
| III. Never Bypass a Security Control | Only `SetThreadExecutionState`; no input injection, no elevation, no policy edits | PASS |
| IV. Zero Side Effects | No network imports, no file or registry writes, no timers | PASS — asserted by import-table test |
| V. Tiny by Construction | Size/RAM/startup budgets checked in CI-style script | PASS — enforced in T012 |

No violations; Complexity Tracking is empty.

### Design decisions and rejected alternatives

| Decision | Alternative rejected | Reason |
|---|---|---|
| Raw Win32 via `windows` crate | Tauri / Electron / WinUI | A webview or runtime costs 50–150× the binary size for one API call and a tray icon |
| Hidden top-level `WS_POPUP` window | Message-only window (`HWND_MESSAGE`) | Message-only windows do not receive the `TaskbarCreated` broadcast, so the icon would never return after an Explorer restart |
| `ES_DISPLAY_REQUIRED` in addition to `ES_SYSTEM_REQUIRED` | System-required only | The display-off timer is what raises the lock screen and forces re-authentication |
| RAII guard owning the request | Explicit on/off calls at each call site | Guarantees release on every exit path including panic |
| Statically linked CRT (`+crt-static`) | Default dynamic CRT | Removes the `VCRUNTIME140.dll` dependency, which is not part of Windows |
| Icons rasterised at runtime | Shipping `.ico` + `build.rs` + resource compiler | No build script, no resource toolchain, and always drawn at the exact shell pixel size |

## Project Structure

### Documentation (this feature)

```text
specs/001-keep-awake/
├── spec.md              # Feature specification
├── plan.md              # This file
├── tasks.md             # Task breakdown
└── quickstart.md        # Build, run, and acceptance protocol
```

### Source Code (repository root)

```text
Cargo.toml
.cargo/config.toml           # static CRT for the MSVC target
src/
├── main.rs                  # entry, CLI flag, single instance, message loop
├── app.rs                   # wiring: state <-> power <-> tray
├── state.rs                 # pure state machine (+ unit tests)
├── power.rs                 # PowerGuard: SetThreadExecutionState, RAII release
├── tray.rs                  # Shell_NotifyIconW v4, icon swap, tooltip, TaskbarCreated
├── menu.rs                  # popup menu construction and command ids
├── icon.rs                  # rasterised artwork -> HICON (+ unit tests)
├── diag.rs                  # --diagnose
└── util.rs                  # wide-string helpers
scripts/
└── verify.ps1               # size, subsystem, imports, tray, RAM, idle CPU
```

Tests live beside the code they exercise (`#[cfg(test)]` in `state.rs` and
`icon.rs`), so the crate stays a single binary with no library target.

**Structure Decision**: single binary crate at the repository root. The repository is the product; no
workspace, no libraries, because nothing needs to be reused.

## Verification Strategy

Three layers, in order of cost:

1. **Unit** — the toggle state machine, headless, no OS calls (`cargo test`).
2. **Static** — parse the built executable's import table and fail if anything outside
   `kernel32/user32/shell32/gdi32/advapi32/comctl32/msvcrt/ucrtbase` appears; fail if the PE subsystem
   is console; fail if the binary exceeds 1 MB.
3. **Behavioural** — the manual battery protocol in `quickstart.md`, judged by Windows Kernel-Power
   events rather than by looking at the screen.

## Risks

| Risk | Response |
|---|---|
| Employer enforces an inactivity lock | Documented limitation; the app cannot and will not work around it |
| Endpoint protection flags an unsigned binary | No network, no injection, no elevation, no writes — minimal surface; source is inspectable |
| Static CRT interacts badly with the `windows` crate's proc macros | Build with `+crt-static` on the target only; validate by running the binary |
| Tray icon hidden in the Windows 11 overflow area | Documented in quickstart: enable "Show this icon" once |
