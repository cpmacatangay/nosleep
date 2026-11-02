# Implementation Plan: Start with Windows

**Branch**: `002-autostart` | **Date**: 2026-11-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/002-autostart/spec.md`

## Summary

Add one checkable tray-menu item that registers the executable under the per-user `Run`
key. The registry layer is isolated in its own module, takes the key path as a parameter so
it can be tested without touching the real autostart entry, and is the only code in the
program permitted to write to the registry.

Also fixes a latent sign-in race: if the notification area is not ready when the app
starts, adding the tray icon fails and the app would have run invisibly forever.

## Technical Context

**Language/Version**: Rust 1.99.1, `x86_64-pc-windows-msvc`

**Primary Dependencies**: `windows-sys`, adding the `Win32_System_Registry` feature only

**Storage**: one `REG_SZ` value at `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\NoSleep`

**Testing**: unit tests against throwaway `HKCU` keys, plus a test pinning the real key path

**Target Platform**: Windows 10 1809+ / 11, x86_64

**Constraints**: no elevation, no COM, no installer, no files written, no cached UI state

**Scale/Scope**: one boolean, one menu item, roughly 150 lines including tests

## Constitution Check

| Principle | Gate | Status |
|---|---|---|
| I. Physically Inert When Off | Autostart runs the app; the app still starts Off | PASS |
| II. One Click, No Ceremony | One menu item, no dialog | PASS |
| III. Never Bypass a Security Control | Writes only to the user's own Run key, visible in Windows' startup controls | PASS |
| IV. Zero Side Effects | Amended in constitution v1.1.0 to permit exactly this one value | PASS (amended) |
| V. Tiny by Construction | `RegSetValueExW` and friends add ~2 KB; budgets unaffected | PASS |

## Design

```text
src/autostart.rs
├── is_enabled()        registry read
├── stored_path()       raw value, as Windows sees it
├── enable()            write quoted current exe
├── disable()           delete, idempotent
├── heal()              rewrite when the stored path is stale
└── private plumbing    open_key / create_key / read_value / write_value / delete_value
```

The public functions hard-code the real key; the plumbing takes the key path as an
argument, which is what makes the unit tests safe and hermetic.

`main` calls `heal()` once after attaching the tray. The menu reads
`is_enabled()` on every open, so it cannot show a state the registry disagrees with.

### Tray retry

`App` gains a two-second timer that retries `Shell_NotifyIconW` up to fifteen times,
then gives up. The timer is cancelled as soon as the icon is registered, on shutdown, and
when the attempt budget is exhausted.

## Rejected alternatives

| Alternative | Why not |
|---|---|
| Startup-folder shortcut | Needs `IShellLink` COM; 20–40 KB and far more code for the same result |
| Scheduled task | Heavier, and looks less like a normal application to IT |
| `HKLM` Run key | Requires administrator rights and would affect every account |
| Writing a config file | The registry value is the state; a file would duplicate it |

## Verification

1. Unit tests: write/read/delete round trip, idempotent delete, empty value, missing value,
   and the real key path constant.
2. Manual: tick the item, confirm with `reg query`, sign out and in.
3. `scripts/verify.ps1` unchanged; it reports autostart state for information.

## Risks

| Risk | Response |
|---|---|
| Corporate policy blocks the `Run` key | Nothing breaks; the menu shows the true state |
| Visibility to IT increases | Fully transparent: the entry appears in Task Manager → Startup apps |
| Windows disables the entry independently | Documented; the app does not fight it |
| Stale path after a move | `heal()` on every launch |
