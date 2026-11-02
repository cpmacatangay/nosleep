# Feature Specification: Keep-Awake Tray Toggle

**Feature Branch**: `001-keep-awake`

**Created**: 2026-11-02

**Status**: Draft

**Input**: User description: "A desktop app that is non-invasive, minimalist, and fast, that with one click prevents my laptop from sleeping. When it is On, it is like pressing an imaginary button that keeps the machine awake."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Turn wakefulness on with one click (Priority: P1)

A person is reading, thinking, or waiting on a long-running task. The machine is configured to sleep
after a short idle period, and every sleep forces a re-authentication and the loss of working context.
They click the tray icon once. From that moment the machine stays awake, with the screen on, until
they click again.

**Why this priority**: This is the entire product. Everything else is affordance around this one
interaction. Without it there is no feature to ship.

**Independent Test**: Run the utility, click the icon once, leave the machine untouched on battery,
and confirm after the configured idle period that the machine never slept and the screen never turned
off. Value delivered: the user's session survives an idle period.

**Acceptance Scenarios**:

1. **Given** the utility is running and Off, **When** the user clicks the tray icon once, **Then** the
   icon visibly changes to the On state and the machine no longer sleeps or turns off its display.
2. **Given** the utility is On and the machine is on battery power, **When** the user does not touch
   the machine for longer than the configured sleep period, **Then** the machine remains awake and the
   session remains signed in.
3. **Given** the utility is On, **When** the user returns after the idle period, **Then** the screen is
   still on and no re-authentication is required.

---

### User Story 2 - Tell at a glance whether it is On (Priority: P1)

The person glances at the tray to decide whether the machine will stay awake. They need an unambiguous
answer without clicking anything or opening a window.

**Why this priority**: A toggle whose state cannot be read is dangerous — the user cannot tell whether
their session is protected, and may leave the machine awake unintentionally. State legibility is part
of the core value, not decoration.

**Independent Test**: With the utility running, toggle it and observe the tray icon and the tooltip
text. An observer who has never seen the app can correctly say whether it is On.

**Acceptance Scenarios**:

1. **Given** the utility is Off, **When** the user hovers the tray icon, **Then** the tooltip states
   that the machine will sleep normally.
2. **Given** the utility is On, **When** the user hovers the tray icon, **Then** the tooltip states
   that the machine is being kept awake.
3. **Given** either state, **When** the user looks at the icon, **Then** the two states are
   distinguishable by shape as well as by colour, so colour-blind users are not disadvantaged.

---

### User Story 3 - Return to normal behaviour (Priority: P1)

The person finishes what they were doing and wants the machine to sleep normally again, so it does not
drain the battery or stay lit in a bag.

**Why this priority**: An irreversible or hard-to-reverse toggle is worse than no toggle. Off must be
as cheap as On.

**Independent Test**: Turn On, then turn Off, and confirm the machine returns to its configured sleep
and display-off behaviour within the configured idle period.

**Acceptance Scenarios**:

1. **Given** the utility is On, **When** the user clicks the tray icon once, **Then** the icon returns
   to the Off state and normal power behaviour resumes.
2. **Given** the utility is On, **When** the app is quit or its process is terminated, **Then** the
   machine returns to normal power behaviour immediately, with no residual wakefulness.
3. **Given** the utility is On, **When** the machine is shut down or the user signs out, **Then** the
   app exits cleanly and leaves nothing behind.

---

### User Story 4 - Start from a clean slate (Priority: P2)

The person launches the utility after a reboot and expects it not to have silently kept the machine
awake from a forgotten previous session.

**Why this priority**: Fail-safe default. Prevents the "why is my laptop hot in my bag" failure mode.
Lower priority only because the app is normally always running.

**Independent Test**: Launch the app twice with a reboot or a quit in between and confirm it always
starts Off.

**Acceptance Scenarios**:

1. **Given** the utility was On when it was quit, **When** it is launched again, **Then** it starts Off.
2. **Given** the machine has just restarted, **When** the utility is launched, **Then** it starts Off.
3. **Given** the utility is already running, **When** the user launches it a second time, **Then** no
   second copy starts and no second tray icon appears.

---

### User Story 5 - Leave no trace (Priority: P2)

The person wants to be able to remove the utility completely and to know it does nothing they did not
ask for, because the machine may be managed by an employer.

**Why this priority**: Trust is a prerequisite for running an unsigned utility on a managed device.
Lower priority than function, but part of the product's definition rather than a nice-to-have.

