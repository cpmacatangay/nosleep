# Feature Specification: Start with Windows

**Feature Branch**: `002-autostart`

**Created**: 2026-11-02

**Status**: Implemented

**Input**: User description: "I want the icon waiting for me after I log in, so I don't have to remember to launch it."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start with Windows (Priority: P2)

A person turns the machine on in the morning. NoSleep is already in the notification
area, Off, ready to be clicked. They never opened it themselves.

**Why this priority**: The product's value is one click away from useless if the app is
not running. Forgetting to launch it is the most likely reason the tool fails to do its
job. Lower than P1 only because the app is fully functional without it.

**Independent Test**: Tick "Start with Windows", sign out, sign back in, and confirm the
icon is present and the app is Off.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** the user ticks "Start with Windows", **Then** a
   value named `NoSleep` exists under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
   whose data is the absolute path of the running executable, in quotes.
2. **Given** autostart is enabled, **When** the user signs out and back in, **Then** NoSleep
   is running, in the Off state, with one tray icon.
3. **Given** autostart is enabled, **When** the user untick it, **Then** the value is removed.
4. **Given** the executable was moved and started from its new location, **When** the app
   launches, **Then** the stored path is corrected without user action.
5. **Given** a fresh download, **When** the app runs for the first time, **Then** nothing is
   written and autostart is Off.

---

### Edge Cases

- **The registry write is refused** (policy, permissions). The app keeps working; the next
  time the menu is opened, the tick shows the true state.
- **The executable is deleted while autostart is on.** Windows silently skips the entry.
  Reinstalling to the same path restores it.
- **Windows itself disables the entry** from Task Manager or Settings. The value remains, so
  the app shows it as enabled; Windows is the authority on whether it actually runs.
- **The path contains spaces or quotes.** Spaces are handled by quoting; a path containing a
  double quote is refused rather than written incorrectly.
- **Multiple instances.** Only one app instance exists, so only one writer.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tray menu MUST contain a checkable item labelled "Start with Windows".
- **FR-002**: The item's checked state MUST be read from the registry each time the menu is
  opened, never from a cached value.
- **FR-003**: Enabling MUST write exactly one `REG_SZ` value named `NoSleep` under
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **FR-004**: The value data MUST be the absolute path of the currently running executable,
  enclosed in double quotes.
- **FR-005**: Disabling MUST remove that value and MUST succeed when it is already absent.
- **FR-006**: Autostart MUST default to Off and MUST NOT be written unless the user enables it.
- **FR-007**: On launch, if autostart is enabled and the stored path differs from the current
  executable, the app MUST rewrite it.
- **FR-008**: A failed registry operation MUST NOT crash the app or change any other state.
- **FR-009**: The app MUST NOT write any other registry key or file, and MUST NOT require
  administrator rights for this feature.
- **FR-010**: If the tray icon cannot be added at sign-in because the notification area is not
  ready, the app MUST retry for a bounded period rather than fail silently.

### Key Entities

- **Autostart Entry**: the single registry value; its presence is the feature's entire state.
- **Stored Command**: the quoted absolute path, which is what Windows will execute.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After enabling and signing out/in, the app is running with zero user action.
- **SC-002**: Enabling or disabling takes effect with no confirmation dialog and no restart.
- **SC-003**: The app writes zero registry values and zero files when the feature is unused.
- **SC-004**: A moved executable self-corrects on the next launch, with no user action.
- **SC-005**: The entry is visible to the user in Windows' own startup controls.

## Assumptions

- The executable lives at a stable path that does not change between sessions.
- The user's account may write to its own `Run` key; this is the case for standard users by
  default and requires no elevation.
- Windows' own startup controls may independently disable the entry; the app does not attempt
  to override that.
