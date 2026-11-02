<!--
SYNC IMPACT REPORT
Version change: (new) → 1.0.0
Modified principles: n/a (initial ratification)
Added sections: Core Principles I–V, Additional Constraints, Development Workflow, Governance
Removed sections: none
Follow-up TODOs: none
-->

# NoSleep Constitution

## Core Principles

### I. Physically Inert When Off

When the utility is Off, or simply not running, the machine MUST behave exactly as if the utility had
never been installed. The wake request MUST be released on Quit, logoff, shutdown, panic, and abrupt
process termination. No background timers, no scheduled tasks, no service, no driver, no autostart
entry created without the user's explicit request, and no state that survives the process.

Rationale: a tool whose purpose is to prevent sleep is only acceptable if it is incapable of causing
unintended wakefulness or battery drain when the user is not asking for it. Fail-safe, not fail-open.

### II. One Click, No Ceremony

The primary interaction MUST be a single click on a tray icon that toggles the state. There MUST be no
main window, no settings screen, no wizard, no first-run dialog, and no balloon on startup. State MUST
be legible at a glance from the icon and the tooltip. Secondary actions (quit) MUST live in a
right-click menu.

Rationale: the value of the tool is measured in seconds saved, so any interaction cost beyond a click
erodes the reason it exists.

### III. Never Bypass a Security Control

The utility MUST request wakefulness only through the documented operating-system API intended for
applications that need the machine to stay awake. It MUST NOT inject synthetic input, move the pointer,
keep a phantom session alive, disable or edit security policy, modify power plans, elevate privileges,
install drivers or services, or otherwise circumvent an administrative control. Where an organization
enforces an inactivity lock, the utility MUST lose gracefully and MUST document that it cannot help.

Rationale: this keeps the tool equivalent to a video call holding a wake lock — a defensible
productivity utility rather than a policy circumvention tool. Technical honesty here is also a
practical safeguard: the app must never claim to do something it cannot.

### IV. Zero Side Effects

The utility MUST NOT access the network, emit telemetry, write files, write registry keys, or display
notifications the user did not ask for. It MUST consume no measurable CPU while idle. Distribution MUST
be a single self-contained executable; removal MUST be deletion of that file, with no residue.

Rationale: non-invasiveness is a functional requirement, not a stylistic preference. A user must be
able to verify the entire footprint by looking at one file.

### V. Tiny by Construction (NON-NEGOTIABLE)

The release artifact MUST stay within these budgets, verified by automated checks and not by
aspiration: binary ≤ 1 MB; working set ≤ 10 MB; cold start to interactive < 100 ms; idle CPU 0.0%;
zero dependencies outside the operating system. Any dependency that violates a budget is rejected, and
the budget is not raised to accommodate it.

Rationale: the smallest possible program is the most auditable, the least likely to be flagged by
endpoint protection, and the fastest to start — all properties the target user actually feels.

## Additional Constraints

**Platform**: Windows 10 1809+ and Windows 11, x86_64. Windows only; other platforms are out of scope.

**Packaging**: one portable executable, no installer, no runtime prerequisite. The process MUST request
`asInvoker` and MUST declare per-monitor DPI awareness. No console window may appear.

**Observability**: exactly one diagnostic affordance, a `--diagnose` command-line flag. No log files.

**Compatibility**: the executable MUST run on a machine with no development tools, no language runtime,
and no redistributable packages installed.

## Development Workflow

Work proceeds spec-first: constitution → specification → plan → tasks → implementation → convergence.
A change is complete only when its acceptance criteria have been demonstrated on the target machine.

Verification MUST prefer objective signals over visual inspection where such signals exist: operating
system power events, process resource counters, and static inspection of the shipped executable's
import table. A criterion that can only be judged by "it looked fine" MUST be replaced with a
measurable proxy.

Any proposal that adds a background component, a network call, a persisted setting, or a second
executable MUST be rejected unless this constitution is amended first.

## Governance

This constitution supersedes other development practices where they conflict. Amendments require a
documented rationale, a semantic version bump (MAJOR for principle removal or redefinition, MINOR for a
new or materially expanded principle, PATCH for clarification), and an update to any dependent
specification or plan. Compliance with Principles I–V is reviewed at every convergence step. Complexity
that violates a budget MUST be removed rather than justified.

**Version**: 1.0.0 | **Ratified**: 2026-11-02 | **Last Amended**: 2026-11-02
