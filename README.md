# NoSleep

A one-click wake lock for Windows. Click the tray icon once and the machine
neither sleeps nor turns off its display, until you click again.

Built for laptops whose sleep timeout is set by policy and cannot be changed, and
where every sleep costs a re-login and the loss of working context.

```
  Off                                On
  ○  normal sleep        ──click──▶  ●  keeping this PC awake
```

- **One click.** No window, no settings, no wizard.
- **216 KB.** Single executable, no installer, no runtime.
- **0.0% idle CPU**, ~9 MB working set, starts in milliseconds.
- **No network, no telemetry, no files written, no registry writes, no admin rights.**
- **Inert when Off.** Quit, crash, or kill the process and normal power behaviour
  resumes immediately — the operating system releases the request with the process.

## Use

1. Copy `nosleep.exe` anywhere writable, for example `%LOCALAPPDATA%\NoSleep\`.
2. Run it. An icon appears in the notification area.
3. **Left-click** to toggle. **Right-click** for the menu (toggle, Quit).

Windows 11 hides new tray icons by default. If you do not see it, click the `^`
overflow chevron and drag the icon onto the taskbar, or use *Settings → Taskbar →
Other system tray icons*.

## What it does, precisely

While On, the process holds an operating-system wake lock
(`SetThreadExecutionState` with `ES_CONTINUOUS | ES_SYSTEM_REQUIRED |
ES_DISPLAY_REQUIRED`). This is the same documented mechanism a video player or a
video call uses to stop the screen from turning off during playback. The display
stays on, so the session never reaches the idle lock screen.

When Off, it releases the lock and nothing else changes.

### What it does not do

| | |
|---|---|
| **Bypass a security policy** | It does not touch power plans, policies, or the registry |
| **Survive an enforced inactivity lock** | If IT configured *Interactive logon: Machine inactivity limit*, the screen locks regardless of this or any other app |
| **Prevent sleep on lid close** | Firmware and the lid action decide that |
| **Prevent hibernation at critical battery** | Windows will hibernate; that is correct behaviour |
| **Block `Win+L`** | Manual locking always works |
| **Start with Windows** | No autostart entry is created. Launch it yourself, or add a shortcut |

## Verify it yourself

```powershell
.\nosleep.exe --diagnose
```

Reports whether the OS accepted the wake request. Exit code `0` means accepted,
`2` means refused. This command releases the request immediately; it does not keep
the PC awake.

To verify the real behaviour, put the machine on battery, turn the app On, leave
it alone for longer than the sleep timeout, then check that no sleep event was
recorded:

```powershell
Get-WinEvent -FilterHashtable @{LogName='System'; Id=42,107; StartTime=(Get-Date).AddMinutes(-30)} |
  Select-Object TimeCreated, Id
```

No event `42` (sleep) in the window means it worked.

## Portable package

To produce a folder and a zip that can be carried to another machine:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package.ps1
```

Output:

```text
dist\NoSleep\NoSleep.exe     the application, self-contained
dist\NoSleep\README.txt      instructions for the target machine
dist\NoSleep\SHA256.txt      checksum, to confirm the copy survived transfer
dist\NoSleep-0.1.0-win64.zip
```

The executable has no external dependencies, reads no files, and writes none, so
the folder runs in place — from `%LOCALAPPDATA%`, from a USB stick, from a path
with spaces. There is nothing to install and nothing to uninstall; deleting the
executable is a complete removal.

When transferring, prefer a USB stick or clear the download mark on arrival:

```powershell
Unblock-File .\NoSleep.exe
```

## Releases

Tagging a commit publishes a build automatically:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

`.github/workflows/release.yml` runs the unit tests, the static checks, and the
packaging script, then attaches `NoSleep-0.1.0-win64.zip` to a GitHub release. It
also fails the build if the tag and the version in `Cargo.toml` disagree.

Downloading a release asset attaches a Mark-of-the-Web, so Windows will show
"Windows protected your PC" on first launch. Either clear it, or download with the
GitHub CLI, which does not add the mark:

```powershell
gh release download --pattern 'NoSleep-win64.zip'
```

The asset name is deliberately constant, so the latest build always lives at:

```text
https://github.com/<owner>/nosleep/releases/latest/download/NoSleep-win64.zip
```

## Build

Requires Rust (MSVC toolchain) and the Visual C++ build tools.

```powershell
cargo build --release
cargo test --release
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1
```

`scripts/verify.ps1` checks the size budget, the GUI subsystem, the import table
(no non-Windows dependencies), the power API, single-instance behaviour, tray-icon
registration, memory, and idle CPU.

## Design

Detailed specification, plan, and task history live in
[`specs/001-keep-awake/`](specs/001-keep-awake/). Project principles are in
[`.specify/memory/constitution.md`](.specify/memory/constitution.md).

The short version:

- ONE dependency (`windows-sys`), no GUI framework, no async runtime.
- The wake request is an RAII guard, so no exit path can leave the machine awake.
- A hidden top-level window (not message-only) so the tray icon survives an
  Explorer restart.
- The C runtime is linked statically, so the executable does not need the Visual
  C++ Redistributable.
- Icons are rasterised at the shell's real pixel size instead of shipping `.ico`
  files, which keeps the build free of resource compilation and the edges crisp.

## Licence

MIT
