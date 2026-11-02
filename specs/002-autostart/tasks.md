# Tasks: Start with Windows

**Input**: Design documents from `specs/002-autostart/`

**Prerequisites**: plan.md, spec.md

**Status**: Implemented. Registry behaviour covered by unit tests; the sign-out/sign-in
check is manual.

## Phase 1: Governance and specification

- [x] T001 Amend the constitution to v1.1.0, permitting one autostart registry value
- [x] T002 Write `spec.md`, `plan.md`, `tasks.md` for this feature

## Phase 2: Registry layer

- [x] T003 Create `src/autostart.rs` with `is_enabled`, `stored_path`, `enable`, `disable`, `heal`
- [x] T004 Parameterise the plumbing by key path so tests never touch the real entry
- [x] T005 Unit tests: round trip, idempotent delete, empty value, missing value, real key constant

## Phase 3: Integration

- [x] T006 Add the "Start with Windows" checkable item to `src/menu.rs`
- [x] T007 Dispatch the command in `src/main.rs`, reading state fresh each time the menu opens
- [x] T008 Call `heal()` once at startup, after the tray is attached
- [x] T009 Add the tray-icon retry timer in `src/app.rs` and handle `WM_TIMER`

## Phase 4: Documentation and release

- [x] T010 README: document the menu item and the CalVer release flow; drop the USB wording
- [x] T011 `scripts/set-version.ps1`: CalVer validation and automatic next-version derivation
- [x] T012 Bump to `2026.11.1`, tag, publish

## Notes

- `autostart.rs` is the only module allowed to call the registry; the rest of the program
  has no registry imports.
- The failure path is deliberately silent: no dialog, no balloon. The next menu open shows
  the truth, which is the only feedback that cannot lie.