**Independent Test**: Inspect the process and the filesystem while the app runs; delete the executable
and confirm nothing remains.

**Acceptance Scenarios**:

1. **Given** the utility is running, **When** the user inspects its network activity, **Then** there is
   none.
2. **Given** the utility has run, **When** the user inspects the filesystem and registry, **Then** the
   app has written nothing.
3. **Given** the user no longer wants the utility, **When** they delete the executable, **Then** no
   files, services, scheduled tasks, or startup entries remain.

---

### Edge Cases

- **An organization enforces a separate inactivity lock.** The screen locks regardless of the app. The
  app must not pretend otherwise; this is documented as a limitation rather than fought.
- **The machine is closed or the lid is shut.** Firmware and policy sleep the machine regardless of any
  application request. Out of scope.
- **Battery reaches a critical level.** The operating system hibernates or shuts down regardless of the
  app. This is correct behaviour and must not be suppressed.
- **The app is killed with a task manager.** Wakefulness must end with the process, not persist.
- **The shell restarts (Explorer crash or update).** The tray icon disappears with it; the app must
  restore its icon and preserve its current state.
- **The machine is on mains power.** It may already never sleep; the toggle still behaves consistently.
- **The user is on a machine where the sleep timer is already disabled.** The toggle remains meaningful
  for the display-off timer and must not error.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST present exactly one tray icon and MUST NOT present a taskbar button or a
  main window.
- **FR-002**: A single primary click on the tray icon MUST toggle wakefulness between On and Off.
- **FR-003**: The tray icon MUST reflect the current state through both shape and colour, and the
  tooltip MUST state the current state in words.
- **FR-004**: When On, the system MUST prevent both idle sleep and idle display-off, because display-off
  is what causes the session lock and the re-authentication the user is trying to avoid.
- **FR-005**: When Off, the system MUST release any wakefulness request it holds and MUST NOT otherwise
  influence power behaviour.
- **FR-006**: The system MUST start in the Off state on every launch and MUST NOT persist state between
  runs.
- **FR-007**: The system MUST allow the user to quit from a secondary (right-click) menu, and quitting
  MUST release wakefulness before exit.
- **FR-008**: The system MUST permit only one running instance; a second launch MUST exit without effect.
- **FR-009**: The system MUST release wakefulness on sign-out, shutdown, and abnormal termination.
- **FR-010**: The system MUST restore its tray icon if the shell restarts, preserving the current state.
- **FR-011**: The system MUST NOT require administrative privileges at any point.
- **FR-012**: The system MUST NOT access the network, collect telemetry, or write to the filesystem or
  registry.
- **FR-013**: The system MUST provide exactly one diagnostic affordance, a command-line flag that
  reports whether the wakefulness request was accepted, and MUST NOT create log files.
- **FR-014**: The system MUST NOT inject synthetic input, alter power plans, or modify security policy.

### Key Entities

- **Awake State**: a two-valued condition (On / Off) owned by the running process, not persisted, and
  released when the process ends.
- **Wakefulness Request**: the operating-system-level claim the app holds while On. Its lifetime is
  bounded by the process lifetime by construction.
- **Tray Presence**: the single icon, its artwork for each state, and its tooltip text.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can turn wakefulness on with exactly one click from a running app, and off with
  exactly one click.
- **SC-002**: With the app On and the machine on battery, the machine completes an idle period at least
  20% longer than its configured sleep timeout with zero sleep events recorded.
- **SC-003**: With the app Off, the machine sleeps within one configured idle period, confirming no
  residual effect.
- **SC-004**: The app is indistinguishable from absent when Off: no sleep events prevented, no files
  written, no network connections opened.
- **SC-005**: The shipped executable is a single file under 1 MB that starts in under 100 ms and uses
  no measurable CPU while idle.
- **SC-006**: The app runs on a machine with no development tools or language runtimes installed.
- **SC-007**: An observer can determine the current state from the tray alone, without clicking, in
  under two seconds.

## Assumptions

- The user runs Windows 10 or Windows 11 on an x86_64 machine and has permission to run a portable
  executable without administrator rights.
- The sleep and display-off timeouts are set by policy that the user cannot change; the user does not
  want the app to change them, only to hold them off while On.
- The employer does not enforce a separate inactivity lock; if it does, this feature cannot deliver its
  value and that is documented rather than worked around.
- The machine has no battery-critical override the user wishes to defeat; hibernation at critical
  battery is accepted.
- A single user is signed in; multi-user fast switching is out of scope.
- English-only labelling is acceptable for version 1.
