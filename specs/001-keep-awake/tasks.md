# Tasks: Keep-Awake Tray Toggle

**Input**: Design documents from `specs/001-keep-awake/`

**Prerequisites**: plan.md, spec.md

**Tests**: Included only where they are cheap and objective: unit tests for the state machine and the
icon rasteriser, plus a static/runtime verification script. Power behaviour is verified by the
documented protocol in `quickstart.md`, judged by Kernel-Power events.

**Status**: Implemented. All automated checks pass; the behavioural acceptance protocol (AC-2) is
outstanding because it must run on the target machine on battery.

## Phase 1: Setup

- [x] T001 Create `Cargo.toml` with `windows-sys` as the only dependency and a size-oriented release profile
- [x] T002 Create `.cargo/config.toml` enabling the static CRT for `x86_64-pc-windows-msvc`
- [x] T003 Add `#![windows_subsystem = "windows"]` and the `--diagnose` argument handling in `src/main.rs`

---

## Phase 2: Foundational (blocking)

- [x] T004 Implement `src/power.rs`: `PowerGuard` with `acquire()` / `release()`, `Drop` releasing, and
      `last_error()`; success is judged by last-error, not by the ambiguous return value
- [x] T005 Implement `src/state.rs`: `State::{Off,On}` with pure `toggled()`, `is_on()`, `tooltip()`
- [x] T006 Implement single-instance guard via a named mutex in `src/main.rs`
- [x] T007 Implement the hidden top-level window, window class, and message loop in `src/main.rs`

**Checkpoint**: the process starts, owns a mutex, and pumps messages.

---

## Phase 3: User Story 1 — Toggle wakefulness on (P1) 🎯 MVP

**Goal**: one click keeps the machine awake.
**Independent Test**: click once on battery, wait past the sleep timeout, observe no sleep event.

- [x] T008 [US1] Implement `src/icon.rs`: rasterise the artwork into an `HICON` via `CreateIconIndirect`
- [x] T009 [US1] Implement `src/tray.rs`: add/modify the tray icon with `Shell_NotifyIconW`, version-4 callbacks
- [x] T010 [US1] Wire click → `State::toggled()` → `PowerGuard` → icon swap in `src/app.rs`
- [x] T011 [US1] Unit-test the state machine in `src/state.rs` (4 tests)
- [x] T011b [US1] Unit-test the icon rasteriser in `src/icon.rs` (4 tests)

**Checkpoint**: the core value works; verified statically and by shell registration.

---

## Phase 4: User Story 2 — Read the state at a glance (P1)

- [x] T012 [US2] Render distinct On/Off artwork: solid amber disc vs hollow grey ring
- [x] T013 [US2] Update the tooltip on every state change with explicit words

---

## Phase 5: User Story 3 — Return to normal (P1)

- [x] T014 [US3] Implement `src/menu.rs`: checkable toggle item plus Quit, with `TPM_RETURNCMD`
- [x] T015 [US3] Release the wake lock on Quit, `WM_QUERYENDSESSION`, `WM_ENDSESSION`, and `WM_DESTROY`
- [x] T016 [US3] Remove the tray icon before exit

---

## Phase 6: User Story 4 — Clean slate (P2)

- [x] T017 [US4] Ensure state is never persisted; always start Off
- [x] T018 [US4] Second launch exits silently without creating a second icon
- [x] T019 [US4] Re-add the tray icon on the `TaskbarCreated` broadcast, preserving state

---

## Phase 7: User Story 5 — Leave no trace (P2)

- [x] T020 [US5] Implement `--diagnose` reporting the API result and exit code, writing to a console,
      pipe, or redirected file, with a dialog fallback
- [x] T021 [US5] Confirm zero filesystem and registry writes by inspection and code review

---

## Phase 8: Polish and verification

- [x] T022 [P] `scripts/verify.ps1`: size, subsystem, import table, wake request, single instance,
      tray registration, working set, idle CPU
- [x] T023 [P] `quickstart.md`: build, run, acceptance protocol, known limitations
- [x] T024 Remove build warnings; `cargo clippy -- -D warnings` is clean
- [x] T025 Confirm the two icon states are distinguishable from the shell's own icon snapshot
- [ ] T026 Run the behavioural acceptance protocol (AC-2) on the target machine, on battery

---

## Dependencies

- Phase 2 blocks everything.
- US1 (Phase 3) is the MVP; US2–US5 layer on it and are independently verifiable.
- Phase 8 depends on a working build; T026 depends on access to the target machine.

## Notes

- Every wake request goes through `PowerGuard`; no direct `SetThreadExecutionState` calls elsewhere.
- No `unsafe` outside the Win32 wrapper modules; the state machine stays pure.
